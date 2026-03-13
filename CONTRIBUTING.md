# Contributing to vex

Thank you for your interest in contributing!

## Development setup

```bash
git clone https://github.com/yourusername/vex
cd vex
cargo build --all
cargo test --all
```

## Commit style

Use [Conventional Commits](https://www.conventionalcommits.org/):

```
feat(engine): add parallel task execution
fix(cache): handle corrupted SQLite entries
docs(readme): update remote cache section
```

## Pull requests

- One feature or fix per PR
- Add tests for new behaviour
- Run `cargo fmt --all && cargo clippy --all` before pushing
- Update `CHANGELOG.md` if relevant

## Reporting bugs

Open an issue with a minimal `vex.toml` that reproduces the problem.
