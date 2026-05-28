# Material 3 DatePicker Locale Strings Packet v1 - Evidence And Gates

Status: Closed
Last updated: 2026-05-28

## Evidence Anchors

- `ecosystem/fret-ui-material3/src/foundation/strings.rs`
- `ecosystem/fret-ui-material3/src/button.rs`
- `ecosystem/fret-ui-material3/src/date_picker.rs`
- `ecosystem/fret-ui-material3/tests/automation_surface.rs`
- `ecosystem/fret-bootstrap/src/lib.rs`
- `docs/workstreams/material3-date-picker-locale-strings-packet-v1/artifacts/date_picker_locale_strings_packet_v1.md`
- `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/material3_picker_packet_v1.md`
- `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/component_alignment_matrix_v1.json`

## Canonical Gates

```powershell
cargo fmt --package fret-ui-material3 --package fret-bootstrap -- --check
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_date_picker_uses_material_string_registry_and_date_descriptions
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_date_picker
cargo test -p fret-bootstrap --lib default_i18n_formats_material3_date_picker_strings
cargo check -p fret-ui-material3 --features diagnostics --tests
cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings
python -m json.tool docs/workstreams/material3-date-picker-locale-strings-packet-v1/WORKSTREAM.json
python -m json.tool docs/workstreams/material3-component-alignment-sweep-v1/artifacts/component_alignment_matrix_v1.json
python tools/check_workstream_catalog.py
git diff --check
```

## Verification Notes

- Local verification set `TEMP`/`TMP` to `target/tmp` because the machine's `C:` temporary directory
  was full. That did not change the tested binaries or source inputs.
- `cargo nextest run -p fret-bootstrap` was not used because the package currently compiles examples
  that require optional features. The focused bootstrap proof uses `cargo test -p fret-bootstrap
  --lib`.
