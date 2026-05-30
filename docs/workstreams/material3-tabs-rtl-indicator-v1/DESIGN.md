# Material3 Tabs RTL Indicator v1 - Design

Status: Closed
Last updated: 2026-05-30

## Problem

Material3 Tabs did not consume the Material layout direction context. That left horizontal keyboard
navigation LTR-only and kept active-indicator fallback geometry tied to logical index order even
when the theme declared RTL directionality.

The crate already had a Material layout direction provider in `foundation::context`, but the public
`fret_ui_material3::context` facade did not re-export it.

## Source Stack

- Compose Material3 `TabRow.kt`: fixed tab rows place tabs and indicators through relative
  placement, and the indicator offset path accounts for `LayoutDirection.Rtl`.
- Existing Fret Material3 components: `ChipSet` and `SegmentedButtonSet` already flip
  ArrowLeft/ArrowRight by `LayoutDirection`.
- Fret Material3 foundation context: `md.sys.fret.layout.is-rtl` and
  `with_material_layout_direction` define the local direction contract.

## Chosen Shape

- Resolve `LayoutDirection` once in `Tabs::into_element` from theme default plus tree-local
  Material context override.
- Route horizontal roving navigation through a small helper:
  - LTR: ArrowRight = forward, ArrowLeft = backward.
  - RTL: ArrowLeft = forward, ArrowRight = backward.
- Keep Home/End logical, matching existing Material3 set behavior.
- Mirror active-indicator fallback tab index when no measured tab geometry is available.
- Prefer measured tab/content bounds whenever present, so future layout-engine RTL mirroring can
  flow through without another component rewrite.
- Publicly re-export Material layout direction context helpers from
  `fret_ui_material3::context`.

## Boundary

This lane does not make `fret-ui` Flex layout globally RTL-aware. Tabs now consume direction
correctly where the component owns the policy, but physical row mirroring and selected-tab
auto-scroll remain separate follow-ons.
