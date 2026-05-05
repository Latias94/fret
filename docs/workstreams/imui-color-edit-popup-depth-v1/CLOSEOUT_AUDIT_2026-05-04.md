# Closeout Audit - 2026-05-04

Status: Closed

## Verdict

Closed. `ColorEdit` no longer ships a visible popup stub in the app-facing IMUI editor-control
path.

## Goal-Backward Audit

- Stub removal:
  - Evidence: `ecosystem/fret-ui-editor/src/controls/color_edit.rs`
  - Result: the popup body is a preset swatch palette instead of placeholder text.
- Model update behavior:
  - Evidence: `ecosystem/fret-ui-editor/src/controls/color_edit.rs`
  - Result: selecting a preset updates the bound `Model<Color>`, syncs the draft hex text, clears
    errors, closes the popup, and requests redraw.
- Regression guard:
  - Evidence: `ecosystem/fret-ui-editor/tests/imui_surface_policy.rs`
  - Result: the test rejects the old stub string and requires the preset-palette surface.
- Public teaching proof:
  - Evidence: `apps/fret-cookbook/examples/imui_editor_controls_basics.rs`
  - Result: the public `fret::imui::editor` cookbook proof still compiles with `ColorEdit`.

## Gates Run

- `cargo fmt --package fret-ui-editor`
- `cargo check --tests -p fret-ui-editor --features imui`
- `cargo nextest run -p fret-ui-editor --features imui color_presets_are_unique_and_hex_formattable color_edit_popup_is_a_real_preset_palette_not_a_stub --no-fail-fast`
- `cargo check -p fret-cookbook --features cookbook-imui --example imui_editor_controls_basics`
- `python tools/check_layering.py`
- `python tools/check_workstream_catalog.py`
- `python -m json.tool docs/workstreams/imui-color-edit-popup-depth-v1/WORKSTREAM.json`
- `git diff --check`

## Follow-On Policy

Do not reopen this lane for full color-picker parity. Start separate follow-ons for HSV/RGB editing,
alpha checkerboard/picker behavior, color history, or eyedropper integration.
