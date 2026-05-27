# Material 3 TimePicker String Registry Packet v1 - Evidence And Gates

Status: Closed
Last updated: 2026-05-28

## Evidence Anchors

- `ecosystem/fret-ui-material3/src/foundation/strings.rs`
- `ecosystem/fret-ui-material3/src/foundation/mod.rs`
- `ecosystem/fret-ui-material3/src/time_picker.rs`
- `ecosystem/fret-ui-material3/tests/automation_surface.rs`
- `ecosystem/fret-bootstrap/src/lib.rs`
- `docs/workstreams/material3-time-picker-string-registry-packet-v1/artifacts/time_picker_string_registry_packet_v1.md`
- `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/material3_picker_packet_v1.md`
- `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/component_alignment_matrix_v1.json`

## Canonical Gates

```powershell
cargo fmt --package fret-ui-material3 --package fret-bootstrap -- --check
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_time_picker_uses_material_string_registry
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_time_picker_uses_compose_aligned_accessibility_labels
cargo nextest run -p fret-ui-material3 --test radio_alignment time_picker_time_input_rejects_invalid_values_and_recovers
cargo test -p fret-bootstrap --lib default_i18n_formats_material3_time_picker_strings
cargo check -p fret-ui-material3 --features diagnostics --tests
cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings
python -m json.tool docs/workstreams/material3-time-picker-string-registry-packet-v1/WORKSTREAM.json
python -m json.tool docs/workstreams/material3-component-alignment-sweep-v1/artifacts/component_alignment_matrix_v1.json
python tools/check_workstream_catalog.py
git diff --check
```

## Verification Notes

- The local verification run set `TEMP`/`TMP` to `target/tmp` because the machine's `C:` temporary
  directory was full. That did not change the tested binaries or source inputs.
- `cargo nextest run -p fret-bootstrap` was not used for this package because the package currently
  compiles examples that require the optional `launch` feature. The focused bootstrap proof uses
  `cargo test -p fret-bootstrap --lib`.
- Cargo still warned that its global last-use cache database could not be updated because `C:` was
  full. The warnings are environmental and did not affect the passing focused tests.
