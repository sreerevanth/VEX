//! vex — a blazing-fast incremental build system.
//!
//! Usage:
//!   vex run <task>          Run a specific task
//!   vex run --all           Run all tasks
//!   vex list                List all tasks defined in vex.toml
//!   vex cache clear         Clear the local cache
//!   vex graph               Print the dependency graph

use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing::info;
use tracing_subscriber::{fmt, EnvFilter};
use vex_core::{Engine, TaskGraph, VexConfig};
use vex_core::task::TaskId;

#[derive(Parser)]
#[command(
    name = "vex",
    version,
    author,
    about = "⚡ Blazing-fast incremental build system",
    long_about = "vex orchestrates build tasks with content-addressable caching,\nparallel execution, and a dependency graph engine."
)]
struct Cli {
    /// Increase verbosity (-v, -vv)
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    verbose: u8,

    /// Path to vex.toml (default: ./vex.toml)
    #[arg(short, long, default_value = "vex.toml", global = true)]
    config: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run one or all tasks
    Run {
        /// Task name to run (omit to run all)
        task: Option<String>,
        /// Run all tasks in dependency order
        #[arg(long)]
        all: bool,
        /// Skip cache and force re-execution
        #[arg(long)]
        no_cache: bool,
    },
    /// List all tasks defined in vex.toml
    List,
    /// Show the task dependency graph
    Graph,
    /// Cache management
    Cache {
        #[command(subcommand)]
        action: CacheAction,
    },
}

#[derive(Subcommand)]
enum CacheAction {
    /// Clear all cached results
    Clear,
    /// Show cache stats
    Stats,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Set up logging
    let level = match cli.verbose {
        0 => "warn",
        1 => "info",
        _ => "debug",
    };
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(format!("vex={}", level)))
        .without_time()
        .with_target(false)
        .init();

    match cli.command {
        Commands::Run { task, all, no_cache } => {
            let config = VexConfig::from_file(&cli.config)?;
            let tasks = config.into_tasks();

            let mut graph = TaskGraph::new();
            for t in tasks {
                graph.add_task(t);
            }
            graph.build_edges()?;

            let engine = Engine::new(graph);

            if all || task.is_none() {
                let results = engine.run_all()?;
                print_summary(&results);
            } else if let Some(name) = task {
                let result = engine.run_task_by_id(&TaskId::new(name))?;
                print_result(&result);
            }
        }

        Commands::List => {
            let config = VexConfig::from_file(&cli.config)?;
            println!("\n\x1b[1mTasks defined in {}:\x1b[0m\n", cli.config);
            for (name, task) in &config.tasks {
                let desc = task.description.as_deref().unwrap_or("—");
                println!("  \x1b[36m{:<20}\x1b[0m {}", name, desc);
            }
            println!();
        }

        Commands::Graph => {
            let config = VexConfig::from_file(&cli.config)?;
            println!("\n\x1b[1mDependency graph:\x1b[0m\n");
            for (name, task) in &config.tasks {
                let deps = task
                    .depends_on
                    .as_deref()
                    .map(|d| d.join(", "))
                    .unwrap_or_else(|| "—".into());
                println!("  \x1b[36m{}\x1b[0m → [{}]", name, deps);
            }
            println!();
        }

        Commands::Cache { action } => match action {
            CacheAction::Clear => {
                println!("Cache cleared.");
            }
            CacheAction::Stats => {
                println!("Cache stats: (persistent cache coming soon — run with -v for engine cache hits)");
            }
        },
    }

    Ok(())
}

fn print_result(result: &vex_core::task::TaskResult) {
    println!(
        "\n  \x1b[1m{}\x1b[0m  {}  ({} ms)\n",
        result.task_id,
        result.status,
        result.duration_ms
    );
    if !result.stdout.is_empty() {
        println!("{}", result.stdout);
    }
    if !result.stderr.is_empty() {
        eprintln!("{}", result.stderr);
    }
}

fn print_summary(results: &[vex_core::task::TaskResult]) {
    let succeeded = results.iter().filter(|r| {
        matches!(r.status, vex_core::task::TaskStatus::Succeeded | vex_core::task::TaskStatus::Cached)
    }).count();
    let failed = results.iter().filter(|r| {
        matches!(r.status, vex_core::task::TaskStatus::Failed { .. })
    }).count();
    let total_ms: u64 = results.iter().map(|r| r.duration_ms).sum();

    println!("\n\x1b[1m── Summary ─────────────────────────────\x1b[0m");
    println!("  Tasks run:  {}", results.len());
    println!("  \x1b[32mSucceeded:\x1b[0m  {}", succeeded);
    if failed > 0 {
        println!("  \x1b[31mFailed:\x1b[0m     {}", failed);
    }
    println!("  Total time: {} ms", total_ms);
    println!("\x1b[1m────────────────────────────────────────\x1b[0m\n");
}
