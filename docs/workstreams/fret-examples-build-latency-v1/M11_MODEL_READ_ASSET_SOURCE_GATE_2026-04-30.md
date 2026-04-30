# Fret Examples Build Latency v1 - M11 Model-Read And Asset Source Gate - 2026-04-30

Status: complete

## Decision

Move the tail source-only model-read and helper-entrypoint checks out of the monolithic
`fret-examples` unit-test module and into `tools/gate_examples_source_tree_policy.py`.

## Migrated Checks

- `direct_leaf_visibility_reads_use_grouped_selector_model_layout`
- `stress_render_roots_use_grouped_selector_model_layout`
- `genui_message_lane_uses_state_owned_model_helpers`
- `driver_owned_example_loops_keep_raw_model_store_reads`
- `asset_helper_entrypoints_prefer_ui_assets_capability_adapters`
- `embedded_viewport_driver_extensions_are_discoverable_from_advanced_prelude`

## Rationale

- The checks only verify source markers and bounded source slices.
- They do not execute runtime behavior, rendering, diagnostics, parser logic, or Rust type-level
  assertions.
- The Python source-tree gate keeps these drift checks cheap while preserving the same marker
  coverage.

## Historical References

The UI assets authoring-state audit names
`asset_helper_entrypoints_prefer_ui_assets_capability_adapters` as historical evidence. That audit
is not rewritten; this lane records `tools/gate_examples_source_tree_policy.py` as the active gate
owner for the migrated source markers.

## Gates

```text
python tools/gate_examples_source_tree_policy.py
python -m py_compile tools/gate_examples_source_tree_policy.py
cargo fmt --package fret-examples --check
cargo check -p fret-examples --lib --jobs 1
```
