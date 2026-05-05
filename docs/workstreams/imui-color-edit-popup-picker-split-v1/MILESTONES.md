# ImUi Color Edit Popup Picker Split v1 Milestones

Status: Closed
Last updated: 2026-05-05

## M0 - Scope

- Chose picker controls as the next smallest popup submodule because they contain distinct pointer
  capture, preview-gradient, accessibility value, and HSV/alpha commit behavior.

## M1 - Implementation

- Added `controls/color_edit/popup/picker.rs`.
- Moved `hsv_picker`, `sv_picker`, `hue_bar`, `alpha_bar`, preview stacks, gradient overlays,
  thumb/spacer helpers, and picker-local commit handlers.
- Kept shared checkerboard and fill-preview helpers in `popup.rs` because preset previews still use
  them.
- Updated `imui_surface_policy` to check picker helpers in the new file.

## M2 - Proof

- `cargo fmt --package fret-ui-editor -- --check` passes.
- `cargo nextest run -p fret-ui-editor --features imui color_edit --no-fail-fast` passes.
- Full `fret-ui-editor --features imui`, layering, workstream catalog, source, skills, and diff
  checks pass.
