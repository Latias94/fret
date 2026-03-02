# fret-node-style-skinning-v3 milestones

## M0: Contract

- [x] ADR 0309 accepted
- [x] Evidence anchors identified (where paint caches live; where overrides apply)

## M1: UI-only paint overrides surface (no behavior yet)

- [x] Define `NodeGraphPaintOverrides` trait (revision + per-node/per-edge queries)
- [x] Add `NodeGraphPaintOverridesMap` reference implementation
- [x] Plumb optional `paint_overrides` handle through the canvas widget builder

## M2: Paint cache invalidation + conformance gates

- [x] Include `paint_overrides.revision()` in paint cache keys only
- [x] Conformance: paint overrides change does **not** rebuild derived geometry/index
- [x] Conformance: overrides do not mutate serialized `Graph`

## M3: Per-edge paint overrides

- [x] Implement `EdgePaintOverrideV1` resolution order:
  - `NodeGraphStyle.paint` defaults → presenter/edge_types hint → skin hints → paint overrides
- [x] Support `stroke_paint`, `stroke_width_mul`, and `dash` overrides
- [x] Markers reuse resolved wire paint binding (except `StrokeS01`, which conservatively falls back)

## M3b: Per-node paint overrides

- [x] Support `body_background`, `border_paint`, and `header_background` overrides

## M4 (deferred): Gradient/material wire styling recipes

- [ ] Provide a small “wire paint cookbook” doc:
  - solid
  - linear gradient in `StrokeS01`
  - viewport-fixed highlight
  - material-based animated “flow” (if enabled)
- [x] Provide a JSON-loadable theme preset file + parsing entry-point for iteration
- [ ] Add a demo / diagnostic harness in `apps/` to validate wire paint presets visually
