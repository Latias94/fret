# ImUi Color Edit Popup Split v1 Closeout Audit - 2026-05-05

Status: Closed.

This lane closes a cleanup-only follow-on after the `ColorEdit` model split. It moves popup UI
composition into a dedicated internal module while preserving the public editor control surface.

## What Shipped

- Added `ecosystem/fret-ui-editor/src/controls/color_edit/popup.rs`.
- Moved popup overlay assembly and popup-local UI helpers:
  - HSV picker composition,
  - saturation/value grid and thumb overlay,
  - HueBar and AlphaBar composition,
  - numeric popup row composition,
  - preset swatches,
  - checkerboard and gradient preview helpers,
  - and popup-local pointer handlers.
- Kept `color_edit.rs` focused on public options, root swatch/input row, and model setup.
- Kept `model.rs` focused on pure parsing/conversion/math helpers.
- Updated `imui_surface_policy` to track the new popup source owner.

## Proof

- `cargo fmt --package fret-ui-editor -- --check` passes.
- `cargo nextest run -p fret-ui-editor --features imui color_edit --no-fail-fast` passes.
- `cargo nextest run -p fret-ui-editor --features imui --test imui_surface_policy --no-fail-fast`
  passes.
- `cargo nextest run -p fret-ui-editor --features imui --test imui_adapter_smoke --no-fail-fast`
  passes.
- `cargo nextest run -p fret-ui-editor --features imui --no-fail-fast` passes.
- `python tools/check_layering.py` passes.
- `python tools/check_workstream_catalog.py` passes.
- `python -m json.tool docs/workstreams/imui-color-edit-popup-split-v1/WORKSTREAM.json` passes.
- `python tools/gate_imui_workstream_source.py` passes.
- `python .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols` passes.
- `git diff --check` passes.

## Remaining Work

This split intentionally does not add color features. Start separate follow-ons for:

- further popup submodule decomposition,
- fixture-driven popup/model conformance,
- color history,
- palette customization,
- eyedropper integration,
- color drag/drop payloads,
- or visual HueWheel fidelity.
