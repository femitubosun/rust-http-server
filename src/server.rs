use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

use crate::request::{self, Request};
use crate::utils;

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
                        // needs to be replaces with tokio::spawn
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
    let raw_path = request_parts[1].to_string();
    let (path, query_params) = parse_qs(raw_path);
    let headers = parse_headers(&lines);
    let body = parse_body(body_str.as_bytes().to_vec(), &headers)?;

    Ok(Request::new(method, path, query_params, body))
}

fn route_request(
    stream: &mut TcpStream,
    req: &Request,
    routes: &HashMap<(request::RequestMethod, &'static str), Handler>,
) {
    match routes.get(&(req.method, req.path.as_str())) {
        Some(handler) => handler(stream, req),
        None => utils::error_response(stream, Some(404), "Not Found"),
    }
}

fn percent_decode(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '%' {
            let hex = format!(
                "{}{}",
                chars.next().unwrap_or('0'),
                chars.next().unwrap_or('0')
            );
            if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                result.push(byte as char);
            }
        } else if c == '+' {
            result.push(' '); // + means space in query strings
        } else {
            result.push(c);
        }
    }
    result
}

fn parse_headers(lines: &Vec<&str>) -> HashMap<String, String> {
    let mut headers = HashMap::new();
    for line in &lines[1..] {
        if let Some((key, value)) = line.split_once(": ") {
            headers.insert(key.to_string(), value.to_string());
        }
    }

    headers
}

fn parse_qs(path: String) -> (String, HashMap<String, String>) {
    match path.split_once('?') {
        Some((path_part, query_part)) => {
            let mut params = HashMap::new();
            for pair in query_part.split('&') {
                if let Some((key, value)) = pair.split_once('=') {
                    params.insert(percent_decode(key), percent_decode(value));
                }
            }
            (path_part.to_string(), params)
        }
        None => (path, HashMap::new()),
    }
}

fn parse_body(
    body_bytes: Vec<u8>,
    headers: &HashMap<String, String>,
) -> Result<request::Body, String> {
    if body_bytes.is_empty() {
        return Ok(request::Body::Empty);
    }

    if headers
        .get("Content-Type")
        .map(|ct| ct.contains("application/json"))
        .unwrap_or(false)
    {
        let json =
            serde_json::from_slice(&body_bytes).map_err(|e| format!("Invalid JSON: {}", e))?;
        return Ok(request::Body::Json(json));
    }

    Ok(request::Body::Raw(body_bytes))
}
