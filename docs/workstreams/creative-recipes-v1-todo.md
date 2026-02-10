# Creative Recipes (v1) — TODO

This is a working TODO list for landing the creative recipes surface end-to-end.

See:

- Overview: `docs/workstreams/creative-recipes-v1.md`
- Milestones: `docs/workstreams/creative-recipes-v1-milestones.md`

## P0 — Foundation (ecosystem)

- [ ] Define `ResolveCtx` + `ResolvedWithFallback<T>` + `DegradationReason` in `ecosystem/fret-ui-kit`.
- [ ] Add a minimal diagnostics seam for `RecipeDegraded { label, reason }` (best-effort).
- [ ] Migrate existing recipes (glass/pixelate) to the shared resolve/fallback/report shape without
      changing their public wrapper signatures.
- [ ] Create a `VisualCatalog`/`MaterialCatalog` skeleton API and decide where it is stored (app model vs service).

## P0 — Kernel primitives (scene + renderer)

- [ ] Paint v1 (ADR 1172):
  - [ ] Add `Paint` types and wire to `SceneOp::Quad`.
  - [ ] Add conformance tests for gradient mapping + sanitization.
- [ ] Materials v1 (ADR 1174):
  - [ ] Add `MaterialId` and registry API.
  - [ ] Implement baseline kinds: `DotGrid`, `Grid`, `Checkerboard`, `Stripe`, `Noise`, `Beam`,
        `Sparkle`, `ConicSweep`.
  - [ ] Add deterministic sanitization + seed/time rules (ADR 1183).
- [ ] Masks v1 (ADR 1178):
  - [ ] Add `PushMask/PopMask` ops and gradient mask evaluation.
  - [ ] Add conformance tests for edge coverage and effect/clip interaction.
- [ ] Composite groups v1 (ADR 1180):
  - [ ] Add `PushCompositeGroup/PopCompositeGroup` and blend mode support.
  - [ ] Wire budgets + deterministic degradation.

## P0 — Motion/pointer seams (UI runtime)

- [ ] Frame clock snapshot read (ADR 1179) exposed to widget contexts (non-reactive).
- [ ] Pointer motion snapshot + local mapping helper surface (ADR 1182 / ADR 1177).

## P1 — Recipes and demos

- [ ] MagicUI parity recipes/wrappers:
  - [ ] `MagicCard` (pointer-follow radial gradient fill/border).
  - [ ] `Lens` (radial mask + content scale + reduced-motion behavior).
  - [ ] `BorderBeam` (beam material + mask/composite; deterministic animation).
  - [ ] Patterns: dot/grid/stripe + animated variants.
  - [ ] Sparkles text (seeded sparkle field; reduced-motion fallback).
- [ ] Add UI gallery entries and `fretboard diag` scripts (screenshots + perf baselines).

## P1 — Effect steps extension

- [ ] Implement `ColorMatrix` + `AlphaThreshold` steps (ADR 1175).
- [ ] Add a “bloom-like” recipe example (threshold -> blur -> add) once blend groups exist.

