# Material 3 Expressive Alignment v1 (Milestones)

This document defines milestone checkpoints for aligning Material 3 Expressive outcomes in Fret.

Plan: `docs/workstreams/material3-expressive-alignment-v1-refactor-plan.md`  
TODO: `docs/workstreams/material3-expressive-alignment-v1-todo.md`

## M0: Evidence-ready baseline

Exit criteria:

- UI gallery surfaces exist for the target components with stable `test_id`.
- At least one diag script v2 exists and runs with:
  - `FRET_DIAG_GPU_SCREENSHOTS=1`
  - `--pack` (shareable artifact)
- At least one headless gate exists for "stable structure while pressed" (or an equivalent invariant).

## M1: Controls parity (checkbox / switch / icon toggle)

Exit criteria:

- Checkbox supports indeterminate outcome (tri-state) with:
  - semantics mapping (`checked: None`) and diag predicate gate (`checked_is_none`),
  - screenshot evidence in both Standard and Expressive variants.
- Switch thumb/track icon + sizing outcomes match upstream references for:
  - selected/unselected, enabled/disabled, with/without icons.
- Icon toggle button expressive shape morph is gated and stable.

## M2: Segmented / slider parity

Exit criteria:

- Segmented buttons: selection indicator geometry, keyboard navigation, and state layers are aligned and gated.
- Slider: value semantics, thumb sizing, and interaction outcomes are aligned and gated.

## M3: Input fields parity (text field / select / autocomplete)

Exit criteria:

- Text field: error/disabled/supporting-text outcomes match upstream, with at least one diag gate per major state.
- Select/autocomplete: overlay anchoring + keyboard navigation + dismissal behavior are stable and gated.

## M4: Navigation + surfaces parity (tabs / nav / dialogs)

Exit criteria:

- Tabs: indicator motion + selection semantics are aligned and gated.
- Navigation components: selection indicator + expressive sizing are aligned and gated.
- Dialog/bottom sheet/snackbar/tooltip: elevation/scrim/motion outcomes are aligned and gated (dismiss policy stays in
  ecosystem).

