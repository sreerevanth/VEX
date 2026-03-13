//! Directed acyclic task graph with topological scheduling.

use petgraph::algo::toposort;
use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::HashMap;
use tracing::debug;

use crate::error::{Result, VexError};
use crate::task::{Task, TaskId};

/// The build graph: tasks as nodes, dependencies as directed edges.
pub struct TaskGraph {
    graph: DiGraph<Task, ()>,
    index_map: HashMap<TaskId, NodeIndex>,
}

impl TaskGraph {
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            index_map: HashMap::new(),
        }
    }

    /// Add a task to the graph.
    pub fn add_task(&mut self, task: Task) {
        let id = task.id.clone();
        let idx = self.graph.add_node(task);
        self.index_map.insert(id, idx);
    }

    /// Wire up dependency edges between tasks.
    pub fn build_edges(&mut self) -> Result<()> {
        let tasks: Vec<Task> = self
            .graph
            .node_indices()
            .map(|i| self.graph[i].clone())
            .collect();

        for task in &tasks {
            let from = self.index_map[&task.id];
            for dep in &task.depends_on {
                let to = self.index_map.get(dep).ok_or_else(|| {
                    VexError::TaskNotFound(dep.to_string())
                })?;
                self.graph.add_edge(*to, from, ());
            }
        }
        Ok(())
    }

    /// Return tasks in topological order (dependencies first).
    pub fn execution_order(&self) -> Result<Vec<&Task>> {
        toposort(&self.graph, None)
            .map_err(|cycle| {
                let task = &self.graph[cycle.node_id()];
                VexError::CycleDetected(task.id.to_string())
            })
            .map(|order| {
                order.into_iter().map(|i| &self.graph[i]).collect()
            })
    }

    /// Find a task by ID.
    pub fn get_task(&self, id: &TaskId) -> Option<&Task> {
        self.index_map.get(id).map(|&i| &self.graph[i])
    }

    pub fn task_count(&self) -> usize {
        self.graph.node_count()
    }

    /// Return all tasks in the graph.
    pub fn all_tasks(&self) -> Vec<&Task> {
        self.graph.node_indices().map(|i| &self.graph[i]).collect()
    }
}

impl Default for TaskGraph {
    fn default() -> Self {
        Self::new()
    }
}
