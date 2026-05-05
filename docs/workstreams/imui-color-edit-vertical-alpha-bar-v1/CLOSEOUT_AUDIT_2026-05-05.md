# ImUi Color Edit Vertical Alpha Bar v1 Closeout Audit - 2026-05-05

Status: Closed.

This lane adds Dear ImGui-style vertical AlphaBar behavior to editor `ColorEdit`'s `HsvHueBar`
picker.

## What Shipped

- Inlined a vertical AlphaBar into `hsv_picker` when alpha editing and AlphaBar are both visible.
- Kept the standalone AlphaBar path for `ColorEditPopupPicker::Hidden` combinations.
- Added vertical alpha gradient and thumb helpers.
- Added local-Y alpha mapping with top = 100% and bottom = 0%.
- Updated focused tests and source-policy anchors.

## Proof

- `cargo fmt --package fret-ui-editor -- --check` passes.
- `cargo nextest run -p fret-ui-editor --features imui color_edit --no-fail-fast` passes.
- `cargo nextest run -p fret-ui-editor --features imui --test imui_surface_policy --no-fail-fast`
  passes.
- `cargo nextest run -p fret-ui-editor --features imui --no-fail-fast` passes.
- `python -m json.tool docs/workstreams/imui-color-edit-vertical-alpha-bar-v1/WORKSTREAM.json`
  passes.
- `python -m json.tool docs/workstreams/imui-editor-grade-product-closure-v1/WORKSTREAM.json`
  passes.
- `python tools/check_layering.py` passes.
- `python tools/check_workstream_catalog.py` passes.
- `python tools/gate_imui_workstream_source.py` passes.
- `python .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols` passes.
- `git diff --check` passes.

## Remaining Work

Continue with separate color feature follow-ons such as HueWheel fidelity, picker options popup,
color history, eyedropper behavior, or palette customization.
