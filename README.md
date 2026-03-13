<div align="center">

# ⚡ vex

**A blazing-fast, incremental build system with content-addressable caching.**

[![CI](https://github.com/yourusername/vex/actions/workflows/ci.yml/badge.svg)](https://github.com/yourusername/vex/actions)
[![Crates.io](https://img.shields.io/crates/v/vex-cli.svg)](https://crates.io/crates/vex-cli)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)
[![Rust 1.75+](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)

[Getting Started](#getting-started) · [Configuration](#configuration) · [Architecture](#architecture) · [Contributing](#contributing)

</div>

---

## What is vex?

vex is a task runner and build orchestrator that knows which work to skip.

Instead of timestamping files (like Make), vex hashes the **content** of your inputs using SHA-256. A task only re-runs when its inputs *actually change*. This eliminates entire classes of stale-build bugs and makes CI significantly faster.

```
$ vex run --all

  fmt          [cached]
  lint         [running] → [ok]   (1.2s)
  test         [cached]
  build        [ok]               (4.8s)

── Summary ──────────────────────────
  Tasks run:   4
  Succeeded:   4
  Total time:  6031 ms
─────────────────────────────────────
```

## Features

- **Content-addressable cache** — inputs are fingerprinted by file content, not mtime
- **Parallel execution** — independent tasks run concurrently across CPU cores
- **Dependency graph** — cyclic dependencies are detected at startup, not mid-run
- **Persistent cache** — SQLite-backed store survives restarts; `vex-server` shares it across machines
- **Remote cache API** — ship a `vex-server` instance and every developer shares warm cache
- **Zero config escape hatches** — every field in `vex.toml` is optional

## Getting Started

### Install

```bash
cargo install vex-cli
```

Or build from source:

```bash
git clone https://github.com/yourusername/vex
cd vex
cargo install --path crates/vex-cli
```

### Define your tasks

Create a `vex.toml` in your project root:

```toml
[project]
name    = "my-app"
version = "1.0.0"

[tasks.fmt]
run         = "cargo fmt --check"
description = "Check formatting"
cache       = false

[tasks.lint]
run         = "cargo clippy -- -D warnings"
inputs      = ["src/**/*.rs", "Cargo.toml"]
description = "Lint with clippy"
depends_on  = ["fmt"]

[tasks.test]
run         = "cargo test"
inputs      = ["src/**/*.rs", "tests/**/*.rs"]
description = "Run the test suite"
depends_on  = ["lint"]

[tasks.build]
run         = "cargo build --release"
inputs      = ["src/**/*.rs", "Cargo.toml"]
outputs     = ["target/release/my-app"]
description = "Compile release binary"
depends_on  = ["test"]
```

### Run

```bash
vex run --all          # run all tasks in dependency order
vex run build          # run a specific task
vex list               # list all tasks
vex graph              # print the dependency graph
vex cache clear        # invalidate the local cache
```

## Configuration

### `vex.toml` reference

```toml
[project]
name        = "string"          # required
version     = "string"          # optional
description = "string"          # optional

[tasks.<name>]
run         = "shell command"   # required — executed via sh -c / cmd /C
inputs      = ["glob patterns"] # files that, when changed, invalidate the cache
outputs     = ["paths"]         # files this task produces
depends_on  = ["task-names"]    # tasks that must complete first
env         = { KEY = "value" } # environment variables injected at runtime
working_dir = "path"            # working directory (default: project root)
cache       = true              # set false to always re-run (default: true)
description = "string"          # shown in `vex list`
```

### Environment variable overrides

| Variable        | Default      | Description                      |
|-----------------|--------------|----------------------------------|
| `VEX_CACHE_DIR` | `.vex/cache` | Local cache database location    |
| `VEX_REMOTE`    | —            | URL of a `vex-server` instance   |
| `VEX_JOBS`      | # of CPUs    | Max parallel task count          |
| `RUST_LOG`      | `warn`       | Log verbosity (use `vex=debug`)  |

## Architecture

vex is structured as a Rust workspace with four crates, each with a single responsibility:

```
vex/
├── crates/
│   ├── vex-core      # task graph, fingerprinting, execution engine
│   ├── vex-cache     # SQLite-backed persistent cache (async)
│   ├── vex-cli       # user-facing binary (`vex`)
│   └── vex-server    # remote cache & API server (`vex-server`)
├── examples/         # example vex.toml configurations
└── tests/            # integration tests
```

### How a build works

```
vex.toml
   │
   ▼
VexConfig::from_file()
   │  parse & validate
   ▼
TaskGraph::build_edges()
   │  topological sort, cycle detection (petgraph)
   ▼
Engine::run_all()
   │
   ├── for each task (in dep order):
   │     │
   │     ├── Fingerprint::from_paths(task.inputs)   ← SHA-256 of file contents
   │     │
   │     ├── cache.get(fingerprint)
   │     │     hit  → return cached result  [cached]
   │     │     miss → continue
   │     │
   │     ├── sh -c "task.run"
   │     │
   │     └── cache.put(fingerprint, result)
   │
   └── print summary
```

### Caching model

vex uses a two-level cache:

1. **In-process** (`DashMap`) — keyed by fingerprint, survives within a single `vex` invocation
2. **Persistent** (`SQLite` via `sqlx`) — survives restarts, lives in `.vex/cache/vex.db`
3. **Remote** (`vex-server`) — shared across machines, keyed by the same fingerprints

Because cache keys are derived from file contents (not times), two developers who check out the same commit will always hit the same cache entries.

## Remote Cache Server

Start a server:

```bash
cargo install vex-server
vex-server --port 7878 --cache-dir /var/vex-cache
```

Point clients at it:

```bash
VEX_REMOTE=http://ci-server:7878 vex run --all
```

### API

| Method | Path                           | Description              |
|--------|-------------------------------|--------------------------|
| GET    | `/health`                     | Health check             |
| GET    | `/api/v1/cache/:fingerprint`  | Retrieve a cached result |
| PUT    | `/api/v1/cache/:fingerprint`  | Store a result           |
| DELETE | `/api/v1/cache/:fingerprint`  | Invalidate an entry      |
| GET    | `/api/v1/tasks`               | List known tasks         |

## Comparison

| Feature                   | vex | Make | Turborepo | Bazel |
|---------------------------|-----|------|-----------|-------|
| Content-addressed cache   | ✅  | ❌   | ✅        | ✅    |
| Remote cache              | ✅  | ❌   | ✅        | ✅    |
| Language agnostic         | ✅  | ✅   | ✅        | ✅    |
| Cycle detection           | ✅  | ❌   | ✅        | ✅    |
| Zero installation deps    | ✅  | ✅   | ❌        | ❌    |
| Single binary             | ✅  | ✅   | ❌        | ❌    |
| Config complexity         | Low | Low  | Medium    | High  |

## Contributing

Contributions are welcome. Please read [CONTRIBUTING.md](CONTRIBUTING.md) first.

```bash
# run the full check suite
cargo fmt --all && cargo clippy --all && cargo test --all
```

## License

Licensed under either of:

- MIT License ([LICENSE-MIT](LICENSE-MIT))
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))

at your option.
