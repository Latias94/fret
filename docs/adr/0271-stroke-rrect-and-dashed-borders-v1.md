---
title: Stroke RRect and Dashed Borders v1
status: Draft
date: 2026-02-13
---

# ADR 0271: Stroke RRect and Dashed Borders v1

## Context

Dashed borders are a common UI affordance (e.g. shadcn/ui uses `border-dashed` in examples like
tasks) and an editor UX primitive (selection rectangles, docking drop zones, drag previews).

ADR 0030 explicitly defers “dashed borders” as part of `SceneOp::Quad` in P0 because the feature is
high entropy: once standardized, we must lock edge cases early (rounded corners, per-edge widths,
phase anchoring, snapping, transforms, clip interaction).

We want first-class dashed borders without turning `SceneOp::Quad` into a general “stroke API”.

## Decision

Add a dedicated stroke primitive for rounded rectangles:

- `SceneOp::StrokeRRect { rect, stroke, stroke_paint, corner_radii, style }`
- `DashPatternV1 { dash, gap, phase }`
- `StrokeStyleV1 { dash: Option<DashPatternV1> }`

This keeps `SceneOp::Quad` semantics stable while enabling dashed borders through “stroke-like”
semantics (consistent with ADR 0030’s future option to prefer stroke primitives over bloating
`Quad`).

## Semantics (v1)

### Units

- `dash`, `gap`, `phase` are defined in **logical pixels** (`Px`).
- The renderer multiplies them by `scale_factor` to evaluate the pattern in physical pixels.

### Pattern evaluation

- `period = dash + gap`
- No perimeter-fitting: the renderer does **not** adjust the pattern to evenly divide the perimeter.
- If `dash <= 0` or `period <= 0`, the dash mask is disabled (stroke renders as solid).
- Coverage rule (phase sign convention):
  - Let `s` be the perimeter coordinate (in physical pixels) and `phase` be the dash phase (also in physical pixels).
  - Define `m = (s + phase) mod period` (Euclidean modulo into `[0, period)`).
  - The dash is **on** iff `m < dash`.

### Perimeter parameterization (rounded rect)

We define a stable perimeter coordinate `s` (in physical pixels):

- Anchor: the top edge, at `(x + r_tl, y)` (immediately after the top-left corner radius).
- Direction: clockwise.
- Segments: top edge → TR arc → right edge → BR arc → bottom edge → BL arc → left edge → TL arc.
- Corner arcs contribute `(π/2) * r` each (quarter circle).

### Interaction with borders

- Stroke coverage is **inside-aligned** (consistent with the existing quad border semantics).
- Dashing masks the stroke’s coverage (i.e. it gates the stroke alpha), not the paint itself.
- Per-edge stroke widths are supported (`Edges`).

### Pixel snapping

- When an authoring layer snaps bounds to device pixels, dash evaluation uses the snapped geometry
  so the pattern does not “swim” relative to the pixels.

### Transforms and clipping

- Dashes are evaluated in the same local space as the rrect primitive; transforms deform the shape
  and its pattern consistently.
- Dash masking must respect the clip and mask stacks (no leaking outside clips).

## Consequences

- We can implement dashed borders without changing `SceneOp::Quad` (ADR 0030 constraint).
- The new primitive provides a future extension point for stroke semantics (joins/caps/dashes on
  paths) without duplicating policy into fill primitives.
- This ADR intentionally does not define a general `StrokePath` surface; it limits scope to the
  rrect case needed by UI parity and editor primitives.

## Evidence / implementation anchors

- Contract types:
  - `crates/fret-core/src/scene/stroke.rs`
  - `crates/fret-core/src/scene/mod.rs` (`SceneOp::StrokeRRect`)
- Renderer:
  - `crates/fret-render-wgpu/src/renderer/render_scene/encode/ops.rs`
  - `crates/fret-render-wgpu/src/renderer/render_scene/encode/draw/quad.rs`
  - `crates/fret-render-wgpu/src/renderer/shaders.rs` (`dash_params`, `rrect_perimeter_s`)
- Conformance gate:
  - `crates/fret-render-wgpu/tests/dashed_border_conformance.rs`

## Related

- ADR 0030: `docs/adr/0030-shape-rendering-and-sdf-semantics.md`
- Workstream: `docs/workstreams/quad-border-styles-v1.md`
