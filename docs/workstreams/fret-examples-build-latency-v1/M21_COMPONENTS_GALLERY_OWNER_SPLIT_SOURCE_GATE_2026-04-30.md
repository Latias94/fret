# Fret Examples Build Latency v1 - M21 Components Gallery Owner Split Source Gate - 2026-04-30

Status: complete

## Decision

Move the components gallery owner-split source/document marker check out of the monolithic
`fret-examples` Rust unit-test module and into the examples source-tree policy gate.

## Migrated Check

- `components_gallery_keeps_retained_render_and_driver_owner_split`

## Behavior

The new `tools/examples_source_tree_policy/owner_split.py` owner module now covers:

- retained render owner markers in `components_gallery.rs`,
- app/theme sync owner helper markers,
- driver/event owner helper markers,
- the components gallery owner split audit note markers.

No runtime behavior change is intended. This slice only moves pure source/document scanning away
from Rust unit tests that require compiling `fret-examples`.

## Evidence

- `tools/examples_source_tree_policy/owner_split.py`
- `tools/examples_source_tree_policy/gate.py`
- `apps/fret-examples/src/lib.rs`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/COMPONENTS_GALLERY_OWNER_SPLIT_AUDIT_2026-04-16.md`

## Result

The Rust `#[test]` count in `apps/fret-examples/src/lib.rs` dropped from 68 to 67.

## Gates

```text
python tools/gate_examples_source_tree_policy.py
python -m py_compile tools/gate_examples_source_tree_policy.py tools/examples_source_tree_policy/__init__.py tools/examples_source_tree_policy/gate.py tools/examples_source_tree_policy/app_facing.py tools/examples_source_tree_policy/interop.py tools/examples_source_tree_policy/manual.py tools/examples_source_tree_policy/owner_split.py
cargo fmt --package fret-examples --check
cargo check -p fret-examples --lib --jobs 1
```
