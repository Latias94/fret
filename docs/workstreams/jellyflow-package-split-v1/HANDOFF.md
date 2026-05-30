# Jellyflow Package Split v1 - Handoff

Status: Active
Last updated: 2026-05-30

## Current State

JF-020 is complete. `jellyflow-core` owns the stable graph model, interaction, type, ops, and
transaction/history helpers. `jellyflow-runtime` now owns the headless `io`, `profile`, `rules`,
`schema`, and `runtime` modules. `fret-node` depends on both Jellyflow crates, preserves old module
paths with compatibility re-exports, and keeps Fret UI/kit/profile recipes in the adapter crate.

## Active Task

- Task ID: JF-030
- Owner: current Codex session
- Status: next
- Claim: decide whether canvas-space geometry, route math, spatial indexes, and hit-test helpers
  belong in `jellyflow-geometry` or should stay in the Fret adapter.

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

## JF-020 Evidence

- `cargo check -p jellyflow-runtime`: passed.
- `cargo nextest run -p jellyflow-runtime`: passed with 67 tests.
- `cargo clippy -p jellyflow-runtime --all-targets -- -D warnings`: passed.
- `cargo check -p fret-node --all-features --tests`: passed.
- `cargo nextest run -p fret-node --no-default-features`: passed with 24 tests.
- `cargo fmt --check`: passed.

## Next Steps

1. Audit `ecosystem/fret-node/src/ui/canvas`, declarative paint-only route math, and spatial
   indexes before moving geometry.
2. Decide whether `runtime::fit_view` and `runtime::utils` stay in runtime or become the first
   `jellyflow-geometry` inputs.
3. Add focused compatibility gates before moving geometry or UI measurement state.
