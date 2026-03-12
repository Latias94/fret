---
title: Path Stroke Style v2 (Join/Cap/Miter + Dash)
status: Draft
date: 2026-02-16
---

# ADR 0277: Path Stroke Style v2 (Join/Cap/Miter + Dash)

## Context

Fret’s vector path system (`fret-core` `PathService`) supports stroking paths, but the stroke style
surface is currently **width-only**:

- `PathStyle::Stroke(StrokeStyle { width })`
- `StrokeStyle` has no join/cap/miter semantics and no dash pattern.

This leaves common UI rendering semantics either unexpressible or forced into approximation
patterns (many quads / per-widget hacks), which:

- increases draw op count,
- makes results less deterministic across scale factors and transforms,
- and complicates portability for wasm/mobile.

We already have a “dashed stroke” contract for rounded-rect strokes (`SceneOp::StrokeRRect`) in
ADR 0271. Vector path strokes should converge on compatible dash semantics where possible.

## Decision

Introduce a **v2 stroke style** for vector path stroking that is bounded and portable, without
breaking existing v1 callsites:

- Add `StrokeStyleV2` (width + join/cap/miter + optional dash pattern).
- Add `PathStyle::StrokeV2(StrokeStyleV2)`.
- Keep `PathStyle::Stroke(StrokeStyle { width })` supported as the width-only v1 form.

Renderers and caches must treat v2 style fields as part of the path preparation key and must keep
fallback/degradation behavior deterministic.

## Semantics (v2)

### Units

- `width` and dash fields are expressed in **logical pixels** (`Px`).
- `PathConstraints.scale_factor` is used during tessellation and caching; the renderer must not
  apply hidden additional scaling heuristics.

### Join/cap/miter

`StrokeStyleV2` defines:

- `join`: `Miter | Bevel | Round`
- `cap`: `Butt | Square | Round`
- `miter_limit`: finite, clamped to a safe range; used only when `join == Miter`

The default renderer maps these directly to the underlying tessellator implementation (lyon).

### Dash pattern

Dash uses the same base model as ADR 0271:

- `DashPatternV1 { dash_px, gap_px, phase_px }`
- `period = dash + gap`
- no “perimeter fitting” (do not adjust to evenly divide lengths)
- Coverage rule (phase sign convention): `m = (s + phase) mod period`; dash is **on** iff `m < dash`.

For vector paths, we must define a stable anchoring rule:

- Dash phase is anchored at the **start of each subpath** (the first `MoveTo` that begins an
  active contour).
- The pattern advances along the contour in command order.
- `Close` contributes the closing segment length; the pattern continues through the close.
- For multi-contour paths, the phase resets per contour (no cross-contour continuation).

If `dash <= 0` or `period <= 0`, dashing is disabled and the stroke renders as solid.

### Transforms

Vector path tessellation is performed in local space. Transforms are applied to generated geometry
during scene encoding. Under non-uniform transforms, strokes and their dash patterns deform; this
is expected and must remain deterministic (no backend-specific “corrections”).

## Deterministic degradation (portability)

Backends must degrade deterministically when a feature is unsupported:

- If dash is unsupported, degrade to solid stroke (dash disabled).
- If round join/cap is unsupported, degrade to bevel/butt.

The v2 contract must keep these degradations observable via existing diagnostics/perf plumbing when
possible, but must not introduce unbounded state surfaces.

## Consequences

- Vector stroke semantics become expressible without bloating fill primitives or scene ops.
- Existing v1 width-only stroke users remain valid and stable.
- The renderer path cache key must include v2 style fields (join/cap/miter/dash) to remain
  deterministic.
- Conformance coverage becomes mandatory because stroke semantics are visually sensitive (joins,
  caps, dash anchoring).

## Evidence / implementation anchors

Anchors:

- Contract types: `crates/fret-core/src/vector_path.rs` (`StrokeStyleV2`, `StrokeJoinV1`, `StrokeCapV1`, `PathStyle::StrokeV2`)
- Renderer path prep + cache keys: `crates/fret-render-wgpu/src/renderer/path.rs` (`mix_path_style`, `build_dashed_lyon_path`, `tessellate_path_commands`, `metrics_from_path_commands`)
- Conformance: `crates/fret-render-wgpu/tests/path_stroke_style_v2_conformance.rs` (GPU readback: join/cap/dash across scale factors)

## Related

- ADR 0271: `docs/adr/0271-stroke-rrect-and-dashed-borders-v1.md` (dash model)
- Workstream: `docs/workstreams/path-stroke-style-v2/path-stroke-style-v2.md`
