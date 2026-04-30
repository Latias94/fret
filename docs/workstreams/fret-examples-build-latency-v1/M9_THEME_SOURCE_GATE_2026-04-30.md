# Fret Examples Build Latency v1 - M9 Theme Source Gate - 2026-04-30

Status: complete

## Decision

Move the source-only theme-read policy checks out of the monolithic `fret-examples` unit-test module
and into `tools/gate_examples_source_tree_policy.py`.

## Migrated Checks

- `default_app_examples_prefer_app_theme_snapshot_helper`
- `selected_advanced_runtime_examples_prefer_context_theme_snapshot_helpers`
- `selected_element_context_examples_prefer_context_theme_reads`
- `renderer_theme_bridge_proofs_keep_explicit_host_theme_reads`

## Rationale

- The checks only verify first-party example source markers.
- They do not need Rust type checking, runtime setup, rendering, or launched diagnostics.
- Keeping them in the Python source-tree gate avoids compiling the large `fret-examples` test module
  for theme policy drift.

## Historical References

`selected_element_context_examples_prefer_context_theme_reads` and
`renderer_theme_bridge_proofs_keep_explicit_host_theme_reads` are referenced by the historical
`public-authoring-state-lanes-and-identity-fearless-refactor-v1` audit note. That audit is not
rewritten; this note records that the active gate owner for those source markers is now
`tools/gate_examples_source_tree_policy.py`.

## Gates

```text
python tools/gate_examples_source_tree_policy.py
python -m py_compile tools/gate_examples_source_tree_policy.py
cargo fmt --package fret-examples --check
cargo check -p fret-examples --lib --jobs 1
```
