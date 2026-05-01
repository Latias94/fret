# Fret Examples Build Latency v1 - M58 App UI Render Accessor Source Gate - 2026-05-01

Status: complete

## Decision

Move the app-UI render accessor source-policy check out of the monolithic `fret-examples` Rust
unit-test module and into `tools/examples_source_tree_policy/app_facing.py`.

## Migrated Check

- `selected_app_ui_roots_prefer_explicit_render_context_accessors_over_deref`

## Behavior

The examples source-tree policy gate now covers explicit `AppUi` accessor usage such as
`cx.app()`, `cx.app_mut()`, `cx.window_id()`, and grouped `cx.data()` access in app-facing render
roots.

This slice only moves source markers. It does not change demo behavior or parser tests.

## Evidence

- `tools/examples_source_tree_policy/app_facing.py`
- `tools/gate_examples_source_tree_policy.py`
- `apps/fret-examples/src/lib.rs`
- `apps/fret-examples/src/embedded_viewport_demo.rs`
- `apps/fret-examples/src/async_playground_demo.rs`
- `apps/fret-examples/src/markdown_demo.rs`
- `apps/fret-examples/src/api_workbench_lite_demo.rs`
- `apps/fret-examples/src/hello_world_compare_demo.rs`

## Result

After this migration, the Rust `#[test]` count in `apps/fret-examples/src/lib.rs` dropped from 8
to 7, and the `include_str!` count dropped from 79 to 78.

## Gates

```text
python tools/gate_examples_source_tree_policy.py
python -m py_compile tools/gate_examples_source_tree_policy.py tools/examples_source_tree_policy/app_facing.py tools/check_workstream_catalog.py
python tools/check_workstream_catalog.py
python -m json.tool docs/workstreams/fret-examples-build-latency-v1/WORKSTREAM.json
cargo fmt --package fret-examples --check
cargo check -p fret-examples --lib --jobs 1
```
