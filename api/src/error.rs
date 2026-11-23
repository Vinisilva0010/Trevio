//! The single error type of the HTTP layer.
//!
//! Every failure leaves the server as an RFC 9457 `application/problem+json`
//! body. Bodies are built only from the fixed strings below, so internal
//! details (SQL errors, hosts, stack traces) cannot leak by construction.

use axum::Json;
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("route not found")]
    NotFound,
    #[error("method not allowed")]
    MethodNotAllowed,
    #[error("payload too large")]
    PayloadTooLarge,
    #[error("request timed out")]
    Timeout,
    #[error("service not ready")]
    NotReady,
    #[error("internal error")]
    Internal,
}

/// RFC 9457 problem details. Mirrors `Problem` in `contracts/openapi.yaml`.
#[derive(Debug, Serialize)]
pub struct Problem {
    #[serde(rename = "type")]
    pub type_uri: &'static str,
    pub title: &'static str,
    pub status: u16,
    pub detail: &'static str,
}

impl ApiError {
    fn status(&self) -> StatusCode {
        match self {
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::MethodNotAllowed => StatusCode::METHOD_NOT_ALLOWED,
            Self::PayloadTooLarge => StatusCode::PAYLOAD_TOO_LARGE,
            Self::Timeout => StatusCode::REQUEST_TIMEOUT,
            Self::NotReady => StatusCode::SERVICE_UNAVAILABLE,
            Self::Internal => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn problem(&self) -> Problem {
        let (type_uri, title, detail) = match self {
            Self::NotFound => (
                "urn:trevio:problem:not-found",
                "Not Found",
                "The requested resource does not exist.",
            ),
            Self::MethodNotAllowed => (
                "urn:trevio:problem:method-not-allowed",
                "Method Not Allowed",
                "This method is not supported on this resource.",
            ),
            Self::PayloadTooLarge => (
                "urn:trevio:problem:payload-too-large",
                "Payload Too Large",
                "The request body exceeds the allowed size.",
            ),
            Self::Timeout => (
                "urn:trevio:problem:timeout",
                "Request Timeout",
                "The request took too long to process.",
            ),
            Self::NotReady => (
                "urn:trevio:problem:not-ready",
                "Service Not Ready",
                "A required dependency is unavailable. Try again later.",
            ),
            Self::Internal => (
                "urn:trevio:problem:internal",
                "Internal Server Error",
                "An unexpected error occurred.",
            ),
        };
        Problem {
            type_uri,
            title,
            status: self.status().as_u16(),
            detail,
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = self.status();
        let mut response = (status, Json(self.problem())).into_response();
        response.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/problem+json"),
        );
        response
    }
}
