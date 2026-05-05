# ImUi Color Edit Model Split v1 Milestones

Status: Closed
Last updated: 2026-05-05

## M0 - Scope

- Identified `ColorEdit` as the immediate architecture hazard after the popup, alpha, HSV, numeric,
  and options slices.
- Chose an internal model split instead of a shared color crate because there is not yet a second
  framework consumer for the editor numeric text and popup mode policy.

## M1 - Implementation

- Added `controls/color_edit/model.rs`.
- Moved `HsvColor`, `ColorNumericInputMode`, numeric mode selection, hex/numeric parsing,
  RGB/HSV conversion, coordinate normalization, sanitization, and a11y value text helpers.
- Added `controls/color_edit/tests.rs` and kept tests colocated with the parent module.
- Updated the policy test so helper ownership is checked in `model.rs`.

## M2 - Proof

- `cargo fmt --package fret-ui-editor -- --check` passes.
- Focused `color_edit` nextest coverage passes after the split.
- Editor IMUI source-policy, adapter smoke, full `fret-ui-editor --features imui`, layering,
  workstream catalog, source, skills, and diff checks pass.
