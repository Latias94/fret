# BottomSheet Chrome Alias Packet v1

Date: 2026-05-28
Task: M3BS-010
Status: complete

## Truth

- Bottom sheet chrome selectors must identify the sheet surface, not the scrim or drag handle.
- Chrome aliases must not change sheet layout or scene output.
- Modal bottom sheet should inherit the same sheet-surface chrome alias from `DockedBottomSheet`.

## Layer Classification

- `material_recipe`: owns bottom sheet part-id taxonomy and where anchors are installed.
- `material_foundation`: owns the reusable hidden diagnostic anchor helper.
- `kit_policy`: modal overlay dismissal/focus remains unchanged.
- `mechanism`: no new mechanism is needed for rectangular sheet chrome aliases.

## First Slice

Add hidden full-region diagnostic anchors for `<base>.chrome` inside `DockedBottomSheet`. Since
`ModalBottomSheet` renders its sheet with `DockedBottomSheet` and a derived `<base>.sheet` id, it
should automatically expose `<base>.sheet.chrome`.

## Proof

- Automation-surface test for `m3-bottom-sheet.chrome` and `m3-modal-bottom-sheet.sheet.chrome`.
- Bottom-sheet headless golden suite remains unchanged.

## Result

Implemented through `foundation::test_id::diagnostic_anchor` inside `DockedBottomSheet`. The
automation-surface red/green test passes and `material3_headless_bottom_sheet_suite_goldens_v1`
passes without updating goldens.
