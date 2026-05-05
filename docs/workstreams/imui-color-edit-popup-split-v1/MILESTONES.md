# ImUi Color Edit Popup Split v1 Milestones

Status: Closed
Last updated: 2026-05-05

## M0 - Scope

- Confirmed the previous `model.rs` split left a small public control file target and one large
  remaining popup composition region.
- Chose an internal module split rather than exposing popup primitives publicly.

## M1 - Implementation

- Added `controls/color_edit/popup.rs`.
- Moved popup overlay assembly, HSV picker UI, saturation/value grid, HueBar, AlphaBar, numeric
  popup rows, preset swatches, checkerboard/gradient preview helpers, and popup-local pointer
  handlers.
- Updated test imports and source-policy anchors for the new `popup.rs` owner.

## M2 - Proof

- `cargo fmt --package fret-ui-editor -- --check` passes.
- Focused `color_edit` nextest coverage passes after the split.
- Editor IMUI source-policy, adapter smoke, full `fret-ui-editor --features imui`, layering,
  workstream catalog, source, skills, and diff checks pass.
