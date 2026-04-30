# Fret Examples Build Latency v1 - M12 Advanced Roster Source Gate - 2026-04-30

Status: complete

## Decision

Move the advanced/reference roster source-policy checks out of the monolithic `fret-examples`
unit-test module and into `tools/gate_examples_source_tree_policy.py`.

## Migrated Checks

- `migrated_examples_use_the_explicit_advanced_surface`
- `advanced_reference_demos_are_explicitly_classified`
- `examples_docs_explicitly_name_the_advanced_reference_roster`

## Rationale

- The checks only verify source and documentation markers.
- They do not execute runtime behavior, rendering, diagnostics, parser logic, or type-level Rust
  assertions.
- The examples source-tree policy gate is the clearer owner for broad first-party examples roster
  drift.

## Historical References

Public authoring-state notes reference `advanced_reference_demos_are_explicitly_classified` as a
historical evidence command. Those notes remain historical records; this lane now records
`tools/gate_examples_source_tree_policy.py` as the active gate owner for the migrated roster
markers.

## Gates

```text
python tools/gate_examples_source_tree_policy.py
python -m py_compile tools/gate_examples_source_tree_policy.py
cargo fmt --package fret-examples --check
cargo check -p fret-examples --lib --jobs 1
```
