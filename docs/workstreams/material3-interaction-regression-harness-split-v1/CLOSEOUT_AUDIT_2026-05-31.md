# Material3 Interaction Regression Harness Split v1 Closeout Audit

Status: Closed
Date: 2026-05-31

## Summary

This lane completed the `radio_alignment.rs` ownership cleanup started by the headless-golden split.
`radio_alignment.rs` now contains only Radio-owned checks, and the historical cross-component
interaction regressions moved to `material3_interaction_regressions.rs`.

## Completed Scope

- Moved 48 non-Radio interaction regression tests.
- Kept the three Radio geometry/ripple/pressed-scene tests in `radio_alignment.rs`.
- Removed stale helpers and imports from `radio_alignment.rs`.
- Added a dedicated workstream and catalog entry.

## Final Gates

- `cargo fmt --package fret-ui-material3 --check`: passed.
- `cargo nextest run -p fret-ui-material3 --features diagnostics --test radio_alignment`:
  3 passed, 0 skipped.
- `cargo nextest run -p fret-ui-material3 --features diagnostics --test material3_interaction_regressions`:
  48 passed, 0 skipped.
- `cargo check -p fret-ui-material3 --features diagnostics --test radio_alignment --test material3_interaction_regressions`:
  passed.
- `cargo check -p fret-ui-material3 --features diagnostics --tests`: passed.
- `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`:
  passed.
- `python -m json.tool docs/workstreams/material3-interaction-regression-harness-split-v1/WORKSTREAM.json | Out-Null`:
  passed.
- `python tools/check_workstream_catalog.py`: passed.
- `python tools/check_layering.py`: passed.
- `git diff --check`: passed.

## Residuals

- The new interaction-regression file is still broad. It is correctly named now, but not yet
  family-owned.
- The TextInput regression may belong in mechanism-layer coverage and needs a separate ownership
  audit before moving across crates.
