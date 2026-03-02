# Fret node graph style & skinning (v3)

Status: Draft

This workstream continues the node graph styling work after:

- v1: paint vs geometry token split (`NodeGraphStyle`)
- v2: UI-only per-entity geometry overrides (`NodeGraphGeometryOverrides`)

v3 focuses on **per-entity paint overrides** so host apps can express “style object” style changes
(edge stroke paint/width/dash; node border/background paint) without mutating the serialized
`Graph` or forcing geometry cache rebuilds.

## Goals

- Add a UI-only per-entity paint override surface for nodes/edges (ADR 0309).
- Keep strict separation:
  - Geometry caches depend on geometry tokens + geometry overrides only.
  - Paint caches depend on paint tokens + skin + paint overrides only.
- Enable “editor-grade” looks (including gradients/materials) by reusing the renderer-level
  `Paint` / `PaintBindingV1` contract (ADR 0306) rather than inventing node-graph-specific paint
  types.

## Non-goals (v3)

- Changing the persisted graph data model to include styling.
- Implementing every upstream “CSS property” equivalent.
- Forcing a specific visual policy (outlines/glows/selection rings remain policy via skin/presenter
  hints).

## References (non-normative)

- XyFlow / React Flow edge: `interactionWidth` + `style?: CSSProperties` (`repo-ref/xyflow`)
- Fret contracts:
  - ADR 0306: paint evaluation spaces (`PaintEvalSpaceV1`, `PaintBindingV1`)
  - ADR 0307: node-graph geometry style surface (paint vs geometry)
  - ADR 0308: per-entity geometry overrides (UI-only)
  - ADR 0309: per-entity paint overrides (this workstream)

