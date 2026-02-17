---
title: Backdrop Warp Effect Step v1
status: Draft
date: 2026-02-17
---

# ADR 0284: Backdrop Warp Effect Step v1

## Context

Fret’s effect system currently supports bounded “image-space” steps that are sufficient for
fake-glass visuals (e.g. backdrop blur + color adjust + subtle dither). Some modern UI designs
also require **spatial distortion** of the already-rendered backdrop:

- refraction-like displacement behind a translucent surface,
- optional subtle chromatic aberration (RGB separation),
- and deterministic, bounded degradation on wasm/WebGPU and mobile GPUs.

Without a contract surface, authors must approximate via bespoke pipelines at call sites, which is
drift-prone, hard to conformance-test, and risky for portability (WebGPU/WGSL uniformity rules).

At the same time, a general “user-provided WGSL shader” contract is intentionally out of scope for
the renderer contract layer in v1:

- it explodes pipeline state space and hurts batching,
- it is difficult to make portable across native + WebGPU backends,
- and it complicates determinism, budgeting, and conformance gating.

## Decision

Add a bounded, portable **backdrop warp** effect step to the `EffectChain` vocabulary:

- Extend `fret-core::scene::EffectStep` with a new variant:
  - `EffectStep::BackdropWarpV1(BackdropWarpV1)`

The step is designed to work under `EffectMode::Backdrop` (sample already-rendered backdrop behind
the group). Under `EffectMode::FilterContent`, renderers MUST degrade deterministically (see
Semantics).

This surface is a mechanism/contract. Higher-level recipes (iOS-like glass tokens, normal map
assets, hover/press response curves, etc.) remain ecosystem policy.

## Contract surface (v1)

```rust
pub struct BackdropWarpV1 {
    /// Displacement strength in logical pixels (pre-scale-factor).
    /// Implementations MUST clamp to a small max (e.g. <= 24 px) deterministically.
    pub strength_px: Px,
    /// Spatial scale in logical pixels used by the warp field (clamped to a non-zero min/max).
    pub scale_px: Px,
    /// Phase/seed parameter for deterministic variation. Animation is achieved by changing this
    /// value explicitly; there is no hidden time dependency.
    pub phase: f32,
    /// Optional chromatic aberration magnitude in logical pixels (pre-scale-factor).
    /// A value of 0 disables chromatic aberration.
    pub chromatic_aberration_px: Px,
    /// Bounded warp vocabulary selector.
    pub kind: BackdropWarpKindV1,
}

pub enum BackdropWarpKindV1 {
    /// A portable analytic warp intended to be cheap and deterministic.
    Wave,
    /// Reserved for a lens-like warp in a future v1.x or v2 (capability-gated).
    /// Renderers may treat this as `Wave` in v1.
    LensReserved,
}
```

Notes:

- All fields MUST be sanitized deterministically (`NaN` → 0, clamp ranges, etc.).
- `phase` MUST be finite after sanitize (non-finite values become 0).
- Renderers MUST keep the vocabulary bounded (no open-ended strings, no shader sources).

## Semantics (v1)

### Mode behavior

- Under `EffectMode::Backdrop`:
  - the renderer samples the already-rendered backdrop behind the effect bounds,
  - applies the warp sampling step to the backdrop image,
  - then composites children on top (as existing backdrop effects do).

- Under `EffectMode::FilterContent`:
  - the renderer MUST degrade deterministically by skipping the step (no warp).
  - Rationale: the v1 contract is explicitly “backdrop warp”; content-warp can be a future surface.

### Sampling behavior

For each pixel `p` in the effect bounds, the renderer computes a displacement vector in logical
pixel units:

- `d_px = warp_field(kind, local_p, scale_px, phase) * strength_px`

The renderer then samples the backdrop at `p + d_px` (after converting to UV in the chosen
implementation), clamped to the backdrop image bounds. `bounds` scissoring MUST be applied (no
implicit expansion).

Chromatic aberration:

- if `chromatic_aberration_px > 0`, the renderer samples three nearby positions (bounded to a small
  max) and reconstructs RGB from those samples deterministically.
- The exact direction can be defined as aligned with `normalize(d_px)` with a deterministic fallback
  when `d_px` is near zero.

### Deterministic degradation order

When budgets/capabilities cannot satisfy the requested effect:

1. Disable chromatic aberration first (`chromatic_aberration_px` treated as 0).
2. Reduce warp strength (bounded clamp) according to quality/budget policy.
3. Skip the warp step entirely.

Degradation MUST be deterministic and observable via renderer perf counters and/or diagnostics
bundles (exact counter naming is renderer-specific, but the workstream requires a gate).

## Portability notes (WebGPU / WGSL)

WGSL imposes uniformity constraints:

- `textureSample` must be executed under uniform control flow.
- derivatives (`fwidth`, etc.) also require uniform control flow.

Implementations of this step MUST avoid divergent branches around sampling. Preferred patterns:

- always compute `d_px` and always sample (using clamp/select to handle out-of-bounds),
- or use pipeline variants (bounded) for “chromatic aberration enabled” vs “disabled”.

## Performance notes

- This step is fragment-bound and increases sampling cost:
  - 1 sample per pixel (warp only),
  - 3 samples per pixel (warp + chromatic aberration).
- It does not inherently require additional offscreen intermediates if implemented as a backdrop
  sampling step, but it does require strict scissoring to `bounds` and budget-driven quality
  choices.

## Alternatives considered

1. **General custom shader surface**
   - Rejected for v1: unbounded, poor portability, hard to gate, pipeline explosion.

2. **Only “fake glass” (blur + color adjust)**
   - Insufficient for refraction/displacement designs; leaves a persistent gap for ecosystem
     authors.

## Consequences

- Ecosystem can express “true liquid glass” style distortion using a stable contract surface.
- The renderer retains a bounded, testable, portable effect vocabulary without opening arbitrary
  shader injection.

