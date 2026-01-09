# ADR 0135: Renderer Effect Clip Masks and Soft Clipping (v1)

Status: Proposed

## Context

Fret has (or is actively landing) two important, future-facing substrates:

1) **Effect groups**: `SceneOp::PushEffect/PopEffect` with `EffectMode::{Backdrop, FilterContent}` (ADR 0119),
   compiled into `fret-render`'s `RenderPlan` substrate (ADR 0118).
2) **Rounded clipping semantics**: `SceneOp::PushClipRRect/PopClip` as a soft clip contract (ADR 0063).

The current renderer v3 MVP implements effects using **rectangular scissor** to bound work. This is correct for
performance and for rectangular clip stacks, but it is insufficient for modern UI composition because:

- Glass-like surfaces (blur + tint) are typically expressed as **rounded panels** with `overflow-hidden`.
- Without soft clipping, effect results appear “wrong”: blurred/pixelated/tinted content extends into corners,
  producing visible seams and halos.
- We want effects to be *composable* with the existing clip stack semantics (ADR 0078 / ADR 0063), without changing
  public `SceneOp` contracts.

We also want to avoid repeated large refactors as the effect surface grows: the solution must fit the v3 renderer
substrate (plan compilation, budgets, deterministic degradation; ADR 0120).

## Decision

### 1) Effects must respect the effective clip stack

For both effect modes (`Backdrop` and `FilterContent`), renderers must treat the effective clip stack as a
first-class constraint on effect evaluation and compositing:

- Rectangular clips can be applied as fast scissor rectangles.
- Rounded clips require **soft clipping** (coverage-based), consistent with ADR 0063.

This does **not** change ADR 0119: `bounds` remains a computation bound (not an implicit clip).

### 2) Introduce a renderer-internal clip mask substrate (v1: rect + rrect)

`fret-render` will introduce an internal representation for “clip used by postprocessing passes”:

- A fast-path `ScissorRect` (rectangular intersection).
- An optional **clip mask** that represents the current soft clip coverage.

This clip mask substrate is renderer-internal and does not leak into `fret-core`.

V1 scope:

- Support rectangular clip stacks via scissor-only (already exists).
- Support rounded-rect clip stacks via a soft clip mask (alpha coverage) derived from the clip stack.

### 3) Deterministic degradation is required

When clip masks cannot be produced within budgets (ADR 0120), the renderer must degrade deterministically:

- Prefer degrading **mask quality** (e.g. lower-resolution mask) before disabling the effect entirely.
- If mask generation is disabled:
  - Effects may fall back to scissor-only behavior (rectangular clip) while preserving ordering.
  - The renderer may also disable the effect step(s) (consistent with ADR 0119 / ADR 0120).

Degradation must be layout-invariant and must not change hit-testing geometry.

## Semantics (Renderer v3)

### A) FilterContent

`FilterContent` is “saveLayer + imageFilter”. With clip masks:

1) Render children inside the effect group into an offscreen intermediate.
2) Apply the effect chain to the intermediate using:
   - `bounds` intersected with the current scissor (to bound computation),
   - and the soft clip mask (to ensure rounded overflow-hidden correctness, especially for blur).
3) Composite back to the parent target using premultiplied alpha over (ADR 0040),
   applying the **clip mask** (but not treating `bounds` as a clip).

Key property:

- The clip stack controls visibility; `bounds` controls *where work is performed*.

### B) Backdrop

`Backdrop` is “backdrop-filter”. With clip masks:

1) Sample the already-rendered parent target at the `PushEffect` boundary, limited by `bounds` and scissor.
2) Apply the effect chain to that sampled region.
3) Write the filtered backdrop result back such that:
   - pixels outside the effect region remain unchanged,
   - and the result respects the effective clip stack via the clip mask.

## Implementation Notes (Non-normative)

### Option 1 (recommended v1): Alpha mask texture (SDF-generated)

Create a mask texture at the working resolution (full size or downsampled) that stores coverage in `R8Unorm`.

- A `MaskPass` rasterizes the current clip stack into the mask.
- Fullscreen effect passes sample the mask and blend/overwrite only where coverage > 0 (or multiply by coverage).

Benefits:

- Works uniformly for all fullscreen-style effect passes (blur/color adjust/pixelate).
- Can be generated at the same resolution as downsampled blur/pixelate for cost control.
- Does not require coupling every effect pipeline to stencil formats or depth attachments.

### Option 2: Stencil-based clip masks

Use a depth-stencil attachment, populate stencil with the clip stack, then run effect passes with stencil tests.

Benefits:

- Avoids sampling an extra mask texture.

Costs/risks:

- Stencil attachment lifecycle becomes a first-class resource in `RenderPlan`.
- More platform-specific pitfalls (WebGPU/mobile constraints, format support, attachment limits).

### Shared SDF foundation (recommended)

To keep clip masks, quads, and future shadows consistent (ADR 0030), use a shared analytic SDF foundation:

- A shared WGSL module for:
  - rounded-rect SDF,
  - derivative-based AA coverage (`fwidth`),
  - and optional distance-to-rect helpers.
- This module is used by:
  - quad rendering,
  - rounded clip mask generation,
  - future shadow primitives,
  - and future “soft mask” effects.

This is a renderer-internal refactor and does not change `SceneOp`.

## Consequences

- Enables “glass panels” and other postprocessing looks to compose correctly with rounded clipping.
- Provides a scalable path to support more complex clip shapes later (paths, masks) without changing public scene contracts.
- Increases renderer complexity: additional passes/resources, and budget policy must cover masks.

## Open Questions

- Mask blending semantics: multiply output alpha vs alpha-clip discard. (Recommendation: coverage multiplication for smooth edges.)
- Nested rounded clips: whether to rasterize full clip stack into one mask, or limit to a bounded maximum and degrade.
- Path clips (`PushClipPath`): when and how to extend beyond rounded rects.

## References

- Rounded clipping semantics: `docs/adr/0063-rounded-clipping-and-soft-clip-masks.md`
- Shape + analytic SDF semantics: `docs/adr/0030-shape-rendering-and-sdf-semantics.md`
- Transform + clip composition: `docs/adr/0078-scene-transform-and-clip-composition.md`
- RenderPlan substrate: `docs/adr/0118-renderer-architecture-v3-render-plan-and-postprocessing-substrate.md`
- Effect semantics: `docs/adr/0119-effect-layers-and-backdrop-filters-scene-semantics-v1.md`
- Budgets + degradation: `docs/adr/0120-renderer-intermediate-budgets-and-effect-degradation-v1.md`
- GPUI reference (rect content masks, premul blending): `repo-ref/zed/crates/gpui`

