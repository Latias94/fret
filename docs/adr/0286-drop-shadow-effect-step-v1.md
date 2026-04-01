---
title: Drop Shadow Effect Step v1 (Bounded, Blur-Based)
status: Accepted
date: 2026-02-18
---

# ADR 0286: Drop Shadow Effect Step v1 (Bounded, Blur-Based)

## Context

Fret already supports a **portable, no-blur elevation baseline** via `ShadowStyle` (ADR 0060) and a
cheap, single-layer `TextShadowV1` (ADR 0283).

Many UI surfaces (cards, popovers, menus, dialogs) also need a **blur-based** drop shadow that is:

- **bounded** (computation bounds; ADR 0117),
- **portable** to wasm/WebGPU and mobile GPUs,
- **deterministic** under missing capabilities / budget pressure,
- and **testable** with conformance + perf gates.

Without a stable mechanism surface, authors tend to:

- emit many quads (high op count, recipe drift), or
- build bespoke offscreen pipelines at call sites (hard to gate, hard to keep portable).

## Goals

- Add a bounded, blur-based shadow mechanism that works for general UI content (not just text).
- Keep the vocabulary small and deterministic.
- Avoid an unbounded “custom shader” contract.

## Non-goals

- CSS `filter: drop-shadow()` parity (arbitrary spread/inset/stack semantics).
- Inner shadows.
- Arbitrary user-provided WGSL shaders.

## Decision

Add a new effect step variant to `fret-core::scene::EffectStep`:

- `EffectStep::DropShadowV1(DropShadowV1)`

Where `DropShadowV1` is a bounded parameter set:

- `offset_px`: logical pixel offset (pre-scale-factor),
- `blur_radius_px`: bounded blur radius (clamped),
- `downsample`: bounded downsample hint (1–4),
- `color`: solid shadow color (premultiplied at render time).

The step is intended for `EffectMode::FilterContent`.

### Mode behavior

- Under `EffectMode::FilterContent`: the renderer MUST attempt to render the shadow per Semantics.
- Under `EffectMode::Backdrop`: the renderer MUST deterministically degrade by skipping the step
  (no shadow).

Rationale:

- The step is defined as “shadow from content coverage”, which requires a FilterContent intermediate.
- Backdrop-mode shadows are a different mechanism surface (and would require sampling semantics
  that are easy to misuse).

## Semantics (v1)

Given an effect scope with computation bounds `bounds` (ADR 0117):

1. Render children into the FilterContent intermediate (existing behavior).
2. Build a **shadow image** from children coverage:
   - Downsample within budgets (deterministic), blur, then upscale back.
   - The renderer may blur RGBA but MUST treat alpha as the coverage source.
3. Composite the shadow **behind** the original content, within `bounds`:
   - shadow is translated by `offset_px`,
   - shadow is tinted by `color`,
   - composition is `shadow under content` (does not darken opaque content).
4. The step is scissored to `bounds`:
   - `bounds` are computation bounds, not an implicit clip (ADR 0117),
   - shadow does not implicitly expand beyond bounds; authors must allocate larger bounds to see
     larger shadows.

## Deterministic degradation (ordered)

1. If `EffectMode::Backdrop`: skip.
2. If intermediate budgets / scratch targets cannot satisfy the blur: skip.

## WebGPU / WGSL portability requirements

- The implementation MUST remain WebGPU-valid WGSL (no non-uniform `textureSample` requirements).
- Prefer `textureLoad` + manual bilinear sampling where needed.

## Consequences

Pros:

- Ecosystem authors can implement elevation recipes with a stable, bounded surface.
- Renderer can centralize budgeting, intermediate reuse, and gate coverage.

Cons:

- Adds pipeline variants and GPU bandwidth for blur-based shadows.

## Tracking

- Workstream: `docs/workstreams/renderer-drop-shadow-effect-v1/renderer-drop-shadow-effect-v1.md`
- TODO: `docs/workstreams/renderer-drop-shadow-effect-v1/renderer-drop-shadow-effect-v1-todo.md`
- Milestones: `docs/workstreams/renderer-drop-shadow-effect-v1/renderer-drop-shadow-effect-v1-milestones.md`
