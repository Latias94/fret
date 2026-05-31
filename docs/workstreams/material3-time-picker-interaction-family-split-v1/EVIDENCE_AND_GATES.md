# Material3 TimePicker Interaction Family Split v1 Evidence And Gates

Status: Closed
Last updated: 2026-05-31

## Gates

- Format:
  - `cargo fmt --package fret-ui-material3 --check`
- Focused TimePicker binary:
  - `cargo nextest run -p fret-ui-material3 --features diagnostics --test material3_time_picker_interactions`
- Residual interaction binary:
  - `cargo nextest run -p fret-ui-material3 --features diagnostics --test material3_interaction_regressions`
- Package checks:
  - `cargo check -p fret-ui-material3 --features diagnostics --test material3_interaction_regressions --test material3_time_picker_interactions`
  - `cargo check -p fret-ui-material3 --features diagnostics --tests`
  - `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`
- Workstream state:
  - `python -m json.tool docs/workstreams/material3-time-picker-interaction-family-split-v1/WORKSTREAM.json | Out-Null`
  - `python tools/check_workstream_catalog.py`
- Layering:
  - `python tools/check_layering.py`
- Diff hygiene:
  - `git diff --check`

## Evidence Anchors

- `ecosystem/fret-ui-material3/tests/material3_time_picker_interactions.rs`
- `ecosystem/fret-ui-material3/tests/material3_interaction_regressions.rs`
- `docs/workstreams/material3-time-picker-interaction-family-split-v1/DESIGN.md`
- `docs/workstreams/material3-time-picker-interaction-family-split-v1/TODO.md`
- `docs/workstreams/material3-time-picker-interaction-family-split-v1/CLOSEOUT_AUDIT_2026-05-31.md`

## Closeout Evidence

Fresh closeout gate results are recorded in
`docs/workstreams/material3-time-picker-interaction-family-split-v1/CLOSEOUT_AUDIT_2026-05-31.md`.
