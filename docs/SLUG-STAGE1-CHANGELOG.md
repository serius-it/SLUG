# SLUG Stage 1 Changelog

## 2026-10-09 — Repository audit and identity foundation

### Changes

- Added `docs/SLUG-STAGE1-AUDIT.md` with the architecture and source-level
  findings for network isolation, monetary units/supply, genesis, covenants,
  package/API identity, compatibility surfaces, unresolved decisions, and
  recommended sequencing.
- Added `docs/SLUG-MIGRATION-MATRIX.md` with status-coded work items, risks,
  prerequisites, and explicit blockers.
- Expanded the root `README.md` with the approved SLUG identity and a warning
  that the code is not yet a usable SLUG node.
- Changed inherited Cargo repository metadata in `Cargo.toml` from the
  upstream URL to the configured repository remote,
  `https://github.com/serius-it/SLUG`. Upstream author attribution remains.
- Did not change crate/binary names, network identifiers, seeds, ports,
  addresses, monetary constants, rewards, genesis, serialization, P2P/RPC
  protocol, or covenant code. Inspection found no runtime identity change safe
  to make without the unresolved design and compatibility decisions.
- Preserved the initially untracked `stage1.md` task file unchanged.

### Validation

- `cargo metadata --no-deps --format-version 1` — succeeded after the metadata
  update; it reported 69 workspace packages and no repository metadata
  mismatches.
- `rustup component add rustfmt --toolchain 1.91.0` — succeeded.
- `cargo fmt --all -- --check` — succeeded after installing `rustfmt`.
- `git diff --check` — succeeded.
- Markdown whitespace, final-newline, README link-target, and migration-matrix
  status-label checks — succeeded.
- Final `git status --short` — showed the intended changes to `Cargo.toml`,
  `README.md`, the three new Stage 1 documents, and the initially untracked
  `stage1.md` task file. The task file remains unchanged.
- No runtime or consensus tests were run because Stage 1 made no Rust source or
  runtime behavior changes. No test results are claimed.
- No Rust source/runtime behavior changed; consensus or runtime tests are not
  claimed for this documentation/metadata-only stage.
