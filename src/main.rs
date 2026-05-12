use tokio::{net::TcpStream, sync::broadcast};

use serde_json::json;

use crate::{server::BoxedFut, utils::parse_json_body};

mod request;
mod server;
mod utils;

#[tokio::main]
async fn main() {
    const PORT: u16 = 3000;
    let mut app = server::Server::init();

    app.get("/health", health_handler);
    app.post("/echo", echo_post_handler);
    app.get("/echo", echo_get_handler);
    app.get("/slow", slow_handler);

    let (shutdown_tx, shutdown_rx) = broadcast::channel(1);

    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        let _ = shutdown_tx.send(());
    });

    app.listen(PORT, shutdown_rx).await;

    println!("Server listening on port: {PORT}")
}
fn health_handler<'a>(stream: &'a mut TcpStream, _req: &'a request::Request) -> BoxedFut<'a> {
    Box::pin(async move {
        utils::json_response(stream, 200, &json!({"status": "OK"})).await;
    })
}

fn echo_post_handler<'a>(stream: &'a mut TcpStream, req: &'a request::Request) -> BoxedFut<'a> {
    Box::pin(async move {
        match parse_json_body(req) {
            Ok(json) => utils::json_response(stream, 200, &json).await,
            Err(e) => utils::error_response(stream, Some(500), e).await,
        }
    })
}

fn echo_get_handler<'a>(stream: &'a mut TcpStream, req: &'a request::Request) -> BoxedFut<'a> {
    Box::pin(async move {
        utils::json_response(
            stream,
            200,
            &serde_json::to_value(&req.query_params).unwrap(),
        )
        .await
    })
}

fn slow_handler<'a>(stream: &'a mut TcpStream, _req: &'a request::Request) -> BoxedFut<'a> {
    Box::pin(async move {
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        utils::json_response(stream, 200, &json!({"status": "slow"})).await;
    })
}
