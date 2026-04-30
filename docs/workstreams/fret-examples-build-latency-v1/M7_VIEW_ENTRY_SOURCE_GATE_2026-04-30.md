# Fret Examples Build Latency v1 - M7 View Entry Source Gate - 2026-04-30

Status: complete

## Decision

Move the broad source-only view runtime entry checks out of the monolithic `fret-examples` unit-test
module and into `tools/gate_examples_source_tree_policy.py`.

## Migrated Checks

- `view_runtime_examples_prefer_app_ui_and_ui_aliases`
- `view_entry_examples_prefer_builder_then_run`

## Rationale

- Both checks only scan source text across first-party examples.
- The checks do not need Rust type checking.
- Keeping them in the examples source-tree gate makes the authoring-surface policy runnable without
  compiling and linking the large examples crate.

## Gates

```text
python tools/gate_examples_source_tree_policy.py
python -m py_compile tools/gate_examples_source_tree_policy.py
cargo fmt --package fret-examples --check
cargo check -p fret-examples --lib --jobs 1
```
