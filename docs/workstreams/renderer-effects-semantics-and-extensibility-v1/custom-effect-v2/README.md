---
title: Custom Effect V2 (High-ceiling, bounded)
status: draft
date: 2026-02-26
scope: renderer, effects, extensibility, abi
---

# Custom Effect V2 (High-ceiling, bounded)

CustomV1 intentionally ships as a small “escape hatch with a ceiling” (single pass, params-only, `src_texture` only).
That is enough for many UI looks, but it is not the end-state for “editor-grade” effects such as:

- acrylic / glass variants that want a noise/LUT/normal-map input,
- stylized post-processing themes (cyberpunk/retro) that want a stable pattern atlas or LUT,
- effect stacks that want a small, fixed multi-pass bundle with an explicit cost model.

This folder tracks a **fearless refactor** path to a CustomV2 ABI that raises the ceiling while keeping the
core contract bounded, budgetable, and capability-gated.

Key invariants:

- No `wgpu` handle leakage into `fret-core` / `fret-ui`.
- Fixed, versioned binding shapes.
- Explicit sampling bounds and predictable scratch usage.
- Deterministic degradation with per-effect counters and plan visibility.

## Decision (M0)

CustomV2 raises the ceiling by adding **one (and only one) extra input**:

- A single **user-provided image texture** referenced by `ImageId` (portable handle).

This unlocks a large set of high-end recipes while keeping the contract bounded:

- color grading via LUT (2D strip or 3D LUT encoded in 2D),
- blue-noise / film grain textures,
- stylized “theme postprocess packs” (scanlines/vignette overlays),
- normal/displacement inputs for glass-like highlights or distortion (data textures),
- editor skins that ship their own patterns without forking the renderer.

Non-goals for v2 (explicitly deferred):

- a general “resource table” (2+ textures),
- arbitrary sampler/addressing/mip configuration,
- multi-pass bundles as a single custom effect unit (can be revisited after v2 lands).

## Contract sketch

### Portable surface (core)

Planned `fret-core` surface (names TBD):

- `EffectStep::CustomV2 { id, params, max_sample_offset_px, input_image }`
- `input_image: Option<CustomEffectImageInputV1>`
  - `image: ImageId`
  - `uv: UvRect` (default `UvRect::FULL`)
  - `sampling: ImageSamplingHint` (`Default`/`Linear`/`Nearest`)

Notes:

- The input is referenced by `ImageId`, reusing the existing portable image registry contract.
- The renderer must clamp addressing (v1/v2 baseline: clamp-to-edge).
- Mips/anisotropy remain out of scope; sampling is LOD 0 only.
- Color space follows `ImageDescriptor.color_space`:
  - sRGB images decode to linear in shader sampling,
  - data textures (LUT/noise/normal maps) should be uploaded as `ImageColorSpace::Linear`.

### WGSL surface (renderer-owned, versioned)

CustomV2 WGSL prelude adds one extra sampled image alongside `src_texture`:

- `input_texture`: `texture_2d<f32>`
- `input_sampler`: `sampler`
- `input_uv_rect`: `vec4<f32>` (`u0, v0, u1, v1`)

And provides small helpers (names TBD) such as:

- `fret_input_uv(pos_px) -> vec2<f32>` (effect-local mapping + `UvRect`)
- `fret_sample_input(uv) -> vec4<f32>` (clamped sampling at LOD 0)

## Capability gating

CustomV2 remains capability-gated:

- Backends that do not support custom effects return `Unsupported` and effect steps deterministically degrade to no-op.
- Backends may support CustomV1 but not CustomV2; capability discovery must allow the app/ecosystem to choose a fallback.

## References

Decision ADR (planned):

- `docs/adr/0300-custom-effect-v2-user-image-input.md`

See also:

- CustomV1 contract: `docs/workstreams/renderer-effects-semantics-and-extensibility-v1/custom-effect-v1-semantics.md`
- ADR 0299 (CustomV1 MVP): `docs/adr/0299-custom-effect-abi-wgpu-only-mvp.md`
