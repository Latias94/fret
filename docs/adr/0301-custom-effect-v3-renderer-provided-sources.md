---
title: "ADR 0301: Custom Effect V3 (Renderer-provided Sources: Raw + Optional Pyramid)"
status: Draft
date: 2026-02-28
---

# ADR 0301: Custom Effect V3 (Renderer-provided Sources: Raw + Optional Pyramid)

## Context

ADR 0299 introduced CustomV1 as a bounded “escape hatch with a ceiling”:

- single-pass fullscreen WGSL,
- params-only (`EffectParamsV1`, 64 bytes),
- `src_texture` only (no user textures),
- deterministic degradation under budgets and capability gating.

ADR 0300 extended the ceiling via CustomV2 by adding exactly one optional user image input (`ImageId` +
`UvRect` + `ImageSamplingHint`) while keeping the ABI bounded and cache-correct.

These surfaces are sufficient for a large class of UI effects (shadcn, Material 3, basic glass), but the
“high fidelity liquid glass” ceiling often needs one of the following *mechanism-level* capabilities:

1) **Dual-source sampling**: access to both an unmodified (“raw”) backdrop and a blurred/adjusted backdrop,
   so edge refraction can stay crisp while the center remains frosted.
2) **A bounded blur pyramid**: a small, renderer-owned, budgeted mip chain so effects can sample “more blurred”
   versions deterministically without embedding heavy blur loops in user WGSL.
3) **(Future) group sharing**: multiple glass surfaces reusing the same pyramid/capture work.

CustomV1/V2 intentionally cannot express (1)(2) without either:

- breaking the fixed bind-shape ABI, or
- pushing blur work into user WGSL (unbudgetable and hard to diagnose).

This ADR defines a versioned CustomV3 contract that raises the ceiling while preserving the non-negotiables:

- no backend handle leakage into `fret-core` / `fret-ui`,
- fixed, versioned bind shapes,
- explicit, budgetable cost model and deterministic degradation,
- WebGPU/WASM portability constraints.

## Goals

- Provide a **renderer-owned raw source** (`src_raw`) alongside the existing chain input (`src`).
- Optionally provide a **renderer-owned blur pyramid** (`src_pyramid`) with explicit upper bounds.
- Keep the portable core surface bounded and cache-correct.
- Preserve determinism under missing resources, unsupported capabilities, and budget pressure.
- Keep the authoring model “single pass” (no arbitrary multi-pass graphs in the ABI).

## Non-goals

- No general resource tables (unbounded textures/buffers/samplers).
- No arbitrary per-effect sampler/addressing/mip configuration.
- No implicit time sources (animation remains app-driven via params).
- No multi-pass custom effect bundles as a single unit.
- Group-level sharing/caching is deferred (may be a follow-up ADR once V3 lands).

## Decision

Introduce **CustomV3** as a new versioned custom effect ABI with a fixed bind shape that includes:

- `src_texture`: the current chain input (same role as v1/v2),
- `src_raw_texture`: the chain **root** (the input before any prior effect steps in the chain),
- `src_pyramid_texture`: an optional renderer-owned **mipped** version of `src_raw` (bounded levels),
- up to two optional user images (`ImageId` inputs) for authored data/textures.

### Core surface (portable)

Add a new effect step (names are illustrative; final naming may follow existing patterns):

- `EffectStep::CustomV3 { id, params, max_sample_offset_px, user0, user1, sources }`

Where:

- `id: EffectId` (renderer-owned registry handle, as in v1/v2).
- `params: EffectParamsV1` (reuse fixed 64B payload initially).
- `max_sample_offset_px: Px` (bounded sampling extent, as in v1/v2).
- `user0: Option<CustomEffectImageInputV1>` (same as CustomV2).
- `user1: Option<CustomEffectImageInputV1>` (same shape; optional second input).
- `sources: CustomEffectSourcesV3`:
  - `want_raw: bool` (default `false`): if `true`, request that `src_raw` be preserved as a distinct source.
  - `pyramid: Option<CustomEffectPyramidRequestV1>`:
    - `max_levels: u8` (clamped to a small constant, e.g. `<= 7`),
    - `max_radius_px: Px` (explicit upper bound used in budgeting).

Semantics:

- If `want_raw == false`, `src_raw` aliases `src` deterministically.
- If `want_raw == true` but the backend cannot provide a read-only raw source (insufficient scratch targets,
  unsupported backend, or insufficient budgets), it deterministically degrades to `src_raw == src` (tracked).
- If `pyramid == None`, `src_pyramid` aliases `src_raw` deterministically with `pyramid_levels == 1`.
- If `pyramid` is requested but budgets/capabilities do not allow it, deterministically degrade to
  `pyramid_levels == 1` and `src_pyramid == src_raw` (tracked via counters).

### Renderer-owned WGSL surface (versioned)

CustomV3 WGSL prelude exposes a fixed set of sources. Conceptually:

- `src_texture: texture_2d<f32>`
- `src_raw_texture: texture_2d<f32>`
- `src_pyramid_texture: texture_2d<f32>` (mipped; `mip_level_count = pyramid_levels`)
- `user0_texture: texture_2d<f32>` (+ optional sampler if filterable sampling is supported)
- `user1_texture: texture_2d<f32>` (+ optional sampler)
- `custom_v3: uniforms` (includes `pyramid_levels` and any source metadata)

The prelude provides helpers that are WebGPU-friendly:

- `fret_sample_src_bilinear(pixel_pos_px) -> vec4<f32>`
- `fret_sample_src_raw_bilinear(pixel_pos_px) -> vec4<f32>`
- `fret_sample_src_pyramid_bilinear(level, pixel_pos_px) -> vec4<f32>` (level clamped to `[0, pyramid_levels-1]`)

The renderer remains responsible for scissor/mask semantics and for deterministic clamping.

### Source semantics (color, premultiply, addressing)

All sources (`src`, `src_raw`, `src_pyramid`) are treated as:

- **linear premultiplied** color storage (consistent with effect evaluation rules in ADR 0117),
- **clamp-to-edge** addressing for any sampling helper the renderer provides,
- **LOD 0** when sampling `src` / `src_raw` helpers.

For the pyramid:

- `src_pyramid` is a renderer-owned mip chain derived from `src_raw`.
- The pyramid is generated deterministically by repeated 2× downsampling from level `k` → `k+1` using a fixed,
  renderer-owned filter (baseline: 2×2 box filter in linear premultiplied space).
- Sampling a pyramid level uses the level’s texel grid deterministically and clamps out-of-bounds access.
- The portable contract does not expose arbitrary mip filtering; authors must use the provided helper
  (`fret_sample_src_pyramid_bilinear`) to keep portability and determinism.

### Capability gating

Renderers must surface CustomV3 capabilities independently from v1/v2, for example:

- supports custom effects v3 at all,
- supports preserving a distinct raw source,
- supports building a pyramid up to `N` levels (or `0` = none),
- supports 0/1/2 user images.

Backends that do not support CustomV3 deterministically degrade `EffectStep::CustomV3` to no-op (tracked).

### Budgeting and determinism

CustomV3 introduces a renderer-visible cost knob (`pyramid` request). The backend:

- uses `max_levels` and `max_radius_px` to decide whether pyramid generation is affordable,
- emits deterministic degradation when budgets are insufficient (alias pyramid to raw with levels=1),
- surfaces counters/plan summaries so the app can see which effects received a pyramid.

### Cache correctness

CustomV3 must be cache-correct:

- registry generations must contribute to the scene encoding cache key (as in v1/v2),
- any capability/budget knobs that change encode output must contribute to the key.

## Consequences

Pros:

- Enables high-fidelity “liquid glass” recipes without unbudgetable blur loops in user WGSL.
- Keeps the ABI bounded: fixed bind shape, explicit pyramid bounds, deterministic fallbacks.
- Keeps layering intact: portable handles only (`EffectId`, `ImageId`), no backend handle leakage.

Cons:

- Adds renderer complexity (source preservation + optional pyramid generation).
- Requires new conformance tests (raw-vs-src correctness; pyramid level determinism; degradation visibility).

## Tracking

- Workstream: `docs/workstreams/renderer-effects-semantics-and-extensibility-v1/custom-effect-v3/README.md`

## References

- ADR 0299: Custom Effect ABI (wgpu-only MVP)
- ADR 0300: Custom Effect V2 (Single User Image Input)
