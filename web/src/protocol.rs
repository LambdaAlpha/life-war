use life_war_core::Action;
use life_war_core::Cell;
use life_war_core::EvolutionStats;
use life_war_core::Game;
use life_war_core::GamePhase;
use life_war_core::GameStatus;
use life_war_core::Player;
use life_war_core::Point;
use life_war_core::TurnResult;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApiPlayer {
    Black,
    White,
}

impl From<Player> for ApiPlayer {
    fn from(player: Player) -> Self {
        match player {
            Player::Black => Self::Black,
            Player::White => Self::White,
        }
    }
}

impl From<ApiPlayer> for Player {
    fn from(player: ApiPlayer) -> Self {
        match player {
            ApiPlayer::Black => Self::Black,
            ApiPlayer::White => Self::White,
        }
    }
}

#[derive(Debug, Copy, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ApiCell {
    Empty,
    Black,
    White,
}

impl From<Cell> for ApiCell {
    fn from(cell: Cell) -> Self {
        match cell {
            Cell::Empty => Self::Empty,
            Cell::Black => Self::Black,
            Cell::White => Self::White,
        }
    }
}

#[derive(Debug, Copy, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ApiPhase {
    Layout,
    Action,
}

impl From<GamePhase> for ApiPhase {
    fn from(phase: GamePhase) -> Self {
        match phase {
            GamePhase::Layout => Self::Layout,
            GamePhase::Action => Self::Action,
        }
    }
}

#[derive(Debug, Copy, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ApiStatus {
    Playing,
    BlackWon,
    WhiteWon,
    Draw,
}

impl From<GameStatus> for ApiStatus {
    fn from(status: GameStatus) -> Self {
        match status {
            GameStatus::Playing => Self::Playing,
            GameStatus::Won(Player::Black) => Self::BlackWon,
            GameStatus::Won(Player::White) => Self::WhiteWon,
            GameStatus::Draw => Self::Draw,
        }
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ApiAction {
    Place { x: u8, y: u8, color: ApiPlayer },
    Pass,
}

impl ApiAction {
    pub fn to_core(self) -> Action {
        match self {
            Self::Place { x, y, color } => {
                Action::Place { point: Point::new(x, y), color: color.into() }
            }
            Self::Pass => Action::Pass,
        }
    }
}

impl From<Action> for ApiAction {
    fn from(action: Action) -> Self {
        match action {
            Action::Place { point, color } => {
                Self::Place { x: point.x, y: point.y, color: color.into() }
            }
            Action::Pass => Self::Pass,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ApiBoard {
    pub size: u8,
    pub cells: Vec<ApiCell>,
}

impl From<&Game> for ApiBoard {
    fn from(game: &Game) -> Self {
        Self { size: game.size(), cells: game.board().iter().copied().map(Into::into).collect() }
    }
}

#[derive(Debug, Copy, Clone, Serialize)]
pub struct ApiEvolutionStats {
    pub born_black: u16,
    pub born_white: u16,
    pub died_black: u16,
    pub died_white: u16,
}

impl From<EvolutionStats> for ApiEvolutionStats {
    fn from(stats: EvolutionStats) -> Self {
        Self {
            born_black: stats.born_black,
            born_white: stats.born_white,
            died_black: stats.died_black,
            died_white: stats.died_white,
        }
    }
}

#[derive(Debug, Copy, Clone, Serialize)]
pub struct ApiCellChange {
    pub x: u8,
    pub y: u8,
    pub before: ApiCell,
    pub after: ApiCell,
}

#[derive(Debug, Clone, Serialize)]
pub struct ApiHistoryEntry {
    pub number: usize,
    pub player: ApiPlayer,
    pub action: ApiAction,
    pub phase: ApiPhase,
    pub generation: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct ApiState {
    pub revision: u64,
    pub board: ApiBoard,
    pub phase: ApiPhase,
    pub current_player: ApiPlayer,
    pub consecutive_passes: u8,
    pub generation: u32,
    pub black_count: u16,
    pub white_count: u16,
    pub status: ApiStatus,
    pub can_undo: bool,
    pub history: Vec<ApiHistoryEntry>,
}

impl ApiState {
    pub(crate) fn from_session(game: &Game, revision: u64, history: &[HistoryEntry]) -> Self {
        let (black_count, white_count) = game.counts();
        let can_undo = !history.is_empty();
        let history: Vec<ApiHistoryEntry> = history
            .iter()
            .enumerate()
            .map(|(index, entry)| ApiHistoryEntry {
                number: index + 1,
                player: entry.player.into(),
                action: entry.result.action.into(),
                phase: entry.result.phase_before.into(),
                generation: entry.result.generation,
            })
            .collect();
        Self {
            revision,
            board: game.into(),
            phase: game.phase().into(),
            current_player: game.current_player().into(),
            consecutive_passes: game.consecutive_passes(),
            generation: game.generation(),
            black_count,
            white_count,
            status: game.status().into(),
            can_undo,
            history,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ApiActionResult {
    pub phase_before: ApiPhase,
    pub phase_after: ApiPhase,
    pub generation: u32,
    pub evolution: Option<ApiEvolutionStats>,
    pub changes: Vec<ApiCellChange>,
}

impl ApiActionResult {
    pub(crate) fn new(result: TurnResult, before: &Game, after: &Game) -> Self {
        let size = usize::from(after.size());
        let changes = before
            .board()
            .iter()
            .copied()
            .zip(after.board().iter().copied())
            .enumerate()
            .filter_map(|(index, (old_cell, new_cell))| {
                (old_cell != new_cell).then_some(ApiCellChange {
                    x: (index % size) as u8,
                    y: (index / size) as u8,
                    before: old_cell.into(),
                    after: new_cell.into(),
                })
            })
            .collect();
        Self {
            phase_before: result.phase_before.into(),
            phase_after: result.phase_after.into(),
            generation: result.generation,
            evolution: result.evolution.map(Into::into),
            changes,
        }
    }
}

#[derive(Debug, Copy, Clone, Deserialize)]
pub struct ApiActionRequest {
    pub revision: u64,
    pub action: ApiAction,
}

#[derive(Debug, Copy, Clone, Deserialize)]
pub struct ApiNewGameRequest {
    pub revision: u64,
    pub size: u8,
}

#[derive(Debug, Copy, Clone, Deserialize)]
pub struct ApiUndoRequest {
    pub revision: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ApiActionResponse {
    pub state: ApiState,
    pub error: Option<String>,
    pub result: Option<ApiActionResult>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ApiStateResponse {
    pub state: ApiState,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct HistoryEntry {
    pub(crate) before: Game,
    pub(crate) player: Player,
    pub(crate) result: TurnResult,
}
