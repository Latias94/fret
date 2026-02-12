# ADR 0138: Renderer Effect Clip Masks and Soft Clipping (v1)

Status: Accepted (v1 substrate landed)

## Context

Fret has (or is actively landing) two important, future-facing substrates:

1) **Effect groups**: `SceneOp::PushEffect/PopEffect` with `EffectMode::{Backdrop, FilterContent}` (ADR 0117),
   compiled into `fret-render`'s `RenderPlan` substrate (ADR 0116).
2) **Rounded clipping semantics**: `SceneOp::PushClipRRect/PopClip` as a soft clip contract (ADR 0063).

The current renderer v3 MVP implements effects using **rectangular scissor** to bound work. This is correct for
performance and for rectangular clip stacks, but it is insufficient for modern UI composition because:

- Glass-like surfaces (blur + tint) are typically expressed as **rounded panels** with `overflow-hidden`.
- Without soft clipping, effect results appear “wrong”: blurred/pixelated/tinted content extends into corners,
  producing visible seams and halos.
- We want effects to be *composable* with the existing clip stack semantics (ADR 0078 / ADR 0063), without changing
  public `SceneOp` contracts.

We also want to avoid repeated large refactors as the effect surface grows: the solution must fit the v3 renderer
substrate (plan compilation, budgets, deterministic degradation; ADR 0118).

## Decision

### 1) Effects must respect the effective clip stack

For both effect modes (`Backdrop` and `FilterContent`), renderers must treat the effective clip stack as a
first-class constraint on effect evaluation and compositing:

- Rectangular clips can be applied as fast scissor rectangles.
- Rounded clips require **soft clipping** (coverage-based), consistent with ADR 0063.

This does **not** change ADR 0117: `bounds` remains a computation bound (not an implicit clip).

### 2) Introduce a renderer-internal clip mask substrate (v1: rect + rrect)

`fret-render` will introduce an internal representation for “clip used by postprocessing passes”:

- A fast-path `ScissorRect` (rectangular intersection).
- An optional **clip mask** that represents the current soft clip coverage.

This clip mask substrate is renderer-internal and does not leak into `fret-core`.

V1 scope:

- Support rectangular clip stacks via scissor-only (already exists).
- Support rounded-rect clip stacks via a soft clip mask (alpha coverage) derived from the clip stack.
- Support **mask resolution tiers** (full / half / quarter) to enable deterministic degradation under budgets (ADR 0118).

### 3) Deterministic degradation is required

When clip masks cannot be produced within budgets (ADR 0118), the renderer must degrade deterministically:

- Prefer degrading **mask quality** (e.g. lower-resolution mask) before disabling the effect entirely.
- If mask generation is disabled:
  - Effects may fall back to scissor-only behavior (rectangular clip) while preserving ordering.
  - The renderer may also disable the effect step(s) (consistent with ADR 0117 / ADR 0118).

Degradation must be layout-invariant and must not change hit-testing geometry.

### 3.1) Mask tier selection policy (v1)

The renderer chooses a **mask resolution tier** per effect boundary (per effective clip stack), subject to
`intermediate_budget_bytes` (ADR 0118).

Available tiers:

- `Mask0`: full-resolution mask for the effect viewport rect, `dst_size == viewport_rect.size`
- `Mask1`: half-resolution mask, `dst_size == downsampled_size(viewport_rect.size, 2)`
- `Mask2`: quarter-resolution mask, `dst_size == downsampled_size(viewport_rect.size, 4)`

`viewport_rect` is the bounded region the effect is operating on (e.g. `bounds` intersected with the effective
scissor/clip). The mask resource carries this rect so sampling can map viewport coordinates into the mask grid
deterministically under degradation.

#### Desired tier (by `EffectQuality`)

- `High`  -> prefer `Mask0`
- `Medium` -> prefer `Mask1`
- `Low`   -> prefer `Mask2`
- `Auto`  -> prefer `Mask0`

#### Degradation rule (deterministic)

If the preferred tier does not fit within `intermediate_budget_bytes`:

1) Degrade to the next lower tier (`Mask0 -> Mask1 -> Mask2`).
2) If no tier fits, do not allocate a mask texture:
   - use clip-stack sampling (`mask_uniform_index`) where supported, or
   - fall back to scissor-only behavior for rect-only clips (existing behavior).

This decision is based only on viewport size and budget, so it is deterministic and layout-invariant.

### 4) Mask semantics are coverage-based and must gate effect outputs

The renderer-internal clip mask represents **coverage** in `[0, 1]` for the effective clip stack:

- Rectangular clips may be represented as a hard scissor or hard mask.
- Rounded clips must be represented as a soft (anti-aliased) coverage mask (ADR 0063 / ADR 0030).

Effect pipelines must apply masks in a way that preserves visual correctness:

- The clip mask must gate **outputs** (writes/composites) of effect passes.
- Implementations may also use the mask to reduce work (early-out), but must not rely on input-only masking
  as the sole mechanism for correctness.

Rationale:

- For blur-like filters, masking only the input causes energy loss near the boundary and produces halos/seams.
  Output gating (coverage multiplication and/or masked composite) preserves smooth edges while keeping `bounds`
  as computation-only (ADR 0117).

## Semantics (Renderer v3)

### Bounds + scissor interaction (important)

`bounds` remains computation-only (ADR 0117). In particular:

- `FilterContent` composite must not treat `bounds` as an implicit clip. The clip mask must be derived from the
  effective clip stack only.
- `Backdrop` may scissor mask generation to the effect region for performance, because the mask is only consumed
  by passes that are already scoped by the same computation bounds.

### A) FilterContent

`FilterContent` is “saveLayer + imageFilter”. With clip masks:

1) Render children inside the effect group into an offscreen intermediate.
2) Apply the effect chain to the intermediate using:
   - `bounds` intersected with the current scissor (to bound computation),
   - optionally (future): a soft clip mask to improve edge quality for blur-like filters near rounded boundaries.
3) Composite back to the parent target using premultiplied alpha over (ADR 0040),
   applying the effective clip stack (but not treating `bounds` as a clip).

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
- Fullscreen effect passes sample the mask and gate outputs using coverage multiplication and/or masked compositing.

Benefits:

- Works uniformly for all fullscreen-style effect passes (blur/color adjust/pixelate).
- Can be generated at the same resolution as downsampled blur/pixelate for cost control.
- Does not require coupling every effect pipeline to stencil formats or depth attachments.

Implementation note:

- When the mask texture resolution differs from the viewport, sampling must map pixel coordinates into the mask
  grid deterministically (nearest sampling + explicit coordinate mapping), so that scissor/bounds remain
  computation-only and clip visibility stays stable under degradation.

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

- Whether some steps should use “hard discard” for performance (rect-only) vs always coverage-based (rounded clips).
- Nested rounded clips: whether to rasterize full clip stack into one mask, or limit to a bounded maximum and degrade.
- Path clips (`PushClipPath`): when and how to extend beyond rounded rects.

## Implementation Status (Branch-local)

- `FilterContent` composite binds the effect-boundary clip stack uniform so rounded clips do not leak on composite.
- `ClipMaskPass` can generate `Mask0` (full-resolution `R8Unorm` coverage) from the effective clip stack.
- `ClipMaskPass` can also generate `Mask1`/`Mask2` (half/quarter) and the plan chooses the best tier within
  `intermediate_budget_bytes`.
- When a blur step is forced to a quarter-resolution path under budgets (i.e. cannot afford the half-resolution
  scratch chain), the plan may cap the clip mask tier to `Mask2` to avoid allocating a full-resolution mask that
  would dominate the remaining budget.
- Quad rendering, clip-mask generation, and clip-evaluating masked fullscreen passes share a single analytic SDF +
  coverage foundation (fwidth-scaled AA), keeping rounded corners and soft clip coverage consistent (ADR 0030).
- Viewport-scoped clip masks are generated and sampled using an explicit `mask_viewport_origin` + `mask_viewport_size`
  (effect scissor rect) carried in the per-scope uniform, so tiered masks map deterministically even when the effect
  region is offset within the window.
- `ScaleNearest` and scissored writeback passes can sample the clip mask (`MaskRef`) to gate writes without per-pixel
  clip-stack iteration. `FilterContent` composite currently uses clip-stack sampling (no mask texture) because `bounds`
  is computation-only and the composite covers the full viewport.
- `RenderPlan` inserts `ClipMaskPass` opportunistically under `intermediate_budget_bytes`, otherwise falls back to clip-stack sampling via `mask_uniform_index`.

Remaining v1 follow-ups:

- Consider downsampled mask tiers for blur/pixelate pipelines.
- Share SDF + coverage helpers across quad rendering and clip-mask generation (ADR 0030).

## References

- Rounded clipping semantics: `docs/adr/0063-rounded-clipping-and-soft-clip-masks.md`
- Shape + analytic SDF semantics: `docs/adr/0030-shape-rendering-and-sdf-semantics.md`
- Transform + clip composition: `docs/adr/0078-scene-transform-and-clip-composition.md`
- RenderPlan substrate: `docs/adr/0116-renderer-architecture-v3-render-plan-and-postprocessing-substrate.md`
- Effect semantics: `docs/adr/0117-effect-layers-and-backdrop-filters-scene-semantics-v1.md`
- Budgets + degradation: `docs/adr/0118-renderer-intermediate-budgets-and-effect-degradation-v1.md`
- GPUI reference (rect content masks, premul blending): `repo-ref/zed/crates/gpui`
