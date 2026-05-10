use std::io::Write;
use std::net::TcpStream;

use serde_json::json;

mod request;
mod server;

fn main() {
    const PORT: u16 = 3000;
    let mut app = server::Server::init();

    app.get("/health", health_handler);
    app.post("/post", post_handler);

    app.listen(&PORT);

    println!("Server listening on port: {PORT}")
}
fn health_handler(stream: &mut TcpStream, req: &request::Request) {
    let response = "HTTP/1.1 200 OK\r\nContent-Length: 18\r\n\r\nHello, from health";
    stream
        .write(response.as_bytes())
        .expect("Failed to write to client!");
}

fn post_handler(stream: &mut TcpStream, req: &request::Request) {
    match req.body.parse_json() {
        Ok(json) => {
            let response_body = json.to_string();
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                response_body.len(),
                response_body
            );
            stream.write_all(response.as_bytes()).ok();
        }
        Err(e) => {
            let response = format!(
                "HTTP/1.1 400 Bad Request\r\nContent-Length: {}\r\n\r\n{}",
                e.len(),
                e
            );
            stream.write_all(response.as_bytes()).ok();
        }
    }
}
