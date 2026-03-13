//! # vex-cache
//!
//! Persistent, content-addressable cache backed by SQLite.
//! Stores task results keyed by input fingerprints so builds
//! remain fast across restarts and CI runs.

pub mod store;
pub use store::CacheStore;

use async_trait::async_trait;
use vex_core::task::TaskResult;

#[async_trait]
pub trait Cache: Send + Sync {
    async fn get(&self, fingerprint: &str) -> anyhow::Result<Option<TaskResult>>;
    async fn put(&self, fingerprint: &str, result: &TaskResult) -> anyhow::Result<()>;
    async fn invalidate(&self, fingerprint: &str) -> anyhow::Result<()>;
    async fn clear(&self) -> anyhow::Result<()>;
}
