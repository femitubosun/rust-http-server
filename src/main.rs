use std::net::TcpStream;

use serde_json::json;

use crate::utils::parse_json_body;

mod request;
mod server;
mod utils;

fn main() {
    const PORT: u16 = 3000;
    let mut app = server::Server::init();

    app.get("/health", health_handler);
    app.post("/echo", echo_post_handler);
    app.get("/echo", echo_get_handler);

    app.listen(&PORT);

    println!("Server listening on port: {PORT}")
}
fn health_handler(stream: &mut TcpStream, _req: &request::Request) {
    utils::json_response(stream, 200, &json!({"status": "OK"}));
}

fn echo_post_handler(stream: &mut TcpStream, req: &request::Request) {
    match parse_json_body(req) {
        Ok(json) => utils::json_response(stream, 200, &json),
        Err(e) => utils::error_response(stream, Some(500), e),
    }
}

fn echo_get_handler(stream: &mut TcpStream, req: &request::Request) {
    utils::json_response(
        stream,
        200,
        &serde_json::to_value(&req.query_params).unwrap(),
    );
}
