---
title: Custom Effect V2 (High-ceiling, bounded)
status: in_progress
date: 2026-02-27
scope: renderer, effects, extensibility, abi
---

# Custom Effect V2 (High-ceiling, bounded)

CustomV1 intentionally ships as a small "escape hatch with a ceiling" (single pass, params-only,
`src_texture` only). That is enough for many UI looks, but it is not the end-state for
"editor-grade" effects such as:

- acrylic / glass variants that want a noise/LUT/normal-map input,
- stylized "theme postprocess packs" (scanlines/vignette overlays),
- effect stacks that want a small, fixed multi-pass bundle with an explicit cost model.

This folder tracks a **fearless refactor** path to a CustomV2 ABI that raises the ceiling while
keeping the core contract bounded, budgetable, and capability-gated.

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
- normal/displacement inputs for glass-like highlights or distortion (data textures),
- editor skins that ship their own patterns without forking the renderer.

Non-goals for v2 (explicitly deferred):

- a general "resource table" (2+ textures),
- arbitrary sampler/addressing/mip configuration,
- multi-pass bundles as a single custom effect unit (can be revisited after v2 lands).

## Contract sketch

### Portable surface (core)

`fret-core` surface:

- `EffectStep::CustomV2 { id, params, max_sample_offset_px, input_image }`
- `input_image: Option<CustomEffectImageInputV1>`
  - `image: ImageId`
  - `uv: UvRect` (default `UvRect::FULL`)
  - `sampling: ImageSamplingHint` (`Default`/`Linear`/`Nearest`)

Notes:

- The input is referenced by `ImageId`, reusing the existing portable image registry contract.
- The renderer clamps addressing (baseline: clamp-to-edge).
- Mips/anisotropy remain out of scope; sampling is LOD 0 only.
- The input texture must be compatible with `texture_2d<f32>` + a filtering sampler
  (`TextureSampleType::Float { filterable: true }`). Incompatible formats deterministically fall
  back to a renderer-owned 1x1 transparent texture.
- Color space follows `ImageDescriptor.color_space`:
  - sRGB images decode to linear in shader sampling,
  - data textures (LUT/noise/normal maps) should be uploaded as `ImageColorSpace::Linear`.

### WGSL surface (renderer-owned, versioned)

CustomV2 WGSL prelude adds one extra sampled image alongside `src_texture`:

- `input_texture`: `texture_2d<f32>`
- `input_sampler`: `sampler`
- `input_uv_rect`: `vec4<f32>` (`u0, v0, u1, v1`)

And provides helpers:

- `fret_input_uv(pos_px) -> vec2<f32>` (effect-local mapping + `UvRect`)
- `fret_sample_input(uv) -> vec4<f32>` (clamped sampling at LOD 0)
- `fret_sample_input_at_pos(pos_px) -> vec4<f32>`

## Capability gating

CustomV2 remains capability-gated:

- Backends that do not support custom effects return `Unsupported`, and effect steps deterministically
  degrade to no-op.
- Backends may support CustomV1 but not CustomV2; capability discovery must allow the app/ecosystem
  to choose a fallback.

## Implementation status (as of 2026-02-27)

Done (evidence anchors):

- Core contract: `crates/fret-core/src/scene/mod.rs` (`EffectStep::CustomV2`, `CustomEffectImageInputV1`).
- Renderer (wgpu): `crates/fret-render-wgpu/src/renderer/pipelines/custom_effect.rs` (fixed v2 bind group shape).
- WGSL prelude (wgpu): `crates/fret-render-wgpu/src/renderer/pipelines/wgsl/custom_effect_v2_*`.
- Conformance: `crates/fret-render-wgpu/tests/effect_custom_v2_conformance.rs`.
- Ecosystem helper: `ecosystem/fret-ui-kit/src/custom_effects.rs` (`CustomEffectProgramV2`).
- Demo: `apps/fret-examples/src/custom_effect_v2_demo.rs` (run via `cargo run -p fret-demo -- custom_effect_v2_demo`).

Pending / follow-ups:

- Capability discovery: expose whether the current backend supports CustomV2 (so apps can pick
  `CustomV1` or no-op).
- Web backend story: define how/when CustomV2 is supported in WebGPU and how it degrades in wasm.

## References

- Decision ADR: `docs/adr/0300-custom-effect-v2-user-image-input.md`
- CustomV1 semantics: `docs/workstreams/renderer-effects-semantics-and-extensibility-v1/custom-effect-v1-semantics.md`
- ADR 0299 (CustomV1 MVP): `docs/adr/0299-custom-effect-abi-wgpu-only-mvp.md`
