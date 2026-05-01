# Fret Examples Build Latency v1 - M59 View Runtime Grouped State Source Gate - 2026-05-01

Status: complete

## Decision

Move the view-runtime grouped state/action source-policy check out of the monolithic
`fret-examples` Rust unit-test module and into `tools/examples_source_tree_policy/app_facing.py`.

## Migrated Check

- `selected_view_runtime_examples_prefer_grouped_state_actions_and_effects`

## Behavior

The examples source-tree policy gate now covers grouped `cx.state()`, `cx.actions()`,
`cx.effects()`, and `cx.data()` usage for the default app/view-runtime examples previously frozen
by the Rust source-marker test.

This slice only moves source markers. It does not change app behavior or parser tests.

## Evidence

- `tools/examples_source_tree_policy/app_facing.py`
- `tools/gate_examples_source_tree_policy.py`
- `apps/fret-examples/src/lib.rs`
- `apps/fret-examples/src/hello_counter_demo.rs`
- `apps/fret-examples/src/query_demo.rs`
- `apps/fret-examples/src/query_async_tokio_demo.rs`
- `apps/fret-examples/src/todo_demo.rs`
- `apps/fret-examples/src/embedded_viewport_demo.rs`

## Result

After this migration, the Rust `#[test]` count in `apps/fret-examples/src/lib.rs` dropped from 7
to 6, and the `include_str!` count dropped from 78 to 74.

## Gates

```text
python tools/gate_examples_source_tree_policy.py
python -m py_compile tools/gate_examples_source_tree_policy.py tools/examples_source_tree_policy/app_facing.py tools/check_workstream_catalog.py
python tools/check_workstream_catalog.py
python -m json.tool docs/workstreams/fret-examples-build-latency-v1/WORKSTREAM.json
cargo fmt --package fret-examples --check
cargo check -p fret-examples --lib --jobs 1
```
