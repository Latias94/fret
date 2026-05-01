# Fret Examples Build Latency v1 - M61 Advanced Helper Context Source Gate - 2026-05-01

Status: complete

## Decision

Move the advanced helper/context source-policy matrix out of the monolithic `fret-examples` Rust
unit-test module and into `tools/examples_source_tree_policy/advanced_helpers.py`.

## Migrated Check

- `advanced_helper_contexts_prefer_app_component_cx`

## Behavior

The examples source-tree policy gate now covers the advanced helper context markers that were
previously frozen in Rust:

- advanced examples that should use `AppComponentCx` keep doing so,
- generic helper surfaces return `impl IntoUiElement<...>` instead of erased `AnyElement`,
- app-facing helpers keep the `AppRenderContext` capability surface,
- default app web examples stay on `ElementContext<'_, App>` and avoid `KernelApp`.

This slice also deletes now-unreferenced `include_str!` constants and unused helper functions from
`apps/fret-examples/src/lib.rs`.

## Evidence

- `tools/examples_source_tree_policy/advanced_helpers.py`
- `tools/examples_source_tree_policy/gate.py`
- `tools/gate_examples_source_tree_policy.py`
- `apps/fret-examples/src/lib.rs`

## Result

After this migration and orphan include cleanup, the Rust `#[test]` count in
`apps/fret-examples/src/lib.rs` dropped from 5 to 4, and the `include_str!` count dropped from 68
to 25.

## Gates

```text
python tools/gate_examples_source_tree_policy.py
python -m py_compile tools/gate_examples_source_tree_policy.py tools/examples_source_tree_policy/advanced_helpers.py tools/examples_source_tree_policy/gate.py tools/check_workstream_catalog.py
python tools/check_workstream_catalog.py
python -m json.tool docs/workstreams/fret-examples-build-latency-v1/WORKSTREAM.json
cargo fmt --package fret-examples --check
cargo check -p fret-examples --lib --jobs 1
```
