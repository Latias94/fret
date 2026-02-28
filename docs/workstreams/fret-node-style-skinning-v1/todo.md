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
- [x] Node chrome: selected/focus ring contract (paint-only) with conformance tests.
- [x] Port chrome hints: shape enum + Diamond/Triangle paint implementation (paint-only).
- [ ] Edge chrome hints: marker/arrowhead overrides (policy stays in ecosystem).
- [ ] Ensure skin ordering is explicit and deterministic (style → presenter → edgeTypes → skin).

## M2 (theme integration + presets)

- [x] Add built-in preset families: `WorkflowClean`, `SchematicContrast`, `GraphDark` (paint-only JSON, hard-coded palette).
- [x] Demo toggle to switch presets at runtime (paint-only invalidation).
- [x] Add in-tree presets derived from `ThemeSnapshot` (`NodeGraphPresetSkinV1::new_from_snapshot`).
- [ ] Extract kit-level presets derived from `ThemeSnapshot` (pure function + documented token mapping).
- [x] Add one scripted/diag gate for “preset switch is paint-only” (node graph demo script).
  - Script: `node-graph-demo-preset-families-paint-only` (`tools/diag-scripts/extras/node-graph-demo-preset-families-paint-only.json`)

## M3 (blueprint-grade effects, v0)

- [x] Node shadow/glow via renderer effect (`PushEffect` + `DropShadowV1`) under `NodeChromeHint.shadow`.
- [x] Wire glow via renderer effect (`PushEffect` + `DropShadowV1`) for selected edges and drag preview wires.
- [x] Add demo toggle for wire glow (`primary+shift+g`) and capture both variants in the diag gate script.
- [ ] Wire outline / dual-stroke strategy (cheap “blueprint readability” baseline, even when effects degrade).
- [ ] Wire gradients (mechanism-level) or a policy-level approximation (two-pass stroke with cached paths).

## Follow-ups (likely)

- [ ] Split “paint-only” vs “geometry-affecting” hint surfaces (or add explicit invalidation keys).
- [ ] Consider a recipe-oriented layer (kit) to compose hints (similar to shadcn recipes).
- [ ] Clarify grid stroke width override semantics (minor vs major).
