# Fret Examples Build Latency v1 - M34 IMUI Collection Select-All Workstream Source Gate - 2026-05-01

Status: complete

## Decision

Move the source-only IMUI collection select-all workstream freeze check out of the monolithic
`fret-examples` Rust unit-test module and into `tools/gate_imui_workstream_source.py`.

## Migrated Check

- `immediate_mode_workstream_freezes_the_p1_collection_select_all_follow_on`

## Behavior

The IMUI workstream source gate now covers:

- the `imui-collection-select-all-v1` design, baseline, landed-slice, and closeout markers,
- the lane-state markers that keep the follow-on and current unit/surface floors explicit,
- the umbrella `imui-editor-grade-product-closure-v1` routing markers,
- and the roadmap/workstream-index/todo-tracker references.

The `proof_collection_select_all_*` tests are real Rust unit behavior checks and remain in Rust.
Only the source-policy/document freeze portion moved to Python.

## Evidence

- `tools/gate_imui_workstream_source.py`
- `apps/fret-examples/src/lib.rs`
- `docs/workstreams/imui-collection-select-all-v1/WORKSTREAM.json`
- `docs/workstreams/imui-collection-select-all-v1/EVIDENCE_AND_GATES.md`

## Result

The Rust `#[test]` count in `apps/fret-examples/src/lib.rs` dropped from 48 to 47, and the
`include_str!` count dropped from 233 to 228.

## Gates

```text
python tools/gate_imui_workstream_source.py
cargo nextest run -p fret-examples --lib proof_collection_select_all_selection_uses_visible_order_and_preserves_active_tile proof_collection_select_all_selection_falls_back_to_first_visible_asset proof_collection_select_all_shortcut_matches_primary_a_only --no-fail-fast
python -m py_compile tools/gate_imui_workstream_source.py
cargo fmt --package fret-examples --check
cargo check -p fret-examples --lib --jobs 1
```
