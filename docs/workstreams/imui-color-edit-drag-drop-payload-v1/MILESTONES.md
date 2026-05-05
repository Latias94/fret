# ImUi Color Edit Drag Drop Payload v1 Milestones

Status: Closed
Last updated: 2026-05-05

## M0 - Scope

- Chose color drag/drop because local `repo-ref/imgui/imgui.h` and
  `repo-ref/imgui/imgui_widgets.cpp` show `ColorButton` publishing `_COL3F` / `_COL4F` payloads
  and `ColorEdit4` accepting both.
- Kept the implementation in `fret-ui-editor` because this is editor control policy, not an
  immediate-mode facade or runtime mechanism contract.

## M1 - Implementation

- Added `ColorEditDragDropOptions` with default local drag/drop enabled and cross-window routing
  explicit.
- Added typed `ColorEditDragDropPayload` and `ColorEditDragDropComponents`.
- Added `color_edit/drag_drop.rs` for the editor-local payload store, thresholded source hooks,
  target hover tracking, delivery, and alpha rules.
- Made the root swatch remain enabled when drag/drop is available even if popup content is hidden.

## M2 - Proof

- `cargo fmt --package fret-ui-editor -- --check` passes.
- `cargo nextest run -p fret-ui-editor --features imui color_edit --no-fail-fast` passes.
- `cargo nextest run -p fret-ui-editor --features imui --test imui_surface_policy --no-fail-fast`
  passes.
- Full `fret-ui-editor --features imui`, layering, workstream catalog, source, skills, and diff
  checks pass.
