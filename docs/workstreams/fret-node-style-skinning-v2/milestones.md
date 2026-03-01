---
title: fret-node style/skin layer v2 — Milestones
status: active
date: 2026-03-01
scope: ecosystem/fret-node
---

# Milestones

## M0 — Contracts + invalidation gates

Status: Done (2026-03-01).

Acceptance:

- An ADR defines the split between paint-only vs geometry-affecting styling surfaces.
- There is a conformance test proving:
  - geometry style changes rebuild derived geometry + hit-testing indexes, and
  - paint-only changes do not.

Evidence anchors (target):

- ADR: `docs/adr/0307-node-graph-geometry-style-surface-v1.md`
- Geometry cache key: `ecosystem/fret-node/src/ui/canvas/widget/derived_geometry/cache_keys.rs`
- Conformance tests under: `ecosystem/fret-node/src/ui/canvas/widget/tests/`

## M1 — Geometry tokens are first-class

Status: Done (2026-03-01).

Acceptance:

- Node graph style tokens are split into:
  - geometry tokens (metrics) and
  - paint tokens (palette).
- Geometry caches are keyed by geometry fingerprint only.
- Neutral naming is used in public APIs (no upstream product names in the surface).

## M2 — Per-entity layout overrides (optional)

Acceptance:

- There is a UI-only override surface for per-node/per-edge geometry (like XyFlow `node.style` /
  `edge.style`), but type-safe and contract-bounded.
- Overrides participate in geometry invalidation and do not leak into `Graph` persistence.
