# Material3 Tabs Leading Icon v1 - Handoff

Status: Closed
Last updated: 2026-05-30

## Current State

The leading-icon follow-on is implemented and closed. The public API is
`TabItem::leading_icon(IconId)`.

## What Changed

- Leading-icon tabs now render icon + label content.
- Icon size is token-backed at 24px for primary and secondary tabs.
- Icon color is routed through primary and secondary navigation-tab tokens.
- Primary content-sized indicators now read real label/icon layout ids instead of wrapper scope ids.
- `tabs_state` has a focused regression for icon size, 8px gap, and content indicator coverage.

## Next Follow-Ons

- Add the stacked 72dp icon + label `Tab` shape.
- Add tab row divider rendering.
- Add gallery snippets for primary/secondary, fixed/scrollable, label-only/leading-icon tabs.
- Audit RTL indicator positioning and selected-tab auto-scroll.
