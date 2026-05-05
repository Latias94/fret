# ImUi Color Edit Popup Numeric Split v1 Milestones

Status: Closed
Last updated: 2026-05-05

## M0 - Scope

- Chose numeric rows as the next smallest popup submodule because they contain distinct validation,
  placeholder, draft sync, and commit behavior.

## M1 - Implementation

- Added `controls/color_edit/popup/numeric.rs`.
- Moved `color_numeric_inputs`, numeric field composition, error line rendering, row-height helper,
  and placeholder helper.
- Updated `imui_surface_policy` to check numeric helpers in the new file.

## M2 - Proof

- `cargo fmt --package fret-ui-editor -- --check` passes.
- `cargo nextest run -p fret-ui-editor --features imui color_edit --no-fail-fast` passes.
- Full `fret-ui-editor --features imui`, layering, workstream catalog, source, skills, and diff
  checks pass.
