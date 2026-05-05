# ImUi Color Edit Alpha Preview Options v1 Milestones

Status: Closed
Last updated: 2026-05-05

## M0 - Scope

- Chose alpha preview modes because local `repo-ref/imgui/imgui.h` and
  `repo-ref/imgui/imgui_widgets.cpp` show this as a concrete ColorButton/ColorEdit parity axis
  after the popup became usable.

## M1 - Implementation

- Added `ColorEditAlphaPreview::{Checkerboard, Opaque, NoBackground, Half}`.
- Added `ColorEditOptions::alpha_preview` with checkerboard as the default.
- Updated root and preset swatches to use the selected preview mode.
- Added focused tests for option coverage and opaque-preview alpha handling.

## M2 - Proof

- `cargo fmt --package fret-ui-editor -- --check` passes.
- `cargo nextest run -p fret-ui-editor --features imui color_edit --no-fail-fast` passes.
- Full `fret-ui-editor --features imui`, layering, workstream catalog, source, skills, and diff
  checks pass.
