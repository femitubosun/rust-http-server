use rust_http_server::{
    self,
    handlers::{echo_get_handler, echo_post_handler, health_handler, slow_handler},
};
use tokio::sync::broadcast;

use crate::rust_http_server::server;

#[tokio::main]
async fn main() {
    const PORT: u16 = 3000;
    let mut app = server::Server::init();

    app.get("/health", health_handler);
    app.post("/echo", echo_post_handler);
    app.get("/echo", echo_get_handler);
    app.get("/slow", slow_handler);

    let (shutdown_tx, shutdown_rx) = broadcast::channel(1);

    app.listen(PORT, shutdown_rx).await;

    tokio::signal::ctrl_c().await.ok();
    let _ = shutdown_tx.send(());
}
