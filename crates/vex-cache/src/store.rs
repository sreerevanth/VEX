//! SQLite-backed persistent cache store.

use async_trait::async_trait;
use sqlx::{SqlitePool, Row};
use tracing::{debug, info};
use vex_core::task::TaskResult;

use crate::Cache;

pub struct CacheStore {
    pool: SqlitePool,
}

impl CacheStore {
    /// Open (or create) the SQLite cache database.
    pub async fn open(path: &str) -> anyhow::Result<Self> {
        let pool = SqlitePool::connect(&format!("sqlite://{}?mode=rwc", path)).await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS task_cache (
                fingerprint TEXT PRIMARY KEY,
                task_id     TEXT NOT NULL,
                status      TEXT NOT NULL,
                stdout      TEXT NOT NULL,
                stderr      TEXT NOT NULL,
                duration_ms INTEGER NOT NULL,
                created_at  TEXT NOT NULL DEFAULT (datetime('now'))
            )
            "#,
        )
        .execute(&pool)
        .await?;

        info!("Cache store opened at {}", path);
        Ok(Self { pool })
    }
}

#[async_trait]
impl Cache for CacheStore {
    async fn get(&self, fingerprint: &str) -> anyhow::Result<Option<TaskResult>> {
        let row = sqlx::query(
            "SELECT task_id, status, stdout, stderr, duration_ms FROM task_cache WHERE fingerprint = ?",
        )
        .bind(fingerprint)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(r) => {
                debug!("Cache hit for {}", fingerprint);
                // Reconstruct minimal TaskResult from DB
                Ok(Some(TaskResult {
                    run_id: uuid::Uuid::new_v4(),
                    task_id: vex_core::task::TaskId::new(r.get::<String, _>("task_id")),
                    status: vex_core::task::TaskStatus::Cached,
                    stdout: r.get("stdout"),
                    stderr: r.get("stderr"),
                    duration_ms: r.get::<i64, _>("duration_ms") as u64,
                    fingerprint: Some(fingerprint.to_string()),
                }))
            }
            None => {
                debug!("Cache miss for {}", fingerprint);
                Ok(None)
            }
        }
    }

    async fn put(&self, fingerprint: &str, result: &TaskResult) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO task_cache
                (fingerprint, task_id, status, stdout, stderr, duration_ms)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(fingerprint)
        .bind(result.task_id.0.as_str())
        .bind(format!("{}", result.status))
        .bind(&result.stdout)
        .bind(&result.stderr)
        .bind(result.duration_ms as i64)
        .execute(&self.pool)
        .await?;

        debug!("Cached result for {}", fingerprint);
        Ok(())
    }

    async fn invalidate(&self, fingerprint: &str) -> anyhow::Result<()> {
        sqlx::query("DELETE FROM task_cache WHERE fingerprint = ?")
            .bind(fingerprint)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn clear(&self) -> anyhow::Result<()> {
        sqlx::query("DELETE FROM task_cache")
            .execute(&self.pool)
            .await?;
        info!("Cache cleared");
        Ok(())
    }
}
