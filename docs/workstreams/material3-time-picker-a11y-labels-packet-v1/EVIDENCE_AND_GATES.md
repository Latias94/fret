# Evidence And Gates

## Evidence

- `repo-ref/compose-multiplatform-core/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/TimePicker.kt`
- `repo-ref/compose-multiplatform-core/compose/material3/material3/src/androidDeviceTest/kotlin/androidx/compose/material3/TimePickerTest.kt`
- `ecosystem/fret-ui-material3/src/time_picker.rs`
- `ecosystem/fret-ui-material3/tests/automation_surface.rs`
- `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/component_alignment_matrix_v1.json`
- `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/material3_picker_packet_v1.md`

## Gates

```powershell
cargo fmt --package fret-ui-material3 -- --check
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_time_picker_uses_compose_aligned_accessibility_labels
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_time_picker_exposes_stable_part_test_ids
cargo check -p fret-ui-material3 --features diagnostics --tests
cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings
python -m json.tool docs/workstreams/material3-component-alignment-sweep-v1/artifacts/component_alignment_matrix_v1.json > $null
python -m json.tool docs/workstreams/material3-time-picker-a11y-labels-packet-v1/WORKSTREAM.json > $null
python tools/check_workstream_catalog.py
git diff --check
```

## Gate Results

Passed on 2026-05-28:

- `cargo fmt --package fret-ui-material3 -- --check`
- `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_time_picker_uses_compose_aligned_accessibility_labels`
- `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_time_picker_exposes_stable_part_test_ids`
- `cargo check -p fret-ui-material3 --features diagnostics --tests`
- `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`
- `python -m json.tool docs/workstreams/material3-component-alignment-sweep-v1/artifacts/component_alignment_matrix_v1.json > $null`
- `python -m json.tool docs/workstreams/material3-time-picker-a11y-labels-packet-v1/WORKSTREAM.json > $null`
- `python -m json.tool docs/workstreams/material3-time-picker-input-error-packet-v1/WORKSTREAM.json > $null`
- `python -m json.tool docs/workstreams/material3-time-picker-dial-accessibility-packet-v1/WORKSTREAM.json > $null`
- `python tools/check_workstream_catalog.py`
- `git diff --check`
