# Material3 Tabs RTL Indicator v1 - Handoff

Status: Closed
Last updated: 2026-05-30

## What Changed

- Tabs consumes Material layout direction from theme/context.
- RTL horizontal roving navigation now follows Material logical direction semantics.
- Active-indicator fallback positioning mirrors logical tab index under RTL.
- `fret_ui_material3::context` now re-exports layout direction provider/resolver helpers.
- Tests cover helper math and a diagnostics integration scenario where RTL ArrowLeft moves from
  the first tab to the next logical tab without wraparound.

## Next Good Follow-Ons

1. Add a layout-engine workstream for global RTL physical placement in Flex-like rows.
2. Add selected-tab auto-scroll for scrollable TabRow.
3. Add RTL tab row gallery/diagnostic examples once physical mirroring exists.
