# Module 12: Networking & Web

HTTP clients and servers, middleware patterns, REST API design, WebSocket communication, and authentication.

## Lessons

| # | File | Topic |
|---|------|-------|
| 01 | `p01_http_clients.rs` | reqwest: GET/POST, headers, cookies, timeouts, retry, connection pooling |
| 02 | `p02_axum_fundamentals.rs` | Router, handlers, extractors, responses, shared state, error handling |
| 03 | `p03_middleware.rs` | Tower middleware, Layer, Service, request/response transformation, logging |
| 04 | `p04_rest_api_design.rs` | RESTful routes, resource naming, status codes, pagination, filtering |
| 05 | `p05_request_validation.rs` | Input validation, serde validation, custom extractors, error responses |
| 06 | `p06_authentication.rs` | JWT tokens, session management, password hashing, auth middleware |
| 07 | `p07_websocket.rs` | WebSocket upgrade, axum WebSocket, message handling, broadcast |
| 08 | `p08_grpc_basics.rs` | tonic, protobuf, service definition, streaming, interceptors |
| 09 | `p09_rate_limiting.rs` | Token bucket, sliding window, per-user limits, tower-based rate limiter |
| 10 | `p10_protocol_design.rs` | Custom protocols, framing, codec, length-delimited, protocol buffers |

## Running Tests

```bash
cargo test -p networking_web
```
