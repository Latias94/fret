---
title: ADR 0300: Custom Effect V2 (Single User Image Input)
status: Draft
date: 2026-02-26
---

# ADR 0300: Custom Effect V2 (Single User Image Input)

## Context

ADR 0299 introduced CustomV1 as a bounded “escape hatch with a ceiling”:

- single-pass fullscreen WGSL,
- params-only (fixed 64B payload),
- `src_texture` only (no user textures),
- deterministic degradation under budgets / capabilities.

CustomV1 is sufficient for many UI looks, but several “high ceiling” recipes require one additional, portable input:

- color grading via LUTs,
- stylized postprocess themes that ship patterns (scanlines/halftone/film grain textures),
- “real glass” variants that need a normal/displacement map as a warp/highlight driver,
- editor skins that want to ship authored data textures without forking the renderer.

The repository boundary rules remain non-negotiable:

- no `wgpu` handles in `fret-core` / `fret-ui`,
- fixed, versioned binding shapes,
- deterministic behavior and diagnosable degradation.

## Goals

- Raise the ceiling of CustomV1 with **one** additional input that unlocks most high-end recipes.
- Reuse an existing portable handle (`ImageId`) and existing sampling vocabulary (`ImageSamplingHint`).
- Keep the contract bounded, cache-correct, and budgetable.
- Keep behavior deterministic under missing images, unsupported capabilities, and budget pressure.

## Non-goals

- No resource tables (2+ textures) in v2.
- No arbitrary sampler/addressing/mip configuration.
- No multi-pass custom effect bundles as a single unit (can be revisited after v2).
- No hidden time sources; animation remains app-driven via params.

## Decision

CustomV2 adds exactly one optional extra input: a **single user-provided image** referenced by `ImageId`.

### Core surface (portable)

Add a new effect step variant (names TBD):

- `EffectStep::CustomV2 { id, params, max_sample_offset_px, input_image }`

Where:

- `id: EffectId` (renderer-owned registry handle).
- `params: EffectParamsV1` (reuse the fixed 64B payload initially).
- `max_sample_offset_px: Px` (bounded sampling extent for deterministic padding, as in v1).
- `input_image: Option<CustomEffectImageInputV1>`
  - `image: ImageId`
  - `uv: UvRect` (default: `UvRect::FULL`)
  - `sampling: ImageSamplingHint` (`Default`/`Linear`/`Nearest`)

### Semantics

- If `input_image` is `None`, the shader receives a deterministic “null” binding (black/zero texture).
- If the referenced `ImageId` is unavailable:
  - bind the same deterministic “null” texture (no panic, no undefined behavior).
- Addressing is clamp-to-edge (v1/v2 baseline).
- Sampling is deterministic:
  - LOD is fixed to 0 (no implicit mip selection).
  - If `Nearest` is unsupported, deterministically degrade to `Default` (v1: effectively `Linear`).

### Color space / data textures

The input texture color space follows the existing image registration contract:

- sRGB images decode to linear when sampled in shader.
- Data textures (LUT/noise/normal maps) should be uploaded as `ImageColorSpace::Linear`.

Custom effect shaders must treat the input as **raw sampled data** (no premultiply/unpremultiply rules are applied to the
input sample by the renderer).

### Capability gating

CustomV2 is capability-gated:

- backends may support CustomV1 but not CustomV2,
- the app/ecosystem must be able to query capabilities and pick a fallback.

Backends that do not support CustomV2 deterministically degrade `EffectStep::CustomV2` to no-op (tracked via counters).

## Consequences

Pros:

- Unlocks a broad set of high-end recipes (LUTs, blue-noise, authored patterns, normal-map-driven glass).
- Keeps the portable contract bounded: one extra `ImageId` + `UvRect` + `ImageSamplingHint`.
- Reuses existing image/sampling contracts and avoids leaking backend handles.
- Preserves determinism and diagnosability.

Cons:

- Adds one extra sampled-image binding shape and a small number of pipeline variants (sampler selection).
- Requires conformance coverage to avoid scissor/mask regressions and ensure deterministic fallbacks.

## Tracking

- Workstream: `docs/workstreams/renderer-effects-semantics-and-extensibility-v1/custom-effect-v2/README.md`
- TODO: `docs/workstreams/renderer-effects-semantics-and-extensibility-v1/custom-effect-v2/todo.md`
- Milestones: `docs/workstreams/renderer-effects-semantics-and-extensibility-v1/custom-effect-v2/milestones.md`
