# ADR 0309: Node graph per-entity paint overrides (v1)

- Status: Accepted
- Date: 2026-03-02
- Scope: `ecosystem/fret-node` (UI-only surface)

## Context

`ecosystem/fret-node` provides an editor-grade node graph canvas. The canvas already separates:

- `NodeGraphStyle` (theme-derived tokens; split into `paint` vs `geometry`)
- `NodeGraphSkin` (paint-only chrome hints; swap-at-runtime)
- `Graph` (domain model; serialized; must stay free of view-only concerns)

In upstream node editors (e.g. XyFlow / React Flow), per-node and per-edge style is commonly
customized via UI-only “style objects” (node `width/height`, edge `interactionWidth`, edge
`style`/`className`).

Fret needs an equivalent per-entity customization surface that:

1. Does **not** mutate or depend on serialized `Graph` state.
2. Can be changed frequently (interactive editing) without invalidating derived geometry or hit
   testing unnecessarily.
3. Uses Fret’s renderer-level `Paint` / `PaintBindingV1` contract (ADR 0306), so advanced looks
   (gradients/materials) remain possible without adding one-off APIs for the node graph.

ADR 0321 introduced the UI-only **geometry** override surface. This ADR defines the matching
UI-only **paint** override surface.

## Decision

Introduce a UI-only per-entity paint override surface for the node graph:

- New trait: `NodeGraphPaintOverrides` (UI-only; `Send + Sync`)
- New default implementation: `NodeGraphPaintOverridesMap` (mutable in-place; revision tracked)
- New optional canvas wiring: `NodeGraphCanvas::with_paint_overrides(...)`

Paint overrides must be **paint-only**:

- They may affect emitted `SceneOp`s and paint caches.
- They must **not** affect derived geometry, routing topology, spatial indexing, or hit-testing
  (except via existing interaction-width mechanisms from ADR 0321).

### v1 override surface (minimum)

The v1 surface should cover the most common “style object” needs while keeping the contract
strictly paint-only:

- Per-edge:
  - `stroke_paint: Option<PaintBindingV1>` (solid/gradient/material; eval space allowed)
  - `stroke_width_mul: Option<f32>` (paint-only thickness multiplier)
  - `dash: Option<DashPatternV1>` (paint-only; may reuse existing dash plumbing)
- Per-node:
  - `body_background: Option<PaintBindingV1>`
  - `border_paint: Option<PaintBindingV1>`
  - (Optional) `header_background: Option<PaintBindingV1>` if header chrome is part of the
    canonical node appearance for the host app.

Notes:

- Geometry-affecting fields (node size, edge interaction width) remain in ADR 0321 overrides.
- This ADR does not mandate *how* chrome is rendered (single-path vs multi-stroke outline/glow).
  That remains a policy decision expressed through `NodeGraphSkin` + presenter hints.

### Revision + cache invalidation

`NodeGraphPaintOverrides::revision()` is required and must be included in **paint cache keys**
only. It must not participate in derived-geometry keys.

Rationale: swapping paint (colors/gradients/material params) should not trigger expensive routing /
index rebuilds. Conversely, paint caches must be invalidated deterministically when overrides
change.

### Gradient/stroke guidance (non-normative)

The renderer already supports gradient paints (including stroke-space evaluation):

- For “wire gradient along the path”, use `PaintBindingV1 { paint: Paint::LinearGradient { ... },
  eval_space: PaintEvalSpaceV1::StrokeS01 }` and author the gradient in the `(s01, 0)` domain
  (e.g. `start=(0,0)`, `end=(1,0)`).
- For “viewport-fixed highlight”, use `PaintEvalSpaceV1::ViewportPx`.

Backends may deterministically degrade unsupported paint forms; such degradations must remain
contract-safe and should be covered by conformance gates in the renderer layer (ADR 0306).

## Alternatives considered

### A) Persist paint overrides into `Graph`

Rejected. Paint overrides are view-only, change frequently, and would pollute serialized graph
files. It also makes shared graph cores harder to reuse across different apps/themes.

### B) Add a generic “CSS-like style map” API

Deferred. A string-keyed style map is flexible but undermines type safety and cache key
determinism. If we need an escape hatch later, it should be layered on top of the typed override
surface with explicit normalization + fingerprinting rules.

### C) Only allow global themes/skins (no per-entity override)

Rejected. Editor-grade apps require per-entity styling (e.g. error states, selection groups,
semantic categories, runtime previews) without cloning whole theme presets or forking render code.

## Consequences

- The node graph gains an extensible, UI-only per-entity paint surface equivalent to upstream
  “style object” capabilities, without coupling the graph core to a UI policy layer.
- Implementations must keep the “paint-only” invariant strong: no geometry/hit-test behavior may
  depend on paint overrides.

## Follow-ups / Workstream

- Track implementation in `docs/workstreams/fret-node-style-skinning-v3/`.
- Add conformance gates:
  - paint overrides bump paint caches but do not rebuild geometry/index caches
  - per-edge overrides do not mutate serialized `Graph`
