#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Real-database test. Ignored by default because it needs Postgres.
//!
//! Run it against the local stack from `infra/` (see `infra/README.md`):
//!
//! ```bash
//! TREVIO_TEST_DATABASE_URL='postgres://USER:PASSWORD@127.0.0.1:5432/DBNAME' \
//!   cargo test -p trevio-api --test database -- --ignored
//! ```

use axum::body::Body;
use axum::http::{Request, StatusCode};
use sqlx::postgres::PgPoolOptions;
use tower::ServiceExt;
use trevio_api::app::{AppState, build_router};

#[tokio::test]
#[ignore = "needs a running Postgres; see the module docs"]
async fn ready_is_200_with_a_real_database() {
    let url = std::env::var("TREVIO_TEST_DATABASE_URL")
        .expect("set TREVIO_TEST_DATABASE_URL to run this test");
    let pool = PgPoolOptions::new().connect_lazy(&url).expect("valid url");
    let res = build_router(AppState { pool })
        .oneshot(Request::get("/ready").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}
