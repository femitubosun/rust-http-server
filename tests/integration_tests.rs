use rust_http_server::handlers::{
    echo_get_handler, echo_post_handler, health_handler, slow_handler,
};
use rust_http_server::server::Server;
use serde_json::json;
use tokio::sync::broadcast;

async fn start_server() -> (u16, broadcast::Sender<()>) {
    let mut app = Server::init();

    app.get("/health", health_handler);
    app.post("/echo", echo_post_handler);
    app.get("/echo", echo_get_handler);
    app.get("/slow", slow_handler);

    let (shutdown_tx, shutdown_rx) = broadcast::channel(1);

    let port = app.listen(0, shutdown_rx).await;

    (port, shutdown_tx)
}

#[tokio::test]
async fn test_health_endpoint() {
    let (port, _shutdown) = start_server().await;
    let client = reqwest::Client::new();

    let res = client
        .get(format!("http://127.0.0.1:{port}/health"))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let body = res.json::<serde_json::Value>().await.unwrap();
    assert_eq!(body, json!({"status": "OK"}));
}
