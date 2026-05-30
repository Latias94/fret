# Material3 Tabs Stacked Icon v1 - Design

Status: Closed
Last updated: 2026-05-30

## Problem

`material3-tabs-leading-icon-v1` shipped Compose `LeadingIconTab`, but the generic Compose
`Tab(text, icon)` shape remained missing. That shape is not the same API: it stacks the icon above
the label and uses the large tab height.

## Source Decision

Truth axis: runtime layout and component behavior.

- Compose Material3 `TabBaselineLayout` uses `LargeTabHeight = 72.dp` when both text and icon are
  present.
- Compose `LeadingIconTab` remains a separate 48dp horizontal icon + label shape.
- Material Web v30 includes `md.comp.primary-navigation-tab.with-icon-and-label-text.container.height`
  at 64px. This lane intentionally does not use that web-token value for the Fret runtime layout,
  because the target is the Compose toolkit behavior for generic `Tab(icon, text)`.

## Shipped Shape

- Add `TabIconPlacement::{Leading, Stacked}`.
- Keep `TabItem::leading_icon(IconId)` as the horizontal 48dp convenience API.
- Add `TabItem::stacked_icon(IconId)` for the Compose generic stacked icon + label slice.
- Add `TabItem::icon(IconId, TabIconPlacement)` as the future-facing shared builder.
- Raise the whole tab row to 72px when any item uses the stacked layout, so fixed/mixed rows keep a
  single indicator baseline.
- Keep primary content-sized indicators measured from the actual label/icon layout probes.
- Seed Fret runtime alias tokens for 72px stacked primary/secondary row height.

## Residuals

This lane does not implement Compose baseline math exactly. It locks the user-visible shape:
72px row height, 24px icon size, vertical icon-over-label placement, and content-sized primary
indicator geometry. Exact first/last baseline offsets can be promoted only if a real text baseline
API exists or becomes necessary for visual QA.
