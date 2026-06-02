# Flex RTL Physical Placement v1 - Design

Status: Closed
Last updated: 2026-05-30

## Problem

`direction-infrastructure-v1` extracted shared component-layer RTL policy, but `fret-ui` Flex rows
still compute physical layout as LTR. This means direction-aware components can navigate in RTL
while their underlying row geometry remains left-to-right unless each recipe hand-rolls placement.

## Boundary

- `fret-core` owns the portable `LayoutDirection` enum.
- `crates/fret-ui` owns mechanism-level physical layout outcomes.
- `ecosystem/*` crates continue to own component-specific policy such as keyboard navigation,
  indicator mirroring, and visual affordances.

## Decision

Capture the current `LayoutDirection` provider value on `AnyElement` during element construction,
copy it into `ElementRecord` during mount, and let layout code read the direction from the node's
record. This avoids adding a required field to `FlexProps`, which would force broad literal churn
across component crates and examples.

For this first slice, horizontal Flex rows use `FlexDirection::RowReverse` under RTL. Vertical Flex
and logical edge/inset remapping are left unchanged.

## Residuals

- Taffy 0.9 does not expose a full CSS `direction` style field, so this slice models horizontal
  row physical placement with `row-reverse`.
- Logical margins, logical insets, and cross-axis column mirroring remain follow-ons.
- Cached subtrees keep the provider snapshot captured when they were built, matching the existing
  documented provider/view-cache contract.
