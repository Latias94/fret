# Material3 Foundation Deepening v1 Milestones

Status: Closed
Last updated: 2026-05-31

## M0 - Context Interface Deepened

Status: Complete

- Material recipes no longer bypass Material context for theme-authoritative direction.
- Residual overlay/popup direction consumers have focused RTL proof.

## M1 - Field Family Deepened

Status: Complete

- Shared Material field chrome lives behind a small private interface.
- TextField, Select, Autocomplete, and ExposedDropdown consume that interface.
- Duplicated label/slot/supporting-text/indicator math is deleted or clearly isolated as a token
  adapter.

## M2 - Token Matrix Split

Status: Complete

- Token source adapters, typed registry, and outcome fixture runners have explicit seams.
- Large visual fixture logic is reduced or split into family modules.
- Material token matrix tests remain the main proof surface.

## M3 - Lane Verified

Status: Complete

- Workstream docs name the final gate set and evidence anchors.
- Formatting, targeted tests, check/clippy, catalog, layering, and diff hygiene pass.
- Closeout records residual risks and any follow-ons.
