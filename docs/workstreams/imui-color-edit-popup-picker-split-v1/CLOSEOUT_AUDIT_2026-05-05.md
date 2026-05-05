# ImUi Color Edit Popup Picker Split v1 Closeout Audit - 2026-05-05

Status: Closed.

This lane moves HSV/SV/Hue/Alpha popup picker controls into an internal `popup/picker.rs` owner.

## What Shipped

- Added `ecosystem/fret-ui-editor/src/controls/color_edit/popup/picker.rs`.
- Moved HSV picker composition, saturation/value grid and thumb overlay, HueBar, AlphaBar,
  gradient/thumb preview helpers, and picker-local pointer commit handlers out of `popup.rs`.
- Kept popup overlay assembly, preset swatches, and shared checkerboard/preview helpers in
  `popup.rs`.
- Updated source-policy anchors and focused tests to track the new picker owner.

## Proof

- `cargo fmt --package fret-ui-editor -- --check` passes.
- `cargo nextest run -p fret-ui-editor --features imui color_edit --no-fail-fast` passes.
- `cargo nextest run -p fret-ui-editor --features imui --test imui_surface_policy --no-fail-fast`
  passes.
- `cargo nextest run -p fret-ui-editor --features imui --no-fail-fast` passes.
- `python tools/check_layering.py` passes.
- `python tools/check_workstream_catalog.py` passes.
- `python -m json.tool docs/workstreams/imui-color-edit-popup-picker-split-v1/WORKSTREAM.json`
  passes.
- `python tools/gate_imui_workstream_source.py` passes.
- `python .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols` passes.
- `git diff --check` passes.

## Remaining Work

Continue with separate popup preview/swatches splits, fixture-driven popup/model conformance, or new
color feature follow-ons such as color history, palette customization, eyedropper integration, color
drag/drop payloads, or HueWheel fidelity.
