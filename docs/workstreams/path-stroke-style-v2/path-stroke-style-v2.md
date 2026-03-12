---
title: Path Stroke Style v2 (Join/Cap/Miter + Dash) — Workstream
status: active
date: 2026-02-16
scope: fret-core vector paths, tessellation/cache keys, fret-render-wgpu path pipeline, portability + conformance
---

# Path Stroke Style v2 (Join/Cap/Miter + Dash) — Workstream

This workstream extends Fret’s **vector path stroke semantics** beyond “width-only” to cover the
common UI renderer needs:

- stroke join (miter/bevel/round),
- stroke cap (butt/square/round),
- miter limit,
- dash pattern (dash/gap/phase),
- and a portability-first, deterministic fallback story (wasm/mobile included).

Detailed TODOs live in:

- `docs/workstreams/path-stroke-style-v2/path-stroke-style-v2-todo.md`

Milestone board (one-screen) lives in:

- `docs/workstreams/path-stroke-style-v2/path-stroke-style-v2-milestones.md`

## Why this exists

Today:

- `fret-core` vector paths support `PathStyle::Stroke(StrokeStyle { width })` only
  (`crates/fret-core/src/vector_path.rs`).
- The default renderer (`fret-render-wgpu`) already tessellates stroke paths via lyon, but the
  style surface is incomplete and cannot express join/cap/dash rules (`crates/fret-render-wgpu/src/renderer/path.rs`).

Most UI frameworks treat “stroke semantics” as a first-class concept. Without it, downstream UI
layers tend to approximate via many quads or per-widget hacks, which is:

- harder to batch,
- harder to keep deterministic under transforms/scale factors,
- and harder to make portable under WebGPU constraints.

## Goals (v2)

1. Add a **bounded** and **portable** stroke style surface for vector paths:
   - join, cap, miter limit, optional dash pattern.
2. Keep tessellation + caching deterministic:
   - style participates in the cache key (no hidden heuristics).
3. Avoid contract churn:
   - allow existing `PathStyle::Stroke(width-only)` users to keep working.
4. Leave at least one hard regression gate:
   - GPU readback conformance test(s) for join/cap/dash stability across scale factors.

## Current status

- Contract + ADR landed (v2 types are available in `fret-core`).
- Default renderer (`fret-render-wgpu`) supports v2 join/cap/miter/dash via lyon tessellation and
  deterministic dash segmentation.
- GPU readback conformance covers join/cap/dash properties across multiple scale factors.

## Non-goals (v2)

- A full “stroke paint” expansion for `SceneOp::Path` (gradients/materials on strokes).
- Non-scaling stroke under arbitrary non-uniform transforms (this requires a per-draw transform-aware
  contract; v2 will document the current behavior and keep it deterministic).
- High-level “CSS stroke” parity (line-dasharray corner cases, dash-offset percent semantics, etc.).

## Contract surface options (core)

We have two viable directions for the public `fret-core` contract:

### Option A — Evolve `StrokeStyle` in place (breaking)

Pros:

- simplest type graph (one style struct).

Cons:

- breaks all struct-literal callsites (`StrokeStyle { width: ... }` is widely used).
- requires careful semver consideration if `fret-core` is published.

### Option B (recommended) — Add `StrokeStyleV2` + `PathStyle::StrokeV2`

Pros:

- keeps existing v1 callsites unchanged (width-only stays valid).
- enables gradual adoption by ecosystem crates and apps.
- makes conformance testing and portability policy explicit per style version.

Cons:

- renderer and cache key code must handle two stroke variants.

Recommendation: **Option B**.

## Proposed v2 semantics (to lock)

Normative contract: ADR 0277 (`docs/adr/0277-path-stroke-style-v2.md`).

### Style fields (bounded)

`StrokeStyleV2` should be small and predictable:

- `width: Px` (logical px, scale-aware via `PathConstraints.scale_factor`)
- `join: StrokeJoinV1` (`Miter`, `Bevel`, `Round`)
- `cap: StrokeCapV1` (`Butt`, `Square`, `Round`)
- `miter_limit: f32` (clamped to a safe finite range)
- `dash: Option<DashPatternV1>` (reuse the existing contract type if possible)

### Dash semantics (compatible with dashed borders)

We should reuse the already-locked dash model from dashed borders where possible:

- pattern: `(dash_px, gap_px, phase_px)`
- units: logical px (scale-aware)
- no perimeter fitting / no attempt to “evenly divide” segment lengths

### Transform interaction

The current path pipeline tessellates in local space and applies transforms to vertices during
encoding. Under non-uniform transforms, strokes will deform. v2 must:

- define that deformation as *expected* (deterministic),
- and avoid “auto-correct” heuristics that are backend-specific.

### Portability + deterministic degradation

Degradation must be deterministic and observable. Examples (to decide explicitly):

- If dash is requested but unsupported in a backend/path pipeline, degrade to solid stroke.
- If round joins/caps are unsupported, degrade to bevel/butt.

The initial default renderer is expected to support all v2 fields via lyon stroke tessellation,
but the contract must still define fallbacks for future backends.

## Evidence / regression gates (required)

- Add at least one GPU readback conformance test that renders:
  - a polyline/path with corners (join),
  - end caps (cap),
  - and a dashed stroke (dash),
  at multiple scale factors (1.0 / 1.5 / 2.0) and compares sampled pixels or fingerprints.
- Add a small headless perf probe (optional) only if tessellation becomes a hotspot; do not add
  new gates without evidence.
