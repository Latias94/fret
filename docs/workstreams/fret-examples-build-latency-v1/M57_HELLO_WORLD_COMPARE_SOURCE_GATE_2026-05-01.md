# Fret Examples Build Latency v1 - M57 Hello World Compare Source Gate - 2026-05-01

Status: complete

## Decision

Move the `hello_world_compare_demo` app-facing helper source-policy check out of the monolithic
`fret-examples` Rust unit-test module and into `tools/examples_source_tree_policy/app_facing.py`.

## Migrated Check

- `closure_local_app_facing_helpers_can_use_app_render_cx_alias`

## Behavior

The examples source-tree policy gate now covers the `AppRenderCx` closure helper, root helper
signature, and `.into_element_in(cx)` app-lane markers for `hello_world_compare_demo`.

This slice only moves source markers. The demo behavior and the remaining runtime-frame sample
source check remain unchanged.

## Evidence

- `tools/examples_source_tree_policy/app_facing.py`
- `tools/gate_examples_source_tree_policy.py`
- `apps/fret-examples/src/lib.rs`
- `apps/fret-examples/src/hello_world_compare_demo.rs`

## Result

After this migration, the Rust `#[test]` count in `apps/fret-examples/src/lib.rs` dropped from 9
to 8, and the `include_str!` count stayed at 79.

## Gates

```text
python tools/gate_examples_source_tree_policy.py
python -m py_compile tools/gate_examples_source_tree_policy.py tools/examples_source_tree_policy/app_facing.py tools/check_workstream_catalog.py
python tools/check_workstream_catalog.py
python -m json.tool docs/workstreams/fret-examples-build-latency-v1/WORKSTREAM.json
cargo fmt --package fret-examples --check
cargo check -p fret-examples --lib --jobs 1
```
