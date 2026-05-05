# ImUi Color Edit Popup Swatches Split v1 Milestones

Status: Closed
Last updated: 2026-05-05

## M0 - Scope

- Chose preset swatches as the next smallest popup submodule because they contain distinct
  selection styling, alpha-preserving commit behavior, and popup-close policy.

## M1 - Implementation

- Added `controls/color_edit/popup/swatches.rs`.
- Moved `preset_swatches` and `preset_swatch` into the new module.
- Kept `popup.rs` responsible for popup content ordering only.
- Updated `imui_surface_policy` to check preset helpers in the new file.

## M2 - Proof

- `cargo fmt --package fret-ui-editor -- --check` passes.
- `cargo nextest run -p fret-ui-editor --features imui color_edit --no-fail-fast` passes.
- Full `fret-ui-editor --features imui`, layering, workstream catalog, source, skills, and diff
  checks pass.
