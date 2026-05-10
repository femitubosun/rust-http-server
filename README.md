# HTTP Server Project Specification

## Overview
Build a bare HTTP/1.1 server using only tokio for async I/O. No axum, no hyper, no warp. The server handles the TCP stream directly.

## Core Capabilities

### 1. Accept TCP Connections
- Use `TcpListener::bind` to accept incoming connections
- Spawn a task per connection with `tokio::spawn`

### 2. Parse Raw HTTP Requests
- Read bytes off the stream
- Parse the request line (`GET /path HTTP/1.1`)
- Parse headers
- Handle the body for POST requests
- No external HTTP parsing library — the parser must be hand-written

### 3. Route Requests
- Implement a simple router that maps `(method, path)` to a handler function

### 4. Send Valid HTTP Responses
- Format status line
- Include headers: `Content-Type`, `Content-Length`, `Connection`
- Include body
- Must be correctly formatted so real browsers or curl accept the response

### 5. Handle Concurrent Connections
- Support multiple clients simultaneously without blocking
- Verify under load

### 6. Graceful Shutdown
- Handle Ctrl+C via `tokio::signal`
- Stop accepting new connections
- Allow in-flight requests to finish

## Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/health` | Returns `200 OK` with `{"status": "ok"}` |
| GET | `/echo?msg=hello` | Returns `200 OK` with the query parameter reflected |
| POST | `/echo` | Returns `200 OK` with the request body reflected |
| GET | `/slow` | Waits 2 seconds, then responds (concurrency test) |
| GET | `*` | Returns `404 Not Found` for any other path |

### Concurrency Verification
The `/slow` endpoint must handle 10 concurrent requests and complete all in ~2 seconds, not 20. This verifies the async model works.

## Challenges

### Partial Reads
TCP does not guarantee a full HTTP request in a single `read()` call. The server must buffer and loop until `\r\n\r\n` (end of headers) is found.

### Ownership in Async
Passing data into `tokio::spawn` requires `'static + Send`. This requires understanding `Arc` and its purpose.

### HTTP Spec Edge Cases
- `Content-Length` must match actual body length or clients hang
- `Connection: close` vs `keep-alive` must be handled correctly

## Learning Outcomes

This project covers:
- Async Rust
- TCP I/O
- Manual parsing
- Ownership across thread boundaries
- HTTP mechanics that frameworks abstract away

## Implementation Approach

1. Build a single-threaded synchronous version first using `std::net::TcpListener`
2. Port to tokio after the synchronous version works
