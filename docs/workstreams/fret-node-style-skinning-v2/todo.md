---
title: fret-node style/skin layer v2 — TODO
status: active
date: 2026-03-01
scope: ecosystem/fret-node
---

# TODO

## M0 (contracts + gates)

- [ ] Land ADR 0307 (paint vs geometry style surfaces + invalidation rules).
- [ ] Add conformance gates:
  - [ ] Geometry style change rebuilds derived geometry + spatial index.
  - [ ] Paint-only style change does **not** rebuild derived geometry (retain v1 gate).

## M1 (split tokens: metrics vs palette)

- [ ] Replace the monolithic `NodeGraphStyle` with:
  - a geometry-affecting token bundle (metrics),
  - a paint-only token bundle (palette),
  - and an explicit revision/fingerprint split.
- [ ] Update cache keys so geometry caches depend on geometry fingerprint only.
- [ ] Rename public “xyflow” naming in APIs to neutral naming (keep references in docs only).
- [ ] Provide a single “compact defaults” helper preset for migration (typed, theme-friendly).

## M2 (per-entity geometry overrides, optional)

- [ ] Define a UI-only per-node/per-edge override surface for geometry (width/height/constraints,
      port metrics overrides, per-edge interaction width).
- [ ] Keep overrides out of the serialized graph document (same rule as `NodeGraphSkin`).
- [ ] Define deterministic resolution order:
  - base style → presenter → override provider → middleware (if any).

## Explicitly out of scope (v2)

- [ ] Stroke-space wire gradients (renderer mechanism work; track separately).

