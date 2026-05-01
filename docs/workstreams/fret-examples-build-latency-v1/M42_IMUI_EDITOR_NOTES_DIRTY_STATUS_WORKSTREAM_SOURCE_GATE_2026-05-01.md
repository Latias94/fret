# Fret Examples Build Latency v1 - M42 IMUI Editor Notes Dirty Status Workstream Source Gate - 2026-05-01

Status: complete

## Decision

Move the source-only IMUI editor-notes dirty status closeout check out of the monolithic
`fret-examples` Rust unit-test module and into `tools/gate_imui_workstream_source.py`.

## Migrated Check

- `immediate_mode_workstream_closes_the_p1_editor_notes_dirty_status_follow_on`

## Behavior

The IMUI workstream source gate now covers:

- the `imui-editor-notes-dirty-status-v1` design, TODO, milestones, evidence, M1 landed slice, and
  closeout markers,
- the `editor_notes_demo.rs` source markers that keep the `Draft status` row visible,
- the lane-state markers that keep the Python source-policy gate and editor rail/device shell
  surface floors explicit,
- and the roadmap/workstream-index/todo-tracker references.

The editor rail and device shell surface tests are real Rust integration checks and remain in Rust.
Only the source-policy/document/source-shape freeze portion moved to Python.

## Evidence

- `tools/gate_imui_workstream_source.py`
- `apps/fret-examples/src/lib.rs`
- `apps/fret-examples/src/editor_notes_demo.rs`
- `docs/workstreams/imui-editor-notes-dirty-status-v1/WORKSTREAM.json`
- `docs/workstreams/imui-editor-notes-dirty-status-v1/EVIDENCE_AND_GATES.md`

## Result

The Rust `#[test]` count in `apps/fret-examples/src/lib.rs` dropped from 40 to 39, and the
`include_str!` count dropped from 191 to 184.

## Gates

```text
python tools/gate_imui_workstream_source.py
cargo nextest run -p fret-examples --test editor_notes_editor_rail_surface --test editor_notes_device_shell_surface --no-fail-fast
python -m py_compile tools/gate_imui_workstream_source.py
cargo fmt --package fret-examples --check
cargo check -p fret-examples --lib --jobs 1
```
