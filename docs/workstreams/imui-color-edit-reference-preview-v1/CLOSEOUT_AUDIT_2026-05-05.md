# ImUi Color Edit Reference Preview v1 Closeout Audit - 2026-05-05

Status: Closed.

This lane adds Dear ImGui-style current/original reference preview behavior to editor `ColorEdit`
popups.

## What Shipped

- Added `ColorEditPopupSidePreview` with `Hidden`, `Current`, and `CurrentAndOriginal` modes.
- Defaulted `ColorEditPopupOptions` to current + original side previews.
- Captured the reference color when the popup opens.
- Added a popup preview row that renders current and original color chips.
- Made original activation restore RGB only when alpha editing is hidden and RGBA when alpha editing
  is visible.
- Kept the behavior in `fret-ui-editor`; `fret-imui` remains a thin adapter.

## Proof

- `cargo fmt --package fret-ui-editor -- --check` passes.
- `cargo nextest run -p fret-ui-editor --features imui color_edit --no-fail-fast` passes.
- `cargo nextest run -p fret-ui-editor --features imui --test imui_surface_policy --no-fail-fast`
  passes.
- `cargo nextest run -p fret-ui-editor --features imui --no-fail-fast` passes.
- `python -m json.tool docs/workstreams/imui-color-edit-reference-preview-v1/WORKSTREAM.json`
  passes.
- `python -m json.tool docs/workstreams/imui-editor-grade-product-closure-v1/WORKSTREAM.json`
  passes.
- `python tools/check_layering.py` passes.
- `python tools/check_workstream_catalog.py` passes.
- `python tools/gate_imui_workstream_source.py` passes.
- `python .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols` passes.
- `git diff --check` passes.

## Remaining Work

Continue with separate color feature follow-ons such as HueWheel fidelity, color history, eyedropper
behavior, palette customization, or fixture-driven popup conformance.
