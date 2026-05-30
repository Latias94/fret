# Material3 Layout Direction Provider Bridge v1

Status: Closed
Last updated: 2026-05-30

## Problem

The Flex RTL mechanism now reads `fret_core::LayoutDirection` from the element provider stack when
an `AnyElement` is built. Material3 components already have their own direction context
(`MaterialLayoutDirectionOverride`) and theme fallback (`md.sys.fret.layout.is-rtl`), but those
resolved values were not consistently installed as the core provider that layout mechanisms consume.

This creates a split-brain direction model:

- Material3 component behavior can be RTL-aware.
- Core layout can still see default LTR.
- Horizontal rows can keep LTR physical order even when Material3 logic uses RTL keyboard behavior.

## Scope

- Add a Material3 foundation helper that resolves Material layout direction and installs the core
  `LayoutDirection` provider for the subtree.
- Make explicit Material direction scopes (`with_material_layout_direction`) also provide the core
  direction.
- Use Tabs as the first consumer-level proof because it already has Material RTL keyboard and
  indicator behavior, and it builds a horizontal row where physical order is observable.

## Non-Goals

- Do not solve every logical edge (`padding-inline-start`, `margin-inline-end`, inset) in this lane.
- Do not claim full component visual parity for every Material3 component.
- Do not move Material policy into `crates/fret-ui`; the core layer should only consume the generic
  direction provider.

## Assumptions

- Area: closed lane handling
  - Assumption: `flex-rtl-physical-placement-v1` stays closed; this is a narrow follow-on.
  - Evidence: `docs/workstreams/flex-rtl-physical-placement-v1/WORKSTREAM.json`.
  - Confidence: Confident.
  - Consequence if wrong: this work would blur a closed mechanism lane with design-system adoption.

- Area: ownership
  - Assumption: Material3 direction fallback and override policy belongs in
    `ecosystem/fret-ui-material3::foundation`, while the core provider type stays
    `fret_core::LayoutDirection`.
  - Evidence: `docs/adr/0066-fret-ui-runtime-contract-surface.md`,
    `ecosystem/fret-ui-material3/src/foundation/context.rs`.
  - Confidence: Confident.
  - Consequence if wrong: component policy could leak into the mechanism layer.

- Area: first consumer
  - Assumption: Tabs is the right first consumer proof because its RTL behavior is already tested and
    its row geometry is easy to assert.
  - Evidence: `ecosystem/fret-ui-material3/src/tabs.rs`,
    `ecosystem/fret-ui-material3/tests/tabs_state.rs`.
  - Confidence: Likely.
  - Consequence if wrong: the bridge would still be correct, but the consumer proof might not cover
    the most fragile Material component.

## Parity Proof Note

- Truth: When Material3 resolves RTL from an explicit override or theme fallback, descendant
  elements that rely on core layout should capture RTL.
- Artifacts: `with_material_resolved_layout_direction` helper and context tests.
- Wiring: Tabs wraps its horizontal row subtree with the resolved Material direction provider.
- Proof: Tabs rendered under the RTL theme places the first logical tab to the physical right of the
  second logical tab.
- Residual risk: Logical edge padding and inline insets are still separate follow-ons.
