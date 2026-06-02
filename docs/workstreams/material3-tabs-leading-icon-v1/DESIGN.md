# Material3 Tabs Leading Icon v1 - Design

Status: Closed
Last updated: 2026-05-30

## Problem

`material3-tabs-api-breadth-v1` closed the primary/secondary tab variant surface, but left icon and
label tabs as a follow-on. Compose Material3 exposes `LeadingIconTab` as a 48dp tab with an icon in
front of the text label, a 24dp icon, and an 8dp gap. Fret Material3 only supported label-only
`TabItem`s, so users could not express that source-backed shape.

## Source Facts

- Compose `LeadingIconTab` renders an icon before the text label and keeps the small 48dp tab
  height.
- Compose uses `TextDistanceFromLeadingIcon = 8.dp`.
- Primary and secondary navigation-tab tokens both define `IconSize = 24.dp`.
- Primary active icon color follows primary, inactive icon color follows on-surface-variant.
- Secondary active icon color follows on-surface, inactive icon color follows on-surface-variant.

Reference files:

- `repo-ref/compose-multiplatform-core/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Tab.kt`
- `repo-ref/compose-multiplatform-core/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/PrimaryNavigationTabTokens.kt`
- `repo-ref/compose-multiplatform-core/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/SecondaryNavigationTabTokens.kt`

## Shipped Shape

- Add `TabItem::leading_icon(IconId)` for the Compose `LeadingIconTab` slice.
- Keep label-only tabs as the default and preserve existing selection, roving focus, semantics, and
  active-indicator behavior.
- Render leading-icon tabs as horizontal icon + label content with the Material 8px gap.
- Route icon size and icon color through typed tabs token helpers for primary and secondary tabs.
- Seed the missing secondary v30 icon aliases from the Compose secondary token facts.
- Fix the layout probe bug that stored wrapper scope ids instead of the actual text/icon layout
  element ids.

## Intentional Residuals

This lane intentionally does not implement the taller stacked icon + label `Tab` shape. Compose
models that separately through the generic `Tab(icon, text)` path with a 72dp large tab height. That
needs a separate API decision because it changes tab height and content layout.
