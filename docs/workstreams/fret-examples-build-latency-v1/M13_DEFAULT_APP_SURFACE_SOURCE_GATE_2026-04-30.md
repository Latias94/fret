# Fret Examples Build Latency v1 - M13 Default App Surface Source Gate - 2026-04-30

Status: complete

## Decision

Move a small default app surface source-policy batch out of the monolithic `fret-examples` unit-test
module and into `tools/gate_examples_source_tree_policy.py`.

## Migrated Checks

- `simple_todo_demo_prefers_default_app_surface`
- `query_demos_prefer_default_app_surface`
- `hello_counter_demo_prefers_root_helper_surface`
- `hello_counter_demo_prefers_app_lane_text_builders_and_capability_first_landing`

## Rationale

- The checks only verify source markers.
- They do not execute runtime behavior, rendering, diagnostics, parser logic, or Rust type-level
  assertions.
- The examples source-tree policy gate already owns broad default app source-policy drift.

## Historical References

Some historical default-app and public-authoring notes name the old Rust test functions as evidence
commands. Those notes remain historical records; this lane now records
`tools/gate_examples_source_tree_policy.py` as the active gate owner for the migrated markers.

## Gates

```text
python tools/gate_examples_source_tree_policy.py
python -m py_compile tools/gate_examples_source_tree_policy.py
cargo fmt --package fret-examples --check
cargo check -p fret-examples --lib --jobs 1
```
