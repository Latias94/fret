---
title: fret-node style/skin layer v2 — TODO
status: active
date: 2026-03-01
scope: ecosystem/fret-node
---

# TODO

## M0 (contracts + gates)

- [x] Land ADR 0307 (paint vs geometry style surfaces + invalidation rules).
- [x] Add conformance gates:
  - [x] Geometry style change rebuilds derived geometry + spatial index.
  - [x] Paint-only style change does **not** rebuild derived geometry (retain v1 gate).

## M1 (split tokens: metrics vs palette)

- [x] Replace the monolithic `NodeGraphStyle` with:
  - a geometry-affecting token bundle (metrics),
  - a paint-only token bundle (palette),
  - and an explicit revision/fingerprint split.
- [x] Update cache keys so geometry caches depend on geometry fingerprint only.
- [x] Rename public “xyflow” naming in APIs to neutral naming (keep references in docs only).
- [x] Provide a single “compact defaults” helper preset for migration (typed, theme-friendly).

## M2 (per-entity geometry overrides, optional)

- [ ] Land ADR 0308 (UI-only per-entity geometry overrides + invalidation rules).
- [ ] Define a UI-only per-node/per-edge override surface for geometry:
  - node size overrides (width/height, screen-space px),
  - per-edge interaction-width overrides (screen-space px).
- [ ] Keep overrides out of the serialized graph document (same rule as `NodeGraphSkin`).
- [ ] Define deterministic resolution order:
  - graph document size → overrides → presenter/measured hints → style defaults.
- [ ] Add conformance gates:
  - overrides revision invalidates derived geometry + spatial index,
  - overrides do not leak into `Graph` persistence.

## Explicitly out of scope (v2)

- [ ] Stroke-space wire gradients (renderer mechanism work; track separately; do not block M2).
