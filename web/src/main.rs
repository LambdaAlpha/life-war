mod protocol;
mod server;

#[tokio::main]
async fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let port = arguments.iter().find_map(|value| value.parse().ok()).unwrap_or(0);
    let open_browser = !arguments.iter().any(|value| value == "--no-open");
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", port))
        .await
        .expect("failed to bind local listener");
    let address = listener.local_addr().expect("failed to read local address");
    let url = format!("http://{address}");

    println!("Life War demo: {url}");
    if open_browser {
        let _ = webbrowser::open(&url);
    }
    axum::serve(listener, server::build_app()).await.expect("web server failed");
}
