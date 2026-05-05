# ImUi Color Edit Drag Drop Payload v1 Closeout Audit - 2026-05-05

Status: Closed.

This lane adds Dear ImGui-style color drag/drop payloads to editor `ColorEdit` swatches while
preserving Fret's layer split: typed editor policy in `fret-ui-editor`, thin `fret-imui`, and no
runtime drag contract widening.

## What Shipped

- Added `ColorEditDragDropOptions`, defaulting to local drag/drop enabled and explicit cross-window
  routing.
- Added typed `ColorEditDragDropPayload` and `ColorEditDragDropComponents::{Rgb, Rgba}`.
- Added `color_edit/drag_drop.rs` for the editor-local payload store, drag source hooks, drop target
  hover tracking, delivery, and alpha application.
- Wired the root `ColorEdit` swatch as both source and target.
- Matched Dear ImGui `_COL3F` / `_COL4F` alpha behavior:
  - RGB payloads preserve target alpha.
  - RGBA payloads apply alpha only when the target shows alpha.
- Kept the swatch enabled for payload interactions even when popup content is intentionally hidden.

## Proof

- `cargo fmt --package fret-ui-editor -- --check` passes.
- `cargo nextest run -p fret-ui-editor --features imui color_edit --no-fail-fast` passes.
- `cargo nextest run -p fret-ui-editor --features imui --test imui_surface_policy --no-fail-fast`
  passes.
- `cargo nextest run -p fret-ui-editor --features imui --no-fail-fast` passes.
- `python tools/check_layering.py` passes.
- `python tools/check_workstream_catalog.py` passes.
- `python -m json.tool docs/workstreams/imui-color-edit-drag-drop-payload-v1/WORKSTREAM.json`
  passes.
- `python tools/gate_imui_workstream_source.py` passes.
- `python .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols` passes.
- `git diff --check` passes.

## Remaining Work

Continue with separate ColorEdit/ColorPicker feature follow-ons: HueWheel fidelity, eyedropper
behavior, palette customization, color history, and richer side-preview/reference-color affordances.
