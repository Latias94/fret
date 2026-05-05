# ImUi Color Edit Vertical Hue Bar v1 Closeout Audit - 2026-05-05

Status: Closed.

This lane changes editor `ColorEdit`'s `HsvHueBar` picker from a horizontal hue strip to the Dear
ImGui-shaped SV square plus vertical HueBar layout.

## What Shipped

- Replaced hue local-coordinate mapping with local Y over bar height.
- Changed the HSV picker layout to render the SV square and HueBar side by side.
- Added vertical hue gradient and thumb overlay helpers.
- Updated focused tests and source-policy anchors.
- Kept AlphaBar orientation, HueWheel, picker options, and context-menu behavior as separate
  follow-ons.

## Proof

- `cargo fmt --package fret-ui-editor -- --check` passes.
- `cargo nextest run -p fret-ui-editor --features imui color_edit --no-fail-fast` passes.
- `cargo nextest run -p fret-ui-editor --features imui --test imui_surface_policy --no-fail-fast`
  passes.
- `cargo nextest run -p fret-ui-editor --features imui --no-fail-fast` passes.
- `python -m json.tool docs/workstreams/imui-color-edit-vertical-hue-bar-v1/WORKSTREAM.json`
  passes.
- `python -m json.tool docs/workstreams/imui-editor-grade-product-closure-v1/WORKSTREAM.json`
  passes.
- `python tools/check_layering.py` passes.
- `python tools/check_workstream_catalog.py` passes.
- `python tools/gate_imui_workstream_source.py` passes.
- `python .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols` passes.
- `git diff --check` passes.

## Remaining Work

Continue with separate color feature follow-ons such as vertical AlphaBar parity, HueWheel fidelity,
picker options popup, color history, eyedropper behavior, or palette customization.
