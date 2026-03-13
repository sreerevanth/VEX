//! vex.toml config parser.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use crate::error::{Result, VexError};
use crate::task::{Task, TaskId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VexConfig {
    pub project: ProjectConfig,
    pub tasks: HashMap<String, TaskConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub name: String,
    pub version: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskConfig {
    pub run: String,
    pub inputs: Option<Vec<String>>,
    pub outputs: Option<Vec<String>>,
    pub depends_on: Option<Vec<String>>,
    pub env: Option<HashMap<String, String>>,
    pub cache: Option<bool>,
    pub description: Option<String>,
    pub working_dir: Option<String>,
}

impl VexConfig {
    /// Load and parse a `vex.toml` file.
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let content = std::fs::read_to_string(path.as_ref())
            .map_err(|_| VexError::Config("vex.toml not found".into()))?;
        toml::from_str(&content)
            .map_err(|e| VexError::Config(e.to_string()))
    }

    /// Convert config tasks into domain Task objects.
    pub fn into_tasks(self) -> Vec<Task> {
        self.tasks
            .into_iter()
            .map(|(name, cfg)| {
                let mut task = Task::new(name, cfg.run);
                if let Some(inputs) = cfg.inputs {
                    task.inputs = inputs;
                }
                if let Some(outputs) = cfg.outputs {
                    task.outputs = outputs;
                }
                if let Some(deps) = cfg.depends_on {
                    task.depends_on = deps.into_iter().map(TaskId::new).collect();
                }
                if let Some(env) = cfg.env {
                    task.env = env;
                }
                if let Some(cache) = cfg.cache {
                    task.cache = cache;
                }
                task.description = cfg.description;
                task.working_dir = cfg.working_dir;
                task
            })
            .collect()
    }
}
