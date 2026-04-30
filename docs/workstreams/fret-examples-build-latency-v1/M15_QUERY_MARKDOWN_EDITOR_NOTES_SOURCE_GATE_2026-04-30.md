# Fret Examples Build Latency v1 - M15 Query Markdown Editor Notes Source Gate - 2026-04-30

Status: complete

## Decision

Move another source-only marker group out of the monolithic `fret-examples` Rust unit-test module
and into `python tools/gate_examples_source_tree_policy.py`.

## Migrated Checks

- `query_demos_prefer_capability_first_landing_for_root_detail_builders`
- `markdown_demo_keeps_layout_query_authoring_on_app_ui_lane`
- `markdown_demo_prefers_capability_first_landing_for_root_and_layout_query_shells`
- `editor_notes_demo_keeps_reusable_panels_on_generic_element_context_access`
- `editor_notes_demo_prefers_capability_first_landing_for_workspace_shell_root`

## Behavior

The examples source-tree policy gate now owns:

- query/query-async capability-first detail builder landing markers,
- markdown AppUi-lane layout-query authoring markers,
- markdown capability-first root/layout-query shell markers,
- editor notes generic reusable-panel helper markers,
- editor notes workspace-shell root capability-first landing markers.

No runtime behavior change is intended. This slice only moves pure source scanning away from Rust
unit tests that require compiling `fret-examples`.

## Evidence

- `tools/examples_source_tree_policy/gate.py`
- `apps/fret-examples/src/lib.rs`

## Result

The Rust `#[test]` count in `apps/fret-examples/src/lib.rs` dropped from 79 to 74.

## Gates

```text
python tools/gate_examples_source_tree_policy.py
python -m py_compile tools/gate_examples_source_tree_policy.py tools/examples_source_tree_policy/__init__.py tools/examples_source_tree_policy/gate.py
cargo fmt --package fret-examples --check
cargo check -p fret-examples --lib --jobs 1
```
