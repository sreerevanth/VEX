# Changelog

All notable changes to this project will be documented here.

Format follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [Unreleased]

### Added
- Initial workspace scaffold with `vex-core`, `vex-cache`, `vex-cli`, `vex-server`
- Content-addressable fingerprinting via SHA-256
- Topological task scheduling with cycle detection (petgraph)
- In-process DashMap cache
- SQLite-backed persistent cache (vex-cache)
- Axum-based remote cache API (vex-server)
- `vex run`, `vex list`, `vex graph`, `vex cache clear` CLI commands
- GitHub Actions CI (fmt, clippy, test on ubuntu/macos/windows)
