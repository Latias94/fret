# ImUi Color Edit Editable Palette Slots v1 Closeout Audit - 2026-05-05

Status: closed closeout record.

## What Shipped

- Added `ColorEditPaletteSlotDrop` and `OnColorEditPaletteSlotDrop` for app-owned palette slot
  mutation.
- Added RGB-only palette slot projection from typed color drag payloads.
- Made popup palette swatches publish RGB drag payloads, matching Dear ImGui's `NoAlpha`
  `ColorButton` payload shape.
- Made popup palette swatches optional drop targets when the app provides the mutation callback.
- Kept the default palette and custom palette source API unchanged.

## Evidence

- `repo-ref/imgui/imgui_demo.cpp`
- `repo-ref/imgui/imgui_widgets.cpp`
- `ecosystem/fret-ui-editor/src/controls/mod.rs`
- `ecosystem/fret-ui-editor/src/controls/color_edit.rs`
- `ecosystem/fret-ui-editor/src/controls/color_edit/drag_drop.rs`
- `ecosystem/fret-ui-editor/src/controls/color_edit/popup.rs`
- `ecosystem/fret-ui-editor/src/controls/color_edit/popup/swatches.rs`
- `ecosystem/fret-ui-editor/src/controls/color_edit/tests.rs`
- `ecosystem/fret-ui-editor/tests/imui_surface_policy.rs`
- `ecosystem/fret-ui-editor/tests/imui_adapter_smoke.rs`

## Gates Run

```bash
cargo fmt --package fret-ui-editor
cargo nextest run -p fret-ui-editor --features imui color_edit --no-fail-fast
cargo nextest run -p fret-ui-editor --features imui --test imui_surface_policy --no-fail-fast
cargo nextest run -p fret-ui-editor --features imui --test imui_adapter_smoke --no-fail-fast
cargo nextest run -p fret-ui-editor --features imui --no-fail-fast
python -m json.tool docs/workstreams/imui-color-edit-editable-palette-slots-v1/WORKSTREAM.json
python -m json.tool docs/workstreams/imui-editor-grade-product-closure-v1/WORKSTREAM.json
python tools/check_workstream_catalog.py
python tools/check_layering.py
python tools/gate_imui_workstream_source.py
python .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols
git diff --check
```

## Residual Gaps

- Color history / recent colors.
- Eyedropper behavior.
- Full picker preview and right-click context popup polish.
