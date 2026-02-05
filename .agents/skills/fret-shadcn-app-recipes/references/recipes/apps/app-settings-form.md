# Recipe: Settings form (fields + keyboard nav)

Goal: build a dense settings page (labels + inputs/select/switch) that is consistent, accessible,
and easy to debug.

## Building blocks

- `Field`, `Label`, `Input`, `Textarea`, `Select`, `Switch` from `fret-ui-shadcn`.
- Use tokens for row height, paddings, and spacing (see `../../mind-models/mm-layout-and-sizing.md`).

## Checklist

- Tab order is predictable; focus ring is visible (focus-visible).
- Disabled controls are visually distinct and not clickable (command gating + disabled styling).
- Validation/error affordances don’t cause layout jumps.

## Regression gates (recommended)

- Add `test_id` per row and key controls so scripts can click/type reliably.
- Add a `tools/diag-scripts/` repro that tabs through the form and captures a bundle at the end.
