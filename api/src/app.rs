//! Router, handlers and middleware.

use std::time::{Duration, Instant};

use axum::Json;
use axum::extract::{Request, State};
use axum::http::{HeaderName, HeaderValue, Method, StatusCode, header};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::{MethodRouter, get};
use serde_json::json;
use sqlx::PgPool;
use tower_http::limit::RequestBodyLimitLayer;
use tracing::{Instrument, info, info_span, warn};
use uuid::Uuid;

use crate::config::{MAX_BODY_BYTES, READY_TIMEOUT, REQUEST_TIMEOUT};
use crate::error::ApiError;

pub const REQUEST_ID_HEADER: HeaderName = HeaderName::from_static("x-request-id");

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
}

/// A served route. The router is built from this list, and the contract test
/// compares the same list with `contracts/openapi.yaml`, so a route cannot be
/// served without being documented.
pub struct RouteSpec {
    pub method: Method,
    pub path: &'static str,
    handler: MethodRouter<AppState>,
}

pub fn routes() -> Vec<RouteSpec> {
    vec![
        RouteSpec {
            method: Method::GET,
            path: "/health",
            handler: get(health),
        },
        RouteSpec {
            method: Method::GET,
            path: "/ready",
            handler: get(ready),
        },
    ]
}

pub fn build_router(state: AppState) -> axum::Router {
    let router = routes()
        .into_iter()
        .fold(axum::Router::new(), |r, spec| {
            r.route(spec.path, spec.handler)
        })
        .fallback(|| async { ApiError::NotFound })
        .method_not_allowed_fallback(|| async { ApiError::MethodNotAllowed })
        .with_state(state);

    with_middleware(router)
}

/// Layers added last run first: request id wraps everything so even timeouts
/// and oversized bodies carry an id. Public so tests can put the same stack in
/// front of a handler that actually reads the body.
pub fn with_middleware<S: Clone + Send + Sync + 'static>(
    router: axum::Router<S>,
) -> axum::Router<S> {
    router
        .layer(RequestBodyLimitLayer::new(MAX_BODY_BYTES))
        .layer(middleware::from_fn(request_timeout))
        .layer(middleware::from_fn(problem_for_bare_413))
        .layer(middleware::from_fn(request_id))
}

/// Liveness: proves the process serves HTTP. Must not touch any dependency,
/// otherwise a database outage would make the orchestrator restart a healthy
/// process.
async fn health() -> Json<serde_json::Value> {
    Json(json!({ "status": "ok" }))
}

/// Readiness: proves the service can do useful work right now.
async fn ready(State(state): State<AppState>) -> Result<Json<serde_json::Value>, ApiError> {
    check_database(&state.pool, READY_TIMEOUT).await?;
    Ok(Json(json!({ "status": "ready" })))
}

async fn check_database(pool: &PgPool, limit: Duration) -> Result<(), ApiError> {
    // Unknown result (timeout) is treated as not ready: fail closed.
    match tokio::time::timeout(limit, sqlx::query("SELECT 1").execute(pool)).await {
        Ok(Ok(_)) => Ok(()),
        Ok(Err(_)) => {
            warn!("readiness check failed: database error");
            Err(ApiError::NotReady)
        }
        Err(_) => {
            warn!("readiness check failed: database timeout");
            Err(ApiError::NotReady)
        }
    }
}

/// A client-supplied id is only trusted if it is short and made of safe
/// characters; anything else is replaced to keep logs and headers clean.
fn is_valid_request_id(value: &str) -> bool {
    (1..=64).contains(&value.len())
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

async fn request_id(req: Request, next: Next) -> Response {
    let id = req
        .headers()
        .get(&REQUEST_ID_HEADER)
        .and_then(|v| v.to_str().ok())
        .filter(|v| is_valid_request_id(v))
        .map(str::to_owned)
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    // Only method and path: never the query string, headers or body.
    let span = info_span!(
        "request",
        request_id = %id,
        method = %req.method(),
        path = %req.uri().path()
    );
    let started = Instant::now();

    let mut response = next.run(req).instrument(span.clone()).await;

    span.in_scope(|| {
        info!(
            status = response.status().as_u16(),
            latency_ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX),
            "request completed"
        );
    });
    if let Ok(value) = HeaderValue::from_str(&id) {
        response.headers_mut().insert(REQUEST_ID_HEADER, value);
    }
    response
}

async fn request_timeout(req: Request, next: Next) -> Response {
    match tokio::time::timeout(REQUEST_TIMEOUT, next.run(req)).await {
        Ok(response) => response,
        Err(_) => ApiError::Timeout.into_response(),
    }
}

/// The body-limit layer and axum's body extractors answer 413 with plain text.
/// Rewrite those so every error a client sees has the same problem+json shape.
async fn problem_for_bare_413(req: Request, next: Next) -> Response {
    let response = next.run(req).await;
    let is_problem = response
        .headers()
        .get(header::CONTENT_TYPE)
        .is_some_and(|v| v.as_bytes().starts_with(b"application/problem+json"));
    if response.status() == StatusCode::PAYLOAD_TOO_LARGE && !is_problem {
        return ApiError::PayloadTooLarge.into_response();
    }
    response
}
