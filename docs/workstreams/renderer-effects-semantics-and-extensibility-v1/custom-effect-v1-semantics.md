---
title: Custom Effect V1 Semantics (WGSL contract)
status: draft
date: 2026-02-26
scope: renderer, effects, custom-effects, wgsl, abi
---

# Custom Effect V1 Semantics (WGSL contract)

This document is the **source-of-truth for what custom effects (CustomV1) may rely on** when authoring WGSL
snippets registered via `CustomEffectService::register_custom_effect_v1` (ADR 0299).

Custom effects are an **escape hatch with a ceiling**:

- They run as a single fullscreen fragment pass within Fret’s bounded `EffectChain` mechanism.
- They do not get access to `wgpu` handles.
- They must remain deterministic and budgetable.

## Where CustomV1 runs

CustomV1 only runs as part of an `EffectChain` between `SceneOp::PushEffect` / `SceneOp::PopEffect`
(ADR 0117 + ADR 0118).

It can be used in both:

- `EffectMode::Backdrop` (src is the already-rendered backdrop behind the bounds), and
- `EffectMode::FilterContent` (src is the rendered subtree within the bounds).

## Required WGSL entrypoint

The registered snippet MUST define:

```wgsl
fn fret_custom_effect(
  src: vec4<f32>,
  uv: vec2<f32>,
  pos_px: vec2<f32>,
  params: EffectParamsV1
) -> vec4<f32>
```

Semantics:

- `src` is the source pixel at `pos_px` from `src_texture`.
- `uv` is `pos_px / src_texture_dimensions`.
- `pos_px` is pixel-space with a `0.5` center convention (the same `pos_px` you would use for `textureLoad` math).
- `params` is a fixed-size payload (64 bytes) and is sanitized (non-finite → `0` at the contract layer).

### Color space and alpha

- Inputs/outputs are treated as **linear premultiplied RGBA**.
- Clip/mask coverage is applied by the renderer after `fret_custom_effect` returns.

Practical rule:

- If you want to do color math in un-premultiplied space, `unpremul → operate → premul` before returning.

## Built-in resources (stable)

### `render_space` (effect-local coordinates)

CustomV1 receives a renderer-owned uniform:

```wgsl
struct RenderSpace {
  origin_px: vec2<f32>,
  size_px: vec2<f32>,
};
@group(0) @binding(5) var<uniform> render_space: RenderSpace;
```

This lets effect authors implement Android/Flutter-style “bounded” shaders without re-deriving bounds from
layout/scissor state.

Recommended pattern:

- local pixel position: `let local_px = pos_px - render_space.origin_px;`
- centered: `let centered = local_px - render_space.size_px * 0.5;`

### Material catalog patterns (noise/dither)

The renderer maintains a small, deterministic “material catalog” texture array that includes reusable patterns:

- Layer `0`: hash noise (64×64)
- Layer `1`: Bayer 8×8 repeated (64×64)

CustomV1 may rely on these helpers being available in the shader prelude:

- `fret_catalog_hash_noise01(pos_px: vec2<f32>) -> f32`
- `fret_catalog_bayer8x8_01(pos_px: vec2<f32>) -> f32`
- `fret_local_px(pos_px: vec2<f32>) -> vec2<f32>` (convenience)

Notes:

- These are intended to make acrylic / film-grain / scanlines / ordered-dither recipes easy to author without
  introducing user-provided textures in v1.
- The pattern origin is intentionally “screen-like” (based on the pass `pos_px`). If you want a panel-anchored pattern,
  use `fret_local_px(pos_px)` as the coordinate source.

## Texture sampling constraints (v1 ceiling)

CustomV1’s `src_texture` is currently bound without a sampler and is intended to be read via `textureLoad`.

Implications:

- You must implement your own bilinear sampling if you want filtered reads.
- `uv` is provided as a convenience but does not imply that `textureSample` is available.

## Padding contract (`max_sample_offset_px`)

`EffectStep::CustomV1` includes `max_sample_offset_px`. This is a conservative bound on how far your shader may
sample away from `pos_px` when reading `src_texture`.

This value is used by the render plan compiler to:

- deterministically expand earlier step scissors (e.g. blur → custom) so the final custom step can safely read
  outside the previous step’s output pixel, and
- avoid `clip^2` artifacts by applying coverage only once on the final commit pass.

If your effect reads up to `pos_px ± 12px`, set `max_sample_offset_px` to at least `12px`.

## What is intentionally *not* stable

CustomV1 WGSL is concatenated into a renderer-owned prelude. Only the symbols described above are stable.

Do not rely on:

- internal bind groups/bindings other than the documented ones,
- internal storage buffer layouts for clip/mask stacks,
- private helper functions from other renderer shaders.

If you need more inputs (user textures, LUTs, history buffers), track the `CustomV2` design workstream.

