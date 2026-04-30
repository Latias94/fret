# Fret Examples Build Latency v1 - M8 Authoring Import Source Gate - 2026-04-30

Status: complete

## Decision

Move a small batch of source-only authoring/import policy checks out of the monolithic
`fret-examples` unit-test module and into `tools/gate_examples_source_tree_policy.py`.

## Migrated Checks

- `app_facing_state_examples_prefer_grouped_data_surface`
- `helper_heavy_examples_prefer_grouped_data_surface`
- `app_facing_query_examples_prefer_fret_query_facade`
- `advanced_entry_examples_prefer_view_elements_aliases`
- `app_facing_docking_examples_use_owning_fret_docking_crate`
- `advanced_docking_harnesses_keep_raw_fret_docking_imports`

## Rationale

- The checks only scan first-party example source text.
- They do not need Rust type checking or launched diagnostics.
- The examples source-tree gate is already the owner for cross-example authoring policy.

## Gates

```text
python tools/gate_examples_source_tree_policy.py
python -m py_compile tools/gate_examples_source_tree_policy.py
cargo fmt --package fret-examples --check
cargo check -p fret-examples --lib --jobs 1
```
