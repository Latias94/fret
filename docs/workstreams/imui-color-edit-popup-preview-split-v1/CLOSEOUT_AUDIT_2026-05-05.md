# ImUi Color Edit Popup Preview Split v1 Closeout Audit - 2026-05-05

Status: Closed.

This lane moves shared checkerboard and preview helpers into an internal `popup/preview.rs` owner.

## What Shipped

- Added `ecosystem/fret-ui-editor/src/controls/color_edit/popup/preview.rs`.
- Moved checkerboard grid, checkerboard cell color, fill-preview layout, absolute fill-preview
  layout, and color preview stack helpers out of `popup.rs`.
- Kept `popup.rs` focused on popup overlay and swatch composition while `popup/picker.rs` imports
  preview helpers for AlphaBar rendering.
- Updated source-policy anchors and focused tests to track the new preview owner.

## Proof

- `cargo fmt --package fret-ui-editor -- --check` passes.
- `cargo nextest run -p fret-ui-editor --features imui color_edit --no-fail-fast` passes.
- `cargo nextest run -p fret-ui-editor --features imui --test imui_surface_policy --no-fail-fast`
  passes.
- `cargo nextest run -p fret-ui-editor --features imui --no-fail-fast` passes.
- `python tools/check_layering.py` passes.
- `python tools/check_workstream_catalog.py` passes.
- `python -m json.tool docs/workstreams/imui-color-edit-popup-preview-split-v1/WORKSTREAM.json`
  passes.
- `python tools/gate_imui_workstream_source.py` passes.
- `python .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols` passes.
- `git diff --check` passes.

## Remaining Work

Continue with a separate popup swatches split, fixture-driven popup/model conformance, or new color
feature follow-ons such as color history, palette customization, eyedropper integration, color
drag/drop payloads, or HueWheel fidelity.
