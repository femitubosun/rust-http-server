use crate::request::Request;
use crate::server::BoxedFut;
use crate::utils;
use tokio::net::TcpStream;

pub fn health_handler<'a>(stream: &'a mut TcpStream, _req: &'a Request) -> BoxedFut<'a> {
    Box::pin(async move {
        let body = serde_json::json!({"status": "OK"});
        utils::json_response(stream, 200, &body).await;
    })
}

pub fn echo_post_handler<'a>(stream: &'a mut TcpStream, req: &'a Request) -> BoxedFut<'a> {
    Box::pin(async move {
        match utils::parse_json_body(req) {
            Ok(json) => utils::json_response(stream, 200, &json).await,
            Err(e) => utils::error_response(stream, Some(500), e).await,
        }
    })
}

pub fn echo_get_handler<'a>(stream: &'a mut TcpStream, req: &'a Request) -> BoxedFut<'a> {
    Box::pin(async move {
        let value = serde_json::to_value(&req.query_params).unwrap();
        utils::json_response(stream, 200, &value).await;
    })
}

pub fn slow_handler<'a>(stream: &'a mut TcpStream, _req: &'a Request) -> BoxedFut<'a> {
    Box::pin(async move {
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        let body = serde_json::json!({"status": "slow"});
        utils::json_response(stream, 200, &body).await;
    })
}
