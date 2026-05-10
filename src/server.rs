use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

use crate::request::{self, Request};

type Handler = fn(&mut TcpStream, &request::Request);

pub struct Server {
    tcp_listener: Option<TcpListener>,
    routes: HashMap<(request::RequestMethod, &'static str), Handler>,
}

impl Server {
    pub fn init() -> Self {
        Server {
            tcp_listener: None,
            routes: HashMap::new(),
        }
    }

    pub fn listen(&mut self, port: &u16) {
        self.tcp_listener = Some(
            TcpListener::bind(format!("127.0.0.1:{port}")).expect("Failed to bind to address"),
        );

        if let Some(ref listener) = self.tcp_listener {
            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => {
                        let routes = self.routes.clone();
                        let _ = std::thread::spawn(move || handle_client(stream, routes));
                    }
                    Err(e) => {
                        eprintln!("Failed to establish connection: {e}")
                    }
                }
            }
        }
    }

    fn add_route(
        &mut self,
        method: request::RequestMethod,
        url: &'static str,
        handler: Handler,
    ) -> &mut Self {
        self.routes.insert((method, url), handler);
        self
    }

    pub fn get(&mut self, url: &'static str, handler: Handler) -> &mut Self {
        self.add_route(request::RequestMethod::GET, url, handler)
    }

    pub fn post(&mut self, url: &'static str, handler: Handler) -> &mut Self {
        self.add_route(request::RequestMethod::POST, url, handler)
    }
}

fn handle_client(
    mut stream: TcpStream,
    routes: HashMap<(request::RequestMethod, &'static str), Handler>,
) {
    let mut buf = [0; 1024];
    let byte_read = stream.read(&mut buf).expect("Failed to read from client!");

    if byte_read == 0 {
        return;
    }

    let req = String::from_utf8_lossy(&buf[..byte_read]);

    match parse_request(req.to_string()) {
        Ok(request) => {
            route_request(&mut stream, &request, &routes);
        }
        Err(error) => {
            let response = format!(
                "HTTP/1.1 400 Bad Request\r\nContent-Length: {}\r\n\r\n{}",
                error.len(),
                error
            );
            stream.write_all(response.as_bytes()).ok();
        }
    }
}

fn parse_request(req: String) -> Result<Request, String> {
    let mut parts = req.split("\r\n\r\n");
    let headers = parts.next().unwrap_or("");
    let body_str = parts.next().unwrap_or("");

    let lines: Vec<&str> = headers.split("\r\n").collect();

    if lines.is_empty() {
        return Err("Empty request".to_string());
    }

    let request_parts: Vec<&str> = lines[0].split_whitespace().collect();
    if request_parts.len() < 2 {
        return Err("Invalid request line".to_string());
    }

    let method = match request::RequestMethod::from_str(request_parts[0]) {
        Some(m) => m,
        None => return Err(format!("Unknown method: {}", request_parts[0])),
    };
    let path = request_parts[1].to_string();

    let mut headers_map = HashMap::new();
    for line in &lines[1..] {
        if let Some((key, value)) = line.split_once(": ") {
            headers_map.insert(key.to_string(), value.to_string());
        }
    }

    let body_bytes = body_str.as_bytes().to_vec();

    let body = if body_bytes.is_empty() {
        request::Body::Empty
    } else {
        request::Body::Raw(body_bytes)
    };

    Ok(Request::new(method, path, headers_map, body))
}

fn route_request(
    stream: &mut TcpStream,
    req: &Request,
    routes: &HashMap<(request::RequestMethod, &'static str), Handler>,
) {
    match routes.get(&(req.method, req.path.as_str())) {
        Some(handler) => handler(stream, req),
        None => {
            let response = "HTTP/1.1 404 Not Found\r\nContent-Length: 9\r\n\r\nNot Found";
            stream.write_all(response.as_bytes()).ok();
        }
    }
}
