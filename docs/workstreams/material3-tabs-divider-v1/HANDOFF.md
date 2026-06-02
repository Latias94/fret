# Material3 Tabs Divider v1 - Handoff

Status: Closed
Last updated: 2026-05-30

## Current State

The TabRow divider follow-on is implemented and closed.

## What Changed

- Tabs render a bottom divider by default.
- `m3-tabs.divider` is exposed as a stable diagnostics/test id.
- Divider height/color are routed through tabs token helpers with generic divider fallbacks.
- Secondary navigation-tab divider aliases are seeded in v30.
- Focused tests prove the divider spans the row bottom and shares the bottom edge with the active
  indicator.

## Next Follow-Ons

- Public divider customization or disable API.
- RTL indicator positioning.
- Selected-tab auto-scroll.
- Gallery snippets for the completed Tabs matrix.
