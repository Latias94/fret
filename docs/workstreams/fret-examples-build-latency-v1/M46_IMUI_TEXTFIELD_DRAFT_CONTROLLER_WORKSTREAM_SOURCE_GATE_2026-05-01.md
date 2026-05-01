# Fret Examples Build Latency v1 - M46 IMUI TextField Draft Controller Workstream Source Gate - 2026-05-01

Status: complete

## Decision

Move the source-only IMUI TextField draft-controller API proof closeout check out of the monolithic
`fret-examples` Rust unit-test module and into `tools/gate_imui_workstream_source.py`.

## Migrated Check

- `immediate_mode_workstream_closes_the_p1_textfield_draft_controller_api_proof`

## Behavior

The IMUI workstream source gate now covers:

- the `imui-textfield-draft-controller-api-proof-v1` design, TODO, milestones, evidence, and
  closeout markers,
- the `text_field.rs`, `editor_notes_demo.rs`, and draft-controller diagnostics script source
  markers that keep the opaque controller proof visible,
- the lane-state markers that keep launched diagnostics, API smoke, surface tests, and the Python
  source-policy gate explicit,
- and the roadmap/workstream-index/todo-tracker references.

The real `fret-ui-editor` API smoke, editor-notes surface tests, and launched diagnostics proof
remain as behavior gates. Only the source-policy/document/source-shape freeze moved to Python.

## Evidence

- `tools/gate_imui_workstream_source.py`
- `apps/fret-examples/src/lib.rs`
- `ecosystem/fret-ui-editor/src/controls/text_field.rs`
- `apps/fret-examples/src/editor_notes_demo.rs`
- `tools/diag-scripts/ui-editor/editor-notes-demo/editor-notes-demo-draft-controller-proof.json`
- `docs/workstreams/imui-textfield-draft-controller-api-proof-v1/WORKSTREAM.json`
- `docs/workstreams/imui-textfield-draft-controller-api-proof-v1/EVIDENCE_AND_GATES.md`

## Result

The Rust `#[test]` count in `apps/fret-examples/src/lib.rs` dropped from 36 to 35, and the
`include_str!` count dropped from 163 to 155.

## Gates

```text
python tools/gate_imui_workstream_source.py
cargo run -p fretboard-dev -- diag script validate tools/diag-scripts/ui-editor/editor-notes-demo/editor-notes-demo-draft-controller-proof.json
cargo nextest run -p fret-ui-editor --test text_field_api_smoke --no-fail-fast
cargo nextest run -p fret-examples --test editor_notes_editor_rail_surface --test editor_notes_device_shell_surface --no-fail-fast
python -m py_compile tools/gate_imui_workstream_source.py
cargo fmt --package fret-examples --check
cargo check -p fret-examples --lib --jobs 1
```
