# Creative Recipes (v1) — TODO

This is a working TODO list for landing the creative recipes surface end-to-end.

See:

- Overview: `docs/workstreams/creative-recipes-v1.md`
- Milestones: `docs/workstreams/creative-recipes-v1-milestones.md`

## P0 — Foundation (ecosystem)

- [x] Define `ResolveCtx` + `ResolvedWithFallback<T>` + `DegradationReason` in `ecosystem/fret-ui-kit`.
- [x] Add a minimal diagnostics seam for `RecipeDegraded { label, reason }` (best-effort).
- [x] Migrate existing recipes (glass/pixelate) to the shared resolve/fallback/report shape without
      changing their public wrapper signatures.
- [x] Create a `VisualCatalog`/`MaterialCatalog` skeleton API and decide where it is stored (app model vs service).

Next (recommended order):

1. `VisualCatalog`/`MaterialCatalog` skeleton (P0 ecosystem foundation)
2. M7: sampled materials v2a (catalog textures; ADR 0242)
3. M8: `ecosystem/fret-ui-magic` Phase 0
4. M9: external texture imports (contract-path demo + capability-gated backend)

## P0 — Kernel primitives (scene + renderer)

- [x] Paint v1 (ADR 0233):
  - [x] Add `Paint` types and wire to `SceneOp::Quad`.
  - [x] Add conformance tests for gradient mapping + sanitization.
- [x] Materials v1 (ADR 0235):
  - [x] Add `MaterialId` and registry API.
  - [x] Implement baseline kinds: `DotGrid`, `Grid`, `Checkerboard`, `Stripe`, `Noise`, `Beam`,
        `Sparkle`, `ConicSweep`.
  - [x] Add deterministic sanitization + seed/time rules (ADR 0244).
- [x] Masks v1 (ADR 0239):
  - [x] Add `PushMask/PopMask` ops and gradient mask evaluation.
  - [x] Add conformance tests for edge coverage and effect/clip interaction.
- [x] Composite groups v1 (ADR 0247):
  - [x] Add `PushCompositeGroup/PopCompositeGroup` and blend mode support.
  - [x] Wire budgets + deterministic degradation.

## P0 — Motion/pointer seams (UI runtime)

- [x] Frame clock snapshot read (ADR 0240) exposed to widget contexts (non-reactive).
- [x] Pointer motion snapshot + local mapping helper surface (ADR 0243 / ADR 0238).

## P0 — Authoring seams (UI mechanism surface)

These are the `fret-ui` authoring surfaces that make the kernel primitives (mask/composite) usable
from ecosystem recipes without falling back to ad-hoc canvas-only hacks.

- [ ] Add a mask layer element in `crates/fret-ui` that emits `SceneOp::PushMask/PopMask` (ADR 0239).
- [ ] Add a compositing group element in `crates/fret-ui` that emits
      `SceneOp::PushCompositeGroup/PopCompositeGroup` (ADR 0247).

## P1 — Recipes and demos

- [ ] MagicUI parity recipes/wrappers:
  - [x] `MagicCard` (pointer-follow radial gradient fill/border).
  - [x] `Lens` (radial mask + content scale + reduced-motion behavior).
  - [ ] `BorderBeam` (beam material + mask/composite; deterministic animation).
  - [ ] Patterns: dot/grid/stripe + animated variants.
  - [ ] Sparkles text (seeded sparkle field; reduced-motion fallback).
- [ ] Add UI gallery entries and `fretboard diag` scripts (screenshots + perf baselines).

## P1 — Effect steps extension

- [x] Implement `ColorMatrix` + `AlphaThreshold` steps (ADR 0236).
- [ ] Add a “bloom-like” recipe example (threshold -> blur -> add) once blend groups exist.

## P1 — Sampled materials (v2a, catalog textures)

This is the recommended first step for ADR 0242: sampled materials bind a renderer-owned catalog
texture selected at registration time (no per-instance `ImageId` yet).

- [x] Define `BindingShape::ParamsPlusCatalogTexture` in the renderer material registry and
      capability-gate it (ADR 0124).
- [x] Add a small catalog texture set (blue-noise/dither) and wire upload + lifetime to the renderer.
- [x] Implement at least one sampled baseline material kind (e.g. a higher quality noise/dither
      overlay) that uses the catalog texture in the shader.
- [x] Add a conformance test for sampled material rendering and deterministic fallbacks.

## P1 — `fret-ui-magic` (Phase 0)

Land a MagicUI-aligned ecosystem crate that composes the existing kernel primitives via
`fret-ui-kit` recipes. The goal is fast “creative baseline” parity with stable fallbacks and
diagnostics, not perfect CSS parity.

- [x] Create `ecosystem/fret-ui-magic` (crate + minimal public surface).
- [ ] Implement 3–5 seed components (Phase 0):
  - [x] `Lens`
  - [x] `MagicCard`
  - [ ] `BorderBeam`
  - [x] `Marquee`
  - [ ] `Dock`
- [ ] Add UI gallery entries + `fretboard diag` scripts for each seed component:
  - [x] `Marquee`
  - [x] `Lens`
  - [x] `MagicCard`
  - [ ] `BorderBeam`
  - [ ] `Dock`
- [ ] Verify deterministic behavior under `--fixed-frame-delta-ms` (diag-controlled time).

## P1 — External texture imports (v1)

This closes the loop for “real import and run” beyond `ImageId` uploads: platform decoders and/or
external systems produce GPU textures and the runner imports them without leaking backend handles
to `fret-ui` (ADR 0234).

- [ ] Land a “contract-path demo” that imports a renderer-owned `wgpu::TextureView` via
      `ImportedViewportRenderTarget` and shows it in the UI as a `ViewportSurface`:
  - [ ] resize + fit + lifecycle
  - [ ] diag bundle evidence (snapshot + screenshot)
  - [ ] perf baseline for steady-state updates
- [ ] Add capability gating for a first real backend path:
  - [ ] web: `VideoFrame`/WebCodecs (if available), or a copy-based fallback
  - [ ] native: a decode path (software or hardware) with a clear copy/zero-copy policy
