# Material 3 BottomSheet Chrome Alias Packet v1 - Handoff

Status: Closed
Last updated: 2026-05-28

## Current State

This lane is closed.

`DockedBottomSheet` now exposes `<base>.chrome` via a hidden diagnostic anchor. `ModalBottomSheet`
renders its sheet through `DockedBottomSheet` with the derived `<base>.sheet` id, so it now exposes
`<base>.sheet.chrome`.

## Completed Tasks

- M3BS-010: gap packet and layer classification.
- M3BS-020: red/green automation selector proof.
- M3BS-030: bottom-sheet golden guard and closeout gates.

## Guardrails

- Do not alter modal overlay/focus policy.
- Do not refresh bottom-sheet goldens unless a real visual correction is proven.
- Do not add markers that change sheet width/height.
