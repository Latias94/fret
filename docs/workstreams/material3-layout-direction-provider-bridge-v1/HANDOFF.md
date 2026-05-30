# Material3 Layout Direction Provider Bridge v1 Handoff

Status: Closed
Last updated: 2026-05-30

## Shipped Slice

Material3 foundation now bridges resolved layout direction into the core `LayoutDirection` provider.
Tabs consumes that bridge and has a diagnostics test proving RTL theme rendering mirrors physical tab
order.

## Residual Follow-Ons

- Logical edge helpers for padding/margins/insets in Material and shared ecosystem layers.
- Full Material3 RTL visual audit across chips, segmented buttons, slider, badge, and navigation.
- Gallery diagnostics that exercise Material3 RTL rows with stable `test_id` anchors.
