use std::io::Write;
use std::net::TcpStream;

use serde_json::from_slice;

use crate::request::{Body, Request};

pub fn parse_json_body(req: &Request) -> Result<serde_json::Value, String> {
    match &req.body {
        Body::Raw(bytes) => from_slice(bytes).map_err(|e| format!("Invalid JSON: {e}")),
        Body::Json(val) => Ok(val.clone()),
        Body::Empty => Err("No body provided".to_string()),
    }
}

pub fn json_response(stream: &mut TcpStream, status: u16, body: &serde_json::Value) {
    let bytes = body.to_string();
    let response = format!(
        "HTTP/1.1 {} {}\r\nContent-Length: {}\r\nContent-Type: application/json\r\n\r\n{}",
        status,
        match status {
            200 => "OK",
            201 => "Created",
            400 => "Bad Request",
            404 => "Not Found",
            500 => "Internal Server Error",
            _ => "Unknown",
        },
        bytes.len(),
        bytes
    );
    stream.write_all(response.as_bytes()).ok();
}

pub fn error_response(stream: &mut TcpStream, status: Option<u16>, e: impl std::fmt::Display) {
    let status = status.unwrap_or(500);
    let bytes = e.to_string();
    let response = format!(
        "HTTP/1.1 {} {}\r\nContent-Length: {}\r\nContent-Type: application/json\r\n\r\n{}",
        status,
        match status {
            400 => "Bad Request",
            404 => "Not Found",
            500 => "Internal Server Error",
            _ => "Error",
        },
        bytes.len(),
        bytes
    );
    stream.write_all(response.as_bytes()).ok();
}
