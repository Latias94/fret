# Closeout Audit - 2026-05-04

Status: Closed

## Verdict

Closed. `ColorEdit` now treats RGB-only edit paths as RGB-only: they preserve alpha instead of
silently forcing the bound color to opaque.

## Goal-Backward Audit

- RGB hex input:
  - Evidence: `ecosystem/fret-ui-editor/src/controls/color_edit.rs`
  - Result: committing `#RRGGBB` preserves the current alpha channel.
- RGBA hex input:
  - Evidence: `ecosystem/fret-ui-editor/src/controls/color_edit.rs`
  - Result: committing `#RRGGBBAA` is accepted only when `show_alpha=true`.
- Preset swatches:
  - Evidence: `ecosystem/fret-ui-editor/src/controls/color_edit.rs`
  - Result: activating a preset changes RGB, preserves current alpha, syncs draft hex, clears
    errors, closes the popup, and requests redraw.
- Regression floor:
  - Evidence: `ecosystem/fret-ui-editor/src/controls/color_edit.rs` and
    `ecosystem/fret-ui-editor/tests/imui_surface_policy.rs`
  - Result: focused parser/preset tests now cover the alpha-preservation policy, while the existing
    policy test keeps the stub from returning.

## Gates Run

- `cargo fmt --package fret-ui-editor`
- `cargo nextest run -p fret-ui-editor --features imui color_presets_are_unique_and_hex_formattable rgb_hex_parse_preserves_alpha_when_alpha_is_not_explicit rgba_hex_parse_is_only_available_when_alpha_is_visible rgb_presets_preserve_current_alpha color_edit_popup_is_a_real_preset_palette_not_a_stub --no-fail-fast`
- `cargo check --tests -p fret-ui-editor --features imui`
- `cargo check -p fret-cookbook --features cookbook-imui --example imui_editor_controls_basics`
- `python tools/check_layering.py`
- `python tools/check_workstream_catalog.py`
- `python -m json.tool docs/workstreams/imui-color-edit-alpha-policy-v1/WORKSTREAM.json`
- `git diff --check`

## Follow-On Policy

Do not reopen this lane for full picker parity. Start separate follow-ons for alpha checkerboard
preview, AlphaBar-style controls, HSV/RGB picker depth, color history, or eyedropper integration.
