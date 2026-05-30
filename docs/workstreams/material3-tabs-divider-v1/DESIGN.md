# Material3 Tabs Divider v1 - Design

Status: Closed
Last updated: 2026-05-30

## Problem

Material3 Tabs now cover primary/secondary variants plus leading and stacked icon item shapes, but
the TabRow bottom divider was still missing. Compose `PrimaryTabRow`, `SecondaryTabRow`, and their
scrollable variants default `divider` to `HorizontalDivider()`, then place the active indicator on
top of the divider at the row bottom.

## Source Facts

- Compose TabRow defaults `divider` to `HorizontalDivider()`.
- `HorizontalDivider` uses `DividerTokens.Thickness = 1.dp`.
- Fixed TabRow places divider at `tabRowHeight - divider.height`.
- Indicator is placed at `tabRowHeight - indicator.height`, so it shares the bottom edge and paints
  on top of the divider.
- Material Web v30 also exposes primary navigation-tab divider tokens. Fret should prefer
  navigation-tab-specific divider tokens when available, with generic divider tokens as fallback.

## Shipped Shape

- Add an internal absolute bottom divider layer to `Tabs`.
- Expose stable `m3-tabs.divider` diagnostics/test id.
- Route divider height/color through tabs token helpers:
  - primary uses generated `md.comp.primary-navigation-tab.divider.*` when present,
  - secondary uses seeded `md.comp.secondary-navigation-tab.divider.*`,
  - both fall back to generic `md.comp.divider.*`.
- Keep active indicator on the same bottom edge as the divider.

## Residuals

The divider is not yet user-disableable like Compose's `divider` slot. A later API breadth lane can
add a public slot or style override if app consumers need to suppress or customize it beyond theme
tokens.
