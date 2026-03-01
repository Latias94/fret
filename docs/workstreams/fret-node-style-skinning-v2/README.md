---
title: fret-node style/skin layer v2 (geometry-affecting styling)
status: active
date: 2026-03-01
scope: ecosystem/fret-node
---

# fret-node style/skin layer v2 (geometry-affecting styling)

v1 established a **paint-only** skin surface (`NodeGraphSkin`) with explicit cache-safe invalidation.
This v2 workstream adds the missing half needed to match editor-grade node graph systems like
XyFlow: **layout / hit-testing / geometry-affecting style knobs**, with explicit invalidation
contracts.

Hard boundaries (kept):

- `Graph` stays document/logic only (no UI policy, no serialized styling).
- `NodeGraphSkin` stays **UI-only** and remains paint-first (per-entity chrome hints).

What changes in v2:

- Split the node graph “style bundle” into **paint tokens** vs **geometry tokens**.
- Ensure geometry-affecting style changes rebuild:
  - derived geometry,
  - spatial index / hit-testing,
  - edge routing anchors (where applicable).
- Keep paint-only switching fast and stable (no geometry rebuild).

Primary reference outcome:

- XyFlow allows both:
  - global theming via CSS variables, and
  - per-node/per-edge overrides via `node.style` / `edge.style` (including width/height).

In Fret terms, this becomes:

- a theme-derived base style (stable, typed tokens), and
- an optional UI-only per-entity override surface (policy/recipes live in ecosystem).

Canonical tracking docs:

- TODO: `docs/workstreams/fret-node-style-skinning-v2/todo.md`
- Milestones: `docs/workstreams/fret-node-style-skinning-v2/milestones.md`

Design contract (source of truth):

- ADR: `docs/adr/0307-node-graph-geometry-style-surface-v1.md`

## Current status (2026-03-01)

- M0 is complete: ADR + invalidation conformance gates are in place.
- M1 is complete: `NodeGraphStyle` is split into paint vs geometry token bundles and geometry
  caches key off the geometry fingerprint only.
- Next: M2 (optional) — add a UI-only per-entity geometry override surface (type-safe, bounded).
