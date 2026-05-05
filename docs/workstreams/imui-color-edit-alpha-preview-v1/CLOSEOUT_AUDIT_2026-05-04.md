# Closeout Audit - 2026-05-04

Status: Closed

## Verdict

Closed. `ColorEdit` swatches now communicate alpha visually through a checkerboard-backed preview
without turning the editor control into a full picker.

## Goal-Backward Audit

- Main swatch preview:
  - Evidence: `ecosystem/fret-ui-editor/src/controls/color_edit.rs`
  - Result: the preview uses `color_preview_stack(...)`, which paints a checkerboard base and
    overlays the current color.
- Preset swatches:
  - Evidence: `ecosystem/fret-ui-editor/src/controls/color_edit.rs`
  - Result: popup presets share the same preview helper, so RGB preset selection remains visually
    honest when the current alpha is non-opaque.
- Regression floor:
  - Evidence: `ecosystem/fret-ui-editor/src/controls/color_edit.rs` and
    `ecosystem/fret-ui-editor/tests/imui_surface_policy.rs`
  - Result: checkerboard colors are unit-tested for stable alternation, and the policy test keeps
    the preset popup from regressing to a stub or losing the alpha-preview helper.

## Gates Run

- `cargo fmt --package fret-ui-editor`
- `cargo nextest run -p fret-ui-editor --features imui alpha_checkerboard_colors_are_stable_and_alternating color_edit_popup_is_a_real_preset_palette_not_a_stub --no-fail-fast`
- `cargo nextest run -p fret-ui-editor --features imui alpha_checkerboard_colors_are_stable_and_alternating color_edit_popup_is_a_real_preset_palette_not_a_stub color_presets_are_unique_and_hex_formattable rgb_hex_parse_preserves_alpha_when_alpha_is_not_explicit rgba_hex_parse_is_only_available_when_alpha_is_visible rgb_presets_preserve_current_alpha --no-fail-fast`
- `cargo nextest run -p fret-ui-editor --features imui --no-fail-fast`
- `cargo check --tests -p fret-ui-editor --features imui`
- `cargo check -p fret-cookbook --features cookbook-imui --example imui_editor_controls_basics`
- `python tools/check_layering.py`
- `python tools/check_workstream_catalog.py`
- `python -m json.tool docs/workstreams/imui-color-edit-alpha-preview-v1/WORKSTREAM.json`
- `git diff --check`

## Follow-On Policy

Do not reopen this lane for full picker parity. The AlphaBar follow-on is tracked by
`docs/workstreams/imui-color-edit-alpha-bar-v1/`; start separate follow-ons for HSV/RGB picker
depth, color history, eyedropper integration, or color drag/drop payloads.
