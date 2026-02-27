---
title: fret-node style/skin layer v1 — TODO
status: active
date: 2026-02-27
scope: ecosystem/fret-node
---

# TODO

## M1 (chrome hints)

- [x] Add UI-only skin surface (`NodeGraphSkin`) with revision invalidation.
- [x] Plumb dashed wires end-to-end (renderer-native `StrokeV2.dash`) + cache key coverage.
- [x] Per-node header palette (header background + title text color) on the main paint path.
- [x] Port chrome hints (paint-only) for fill/stroke/inner scale with conformance coverage.
- [ ] Node chrome: selected/hover/focus ring contract (paint-only) with conformance tests.
- [ ] Port chrome hints: shape enum contract (only `Circle` implemented).
- [ ] Edge chrome hints: marker/arrowhead overrides (policy stays in ecosystem).
- [ ] Ensure skin ordering is explicit and deterministic (style → presenter → edgeTypes → skin).

## M2 (theme integration + presets)

- [ ] Add kit-level presets: `Dify`, `Blueprint`, `ShaderGraph` (pure function of `ThemeSnapshot`).
- [ ] Demo toggle to switch presets at runtime (paint-only invalidation).
- [ ] Add one scripted/diag gate for “preset switch is paint-only” (optional but recommended).

## Follow-ups (likely)

- [ ] Split “paint-only” vs “geometry-affecting” hint surfaces (or add explicit invalidation keys).
- [ ] Consider a recipe-oriented layer (kit) to compose hints (similar to shadcn recipes).
