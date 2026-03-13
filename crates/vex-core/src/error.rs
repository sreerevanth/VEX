use thiserror::Error;

#[derive(Error, Debug)]
pub enum VexError {
    #[error("task `{0}` not found")]
    TaskNotFound(String),

    #[error("cycle detected in task graph involving task `{0}`")]
    CycleDetected(String),

    #[error("task `{task}` failed with exit code {code}")]
    TaskFailed { task: String, code: i32 },

    #[error("fingerprint mismatch for `{0}`")]
    FingerprintMismatch(String),

    #[error("config error: {0}")]
    Config(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, VexError>;
