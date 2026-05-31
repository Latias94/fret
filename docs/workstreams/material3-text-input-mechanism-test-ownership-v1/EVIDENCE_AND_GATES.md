# Material3 TextInput Mechanism Test Ownership v1 Evidence And Gates

Status: Closed
Last updated: 2026-05-31

## Gates

- Format:
  - `cargo fmt --package fret-ui --package fret-ui-material3 --check`
- Focused mechanism test:
  - `cargo nextest run -p fret-ui text_input_text_input_event_updates_model`
- Package checks:
  - `cargo check -p fret-ui --tests`
  - `cargo check -p fret-ui-material3 --features diagnostics --tests`
  - `cargo clippy -p fret-ui --tests --no-deps -- -D warnings`
  - `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`
- Workstream state:
  - `python -m json.tool docs/workstreams/material3-text-input-mechanism-test-ownership-v1/WORKSTREAM.json | Out-Null`
  - `python tools/check_workstream_catalog.py`
- Layering:
  - `python tools/check_layering.py`
- Diff hygiene:
  - `git diff --check`

## Evidence Anchors

- `crates/fret-ui/src/declarative/tests/interactions/text_input.rs`
- `ecosystem/fret-ui-material3/tests/material3_interaction_regressions.rs`
- `docs/workstreams/material3-text-input-mechanism-test-ownership-v1/DESIGN.md`
- `docs/workstreams/material3-text-input-mechanism-test-ownership-v1/TODO.md`
- `docs/workstreams/material3-text-input-mechanism-test-ownership-v1/CLOSEOUT_AUDIT_2026-05-31.md`

## Closeout Evidence

Fresh closeout gate results are recorded in
`docs/workstreams/material3-text-input-mechanism-test-ownership-v1/CLOSEOUT_AUDIT_2026-05-31.md`.
