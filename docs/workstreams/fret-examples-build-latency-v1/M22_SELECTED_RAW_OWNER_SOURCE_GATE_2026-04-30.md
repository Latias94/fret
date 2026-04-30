# Fret Examples Build Latency v1 - M22 Selected Raw Owner Source Gate - 2026-04-30

Status: complete

## Decision

Move the selected raw-owner escape-hatch source marker check out of the monolithic
`fret-examples` Rust unit-test module and into the examples source-tree policy gate.

## Migrated Check

- `selected_raw_owner_examples_keep_escape_hatches_explicit`

## Behavior

The `tools/examples_source_tree_policy/owner_split.py` owner module now covers selected raw-owner
markers for examples that intentionally cross from `AppUi` into explicit `ElementContext` authoring
or advanced raw helper calls.

The source policy is root-aware: most files are read from `apps/fret-examples/src`, while
`imui_interaction_showcase_demo.rs` is read from `apps/fret-examples-imui/src`. This keeps IMUI
split-crate ownership explicit instead of relying on filename-only source lookup.

No runtime behavior change is intended. This slice only moves pure source scanning away from Rust
unit tests that require compiling `fret-examples`.

## Evidence

- `tools/examples_source_tree_policy/owner_split.py`
- `tools/examples_source_tree_policy/gate.py`
- `apps/fret-examples/src/lib.rs`
- `apps/fret-examples-imui/src/imui_interaction_showcase_demo.rs`

## Result

The Rust `#[test]` count in `apps/fret-examples/src/lib.rs` dropped from 67 to 66.

## Gates

```text
python tools/gate_examples_source_tree_policy.py
python -m py_compile tools/gate_examples_source_tree_policy.py tools/examples_source_tree_policy/__init__.py tools/examples_source_tree_policy/gate.py tools/examples_source_tree_policy/app_facing.py tools/examples_source_tree_policy/interop.py tools/examples_source_tree_policy/manual.py tools/examples_source_tree_policy/owner_split.py
cargo fmt --package fret-examples --check
cargo check -p fret-examples --lib --jobs 1
```
