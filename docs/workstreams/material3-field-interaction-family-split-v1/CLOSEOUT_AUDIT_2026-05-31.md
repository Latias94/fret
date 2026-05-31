# Material3 Field Interaction Family Split v1 Closeout Audit

Status: Closed
Date: 2026-05-31

## Summary

This lane removed the field-family tests from the residual Material3 interaction-regression owner.
Autocomplete and ExposedDropdown now have a focused interaction binary, and the residual file is
down to one plain TextInput mechanism ownership decision.

## Completed Scope

- Moved 5 field-family interaction tests out of `material3_interaction_regressions.rs`.
- Added `material3_field_interactions.rs`.
- Tightened residual imports to the single TextInput test.
- Added a dedicated workstream and catalog entry.

## Final Gates

- `cargo fmt --package fret-ui-material3 --check`: passed.
- `cargo nextest run -p fret-ui-material3 --features diagnostics --test material3_field_interactions`:
  5 passed, 0 skipped.
- `cargo nextest run -p fret-ui-material3 --features diagnostics --test material3_interaction_regressions`:
  1 passed, 0 skipped.
- `cargo check -p fret-ui-material3 --features diagnostics --test material3_interaction_regressions --test material3_field_interactions`:
  passed.
- `cargo check -p fret-ui-material3 --features diagnostics --tests`: passed.
- `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`:
  passed.
- `python -m json.tool docs/workstreams/material3-field-interaction-family-split-v1/WORKSTREAM.json | Out-Null`:
  passed.
- `python tools/check_workstream_catalog.py`: passed.
- `python tools/check_layering.py`: passed.
- `git diff --check`: passed.

## Residuals

- Plain TextInput remains until a mechanism-layer ownership audit decides whether it belongs in
  `fret-ui` coverage.
