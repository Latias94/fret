# Direction Infrastructure v1 - Handoff

Status: Closed
Last updated: 2026-05-30

## What Changed

- Added shared horizontal RTL arrow semantics to `fret-ui-kit::primitives::direction`.
- Added shared horizontal visual item position helper to `fret-ui-kit::primitives::direction`.
- Routed `fret-ui-kit::primitives::roving_focus_group` through the shared helper.
- Kept `fret-ui-shadcn::rtl` as a local recipe facade while delegating its horizontal visual
  position helper to kit.
- Migrated Material3 ChipSet, SegmentedButton, and Tabs horizontal navigation to the shared helper.

## Residuals

- Flex RTL physical placement is not implemented here.
- Material3 has remaining component-specific direction sites that should be migrated when those
  components are next touched.

## Next Follow-On

Open a mechanism-layer lane for `fret-ui` layout direction input and Flex row physical RTL
placement, with shadcn ButtonGroup/AvatarGroup and Material3 TabRow as representative consumers.
