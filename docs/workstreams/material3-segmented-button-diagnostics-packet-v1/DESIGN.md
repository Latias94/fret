# Material 3 Segmented Button Diagnostics Packet v1 - Design

Status: Closed
Last updated: 2026-05-28

## Problem

The component matrix still left `SegmentedButtonSet` in known-follow-on state even though the
component already had a dedicated Material3 gallery roving-semantics script and focused Rust gates
for single/multi selection semantics and headless goldens. The missing work was a dedicated packet
that promoted the diagnostics suite and closed the row as diagnostics-aligned.

## Truth

- `SegmentedButtonSet` is a Material recipe component, not a new kit abstraction.
- The recipe owns single/multi selection semantics, per-segment chrome, checked state, and
  roving focus behavior.
- The shared Material foundation owns indication and minimum interactive target policy.
- Diagnostics should prove the gallery page exposes stable group/item semantics and roving focus
  behavior across the single-select, multi-select, and expressive variants.

## Boundaries

- Do not move segmented button group policy into `fret-ui-kit` unless another design system needs
  the same roving policy.
- Do not widen `crates/*`; no core mechanism gap is proven by this packet.
- Keep the gallery script and the focused Rust semantics/golden tests as the proof surface.
