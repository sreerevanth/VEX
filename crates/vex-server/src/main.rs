//! vex-server — remote cache and build orchestration API.
//!
//! Exposes an HTTP API so CI machines and developer workstations
//! can share a warm cache and avoid redundant builds.

use axum::{
    extract::Path,
    http::StatusCode,
    response::Json,
    routing::{delete, get, put},
    Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;
use tracing::info;
use tracing_subscriber::{fmt, EnvFilter};

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    version: &'static str,
}

#[derive(Serialize)]
struct CacheEntry {
    fingerprint: String,
    task_id: String,
    status: String,
    duration_ms: u64,
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        version: env!("CARGO_PKG_VERSION"),
    })
}

async fn get_cache(Path(fingerprint): Path<String>) -> (StatusCode, Json<serde_json::Value>) {
    // Stub — real impl queries CacheStore
    info!("Cache lookup: {}", fingerprint);
    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({ "error": "not found" })),
    )
}

async fn put_cache(
    Path(fingerprint): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> StatusCode {
    info!("Cache store: {}", fingerprint);
    StatusCode::CREATED
}

async fn delete_cache(Path(fingerprint): Path<String>) -> StatusCode {
    info!("Cache invalidate: {}", fingerprint);
    StatusCode::NO_CONTENT
}

async fn list_tasks() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "tasks": [] }))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new("vex_server=debug,tower_http=info"))
        .init();

    let app = Router::new()
        .route("/health", get(health))
        .route("/api/v1/cache/:fingerprint", get(get_cache))
        .route("/api/v1/cache/:fingerprint", put(put_cache))
        .route("/api/v1/cache/:fingerprint", delete(delete_cache))
        .route("/api/v1/tasks", get(list_tasks))
        .layer(TraceLayer::new_for_http());

    let addr: SocketAddr = "0.0.0.0:7878".parse()?;
    info!("vex-server listening on {}", addr);
     let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, app).await?;
        

    Ok(())
}
