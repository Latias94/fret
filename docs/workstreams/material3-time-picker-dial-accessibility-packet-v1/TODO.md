# TODO - Material 3 TimePicker Dial Accessibility Packet v1

## Task Ledger

### M3TPD-010 - Open the dial accessibility packet

Status: Done
Owner: Codex
Scope:

- Record the source-truth ordering, target outcomes, and non-goals.
- Keep this lane narrow enough to close independently from broader picker accessibility work.

Validation:

- Workstream docs exist and name the first executable slice.

### M3TPD-020 - Expose value-derived dial label ids

Status: Done
Owner: Codex
Depends on: M3TPD-010
Scope:

- Add `clock-dial.hour.<HH>` and `clock-dial.minute.<MM>` label ids.
- Preserve existing parent dial, chrome, selector, period, and modal ids.
- Avoid positional selectors such as row, index, or traversal order.

Validation:

- `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_time_picker_exposes_stable_part_test_ids`

### M3TPD-030 - Update matrix and proof artifacts

Status: Done
Owner: Codex
Depends on: M3TPD-020
Scope:

- Update `component_alignment_matrix_v1.json`.
- Update `material3_picker_packet_v1.md` residual-risk wording.
- Add the parity proof artifact for this slice.

Validation:

- `python -m json.tool docs/workstreams/material3-component-alignment-sweep-v1/artifacts/component_alignment_matrix_v1.json > $null`
- `python tools/check_workstream_catalog.py`

### M3TPD-040 - Verify and close the packet

Status: Done
Owner: Codex
Depends on: M3TPD-030
Scope:

- Run targeted format, test, check, and lint gates.
- Record exact evidence.
- Mark the lane closed only if the gates pass.

Validation:

- `cargo fmt --package fret-ui-material3 -- --check`
- `cargo check -p fret-ui-material3 --features diagnostics --tests`
- `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`
- `git diff --check`
