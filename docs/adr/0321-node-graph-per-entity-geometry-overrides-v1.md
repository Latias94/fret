# ADR 0321: Node Graph Per-Entity Geometry Overrides (v1)

Status: Proposed

## Context

`ecosystem/fret-node` targets editor-grade node graphs. Reference systems (XyFlow / React Flow,
Unreal/Unity shader graphs, etc.) commonly expose a per-entity “escape hatch” for layout-affecting
styling:

- per-node width/height overrides (`node.style.width/height` in XyFlow),
- per-edge interaction width overrides (`edge.interactionWidth` in XyFlow),
- occasional per-node/per-edge metric tweaks needed for domain-specific node families.

Fret has two existing, intentionally separated styling surfaces:

- **Base tokens**: `NodeGraphStyle` (typed, theme-derived).
- **Paint-only per-entity chrome**: `NodeGraphSkin` (UI-only; must not affect geometry/hit-testing).

ADR 0320 split style into **paint** vs **geometry** planes and reserved space for per-entity
geometry overrides. This ADR defines that missing per-entity surface.

Hard boundary:

- `Graph` remains document/logic only: it must not serialize UI-only styling overrides.

## Decision

### D1 — Add a UI-only per-entity geometry override provider

Introduce a new UI-only surface:

- `NodeGraphGeometryOverrides` (name TBD, but must be neutral; no upstream product names in APIs)

with the minimum contract:

- `revision() -> u64` (explicit invalidation; geometry-affecting).
- `node_geometry_override(node) -> NodeGeometryOverrideV1` (optional width/height overrides).
- `edge_geometry_override(edge) -> EdgeGeometryOverrideV1` (optional interaction-width overrides).

Notes:

- Overrides are **typed** and **bounded**: no arbitrary “CSS style bag”.
- Overrides are **UI-only**: they do not live in `Graph`, and must not be persisted by default.

### D2 — Deterministic resolution order

The canvas resolves geometry for a node/edge with a deterministic order:

1) Serialized graph geometry (when present): e.g. `node.size` (document-level explicit size).
2) Per-entity geometry overrides (this ADR).
3) Presenter / measured hints (`NodeGraphPresenter` and wrappers like measured presenters).
4) Global defaults from `NodeGraphStyle.geometry`.

Rationale:

- document data wins (stable persistence),
- UI-only overrides act like “local layout policy”,
- presenter/measured hints remain a fallback,
- global style stays the baseline.

### D3 — Geometry invalidation is explicit and cache-safe

Any change that can affect:

- derived node bounds / port anchors,
- hit-testing spatial indexes,
- edge routing anchors,

must be captured by:

- the override provider `revision()` (or an equivalent fingerprint), and
- the derived-geometry cache key (directly or via an equivalent “forced rebuild” mechanism).

Paint-only caches must not rebuild when only geometry overrides are unchanged.

### D4 — Scope is “metrics only”; paint stays in `NodeGraphSkin`

This ADR explicitly keeps paint in the paint plane:

- Colors / dashes / shadows / rings remain in `NodeGraphStyle.paint` + `NodeGraphSkin`.
- Geometry overrides cover only **layout / hit-testing / routing** knobs.

This keeps the mechanism/policy boundary stable: “paint policy” can iterate without risking silent
geometry churn.

## Consequences

- This adds a third styling surface for node graphs (base style, paint-only skin, geometry overrides)
  but keeps each surface narrow and auditable.
- Ecosystem crates can build higher-level APIs that feel like XyFlow’s `node.style` / `edge.style`,
  while the core contracts remain typed and cache-safe.

## Out of scope

- Stroke-space wire gradients (renderer + paint evaluation space work; track separately).
- Arbitrary per-entity “CSS bag” style overrides.
