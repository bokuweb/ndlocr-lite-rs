# CI

## Scope

- Run the default, non-ONNX Rust checks on Linux, macOS, and Windows.
- Keep the workflow status visible from `README.md`.

## GitHub Actions

- Workflow: `.github/workflows/ci.yml`
- Job: `rust`
- OS matrix:
  - `ubuntu-latest`
  - `macos-latest`
  - `windows-latest`

Each matrix entry runs:

```bash
cargo fmt --check
cargo clippy --no-deps
cargo test
```

ONNX/model-backed checks remain outside the default CI path because they require
runtime binaries and model fixtures.
