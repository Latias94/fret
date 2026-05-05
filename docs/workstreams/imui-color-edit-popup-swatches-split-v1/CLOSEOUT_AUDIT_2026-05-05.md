# ImUi Color Edit Popup Swatches Split v1 Closeout Audit - 2026-05-05

Status: Closed.

This lane moves preset swatch row composition and activation handling into an internal
`popup/swatches.rs` owner.

## What Shipped

- Added `ecosystem/fret-ui-editor/src/controls/color_edit/popup/swatches.rs`.
- Moved preset row composition, swatch item composition, selected-state styling,
  alpha-preserving preset activation, draft sync, error clear, popup close, and swatch test-id
  derivation out of `popup.rs`.
- Kept `popup.rs` focused on popup overlay assembly and content ordering.
- Updated source-policy anchors to track the new swatches owner.

## Proof

- `cargo fmt --package fret-ui-editor -- --check` passes.
- `cargo nextest run -p fret-ui-editor --features imui color_edit --no-fail-fast` passes.
- `cargo nextest run -p fret-ui-editor --features imui --test imui_surface_policy --no-fail-fast`
  passes.
- `cargo nextest run -p fret-ui-editor --features imui --no-fail-fast` passes.
- `python tools/check_layering.py` passes.
- `python tools/check_workstream_catalog.py` passes.
- `python -m json.tool docs/workstreams/imui-color-edit-popup-swatches-split-v1/WORKSTREAM.json`
  passes.
- `python tools/gate_imui_workstream_source.py` passes.
- `python .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols` passes.
- `git diff --check` passes.

## Remaining Work

Continue with feature follow-ons such as color history, palette customization, eyedropper
integration, color drag/drop payloads, or HueWheel fidelity.
