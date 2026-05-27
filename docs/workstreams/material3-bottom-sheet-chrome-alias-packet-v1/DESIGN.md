# Material 3 BottomSheet Chrome Alias Packet v1

Status: Closed
Last updated: 2026-05-28

## Why This Lane Exists

The Material3 overlay/feedback packet intentionally withheld `bottom_sheet.chrome` and
`modal_bottom_sheet.sheet.chrome` because visible or layout-participating markers changed headless
sheet sizing. The later diagnostic-anchor work proved a layout-only hidden marker pattern that can
represent rectangular chrome bounds without paint or focus effects.

## Target State

- Docked bottom sheets expose `<base>.chrome`.
- Modal bottom sheets expose `<base>.sheet.chrome`.
- The aliases are hidden, non-focusable, and layout-only.
- Existing bottom-sheet headless goldens do not drift.

## Layer Ownership

- `ecosystem/fret-ui-material3/src/bottom_sheet.rs`: Material sheet part ids and chrome aliases.
- `ecosystem/fret-ui-material3/src/foundation/test_id.rs`: reusable hidden diagnostic anchors.
- `ecosystem/fret-ui-kit`: existing modal overlay/focus policy only.
- `crates/*`: out of scope.

## In Scope

- Add red/green automation proof for bottom-sheet chrome aliases.
- Add hidden full-region chrome aliases to `DockedBottomSheet`.
- Confirm modal sheets inherit `sheet.chrome`.
- Keep bottom-sheet headless goldens stable.

## Out Of Scope

- New bottom sheet gestures or drag mechanics.
- Modal overlay policy changes.
- Sheet motion timing changes.
- Any `crates/*` diagnostic contract.

## Closeout Condition

This lane can close when automation selectors, bottom-sheet goldens, check/clippy, JSON/catalog, and
matrix updates pass.

Closeout result on 2026-05-28: closed. `DockedBottomSheet` now installs a hidden full-region
diagnostic anchor for `<base>.chrome`; modal sheets inherit `<base>.sheet.chrome`. Bottom-sheet
headless goldens pass without refresh.
