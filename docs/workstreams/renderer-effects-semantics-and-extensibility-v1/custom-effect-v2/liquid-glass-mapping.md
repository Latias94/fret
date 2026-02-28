---
title: CustomV2 - Liquid Glass Mapping (Reference-derived)
status: draft
date: 2026-02-28
scope: renderer, effects, custom-v2, authoring, portability
---

# CustomV2 - Liquid Glass Mapping

This note maps a "liquid glass" UI look (refracted bevel edges + frosted center) onto Fret's
current renderer semantics, focusing on **CustomV2**.

The goal is to help effect authors build a high-ceiling "glass card" without requiring new core
renderer contracts, while keeping WebGPU/WASM constraints in mind.

## Decompose the look into mechanism-friendly steps

A practical liquid-glass card can be decomposed into these *mechanism-friendly* components:

1. **Backdrop capture**: sample content behind the card (not the card's own children).
2. **Backdrop blur**: blur the backdrop in a deterministic, budgetable way.
3. **Vibrancy**: saturation / brightness / tint shaping in linear space.
4. **Edge bevel**: a wide smooth transition near the boundary (not a thin outline).
5. **Refraction**: displace backdrop sampling near the boundary along a stable normal direction.
6. **Highlight + inner shadow**: subtle depth cues coupled to the same edge distance field.
7. **Grain/noise**: low-amplitude, deterministic, effect-local noise to avoid banding/flatness.
8. **Foreground crispness**: content drawn on top stays unblurred (controls, icons, text).

In Fret, items (1)(2)(3)(8) are primarily handled by `EffectMode::Backdrop` plus built-in effect
steps (blur + color adjust), while items (4)(5)(6)(7) are a good fit for `CustomV2`.

## What “correct” typically means in open implementations

Across open implementations, the “liquid glass” impression usually comes from the combination of:

- A *backdrop blur/vibrancy base* (center reads as frosted),
- A *shape-aware refraction zone* near the boundary (warps more at edges, not at the center),
- *Depth cues* (rim highlight + inner shadow) coupled to the same boundary distance,
- A small amount of *grain/noise* to avoid a flat/plastic look.

The “shape-aware refraction zone” is the main differentiator from a generic “wavy distortion”:
it needs a signed-distance-to-boundary value and a stable direction (a normal) so corners behave
correctly and the refraction follows the rounded rect.

### A common “renderer-owned glass primitive” architecture

Some GPU UI renderers treat “glass” as a first-class primitive with a renderer-owned pipeline, rather
than an author-defined single-pass shader. In that style, the renderer typically provides:

- a **backbuffer / offscreen backdrop** texture that glass can sample (often previous frame or a
  composited intermediate, not the swapchain directly),
- a **multi-pass blur** implementation (often Kawase-like downsample/upsample) that acts like a small
  blur pyramid,
- optional shared caching so multiple glass surfaces can reuse the same blurred backdrop work.

This approach aligns closely with the “blur pyramid” / “shared cache” ceiling items listed later in
this note. It does not automatically imply “dual-source sampling” (unblurred + blurred) in one pass,
but it makes it much easier to add if the renderer owns both textures.

## Map to Fret surfaces

### Option A (recommended baseline): built-in blur + CustomV2 edge pass

- Use **`EffectMode::Backdrop`**.
- Use built-in steps for the "cheap and correct" parts:
  - `GaussianBlur` for the frosted center (budgeted + deterministic),
  - `ColorAdjust` for vibrancy shaping,
  - `NoiseV1` (optional) for grain, if you want a built-in, contract-owned grain surface.
- Use **`CustomV2`** as the final step to implement edge bevel + edge-only refraction + highlights,
  sampling from:
  - `src_texture` (the already-blurred backdrop), and
  - the v2 user image input (`ImageId`) as a **data texture** (normal/noise/LUT, etc.).

Example chain structure (conceptual):

- `GaussianBlur { radius_px, downsample }`
- `ColorAdjust { saturation, brightness, contrast }`
- `CustomV2 { max_sample_offset_px, input_image, params }`

Key contract hooks to use:

- `EffectStep::CustomV2.max_sample_offset_px` must cover your maximum displacement and sampling
  radius so earlier steps can expand scissors deterministically.
- Use `render_space` + `fret_local_px(pos_px)` so the edge math is stable under transforms and
  does not depend on global window coordinates.

Important limitation (by design in v2):

- If you blur first and then run `CustomV2`, the custom shader **cannot access the unblurred
  backdrop** anymore. That means edge refraction will distort a *blurred* input, not the original
  crisp background. This is still a useful and often good-looking approximation (especially when
  highlights/shadows do most of the perceptual work), but it is not the highest-fidelity variant.

### Option B (higher fidelity): single-pass CustomV2 does blur + refraction

If you want the more "correct" look (crisp refraction near the edge + frosted center), run a
single `CustomV2` pass first (on the unblurred backdrop) and do both:

- edge-only refraction (sampling along the rounded-rect normal), and
- an in-shader multi-sample blur for the center region (spiral / small-kernel / kawase-like),
  blended by distance-to-edge.

You can optionally follow with `ColorAdjust` if you prefer to keep vibrancy shaping outside the
custom shader.

This raises the ceiling, but:

- it is harder to budget (pass cost lives in the shader),
- it is easier to violate WebGPU uniformity rules,
- it is easier to regress perf on large radii.

## CustomV2 parameter packing (recommended baseline)

CustomV2 only receives `EffectParamsV1` (64 bytes = 4×`vec4<f32>`). A practical packing for
liquid-glass authoring (edge-only refraction + highlights):

- `params.vec4s[0]`:
  - `x`: `refraction_height_px` (>= 0, thickness of the refracting edge zone)
  - `y`: `refraction_amount_px` (>= 0, peak displacement in px)
  - `z`: `depth_effect01` (0..1, optional bias toward radial depth)
  - `w`: `chroma_aberration01` (0..1, optional; keep 0 for portable baseline)
- `params.vec4s[1]`:
  - `xyzw`: `tint_rgba` (premul in shader; alpha controls tint strength)
- `params.vec4s[2]`:
  - `x`: `highlight_strength` (0..1)
  - `y`: `inner_shadow_strength` (0..1)
  - `z`: `grain_strength` (0..1)
  - `w`: `grain_scale` (>= 0.1)
- `params.vec4s[3]`:
  - `xyzw`: `corner_radii_px` (tl, tr, br, bl), **effect-local** radii.

Notes:

- Keep `tint_rgba` separate from vibrancy `ColorAdjust` so authors can decide whether tint is
  applied pre- or post-highlight.
- Prefer "wide smooth transitions" for bevel (`smoothstep` ranges measured in pixels), not a thin
  1px outline, to avoid corner-looking artifacts.
- `refraction_height_px` and `refraction_amount_px` are intentionally separate: height controls
  *where* refraction happens, amount controls *how much* it bends.

## Input image conventions (v2 user image)

For refraction you usually want a data texture (linear space), for example:

- **Normal/noise map**:
  - `rg`: signed displacement direction encoded in [0..1] → [-1..1],
  - `b`: optional falloff/control term (e.g. keep center stable),
  - `a`: reserved.

For color grading:

- **LUT in 2D** (linear data texture) sampled with `fret_sample_input(...)` and a custom mapping.

Always set the input `ImageDescriptor.color_space` correctly:

- `Linear` for data textures (noise/normal/LUT),
- `Srgb` for color images intended to decode to linear on sample.

## Edge bevel + refraction implementation sketch (WGSL)

The key is to derive a stable edge distance and normal direction **without derivatives**:

- compute a rounded-rect signed distance in effect-local pixel space,
- derive a normal direction from the distance field using an analytic gradient (preferred) or
  finite differences,
- displace sampling along the outward normal *only near the boundary* (edge zone),
- layer highlight/shadow as functions of the same distance.

This keeps the shader WebGPU-friendly and corner-stable.

Two practical implementation notes:

- Use an *edge-only zone*: if `dist_in_px >= refraction_height_px`, sample `src_texture` unchanged.
- Use a non-linear mapping from normalized edge depth to displacement (e.g. a "circle map") so the
  refraction ramps smoothly:
  - `circle_map(x) = 1 - sqrt(1 - x*x)` for `x in [0,1]`

This avoids a "flat" bevel and produces the expected liquid-like curvature.

### A concrete “reference-derived” shape-normal recipe

One robust pattern (portable to WebGPU and stable at corners) is:

- Pick the active corner radius based on the centered quadrant (tl/tr/br/bl).
- Compute `sd = sd_rounded_rect(centered, half_size, radius)`.
- Convert to inside-distance: `dist_in = max(0, -sd)`.
- If `dist_in >= refraction_height_px`, return `src` unchanged.
- Else, compute an edge depth `x = 1 - dist_in / refraction_height_px` in `[0, 1]`.
- Map depth to displacement using `circle_map(x)`.
- Compute a stable normal direction from an analytic SDF gradient:
  - Use a slightly inflated radius (e.g. `min(radius * 1.5, min(half_size.x, half_size.y))`) when
    computing the gradient to avoid overly sharp corner normals.
- Sample along the normal: `pos_px + disp_px * normal`.

This is close to the “lens” refraction strategy used in Android runtime shader implementations:
the refraction is *edge-only* and follows the rounded rect geometry.

## WebGPU/WASM constraints (author checklist)

- Avoid `discard` and non-uniform early returns before derivative operations.
  - If you need SDF AA via derivatives, ensure uniform control flow around `dpdx`/`dpdy`/`fwidth`.
  - Prefer analytic or finite-difference gradients for liquid-glass normals (portable; no derivatives).
- Clamp sampling deterministically (no out-of-bounds textureLoad).
- Keep `max_sample_offset_px` accurate so plan padding remains correct.

## Do v1 + v2 cover the ceiling for liquid glass?

For the “reference-derived” ceiling described above (blur/vibrancy base + edge-only refraction +
rim/inner-shadow + grain):

- **Yes, with CustomV2**, as long as you accept the sequential-chain model (Option A) where the
  refraction samples the already-blurred backdrop.
- **Yes, even with CustomV1**, if you do not need a user image input (no authored normal/noise/LUT);
  the refraction math itself does not require extra textures.

What is *not* fully covered today (and would imply a new contract / refactor):

- A single effect step sampling **both** an unblurred backdrop and a blurred backdrop (dual-source),
- A renderer-owned **blur pyramid / MIP chain** exposed to custom effects with an explicit cost model,
- Cross-element “glass group” sharing (multiple shapes blending and reusing a cached backdrop capture).

If we want to treat “full liquid glass systems” (group blending + shared blur + higher-fidelity
refraction) as the next ceiling, the likely direction is a **CustomV3** story that introduces one
additional bounded input surface (dual-source or pyramid) rather than unbounded multi-pass graphs.

## Evidence anchors in this repo

- CustomV2 contract + helpers:
  - `docs/workstreams/renderer-effects-semantics-and-extensibility-v1/custom-effect-v2/README.md`
  - `ecosystem/fret-ui-kit/src/custom_effects.rs`
- Existing authoring templates:
  - `apps/fret-examples/src/custom_effect_v2_web_demo.rs` (warp + controls harness)
  - `apps/fret-examples/src/custom_effect_v2_glass_chrome_web_demo.rs` (normal-driven highlight)
  - `apps/fret-examples/src/liquid_glass_demo.rs` (warp + blur + color adjust recipes, native)
