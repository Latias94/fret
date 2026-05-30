# TODO - Material 3 DatePicker Day Cell Selectors Packet v1

## Task Ledger

### M3DCS-010 - Open the packet

Status: Done
Owner: Codex
Scope:

- Record the source truth, scope, non-goals, and layer ownership.

Validation:

- Workstream docs exist and identify the first executable task.

### M3DCS-020 - Add value-derived day cell selector aliases

Status: Done
Owner: Codex
Depends on: M3DCS-010
Scope:

- Stamp `date_picker.cell.<yyyy-mm-dd>` anchors for rendered day cells.
- Preserve existing `date_picker.cell.<row>.<col>` semantic ids.
- Cover docked and modal picker surfaces in automation tests.

Validation:

- `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_date_picker_exposes_stable_part_test_ids`

### M3DCS-030 - Update matrix and evidence docs

Status: Done
Owner: Codex
Depends on: M3DCS-020
Scope:

- Update the component matrix row for DatePicker.
- Update the picker packet to record value-derived cell ids and residual follow-ons.
- Add the proof artifact.

Validation:

- `python -m json.tool docs/workstreams/material3-component-alignment-sweep-v1/artifacts/component_alignment_matrix_v1.json > $null`
- `python tools/check_workstream_catalog.py`

### M3DCS-040 - Verify and close

Status: Done
Owner: Codex
Depends on: M3DCS-030
Scope:

- Run focused and crate-level gates.
- Record command evidence.
- Close the workstream if all gates pass.

Validation:

- `cargo fmt --package fret-ui-material3 -- --check`
- `cargo check -p fret-ui-material3 --features diagnostics --tests`
- `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`
- `git diff --check`
