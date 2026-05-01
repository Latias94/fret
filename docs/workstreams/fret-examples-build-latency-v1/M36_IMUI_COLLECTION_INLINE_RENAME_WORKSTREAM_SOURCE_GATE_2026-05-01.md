# Fret Examples Build Latency v1 - M36 IMUI Collection Inline Rename Workstream Source Gate - 2026-05-01

Status: complete

## Decision

Move the source-only IMUI collection inline-rename workstream freeze check out of the monolithic
`fret-examples` Rust unit-test module and into `tools/gate_imui_workstream_source.py`.

## Migrated Check

- `immediate_mode_workstream_freezes_the_p1_collection_inline_rename_follow_on`

## Behavior

The IMUI workstream source gate now covers:

- the `imui-collection-inline-rename-v1` design, baseline, landed-slice, and closeout markers,
- the lane-state markers that keep the follow-on and current unit/surface floors explicit,
- the umbrella `imui-editor-grade-product-closure-v1` routing markers,
- and the roadmap/workstream-index/todo-tracker references.

The rename session, rename shortcut, rename commit, and empty-label rejection tests are real Rust
unit behavior checks and remain in Rust. Only the source-policy/document freeze portion moved to
Python.

## Evidence

- `tools/gate_imui_workstream_source.py`
- `apps/fret-examples/src/lib.rs`
- `docs/workstreams/imui-collection-inline-rename-v1/WORKSTREAM.json`
- `docs/workstreams/imui-collection-inline-rename-v1/EVIDENCE_AND_GATES.md`

## Result

The Rust `#[test]` count in `apps/fret-examples/src/lib.rs` dropped from 46 to 45, and the
`include_str!` count dropped from 223 to 218.

## Gates

```text
python tools/gate_imui_workstream_source.py
cargo nextest run -p fret-examples --lib proof_collection_begin_rename_session_prefers_active_visible_asset proof_collection_begin_rename_session_falls_back_to_first_visible_asset proof_collection_rename_shortcut_matches_plain_f2_only proof_collection_commit_rename_updates_label_without_touching_order_or_ids proof_collection_commit_rename_rejects_empty_trimmed_label --no-fail-fast
python -m py_compile tools/gate_imui_workstream_source.py
cargo fmt --package fret-examples --check
cargo check -p fret-examples --lib --jobs 1
```
