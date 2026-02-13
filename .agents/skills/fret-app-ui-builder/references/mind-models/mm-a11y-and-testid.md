# Mind model: Semantics + `test_id` (automation-first)

Goal: make automated repros stable so you stop shipping regressions.

## Guidelines

- Every interactive target used by a repro script should have a stable `test_id`.
  - Prefer component-level IDs (`select-trigger`, `menu-item-apple`) rather than layout-derived selectors.
- Add semantics roles/labels that match intent (button/menu/listbox/option).
  - This improves both accessibility and automation signal.

## Script stability

When you create `tools/diag-scripts/<scenario>.json`:

- prefer `wait_until exists` on a `test_id` instead of “wait frames” as a primary sync,
- capture a bundle right after the relevant state transition (open/close/select),
- only add screenshots if geometry/semantics gates are insufficient.

## See also

- `fret-diag-workflow` (scripted repro + packaging)
