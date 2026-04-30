# Fret Examples Build Latency v1 - M20 Manual UI Tree Source Gate - 2026-04-30

Status: complete

## Decision

Move the manual `UiTree<App>` root-wrapper source-only marker check out of the monolithic
`fret-examples` Rust unit-test module and into the examples source-tree policy gate.

## Migrated Check

- `manual_ui_tree_examples_keep_root_wrappers_on_local_typed_helpers`

## Behavior

The new `tools/examples_source_tree_policy/manual.py` owner module now covers root-wrapper markers
for:

- `cjk_conformance_demo.rs`,
- `emoji_conformance_demo.rs`.

No runtime behavior change is intended. This slice only moves pure source scanning away from Rust
unit tests that require compiling `fret-examples`.

## Evidence

- `tools/examples_source_tree_policy/manual.py`
- `tools/examples_source_tree_policy/gate.py`
- `apps/fret-examples/src/lib.rs`

## Result

The Rust `#[test]` count in `apps/fret-examples/src/lib.rs` dropped from 69 to 68.

## Gates

```text
python tools/gate_examples_source_tree_policy.py
python -m py_compile tools/gate_examples_source_tree_policy.py tools/examples_source_tree_policy/__init__.py tools/examples_source_tree_policy/gate.py tools/examples_source_tree_policy/app_facing.py tools/examples_source_tree_policy/interop.py tools/examples_source_tree_policy/manual.py
cargo fmt --package fret-examples --check
cargo check -p fret-examples --lib --jobs 1
```
