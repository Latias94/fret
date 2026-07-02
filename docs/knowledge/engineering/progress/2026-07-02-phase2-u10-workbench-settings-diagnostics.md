---
type: Work Progress
title: Phase 2 U10 Workbench-lite settings diagnostics
tags: fret,phase2,u10,public-app,diagnostics,command-routing
timestamp: 2026-07-02
related_plan: docs/plans/2026-07-02-001-refactor-ui-framework-phase2-plan.md
git_branch: feat/ui-framework-phase2-refactor
---

# Phase 2 U10 Workbench-Lite Settings Diagnostics

## Summary

Phase 2 U10 turns the generated `workbench-lite` scaffold settings dialog into a public-app
diagnostics gate. The scaffold now enables diagnostics, exposes stable settings label/close IDs,
and separates committed project/owner state from draft input state so Cancel/Escape discard edits
while Save commits trimmed values.

The public script also exposed a real command-routing bug: modal overlay buttons emitted command
effects, but modal barrier dispatch did not fall back to the base view's `AppUi` action root. Command
dispatch now evaluates explicit action-route fallback roots through the base dispatch snapshot even
when a modal barrier is active, without opening arbitrary underlay subtree dispatch.

## Changes

- `workbench-lite` template enables the `diagnostics` feature and documents the public settings
  script in generated README output.
- Settings state is split into committed `project_name` / `owner_name` and draft
  `draft_project_name` / `draft_owner_name`.
- `OpenSettings` copies committed state into drafts, `SaveSettings` commits trimmed non-empty drafts,
  and `CancelSettings` only closes the dialog.
- Stable selectors now cover the project label, owner label, inputs, save/cancel buttons, and close
  button.
- `tools/diag-scripts/public-app/workbench-lite-settings-dialog.json` covers open, initial focus,
  focus containment, Cancel, Save, Escape, focus restore, and label updates.
- `UiTree::dispatch_command` now uses base action-route fallback roots under modal barriers so
  overlay commands can route to the owning `AppUi` action host.

## Verification

Focused red/green evidence:

- Before implementation, the scaffold template tests failed on the missing close selector and
  missing diagnostics README script reference.
- During the real public app run, Cancel keyboard activation failed to close the dialog even though
  the button was focused and emitted Enter events. The same path passed for a standalone shadcn
  button action, isolating the defect to modal command fallback routing.
- After command routing changed, the public script passed against a generated workbench-lite app.

Verification passed before commit:

- `cargo nextest run -p fretboard workbench_lite_template_uses_public_app_facade_only workbench_lite_template_cargo_toml_enables_command_palette_without_state workbench_lite_readme_documents_second_hour_position --no-fail-fast`
- `cargo run -p fretboard-dev -- diag script validate tools/diag-scripts/public-app/workbench-lite-settings-dialog.json`
- `cargo run -p fretboard-dev -- diag run tools/diag-scripts/public-app/workbench-lite-settings-dialog.json --timeout-ms 240000 --launch -- cargo run --manifest-path local/u10-workbench-lite/Cargo.toml`
- `cargo nextest run -p fret shadcn_button_action_keyboard_activation_dispatches_app_ui_action --features shadcn --no-fail-fast`
- `cargo check -p fret-ui`
- `cargo nextest run -p fretboard scaffold --no-fail-fast`
- `cargo fmt --all --check`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`
- `python3 tools/check_execution_surface.py`
- `git diff --check`

## Next Action

Continue to U11 by moving query/mutation/toast cookbook affordances behind the public `AppUi`
facade. Keep raw model-store access out of public examples unless the example is explicitly teaching
raw runtime escape hatches.

# Citations

- [Phase 2 plan](../../../plans/2026-07-02-001-refactor-ui-framework-phase2-plan.md)
- [Workbench-lite template](../../../../crates/fretboard/src/scaffold/templates.rs)
- [Settings diagnostics script](../../../../tools/diag-scripts/public-app/workbench-lite-settings-dialog.json)
- [Command routing](../../../../crates/fret-ui/src/tree/commands.rs)
