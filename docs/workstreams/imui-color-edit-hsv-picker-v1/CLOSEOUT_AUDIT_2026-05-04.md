# Closeout Audit - 2026-05-04

Status: Closed

## Verdict

Closed. Editor `ColorEdit` now has a bounded HSV picker slice in its popup: RGB/HSV conversion,
a saturation/value area, a HueBar, alpha-preserving writes, and focused regression coverage.

## Goal-Backward Audit

- HSV conversion foundation:
  - Evidence: `ecosystem/fret-ui-editor/src/controls/color_edit.rs`
  - Result: private RGB/HSV helpers cover primary colors, grayscale behavior, preset roundtrips,
    and sanitized unit/hue handling.
- Popup picker controls:
  - Evidence: `ecosystem/fret-ui-editor/src/controls/color_edit.rs`
  - Result: popup content now renders the HSV picker before preset swatches and optional AlphaBar.
- Alpha preservation:
  - Evidence: `ecosystem/fret-ui-editor/src/controls/color_edit.rs`
  - Result: HSV picker writes preserve the current alpha channel and format draft hex based on
    `show_alpha`.
- Regression floor:
  - Evidence: `ecosystem/fret-ui-editor/tests/imui_surface_policy.rs`
  - Result: source-policy guard keeps picker infrastructure in the editor-owned implementation and
    prevents a stub popup regression.

## Gates Run

- `cargo fmt --package fret-ui-editor`
- `cargo nextest run -p fret-ui-editor --features imui color_edit --no-fail-fast`
- `cargo check --tests -p fret-ui-editor --features imui`
- `cargo check -p fret-cookbook --features cookbook-imui --example imui_editor_controls_basics`
- `python tools/check_layering.py`
- `python tools/check_workstream_catalog.py`
- `python -m json.tool docs/workstreams/imui-color-edit-hsv-picker-v1/WORKSTREAM.json`
- `cargo nextest run -p fret-ui-editor --features imui --no-fail-fast`
- `python .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols`
- `git diff --check`

## Follow-On Policy

Do not reopen this lane for full `ColorPicker4` parity. RGB/HSV numeric display and edit modes are
covered by `docs/workstreams/imui-color-edit-numeric-readout-v1/CLOSEOUT_AUDIT_2026-05-04.md`
and `docs/workstreams/imui-color-edit-numeric-input-v1/CLOSEOUT_AUDIT_2026-05-04.md`; per-control
popup defaults are covered by
`docs/workstreams/imui-color-edit-popup-options-v1/CLOSEOUT_AUDIT_2026-05-05.md`. Start separate
follow-ons for vertical HueBar fidelity, color history, eyedropper integration, or color drag/drop
payloads.
