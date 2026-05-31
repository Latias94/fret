# Material3 Field Family Production Alignment v1 Evidence And Gates

Status: Closed
Last updated: 2026-05-31

## Repro

- `cargo nextest run -p fret-ui-material3 --features diagnostics --test text_field_hover --test automation_surface`
- `cargo check -p fret-ui-material3 --features diagnostics --tests`

## Gates

- Passed: `cargo fmt --package fret-ui-material3 --check`
- Passed: `cargo nextest run -p fret-ui-material3 --features diagnostics --test text_field_hover --test automation_surface`
- Passed: `cargo check -p fret-ui-material3 --features diagnostics --tests`
- Passed: `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`
- Passed: `python -m json.tool docs/workstreams/material3-field-family-production-alignment-v1/WORKSTREAM.json | Out-Null`
- Passed: `python tools/check_workstream_catalog.py`
- Passed: `python tools/check_layering.py`
- Passed: `git diff --check`

## Evidence Anchors

- `ecosystem/fret-ui-material3/src/foundation/field.rs`
- `ecosystem/fret-ui-material3/src/text_field.rs`
- `ecosystem/fret-ui-material3/src/select.rs`
- `ecosystem/fret-ui-material3/tests/text_field_hover.rs`
- `ecosystem/fret-ui-material3/tests/automation_surface.rs`
- `docs/workstreams/material3-field-family-production-alignment-v1/CLOSEOUT_AUDIT_2026-05-31.md`
