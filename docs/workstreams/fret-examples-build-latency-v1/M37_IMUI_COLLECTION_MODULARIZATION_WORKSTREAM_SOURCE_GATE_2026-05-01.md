# Fret Examples Build Latency v1 - M37 IMUI Collection Modularization Workstream Source Gate - 2026-05-01

Status: complete

## Decision

Move the source-only IMUI editor proof collection modularization workstream freeze check out of the
monolithic `fret-examples` Rust unit-test module and into `tools/gate_imui_workstream_source.py`.

## Migrated Check

- `immediate_mode_workstream_freezes_the_p1_collection_modularization_follow_on`

## Behavior

The IMUI workstream source gate now covers:

- the `imui-editor-proof-collection-modularization-v1` design, baseline, landed-slice, and closeout
  markers,
- the lane-state markers that keep the follow-on and current unit/surface floors explicit,
- the umbrella `imui-editor-grade-product-closure-v1` routing markers,
- the roadmap/workstream-index/todo-tracker references,
- and the host/module source-boundary markers that prove collection implementation stays in
  `imui_editor_proof_demo/collection.rs`.

The drag-rect and empty-label rename tests are real Rust unit behavior checks and remain in Rust.
Only the source-policy/document freeze portion moved to Python.

## Evidence

- `tools/gate_imui_workstream_source.py`
- `apps/fret-examples/src/lib.rs`
- `apps/fret-examples/src/imui_editor_proof_demo.rs`
- `apps/fret-examples/src/imui_editor_proof_demo/collection.rs`
- `docs/workstreams/imui-editor-proof-collection-modularization-v1/WORKSTREAM.json`

## Result

The Rust `#[test]` count in `apps/fret-examples/src/lib.rs` dropped from 45 to 44, and the
`include_str!` count dropped from 218 to 213.

## Gates

```text
python tools/gate_imui_workstream_source.py
cargo nextest run -p fret-examples --lib proof_collection_drag_rect_normalizes_drag_direction proof_collection_commit_rename_rejects_empty_trimmed_label --no-fail-fast
python -m py_compile tools/gate_imui_workstream_source.py
cargo fmt --package fret-examples --check
cargo check -p fret-examples --lib --jobs 1
```
