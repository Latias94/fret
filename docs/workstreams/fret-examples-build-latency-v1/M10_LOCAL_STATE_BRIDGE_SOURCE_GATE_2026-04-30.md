# Fret Examples Build Latency v1 - M10 Local-State Bridge Source Gate - 2026-04-30

Status: complete

## Decision

Move a broad local-state bridge source-policy batch out of the monolithic `fret-examples` unit-test
module and into `tools/gate_examples_source_tree_policy.py`.

## Migrated Checks

- `canonical_default_app_examples_stay_local_state_first`
- `manual_form_demo_uses_app_ui_render_root_bridge`
- `init_phase_local_state_examples_prefer_new_in_over_from_model`
- `manual_date_picker_demo_uses_app_ui_render_root_bridge`
- `manual_sonner_demo_uses_app_ui_render_root_bridge`
- `manual_ime_smoke_demo_uses_app_ui_render_root_bridge`
- `manual_emoji_conformance_demo_uses_app_ui_render_root_bridge`
- `select_examples_prefer_local_state_bridges_over_clone_model`
- `date_picker_examples_prefer_local_state_bridges_over_clone_model`
- `bool_control_examples_prefer_local_state_bridges_over_clone_model`
- `form_examples_prefer_local_state_form_bridges_over_clone_model`
- `manual_components_gallery_uses_app_ui_render_root_bridge`

## Rationale

- The checks only verify checked-in source markers.
- They do not execute parser logic, runtime code, rendering, diagnostics, or Rust type-level
  behavior.
- The examples source-tree gate is the better owner for source-policy drift because it runs without
  compiling the large examples test module.

## Historical References

Some historical authoring-state workstream notes name the old Rust test functions as evidence
commands. Those notes remain historical records; this lane now records
`tools/gate_examples_source_tree_policy.py` as the active gate owner for the migrated source markers.

## Gates

```text
python tools/gate_examples_source_tree_policy.py
python -m py_compile tools/gate_examples_source_tree_policy.py
cargo fmt --package fret-examples --check
cargo check -p fret-examples --lib --jobs 1
```
