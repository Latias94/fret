# Fret Examples Build Latency v1 - M38 IMUI Collection Command Package Workstream Source Gate - 2026-05-01

Status: complete

## Decision

Move the source-only IMUI collection command-package workstream closeout check out of the
monolithic `fret-examples` Rust unit-test module and into `tools/gate_imui_workstream_source.py`.

## Migrated Check

- `immediate_mode_workstream_closes_the_p1_collection_command_package_follow_on`

## Behavior

The IMUI workstream source gate now covers:

- the `imui-collection-command-package-v1` design, baseline, landed duplicate slice, landed rename
  trigger slice, and closeout markers,
- the lane-state markers that keep the closed follow-on, Python source-policy gate, Rust unit
  floor, and surface floors explicit,
- the umbrella `imui-editor-grade-product-closure-v1` routing markers,
- and the roadmap/workstream-index/todo-tracker references.

The duplicate-selected and rename-trigger tests are real Rust unit behavior checks and remain in
Rust. Only the source-policy/document freeze portion moved to Python.

## Evidence

- `tools/gate_imui_workstream_source.py`
- `apps/fret-examples/src/lib.rs`
- `apps/fret-examples/src/imui_editor_proof_demo/collection.rs`
- `docs/workstreams/imui-collection-command-package-v1/WORKSTREAM.json`
- `docs/workstreams/imui-collection-command-package-v1/EVIDENCE_AND_GATES.md`

## Result

The Rust `#[test]` count in `apps/fret-examples/src/lib.rs` dropped from 44 to 43, and the
`include_str!` count dropped from 213 to 207.

## Gates

```text
python tools/gate_imui_workstream_source.py
cargo nextest run -p fret-examples --lib proof_collection_duplicate_shortcut_matches_primary_d_only proof_collection_duplicate_selection_reselects_visible_copies_and_preserves_active_copy proof_collection_duplicate_selection_uses_unique_copy_suffixes_when_copy_exists proof_collection_begin_rename_session_prefers_active_visible_asset proof_collection_begin_rename_session_falls_back_to_first_visible_asset proof_collection_rename_shortcut_matches_plain_f2_only --no-fail-fast
cargo nextest run -p fret-examples --test imui_editor_collection_command_package_surface --test imui_editor_collection_context_menu_surface --test imui_editor_collection_delete_action_surface --test imui_editor_collection_rename_surface --no-fail-fast
python -m py_compile tools/gate_imui_workstream_source.py
cargo fmt --package fret-examples --check
cargo check -p fret-examples --lib --jobs 1
```
