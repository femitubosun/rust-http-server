use std::collections::HashMap;

pub enum Body {
    Json(serde_json::Value),
    Raw(Vec<u8>),
    Empty,
}

pub struct Request {
    pub method: RequestMethod,
    pub path: String,
    pub query_params: HashMap<String, String>,
    pub body: Body,
}

impl Request {
    pub fn new(
        method: RequestMethod,
        path: String,
        query_params: HashMap<String, String>,
        body: Body,
    ) -> Self {
        Request {
            method,
            path,
            query_params,
            body,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RequestMethod {
    GET,
    POST,
    PUT,
    DELETE,
    HEAD,
    OPTIONS,
    PATCH,
}

impl RequestMethod {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "GET" => Some(RequestMethod::GET),
            "POST" => Some(RequestMethod::POST),
            "PUT" => Some(RequestMethod::PUT),
            "DELETE" => Some(RequestMethod::DELETE),
            "HEAD" => Some(RequestMethod::HEAD),
            "OPTIONS" => Some(RequestMethod::OPTIONS),
            "PATCH" => Some(RequestMethod::PATCH),
            _ => None,
        }
    }
}
