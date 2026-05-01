# Fret Examples Build Latency v1 - M56 Embedded Viewport Source Gate - 2026-05-01

Status: complete

## Decision

Move the embedded viewport demo source-policy checks out of the monolithic `fret-examples` Rust
unit-test module and into `tools/examples_source_tree_policy/app_facing.py`.

## Migrated Checks

- `embedded_viewport_demo_models_size_presets_as_required_toggle_group`
- `embedded_viewport_demo_prefers_capability_first_landing_with_explicit_panel_owner`

## Behavior

The examples source-tree policy gate now covers the required toggle-group size preset model and the
capability-first embedded viewport render slice.

This slice only moves source markers. It does not change embedded viewport runtime behavior or the
remaining Rust tests that still cover broader app-facing/source-policy surfaces.

## Evidence

- `tools/examples_source_tree_policy/app_facing.py`
- `tools/gate_examples_source_tree_policy.py`
- `apps/fret-examples/src/lib.rs`
- `apps/fret-examples/src/embedded_viewport_demo.rs`

## Result

After this migration, the Rust `#[test]` count in `apps/fret-examples/src/lib.rs` dropped from 11
to 9, and the `include_str!` count stayed at 79.

## Gates

```text
python tools/gate_examples_source_tree_policy.py
python -m py_compile tools/gate_examples_source_tree_policy.py tools/examples_source_tree_policy/app_facing.py tools/check_workstream_catalog.py
python tools/check_workstream_catalog.py
python -m json.tool docs/workstreams/fret-examples-build-latency-v1/WORKSTREAM.json
cargo fmt --package fret-examples --check
cargo check -p fret-examples --lib --jobs 1
```
