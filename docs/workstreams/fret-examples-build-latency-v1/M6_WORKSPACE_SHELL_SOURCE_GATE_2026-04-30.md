# Fret Examples Build Latency v1 - M6 Workspace Shell Source Gate - 2026-04-30

Status: complete

## Decision

Move the source-only workspace shell capability-helper markers out of the monolithic
`fret-examples` unit-test module and into `tools/gate_examples_source_tree_policy.py`.

## Migrated Checks

- `workspace_shell_demo_prefers_capability_first_command_button_helpers`
- `workspace_shell_demo_prefers_capability_first_editor_rail_helpers`

## Rationale

- The checks only assert required/forbidden source markers in `workspace_shell_demo.rs`.
- They do not need Rust type checking or launched diagnostics.
- `gate_examples_source_tree_policy.py` already owns cross-example source-tree policy, so it is the
  right owner for the workspace-shell helper-shape rule.

## Gates

```text
python tools/gate_examples_source_tree_policy.py
python -m py_compile tools/gate_examples_source_tree_policy.py
cargo fmt --package fret-examples --check
cargo check -p fret-examples --lib --jobs 1
```
