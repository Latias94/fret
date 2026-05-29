# Jellyflow Package Split v1 - Handoff

Status: Active
Last updated: 2026-05-29

## Current State

JF-010 is complete. `jellyflow-core` now owns the first stable headless modules plus `ops`
transaction/history helpers. `fret-node` depends on `jellyflow-core`, preserves the old module
paths with compatibility re-exports, and keeps the XyFlow-style change projection in
`runtime/changes.rs`.

## Active Task

- Task ID: JF-020
- Owner: current Codex session
- Status: next
- Claim: decide whether runtime store/callback helpers and any remaining geometry seams belong in
  `jellyflow-runtime` or should stay in the Fret adapter after the `ops` split.

## JF-001 Evidence

- `cargo check -p jellyflow-core`: passed.
- `cargo nextest run -p jellyflow-core`: passed with 14 tests.
- `cargo clippy -p jellyflow-core --all-targets -- -D warnings`: passed.
- `cargo check -p fret-node --all-features --tests`: passed.
- `cargo nextest run -p fret-node --no-default-features`: passed with 124 tests.
- `cargo fmt --check`: passed.
- `jq empty docs/workstreams/jellyflow-package-split-v1/WORKSTREAM.json`: passed.
- `git diff --check`: passed.
- `python3 tools/check_layering.py`: passed.

## JF-010 Evidence

- `cargo check -p fret-node --all-features --tests`: passed after moving `ops` into
  `jellyflow-core`.
- `cargo nextest run -p fret-node --no-default-features`: passed with 90 tests after the `ops`
  split.
- `cargo fmt --check`: passed after rustfmt cleanup.
- `python3 tools/check_layering.py`: passed after the `ops` split.

## Next Steps

1. Audit the remaining runtime helper ownership in `fret-node/src/runtime`.
2. Decide whether store/callback helpers should move into `jellyflow-runtime`.
3. Add focused compatibility gates before moving the next seam.
