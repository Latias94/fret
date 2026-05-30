# Material 3 DatePicker Locale Strings Packet v1 - TODO

Status: Closed
Last updated: 2026-05-28

## Tasks

- [x] M3DPLS-010 - Source-backed boundary pass
  Goal: Re-read Compose DatePicker string/description semantics and classify owner layer.
  Result: The issue is `material_foundation + material_recipe`; no `fret-ui` mechanism gap was
  found.

- [x] M3DPLS-020 - Material string helpers and recipe wiring
  Goal: Add DatePicker helpers over `I18nService` and route docked/modal DatePicker labels through
  them.
  Result: `foundation::strings`, `DatePicker`, and `Button::a11y_label` cover visible text,
  accessibility labels, date descriptions, and modal affordances.

- [x] M3DPLS-030 - Regression proof and closeout docs
  Goal: Add focused automation/bootstrap tests, update the component matrix, and close stale
  residual-risk notes.
  Result: DatePicker registry tests and bootstrap Fluent formatting tests pass; packet docs and
  matrix entries record the closed state.
