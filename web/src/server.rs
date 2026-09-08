use std::sync::Arc;
use std::sync::Mutex;

use axum::Json;
use axum::Router;
use axum::body::Body;
use axum::extract::Path;
use axum::extract::State;
use axum::http::StatusCode;
use axum::http::header;
use axum::response::IntoResponse;
use axum::response::Response;
use axum::routing::get;
use axum::routing::post;
use life_war_core::Game;
use life_war_core::GameConfig;
use rust_embed::RustEmbed;

use crate::protocol::ApiActionRequest;
use crate::protocol::ApiActionResponse;
use crate::protocol::ApiActionResult;
use crate::protocol::ApiNewGameRequest;
use crate::protocol::ApiState;
use crate::protocol::ApiStateResponse;
use crate::protocol::ApiUndoRequest;
use crate::protocol::HistoryEntry;

#[derive(RustEmbed)]
#[folder = "static/"]
struct Assets;

#[derive(Default)]
struct GameSession {
    game: Game,
    history: Vec<HistoryEntry>,
    revision: u64,
}

impl GameSession {
    fn state(&self) -> ApiState {
        ApiState::from_session(&self.game, self.revision, &self.history)
    }

    fn validate_revision(&self, revision: u64) -> Result<(), String> {
        if revision == self.revision {
            Ok(())
        } else {
            Err(format!("棋局已更新：请求版本 {revision}，当前版本 {}", self.revision))
        }
    }

    fn apply_action(&mut self, request: ApiActionRequest) -> Result<ApiActionResult, String> {
        self.validate_revision(request.revision)?;
        let before = self.game.clone();
        let player = self.game.current_player();
        let result =
            self.game.apply(request.action.to_core()).map_err(|error| error.to_string())?;
        let api_result = ApiActionResult::new(result, &before, &self.game);
        self.history.push(HistoryEntry { before, player, result });
        self.revision += 1;
        Ok(api_result)
    }

    fn new_game(&mut self, request: ApiNewGameRequest) -> Result<(), String> {
        self.validate_revision(request.revision)?;
        let config = GameConfig::new(request.size).map_err(|error| error.to_string())?;
        self.game = Game::new(config);
        self.history.clear();
        self.revision += 1;
        Ok(())
    }

    fn undo(&mut self, request: ApiUndoRequest) -> Result<(), String> {
        self.validate_revision(request.revision)?;
        let entry = self.history.pop().ok_or_else(|| "没有可悔棋的操作".to_owned())?;
        self.game = entry.before;
        self.revision += 1;
        Ok(())
    }
}

struct AppState {
    session: Mutex<GameSession>,
}

type SharedState = Arc<AppState>;

pub fn build_app() -> Router {
    let state = Arc::new(AppState { session: Mutex::new(GameSession::default()) });
    Router::new()
        .route("/", get(index_handler))
        .route("/api/state", get(state_handler))
        .route("/api/action", post(action_handler))
        .route("/api/new", post(new_game_handler))
        .route("/api/undo", post(undo_handler))
        .route("/{*path}", get(asset_handler))
        .with_state(state)
}

async fn index_handler() -> impl IntoResponse {
    serve_asset("index.html")
}

async fn state_handler(State(state): State<SharedState>) -> Json<ApiState> {
    let session = state.session.lock().expect("game session mutex poisoned");
    Json(session.state())
}

async fn action_handler(
    State(state): State<SharedState>,
    Json(request): Json<ApiActionRequest>,
) -> Json<ApiActionResponse> {
    let mut session = state.session.lock().expect("game session mutex poisoned");
    let outcome = session.apply_action(request);
    let (error, result) = match outcome {
        Ok(result) => (None, Some(result)),
        Err(error) => (Some(error), None),
    };
    Json(ApiActionResponse { state: session.state(), error, result })
}

async fn new_game_handler(
    State(state): State<SharedState>,
    Json(request): Json<ApiNewGameRequest>,
) -> Json<ApiStateResponse> {
    let mut session = state.session.lock().expect("game session mutex poisoned");
    let error = session.new_game(request).err();
    Json(ApiStateResponse { state: session.state(), error })
}

async fn undo_handler(
    State(state): State<SharedState>,
    Json(request): Json<ApiUndoRequest>,
) -> Json<ApiStateResponse> {
    let mut session = state.session.lock().expect("game session mutex poisoned");
    let error = session.undo(request).err();
    Json(ApiStateResponse { state: session.state(), error })
}

async fn asset_handler(Path(path): Path<String>) -> impl IntoResponse {
    serve_asset(&path)
}

fn serve_asset(path: &str) -> Response {
    match Assets::get(path) {
        Some(file) => Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, mime_type(path))
            .body(Body::from(file.data.into_owned()))
            .expect("valid asset response"),
        None => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
            .body(Body::from("not found"))
            .expect("valid not-found response"),
    }
}

fn mime_type(path: &str) -> &'static str {
    if path.ends_with(".html") {
        "text/html; charset=utf-8"
    } else if path.ends_with(".css") {
        "text/css; charset=utf-8"
    } else if path.ends_with(".js") {
        "text/javascript; charset=utf-8"
    } else if path.ends_with(".svg") {
        "image/svg+xml"
    } else {
        "application/octet-stream"
    }
}
