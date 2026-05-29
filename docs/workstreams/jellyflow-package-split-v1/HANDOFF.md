# Jellyflow Package Split v1 - Handoff

Status: Active
Last updated: 2026-05-29

## Current State

JF-001 is complete. `jellyflow-core` now exists and owns the first stable headless modules:
`core`, `types`, and `interaction`. `fret-node` depends on `jellyflow-core` and preserves the old
module paths with compatibility re-exports.

## Active Task

- Task ID: JF-010
- Owner: current Codex session
- Status: next
- Claim: decide whether `ops` belongs in `jellyflow-core` or should wait for
  `jellyflow-runtime`, based on the current private normalization/sanity helpers used by
  `fret-node` runtime.

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

## Next Steps

1. Audit `ops` public/private helper usage before moving it.
2. Decide whether `normalize_transaction`, transaction sanity checks, and `GraphHistory` should be
   public Jellyflow core contracts or remain runtime-owned.
3. Add focused compatibility gates before moving `ops`.
