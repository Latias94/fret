---
title: Isolated Opacity (saveLayerAlpha) v1
status: Draft
date: 2026-02-13
---

# ADR 0272: Isolated Opacity (saveLayerAlpha) v1

## Context

Fret’s `SceneOp::PushOpacity/PopOpacity` defines a multiplicative opacity stack. This is a
zero-intermediate, zero-extra-pass mechanism and must remain the default for performance.

However, many UI visuals (CSS-like fades, disabled state, animated transitions, glass overlays)
require an **isolated opacity group** (often described as `saveLayer(alpha)`):

- children are composited into an offscreen intermediate first,
- then the intermediate is composited back to the parent with an additional alpha multiplier.

Without an explicit isolated-opacity contract, component ecosystems either:

- accept visual mismatches (especially with overlapping children), or
- misuse effect/compositing primitives in non-portable ways.

This ADR defines a bounded, portable isolated-opacity mechanism that:

- preserves strict in-order scene semantics,
- participates in renderer intermediate budgets and deterministic degradation,
- and remains wasm/mobile-friendly (bounds required, degradations explicit).

## Decision

### D1 — Isolated opacity is expressed as a compositing group alpha (preferred)

Extend the existing compositing group descriptor to carry an optional opacity multiplier:

- `CompositeGroupDesc { bounds, mode, quality, opacity }`

Semantics:

- The renderer renders the group’s children into an offscreen intermediate.
- On `PopCompositeGroup`, the renderer composites the intermediate into the parent using:
  - the requested `mode`, and
  - an additional premultiplied alpha multiplier `opacity` applied to the intermediate output.

API notes:

- `opacity` defaults to `1.0`.
- Builders should prefer fluent construction (e.g. `CompositeGroupDesc::new(...).with_opacity(...)`).

This reuses the existing “bounded intermediates + deterministic degradation” machinery and avoids
introducing a new stack/op surface.

### D2 — `PushOpacity` remains non-isolated and zero-cost

`SceneOp::PushOpacity/PopOpacity` semantics remain unchanged:

- it multiplies subsequent draw ops’ opacity,
- it does **not** imply offscreen isolation,
- and it is the default path for performance.

## Semantics (normative)

### Opacity

- `opacity` is a finite `f32`.
- Renderers clamp `opacity` to `[0, 1]`.
- `opacity == 1` is the identity (no additional change).

### Order and stacks

- Scene operation order remains authoritative.
- Compositing groups are **sequence points**: renderers must not reorder ops across group
  boundaries.
- Clip/mask/opacity stacks apply normally inside the group; the group opacity is applied only at
  the group composite boundary.

### Budgets and deterministic degradation

Compositing groups allocate intermediates and therefore must participate in renderer budgets.
When a group cannot be allocated within budgets:

1. The renderer SHOULD degrade intermediate resolution according to `quality` (existing ladder).
2. If no intermediate tier fits, the renderer MUST degrade deterministically:
   - behave as if the group was not isolated,
   - and treat `mode` as `Over`.
   - v1 approximation for `opacity` (deterministic):
     - renderers MAY ignore `opacity` when the group is not isolated (equivalent to
       `opacity = 1.0`).

This degradation is visual-only and must not affect layout or hit-testing.

## Consequences

- Component ecosystems gain a portable, bounded “CSS-like isolated fade” primitive.
- Renderers retain a zero-cost default opacity path, and can budget/telemetry isolated opacity
  explicitly.
- wasm/mobile targets have a clear fallback story (bounded degradation, no unbounded allocations).

## Acceptance criteria (recommended gates)

- Add a GPU conformance test that demonstrates the isolated vs non-isolated difference for
  overlapping children and validates the chosen deterministic degradation behavior under forced
  budget failure.

## Related

- Compositing groups and blend modes: `docs/adr/0247-compositing-groups-and-blend-modes-v1.md`
- Renderer budgets and deterministic degradation: `docs/adr/0118-renderer-intermediate-budgets-and-effect-degradation-v1.md`
- Scene ordering and batching: `docs/adr/0009-renderer-ordering-and-batching.md`
