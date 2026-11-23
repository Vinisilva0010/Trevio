#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Contract tests: `contracts/openapi.yaml` must describe exactly the routes
//! the server exposes, and must be internally consistent.

use std::collections::BTreeSet;
use std::path::PathBuf;

use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use serde_norway::Value;
use sqlx::postgres::PgPoolOptions;
use tower::ServiceExt;
use trevio_api::app::{AppState, build_router, routes};

const HTTP_METHODS: [&str; 8] = [
    "get", "put", "post", "delete", "options", "head", "patch", "trace",
];

fn load() -> Value {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../contracts/openapi.yaml");
    let text = std::fs::read_to_string(&path).expect("contracts/openapi.yaml must exist");
    serde_norway::from_str(&text).expect("openapi.yaml must be valid YAML")
}

fn documented(doc: &Value) -> BTreeSet<(String, String)> {
    let mut set = BTreeSet::new();
    let paths = doc["paths"].as_mapping().expect("paths object");
    for (path, item) in paths {
        for (key, _) in item.as_mapping().expect("path item") {
            let key = key.as_str().expect("string key");
            if HTTP_METHODS.contains(&key) {
                set.insert((key.to_uppercase(), path.as_str().unwrap().to_owned()));
            }
        }
    }
    set
}

fn served() -> BTreeSet<(String, String)> {
    routes()
        .into_iter()
        .map(|r| (r.method.to_string(), r.path.to_owned()))
        .collect()
}

#[test]
fn declares_openapi_3_1() {
    let doc = load();
    assert!(doc["openapi"].as_str().unwrap().starts_with("3.1"));
}

#[test]
fn every_served_route_is_documented() {
    let missing: Vec<_> = served().difference(&documented(&load())).cloned().collect();
    assert!(
        missing.is_empty(),
        "served but not in openapi.yaml: {missing:?}"
    );
}

#[test]
fn every_documented_route_is_served() {
    let extra: Vec<_> = documented(&load()).difference(&served()).cloned().collect();
    assert!(
        extra.is_empty(),
        "in openapi.yaml but not served: {extra:?}"
    );
}

#[tokio::test]
async fn documented_routes_do_not_hit_the_fallback() {
    let pool = PgPoolOptions::new()
        .connect_lazy("postgres://u:p@127.0.0.1:1/none")
        .unwrap();
    let app = build_router(AppState { pool });
    for (method, path) in documented(&load()) {
        let req = Request::builder()
            .method(Method::from_bytes(method.as_bytes()).unwrap())
            .uri(&path)
            .body(Body::empty())
            .unwrap();
        let status = app.clone().oneshot(req).await.unwrap().status();
        assert_ne!(status, StatusCode::NOT_FOUND, "{method} {path}");
        assert_ne!(status, StatusCode::METHOD_NOT_ALLOWED, "{method} {path}");
    }
}

fn collect_refs(value: &Value, out: &mut Vec<String>) {
    match value {
        Value::Mapping(m) => {
            for (k, v) in m {
                if k.as_str() == Some("$ref") {
                    if let Some(s) = v.as_str() {
                        out.push(s.to_owned());
                    }
                } else {
                    collect_refs(v, out);
                }
            }
        }
        Value::Sequence(s) => s.iter().for_each(|v| collect_refs(v, out)),
        _ => {}
    }
}

#[test]
fn all_local_refs_resolve() {
    let doc = load();
    let mut refs = Vec::new();
    collect_refs(&doc, &mut refs);
    assert!(!refs.is_empty());
    for r in refs {
        let pointer = r
            .strip_prefix("#/")
            .unwrap_or_else(|| panic!("non-local ref {r}"));
        let mut node = &doc;
        for part in pointer.split('/') {
            node = node.get(part).unwrap_or_else(|| panic!("dangling ref {r}"));
        }
    }
}

#[test]
fn bearer_scheme_is_declared_but_unused() {
    let doc = load();
    assert_eq!(
        doc["components"]["securitySchemes"]["bearerAuth"]["scheme"],
        "bearer"
    );
    for (_, item) in doc["paths"].as_mapping().unwrap() {
        for (_, op) in item.as_mapping().unwrap() {
            if let Some(sec) = op.get("security") {
                assert!(
                    sec.as_sequence().is_some_and(|s| s.is_empty()),
                    "no operation may require auth yet"
                );
            }
        }
    }
}
