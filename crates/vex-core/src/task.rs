use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// A unique identifier for a task within the build graph.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TaskId(pub String);

impl TaskId {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }
}

impl std::fmt::Display for TaskId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Execution status of a task.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    Running,
    Cached,
    Succeeded,
    Failed { exit_code: i32 },
    Skipped,
}

impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskStatus::Pending => write!(f, "pending"),
            TaskStatus::Running => write!(f, "running"),
            TaskStatus::Cached => write!(f, "cached"),
            TaskStatus::Succeeded => write!(f, "succeeded"),
            TaskStatus::Failed { exit_code } => write!(f, "failed (exit {})", exit_code),
            TaskStatus::Skipped => write!(f, "skipped"),
        }
    }
}

/// A single unit of work in the build graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: TaskId,
    pub run: String,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub depends_on: Vec<TaskId>,
    pub env: HashMap<String, String>,
    pub working_dir: Option<String>,
    pub cache: bool,
    pub description: Option<String>,
}

impl Task {
    pub fn new(id: impl Into<String>, run: impl Into<String>) -> Self {
        Self {
            id: TaskId::new(id),
            run: run.into(),
            inputs: Vec::new(),
            outputs: Vec::new(),
            depends_on: Vec::new(),
            env: HashMap::new(),
            working_dir: None,
            cache: true,
            description: None,
        }
    }

    pub fn with_inputs(mut self, inputs: Vec<String>) -> Self {
        self.inputs = inputs;
        self
    }

    pub fn with_outputs(mut self, outputs: Vec<String>) -> Self {
        self.outputs = outputs;
        self
    }

    pub fn depends_on(mut self, deps: Vec<TaskId>) -> Self {
        self.depends_on = deps;
        self
    }

    pub fn with_cache(mut self, cache: bool) -> Self {
        self.cache = cache;
        self
    }
}

/// The result of executing a task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub run_id: Uuid,
    pub task_id: TaskId,
    pub status: TaskStatus,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u64,
    pub fingerprint: Option<String>,
}
