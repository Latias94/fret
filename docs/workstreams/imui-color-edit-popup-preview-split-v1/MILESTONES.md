# ImUi Color Edit Popup Preview Split v1 Milestones

Status: Closed
Last updated: 2026-05-05

## M0 - Scope

- Chose shared preview helpers as the next smallest popup submodule because they are used by both
  swatches and picker alpha rendering but do not own popup policy.

## M1 - Implementation

- Added `controls/color_edit/popup/preview.rs`.
- Moved `color_preview_stack`, `checkerboard_grid`, `fill_preview_layout`,
  `fill_absolute_preview_layout`, and `checkerboard_cell_color`.
- Kept a narrow `popup::color_preview_stack` re-export for the public control module's existing root
  swatch composition path.
- Updated `imui_surface_policy` to check preview helpers in the new file.

## M2 - Proof

- `cargo fmt --package fret-ui-editor -- --check` passes.
- `cargo nextest run -p fret-ui-editor --features imui color_edit --no-fail-fast` passes.
- Full `fret-ui-editor --features imui`, layering, workstream catalog, source, skills, and diff
  checks pass.
