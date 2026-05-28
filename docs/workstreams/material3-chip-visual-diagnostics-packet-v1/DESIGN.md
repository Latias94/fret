# Material 3 Chip Visual Diagnostics Packet v1 - Design

Status: Closed
Last updated: 2026-05-28

## Problem

The Material3 matrix kept AssistChip, SuggestionChip, FilterChip, and InputChip visual follow-ons
open with the condition "only if gallery diagnostics show spacing/elevation drift." The existing
chip script covered action state, not representative visual chrome geometry.

## Truth

- Chip recipes own label/icon composition, selected semantics, variant chrome, and trailing action
  composition.
- Material foundation owns shared state-layer/ripple and minimum interactive target sizing.
- A focused gallery diagnostics script should prove root/chrome geometry for representative flat,
  elevated, selected, and trailing-icon chip states before escalating to component code.

## Boundaries

- Do not move chip roving policy to `fret-ui-kit` in this packet.
- Do not add exact per-paint or elevation inspection mechanisms without a concrete consumer.
- Treat the visual follow-on as closed if current gallery root/chrome diagnostics pass.

## Reference Axis

- Compose Material3 chip components for selected state, chip variants, and minimum touch target
  outcomes.
- Fret Material3 foundation for shared indication and minimum interactive sizing.
- Fret UI Gallery State Matrix as the diagnostics host for representative chip states.
