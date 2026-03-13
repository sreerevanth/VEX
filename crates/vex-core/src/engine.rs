//! The execution engine — runs tasks in dependency order,
//! checks the cache, fingerprints inputs, and records results.

use std::process::Command;
use std::time::Instant;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::error::{Result, VexError};
use crate::fingerprint::Fingerprint;
use crate::graph::TaskGraph;
use crate::task::{Task, TaskId, TaskResult, TaskStatus};

pub struct Engine {
    graph: TaskGraph,
    /// Simple in-memory cache: fingerprint -> TaskResult
    cache: dashmap::DashMap<String, TaskResult>,
}

impl Engine {
    pub fn new(graph: TaskGraph) -> Self {
        Self {
            graph,
            cache: dashmap::DashMap::new(),
        }
    }

    /// Run all tasks in topological order.
    pub fn run_all(&self) -> Result<Vec<TaskResult>> {
        let order = self.graph.execution_order()?;
        info!("Executing {} tasks", order.len());
        let mut results = Vec::new();
        for task in order {
            let result = self.run_task(task)?;
            results.push(result);
        }
        Ok(results)
    }

    /// Run a single named task (and its deps).
    pub fn run_task_by_id(&self, id: &TaskId) -> Result<TaskResult> {
        let task = self
            .graph
            .get_task(id)
            .ok_or_else(|| VexError::TaskNotFound(id.to_string()))?;
        self.run_task(task)
    }

    fn run_task(&self, task: &Task) -> Result<TaskResult> {
        let run_id = Uuid::new_v4();
        let start = Instant::now();

        // Fingerprint inputs
        let fingerprint = if !task.inputs.is_empty() {
            Some(Fingerprint::from_paths(&task.inputs)?)
        } else {
            None
        };

        let fp_key = fingerprint.as_ref().map(|f| f.as_str().to_string());

        // Cache hit
        if task.cache {
            if let Some(key) = &fp_key {
                if let Some(cached) = self.cache.get(key) {
                    info!("  {} \x1b[36m[cached]\x1b[0m", task.id);
                    return Ok(TaskResult {
                        run_id,
                        task_id: task.id.clone(),
                        status: TaskStatus::Cached,
                        stdout: cached.stdout.clone(),
                        stderr: cached.stderr.clone(),
                        duration_ms: 0,
                        fingerprint: fp_key,
                    });
                }
            }
        }

        info!("  {} \x1b[33m[running]\x1b[0m", task.id);

        // Execute
        let mut cmd = if cfg!(target_os = "windows") {
            let mut c = Command::new("cmd");
            c.args(["/C", &task.run]);
            c
        } else {
            let mut c = Command::new("sh");
            c.args(["-c", &task.run]);
            c
        };

        if let Some(dir) = &task.working_dir {
            cmd.current_dir(dir);
        }

        for (k, v) in &task.env {
            cmd.env(k, v);
        }

        let output = cmd.output().map_err(|e| {
            VexError::TaskFailed {
                task: task.id.to_string(),
                code: -1,
            }
        })?;

        let duration_ms = start.elapsed().as_millis() as u64;
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        let status = if output.status.success() {
            info!("  {} \x1b[32m[ok]\x1b[0m ({} ms)", task.id, duration_ms);
            TaskStatus::Succeeded
        } else {
            let code = output.status.code().unwrap_or(-1);
            warn!("  {} \x1b[31m[failed]\x1b[0m (exit {})", task.id, code);
            TaskStatus::Failed { exit_code: code }
        };

        let result = TaskResult {
            run_id,
            task_id: task.id.clone(),
            status: status.clone(),
            stdout,
            stderr,
            duration_ms,
            fingerprint: fp_key.clone(),
        };

        // Store in cache
        if task.cache && status == TaskStatus::Succeeded {
            if let Some(key) = fp_key {
                self.cache.insert(key, result.clone());
            }
        }

        Ok(result)
    }
}
