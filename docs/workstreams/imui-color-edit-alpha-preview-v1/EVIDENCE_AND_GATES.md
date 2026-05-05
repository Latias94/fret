# Evidence and Gates

Status: Closed
Last updated: 2026-05-04

## Smallest Repro

```bash
cargo nextest run -p fret-ui-editor --features imui alpha_checkerboard_colors_are_stable_and_alternating color_edit_popup_is_a_real_preset_palette_not_a_stub --no-fail-fast
```

## Gates

```bash
cargo fmt --package fret-ui-editor
cargo nextest run -p fret-ui-editor --features imui alpha_checkerboard_colors_are_stable_and_alternating color_edit_popup_is_a_real_preset_palette_not_a_stub color_presets_are_unique_and_hex_formattable rgb_hex_parse_preserves_alpha_when_alpha_is_not_explicit rgba_hex_parse_is_only_available_when_alpha_is_visible rgb_presets_preserve_current_alpha --no-fail-fast
cargo check --tests -p fret-ui-editor --features imui
cargo check -p fret-cookbook --features cookbook-imui --example imui_editor_controls_basics
python tools/check_layering.py
python tools/check_workstream_catalog.py
python -m json.tool docs/workstreams/imui-color-edit-alpha-preview-v1/WORKSTREAM.json
git diff --check
```

## Evidence Anchors

- `ecosystem/fret-ui-editor/src/controls/color_edit.rs`
- `ecosystem/fret-ui-editor/tests/imui_surface_policy.rs`
- `apps/fret-cookbook/examples/imui_editor_controls_basics.rs`

## Upstream Reference Anchors

- `repo-ref/imgui/imgui.h`
- `repo-ref/imgui/imgui_widgets.cpp`
