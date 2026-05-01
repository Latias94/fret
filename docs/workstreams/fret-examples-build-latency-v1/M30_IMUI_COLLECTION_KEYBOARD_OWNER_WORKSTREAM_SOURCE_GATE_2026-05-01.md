# Fret Examples Build Latency v1 - M30 IMUI Collection Keyboard Owner Workstream Source Gate - 2026-05-01

Status: complete

## Decision

Move the source-only IMUI collection keyboard-owner workstream freeze check out of the monolithic
`fret-examples` Rust unit-test module and into `tools/gate_imui_workstream_source.py`.

## Migrated Check

- `immediate_mode_workstream_freezes_the_p1_collection_keyboard_owner_follow_on`

## Behavior

The IMUI workstream source gate now covers:

- the `imui-collection-keyboard-owner-v1` design, baseline, landed-slice, and closeout markers,
- the lane-state markers that keep the follow-on and current unit/surface floors explicit,
- the umbrella `imui-editor-grade-product-closure-v1` routing markers,
- and the roadmap/workstream-index/todo-tracker references.

The `proof_collection_keyboard_*` tests are real Rust unit behavior checks and remain in Rust. Only
the source-policy/document freeze portion moved to Python.

## Evidence

- `tools/gate_imui_workstream_source.py`
- `apps/fret-examples/src/lib.rs`
- `docs/workstreams/imui-collection-keyboard-owner-v1/WORKSTREAM.json`
- `docs/workstreams/imui-collection-keyboard-owner-v1/EVIDENCE_AND_GATES.md`

## Result

The Rust `#[test]` count in `apps/fret-examples/src/lib.rs` dropped from 52 to 51, and the
`include_str!` count dropped from 253 to 248.

## Gates

```text
python tools/gate_imui_workstream_source.py
cargo nextest run -p fret-examples --lib proof_collection_keyboard_arrow_replaces_selection_and_moves_active_tile proof_collection_keyboard_shift_navigation_extends_range_from_anchor proof_collection_keyboard_escape_clears_selection_but_keeps_active_tile proof_collection_keyboard_ignores_primary_modifier_shortcuts --no-fail-fast
python -m py_compile tools/gate_imui_workstream_source.py
cargo fmt --package fret-examples --check
cargo check -p fret-examples --lib --jobs 1
```
