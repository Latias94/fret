# Material 3 TimePicker Input Error Packet v1 - Evidence And Gates

Status: Closed
Last updated: 2026-05-28

## Evidence Anchors

- `ecosystem/fret-ui-material3/src/time_picker.rs`
- `ecosystem/fret-ui-material3/src/tokens/time_input.rs`
- `ecosystem/fret-ui-material3/tests/radio_alignment.rs`
- `ecosystem/fret-ui-material3/tests/automation_surface.rs`
- `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/material3_picker_packet_v1.md`
- `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/component_alignment_matrix_v1.json`
- `docs/workstreams/material3-time-picker-input-error-packet-v1/artifacts/time_picker_input_error_packet_v1.md`

## Canonical Gates

```powershell
cargo fmt --package fret-ui-material3 -- --check
cargo nextest run -p fret-ui-material3 --test radio_alignment time_picker_time_input_rejects_invalid_values_and_recovers
cargo nextest run -p fret-ui-material3 --test radio_alignment time_picker_time_input_replaces_and_auto_advances_hour
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_time_picker_exposes_stable_part_test_ids
cargo check -p fret-ui-material3 --features diagnostics --tests
cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings
python -m json.tool docs/workstreams/material3-time-picker-input-error-packet-v1/WORKSTREAM.json
python -m json.tool docs/workstreams/material3-component-alignment-sweep-v1/artifacts/component_alignment_matrix_v1.json
python tools/check_workstream_catalog.py
git diff --check
```

## Verification Notes

- The focused behavior test types `27` into the 24h hour field, verifies committed time remains at
  the previous valid hour, and asserts `SemanticsInvalid::True`.
- The same test verifies the hour supporting text label, polite atomic live-region semantics, and
  recovery after deleting the invalid value and typing `12`.
- The automation-surface test covers the new stable `input.hour.supporting-text` and
  `input.minute.supporting-text` selectors.
