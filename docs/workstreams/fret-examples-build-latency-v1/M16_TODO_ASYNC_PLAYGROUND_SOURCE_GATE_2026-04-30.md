# Fret Examples Build Latency v1 - M16 Todo Async Playground Source Gate - 2026-04-30

Status: complete

## Decision

Move the todo and async playground source-only marker checks out of the monolithic
`fret-examples` Rust unit-test module and into `python tools/gate_examples_source_tree_policy.py`.

## Migrated Checks

- `todo_demo_prefers_default_app_surface`
- `todo_demo_prefers_capability_first_landing_for_root_builders`
- `async_playground_demo_prefers_app_render_context_helpers_and_root_capability_landing`

## Behavior

The examples source-tree policy gate now owns:

- todo default-app surface markers,
- todo local-state and responsive shell markers,
- todo capability-first root builder markers,
- async playground generic `AppRenderContext` helper markers,
- async playground root capability-first landing markers.

No runtime behavior change is intended. This slice only moves pure source scanning away from Rust
unit tests that require compiling `fret-examples`.

## Evidence

- `tools/examples_source_tree_policy/gate.py`
- `apps/fret-examples/src/lib.rs`

## Result

The Rust `#[test]` count in `apps/fret-examples/src/lib.rs` dropped from 74 to 71.

## Gates

```text
python tools/gate_examples_source_tree_policy.py
python -m py_compile tools/gate_examples_source_tree_policy.py tools/examples_source_tree_policy/__init__.py tools/examples_source_tree_policy/gate.py
cargo fmt --package fret-examples --check
cargo check -p fret-examples --lib --jobs 1
```
