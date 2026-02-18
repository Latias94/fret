# ADR 0285: Backdrop Warp Effect Step v2 (Texture-Driven Warp Field)

- Status: Draft
- Date: 2026-02-18

## Context

ADR 0284 introduced `EffectStep::BackdropWarpV1`: a bounded, portable backdrop sampling effect that
supports procedural displacement + optional chromatic aberration, with deterministic degradation on
wasm/mobile.

Many “real liquid glass” implementations rely on an **asset-driven** warp field (normal/displacement
map) to achieve an organic lens-like distortion. Without a contract surface, ecosystem authors must
approximate via multi-pass pipelines or bespoke postprocess stages, which are:

- expensive (extra intermediates, broken batching),
- hard to keep portable to WebGPU/WGSL uniformity rules,
- hard to debug and gate (no stable surface for conformance/perf).

## Goals

- Add an **image-driven** warp field input to the backdrop warp surface.
- Keep the contract **bounded** (no arbitrary shader sources).
- Keep behavior **deterministic** under missing assets, unsupported capabilities, and budget pressure.
- Preserve the layering rule: `fret-ui` stays mechanism/contract; recipes stay in ecosystem.

## Non-goals

- No general “user-provided WGSL shader” plugin surface.
- No open-ended shader graph system.
- No attempt to match a specific platform aesthetic exactly (ecosystem recipes handle that).

## Decision

Add a new effect step variant to `fret-core::scene::EffectStep`:

- `EffectStep::BackdropWarpV2(BackdropWarpV2)`

`BackdropWarpV2` extends the v1 surface with a bounded warp field source:

- `field: BackdropWarpFieldV2`
  - `Procedural(BackdropWarpV1)` (compat)
  - `ImageDisplacementMap { image: ImageId, uv: UvRect, sampling: ImageSamplingHint, encoding: WarpMapEncodingV1 }`

Where `WarpMapEncodingV1` is a small enum that defines how to decode sampled RGBA into a 2D
displacement vector (in effect-local pixels after scaling).

## Semantics

The step is only meaningful when used under `EffectMode::Backdrop`:

1. Evaluate `d_px` (displacement in pixels):
   - Procedural: as defined in ADR 0284.
   - Image map:
     - sample the warp map at effect-local UV,
     - decode into a vector (encoding-defined),
     - scale by `strength_px` (clamped).
2. Convert `d_px` to backdrop UV and sample backdrop at `uv_backdrop + d_uv` (clamped).
3. Optional chromatic aberration:
   - bounded additional offsets for R/G/B sampling.
4. Composite children on top (existing backdrop effect semantics).

### Deterministic degradation (ordered)

1. If used with `EffectMode::FilterContent`: deterministically degrade (one rule, explicitly chosen):
   - Option A: skip the warp (no-op) and preserve the rest of the chain.
   - Option B: treat as `BackdropWarpV1` procedural (for a stable fallback).
   - This ADR chooses **Option A** (skip) to preserve the “backdrop warp” meaning:
     - `EffectMode::FilterContent` does not sample backdrop pixels by definition,
     - and ADR 0284 already requires `BackdropWarpV1` to be ignored under `FilterContent`.
     This keeps layering semantics consistent and avoids accidental “content-warp” behavior.
2. If the warp field image is unavailable or unsupported:
   - degrade to `Procedural(BackdropWarpV1)` (or skip if strength is zero).
3. Under budget pressure:
   - disable chromatic aberration,
   - then scale down displacement strength,
   - then skip the warp entirely (fall back to fake-glass chain).

## WebGPU / WGSL portability requirements

- Texture sampling must remain in uniform control flow.
- Implementations should avoid divergent branches around sampling; use `select`/`clamp`.
- Sample counts are bounded and pipeline variants must be explicit and measurable.

## Consequences

Pros:

- Ecosystem authors can build asset-driven glass recipes without leaking backend handles.
- The renderer can keep costs bounded (scissor, budgets, deterministic degradation).
- Conformance + perf can be gated on a stable mechanism surface.

Cons:

- Adds one extra sampled image binding and pipeline variants to the effect pass.
- Requires new conformance coverage and perf baselines to avoid unbounded regressions.

## Tracking

- Workstream: `docs/workstreams/renderer-effect-backdrop-warp-v2.md`
- TODO: `docs/workstreams/renderer-effect-backdrop-warp-v2-todo.md`
- Milestones: `docs/workstreams/renderer-effect-backdrop-warp-v2-milestones.md`
