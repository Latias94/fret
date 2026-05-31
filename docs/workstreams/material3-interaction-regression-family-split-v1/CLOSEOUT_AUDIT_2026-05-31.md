# Material3 Interaction Regression Family Split v1 Closeout Audit

Status: Closed
Date: 2026-05-31

## Summary

This lane completed the first family split of the intermediate Material3 interaction-regression
binary. The clearly owned navigation, overlay, and choice/action tests now run as separate nextest
targets, and the residual file is small enough for the next ownership audits.

## Completed Scope

- Moved 38 tests out of `material3_interaction_regressions.rs`.
- Added three family-owned test binaries.
- Left 10 residual tests in the original binary with a clear follow-on boundary.
- Tightened imports in all touched binaries after the split.
- Added a dedicated workstream and catalog entry.

## Final Gates

- `cargo fmt --package fret-ui-material3 --check`: passed.
- `cargo nextest run -p fret-ui-material3 --features diagnostics --test material3_navigation_interactions --test material3_overlay_interactions --test material3_choice_action_interactions`:
  38 passed, 0 skipped.
- `cargo nextest run -p fret-ui-material3 --features diagnostics --test material3_interaction_regressions`:
  10 passed, 0 skipped.
- `cargo check -p fret-ui-material3 --features diagnostics --test material3_interaction_regressions --test material3_navigation_interactions --test material3_overlay_interactions --test material3_choice_action_interactions`:
  passed.
- `cargo check -p fret-ui-material3 --features diagnostics --tests`: passed.
- `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`:
  passed.
- `python -m json.tool docs/workstreams/material3-interaction-regression-family-split-v1/WORKSTREAM.json | Out-Null`:
  passed.
- `python tools/check_workstream_catalog.py`: passed.
- `python tools/check_layering.py`: passed.
- `git diff --check`: passed.

## Residuals

- TimePicker remains in `material3_interaction_regressions.rs` until a TimePicker-owned split.
- Autocomplete and ExposedDropdown remain until a field-family split.
- Plain TextInput remains until a mechanism-layer ownership audit decides whether it belongs in
  `fret-ui` coverage.
