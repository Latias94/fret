# Evidence and Gates

Status: Closed
Last updated: 2026-05-04

## Smallest Repro

```bash
cargo check -p fret-cookbook --features cookbook-imui --example imui_editor_controls_basics
```

## Gates

```bash
cargo fmt --package fret-ui-editor
cargo check --tests -p fret-ui-editor --features imui
cargo nextest run -p fret-ui-editor --features imui color_presets_are_unique_and_hex_formattable color_edit_popup_is_a_real_preset_palette_not_a_stub --no-fail-fast
cargo check -p fret-cookbook --features cookbook-imui --example imui_editor_controls_basics
python tools/check_layering.py
python tools/check_workstream_catalog.py
python -m json.tool docs/workstreams/imui-color-edit-popup-depth-v1/WORKSTREAM.json
git diff --check
```

## Evidence Anchors

- `ecosystem/fret-ui-editor/src/controls/color_edit.rs`
- `ecosystem/fret-ui-editor/tests/imui_surface_policy.rs`
- `apps/fret-cookbook/examples/imui_editor_controls_basics.rs`

## Upstream Reference Anchors

- `repo-ref/imgui/imgui_widgets.cpp`
