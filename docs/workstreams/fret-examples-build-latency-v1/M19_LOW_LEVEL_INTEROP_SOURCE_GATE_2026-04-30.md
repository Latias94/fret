# Fret Examples Build Latency v1 - M19 Low Level Interop Source Gate - 2026-04-30

Status: complete

## Decision

Move the low-level interop direct-leaf root source-only marker check out of the monolithic
`fret-examples` Rust unit-test module and into the examples source-tree policy gate.

## Migrated Check

- `low_level_interop_examples_keep_direct_leaf_root_contracts`

## Behavior

The new `tools/examples_source_tree_policy/interop.py` owner module now covers direct-leaf root
markers for:

- external texture imports,
- external texture imports web,
- external video imports AVF,
- external video imports MF,
- chart declarative,
- node graph.

No runtime behavior change is intended. This slice only moves pure source scanning away from Rust
unit tests that require compiling `fret-examples`.

## Evidence

- `tools/examples_source_tree_policy/interop.py`
- `tools/examples_source_tree_policy/gate.py`
- `apps/fret-examples/src/lib.rs`

## Result

The Rust `#[test]` count in `apps/fret-examples/src/lib.rs` dropped from 70 to 69.

## Gates

```text
python tools/gate_examples_source_tree_policy.py
python -m py_compile tools/gate_examples_source_tree_policy.py tools/examples_source_tree_policy/__init__.py tools/examples_source_tree_policy/gate.py tools/examples_source_tree_policy/app_facing.py tools/examples_source_tree_policy/interop.py
cargo fmt --package fret-examples --check
cargo check -p fret-examples --lib --jobs 1
```
