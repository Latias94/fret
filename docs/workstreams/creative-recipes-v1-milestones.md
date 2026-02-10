# Creative Recipes (v1) — Milestones

This file tracks milestone gates for the creative recipes workstream.

See:

- Overview: `docs/workstreams/creative-recipes-v1.md`
- Task list: `docs/workstreams/creative-recipes-v1-todo.md`

## M0 — Recipe foundation (ecosystem-only)

- Introduce a shared “recipe resolve + fallback reporting” shape in `ecosystem/fret-ui-kit`.
- Keep existing public helpers (e.g. glass/pixelate wrappers) stable; migrate internals only.
- Add a minimal diagnostics sink seam for “recipe degraded” events (best-effort).

## M1 — Paint v1 (gradients)

- Land `Paint` and gradient evaluation (ADR 1172) and wire it to `SceneOp::Quad` (and optionally `Path`).
- Add renderer conformance tests for linear/radial mapping and deterministic sanitization.

## M2 — Materials v1 (params-only)

- Land `MaterialId` + registration API and baseline kinds (ADR 1174).
- Implement determinism rules (explicit seeds, no hidden time; ADR 1183).
- Add renderer telemetry counters and a minimal “procedural paint conformance” test.

## M3 — Masks v1 (gradient alpha masks)

- Land `PushMask/PopMask` (ADR 1178) with gradient-only sources.
- Add conformance tests for coverage correctness and clip/effect interaction boundaries.

## M4 — Compositing groups v1 (blend modes)

- Land `PushCompositeGroup/PopCompositeGroup` (ADR 1180) with a small blend vocabulary.
- Budget intermediates and deterministic degradation (ADR 0120).

## M5 — Motion + pointer snapshots

- Land frame clock reads (non-reactive) and pointer motion snapshot seam (ADR 1179 / ADR 1182).
- Add a reduced-motion policy helper in `fret-ui-kit` and verify fallback behavior.

## M6 — Effect steps extension

- Land `ColorMatrix` and `AlphaThreshold` effect steps (ADR 1175) with conformance tests.
- Provide a “bloom-like” Tier B recipe example using threshold + blur + additive composite (depends on M4).

