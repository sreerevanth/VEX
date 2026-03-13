//! # vex-core
//!
//! The core engine powering the vex build system.
//! Provides the task graph, fingerprinting, execution engine, and config parsing.

pub mod config;
pub mod engine;
pub mod error;
pub mod fingerprint;
pub mod graph;
pub mod task;

pub use config::VexConfig;
pub use engine::Engine;
pub use error::VexError;
pub use graph::TaskGraph;
pub use task::{Task, TaskId, TaskStatus};
