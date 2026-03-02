# fret-node-style-skinning-v3 milestones

## M0: Contract

- [ ] ADR 0309 accepted
- [ ] Evidence anchors identified (where paint caches live; where overrides apply)

## M1: UI-only paint overrides surface (no behavior yet)

- [x] Define `NodeGraphPaintOverrides` trait (revision + per-node/per-edge queries)
- [x] Add `NodeGraphPaintOverridesMap` reference implementation
- [x] Plumb optional `paint_overrides` handle through the canvas widget builder

## M2: Paint cache invalidation + conformance gates

- [x] Include `paint_overrides.revision()` in paint cache keys only
- [x] Conformance: paint overrides change does **not** rebuild derived geometry/index
- [ ] Conformance: overrides do not mutate serialized `Graph`

## M3: Per-edge paint overrides

- [ ] Implement `EdgePaintOverrideV1` resolution order:
  - presenter hint (per edge) → paint overrides → skin hints → `NodeGraphStyle.paint` defaults
- [ ] Support `stroke_paint`, `stroke_width_mul`, and `dash` overrides

## M4 (deferred): Gradient/material wire styling recipes

- [ ] Provide a small “wire paint cookbook” doc:
  - solid
  - linear gradient in `StrokeS01`
  - viewport-fixed highlight
  - material-based animated “flow” (if enabled)
- [ ] Add a demo / diagnostic harness in `apps/` to validate wire paint presets visually
