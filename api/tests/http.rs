#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use axum::Router;
use axum::body::Body;
use axum::body::Bytes;
use axum::http::{Request, StatusCode, header};
use axum::routing::post;
use http_body_util::BodyExt;
use serde_json::Value;
use sqlx::postgres::PgPoolOptions;
use std::time::Duration;
use tower::ServiceExt;
use trevio_api::app::{AppState, build_router, with_middleware};

/// A router whose database can never be reached: port 1 on localhost refuses
/// connections. The pool is lazy, so building it succeeds.
fn router_with_dead_database() -> Router {
    let pool = PgPoolOptions::new()
        .acquire_timeout(Duration::from_millis(500))
        .connect_lazy("postgres://u:p@127.0.0.1:1/none")
        .expect("lazy pool");
    build_router(AppState { pool })
}

async fn send(req: Request<Body>) -> (StatusCode, axum::http::HeaderMap, Vec<u8>) {
    let res = router_with_dead_database().oneshot(req).await.unwrap();
    let (parts, body) = res.into_parts();
    let bytes = body.collect().await.unwrap().to_bytes().to_vec();
    (parts.status, parts.headers, bytes)
}

fn get(path: &str) -> Request<Body> {
    Request::get(path).body(Body::empty()).unwrap()
}

#[tokio::test]
async fn health_is_ok_without_database() {
    let (status, _, body) = send(get("/health")).await;
    assert_eq!(status, StatusCode::OK);
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["status"], "ok");
}

#[tokio::test]
async fn ready_returns_503_problem_when_database_is_unreachable() {
    let (status, headers, body) = send(get("/ready")).await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(headers[header::CONTENT_TYPE], "application/problem+json");
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["status"], 503);
    assert_eq!(json["type"], "urn:trevio:problem:not-ready");
    // Nothing about the host, port or driver may leak.
    let text = String::from_utf8(body).unwrap();
    assert!(!text.contains("127.0.0.1") && !text.contains("sqlx"));
}

#[tokio::test]
async fn response_carries_a_generated_request_id() {
    let (_, headers, _) = send(get("/health")).await;
    let id = headers["x-request-id"].to_str().unwrap();
    assert_eq!(id.len(), 36, "expected a UUID, got {id}");
}

#[tokio::test]
async fn valid_client_request_id_is_echoed() {
    let req = Request::get("/health")
        .header("x-request-id", "client-abc_123")
        .body(Body::empty())
        .unwrap();
    let (_, headers, _) = send(req).await;
    assert_eq!(headers["x-request-id"], "client-abc_123");
}

#[tokio::test]
async fn unsafe_client_request_id_is_replaced() {
    for bad in ["has space", "semi;colon", &"x".repeat(65)] {
        let req = Request::get("/health")
            .header("x-request-id", bad)
            .body(Body::empty())
            .unwrap();
        let (_, headers, _) = send(req).await;
        assert_ne!(headers["x-request-id"], bad);
    }
}

#[tokio::test]
async fn unknown_route_is_a_problem_404() {
    let (status, headers, body) = send(get("/nope")).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(headers[header::CONTENT_TYPE], "application/problem+json");
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["status"], 404);
}

#[tokio::test]
async fn wrong_method_is_a_problem_405() {
    let req = Request::post("/health").body(Body::empty()).unwrap();
    let (status, headers, _) = send(req).await;
    assert_eq!(status, StatusCode::METHOD_NOT_ALLOWED);
    assert_eq!(headers[header::CONTENT_TYPE], "application/problem+json");
}

/// No real route reads a body yet, so the limit is tested on a throwaway
/// route behind the production middleware stack.
#[tokio::test]
async fn oversized_body_is_rejected_as_a_problem_413() {
    let app = with_middleware(Router::new().route(
        "/echo",
        post(|body: Bytes| async move { body.len().to_string() }),
    ));
    let big = vec![b'a'; 64 * 1024 + 1];
    let req = Request::post("/echo").body(Body::from(big)).unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::PAYLOAD_TOO_LARGE);
    assert_eq!(
        res.headers()[header::CONTENT_TYPE],
        "application/problem+json"
    );
}
