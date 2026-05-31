# Material3 TimePicker Interaction Family Split v1 Closeout Audit

Status: Closed
Date: 2026-05-31

## Summary

This lane removed the TimePicker tests from the residual Material3 interaction-regression owner.
TimePicker now has a focused interaction binary, and the residual file is down to field-family plus
plain TextInput ownership decisions.

## Completed Scope

- Moved 4 TimePicker interaction tests out of `material3_interaction_regressions.rs`.
- Added `material3_time_picker_interactions.rs`.
- Moved TimePicker-only invalid/live semantics helpers with the tests.
- Tightened residual imports.
- Added a dedicated workstream and catalog entry.

## Final Gates

- `cargo fmt --package fret-ui-material3 --check`: passed.
- `cargo nextest run -p fret-ui-material3 --features diagnostics --test material3_time_picker_interactions`:
  4 passed, 0 skipped.
- `cargo nextest run -p fret-ui-material3 --features diagnostics --test material3_interaction_regressions`:
  6 passed, 0 skipped.
- `cargo check -p fret-ui-material3 --features diagnostics --test material3_interaction_regressions --test material3_time_picker_interactions`:
  passed.
- `cargo check -p fret-ui-material3 --features diagnostics --tests`: passed.
- `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`:
  passed.
- `python -m json.tool docs/workstreams/material3-time-picker-interaction-family-split-v1/WORKSTREAM.json | Out-Null`:
  passed.
- `python tools/check_workstream_catalog.py`: passed.
- `python tools/check_layering.py`: passed.
- `git diff --check`: passed.

## Residuals

- Autocomplete and ExposedDropdown remain in `material3_interaction_regressions.rs` until a
  field-family split.
- Plain TextInput remains until a mechanism-layer ownership audit decides whether it belongs in
  `fret-ui` coverage.
