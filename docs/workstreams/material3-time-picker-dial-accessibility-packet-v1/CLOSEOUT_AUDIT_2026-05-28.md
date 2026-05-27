# Closeout Audit - Material 3 TimePicker Dial Accessibility Packet v1

Date: 2026-05-28
Status: Closed

## Scope Audit

Closed:

- TimePicker dial label ids are derived from the parent clock dial id.
- Hour labels use `clock-dial.hour.<HH>`.
- Minute labels use `clock-dial.minute.<MM>`.
- The existing parent `clock-dial`, `clock-dial.chrome`, selector, period, input, and modal ids were
  preserved.
- The component matrix and picker packet now separate resolved dial label ids from remaining
  accessibility follow-ons.

Not closed by this packet:

- Invalid input supporting/error text.
- Live-region announcements.
- DatePicker disabled-date and localized calendar semantics.
- A fuller 24-hour dual-ring semantics model.

## Evidence

- `ecosystem/fret-ui-material3/src/time_picker.rs`
- `ecosystem/fret-ui-material3/tests/automation_surface.rs`
- `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/component_alignment_matrix_v1.json`
- `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/material3_picker_packet_v1.md`
- `docs/workstreams/material3-time-picker-dial-accessibility-packet-v1/artifacts/time_picker_dial_accessibility_packet_v1.md`

## Verified Gates

```powershell
cargo fmt --package fret-ui-material3 -- --check
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_time_picker_exposes_stable_part_test_ids
cargo check -p fret-ui-material3 --features diagnostics --tests
cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings
python -m json.tool docs/workstreams/material3-time-picker-dial-accessibility-packet-v1/WORKSTREAM.json > $null
python -m json.tool docs/workstreams/material3-component-alignment-sweep-v1/artifacts/component_alignment_matrix_v1.json > $null
python tools/check_workstream_catalog.py
git diff --check
```

## Follow-On Recommendation

The next picker accessibility packet should choose one of two narrow lanes:

- TimePicker invalid input and live-region announcements.
- DatePicker disabled/selectable-date and localized day/month semantics.

