# Material3 Tabs RTL Indicator v1 - Milestones

Status: Closed
Last updated: 2026-05-30

## M0 - Lane Setup

Closed. The lane scope is limited to component-owned Tabs direction policy and public Material3
direction context exposure.

## M1 - Direction-Aware Tabs

Closed. Tabs now resolves `LayoutDirection`, mirrors horizontal arrow semantics for RTL, and uses
direction-aware fallback active-indicator positioning.

## M2 - Closeout

Closed. Remaining RTL gaps are intentionally outside this lane:

- global Flex/layout-engine RTL physical placement,
- scrollable selected-tab auto-scroll,
- visual gallery/diagnostic examples for RTL tab rows.
