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
    let (port, _shutdown_tx) = start_server().await;
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

#[tokio::test]
async fn test_get_echo_endpoint() {
    let (port, _shutdown_tx) = start_server().await;
    let client = reqwest::Client::new();

    let body = json!({"name": "John Doe", "age": "12", "query":"search term"});

    let res = client
        .get(format!(
            "http://127.0.0.1:{port}/echo?name=John%20Doe&age=12&query=search%20term"
        ))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let resp_body = res.json::<serde_json::Value>().await.unwrap();
    assert_eq!(&body, &resp_body);
}

#[tokio::test]
async fn test_post_echo_endpoint() {
    let (port, _shutdown_tx) = start_server().await;
    let client = reqwest::Client::new();

    let body = json!({"name": "echo test"});

    let res = client
        .post(format!("http://127.0.0.1:{port}/echo"))
        .json(&body)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let resp_body = res.json::<serde_json::Value>().await.unwrap();
    assert_eq!(&body, &resp_body);
}

#[tokio::test]
async fn test_slow_endpoint() {
    let (port, _shutdown_tx) = start_server().await;
    let client = reqwest::Client::new();

    let start = std::time::Instant::now();

    let mut handles = vec![];
    for _ in 0..10 {
        let client = client.clone();
        let url = format!("http://127.0.0.1:{port}/slow");
        handles.push(tokio::spawn(async move {
            let res = client.get(&url).send().await.unwrap();
            assert_eq!(res.status(), 200);
            let body = res.json::<serde_json::Value>().await.unwrap();
            assert_eq!(body, json!({"status": "slow"}));
        }));
    }

    for handle in handles {
        handle.await.unwrap();
    }

    let elapsed = start.elapsed();
    assert!(
        elapsed < std::time::Duration::from_secs(4),
        "10 concurrent slow requests took too long: {:?}",
        elapsed
    );
}
