# Viewport Gizmo Workstream (Roadmap)

Notes only. This document is a tactical roadmap for implementing editor-grade 3D transform gizmos in Fret.
It is not the project source of truth; see `docs/roadmap.md` for priorities.

## North Star

Provide Unity/Godot/Unreal-class transform gizmos for engine viewports:

- depth-tested 3D rendering (occlusion, depth bias, engine AA),
- consistent “screen-space size” feel,
- mature interaction defaults (modes, snapping, local/world, pivot),
- clean separation between framework mechanisms and editor policy.

Locked boundary:

- 3D gizmo geometry renders in the engine pass, not as UI `SceneOp` primitives:
  `docs/adr/0130-viewport-gizmos-engine-pass-and-ui-overlay-boundary.md`.

## Current State

- Engine viewport integration hook exists: `WinitAppDriver::record_engine_frame`
  (`apps/fret-examples/src/plot3d_demo.rs` is an end-to-end reference).
- Gizmo subsystem exists as an ecosystem crate: `ecosystem/fret-gizmo`
  - built-in transform manipulator (Translate/Rotate/Scale/Universal) with begin/update/commit/cancel phases
  - view gizmo (`ViewGizmo`) for camera orbit/snaps
  - pick primitives + policy (`picking.rs`, `GizmoPickPolicy`)
  - plugin contract + manager (`GizmoPlugin`, `GizmoPluginManager`) and example plugins
- End-to-end demo exists: `apps/fret-examples/src/gizmo3d_demo.rs`
  - engine-pass depth-tested rendering via viewport overlay hooks
  - UI overlay HUD readouts above the viewport surface (ADR 0130 boundary)

## MVPs

### MVP 0 — Contracts + Scaffolding (DONE)

- [x] Add ADR 0130 and link from ADR index.
- [x] Create `ecosystem/fret-gizmo` crate.
- [x] Use `glam` internally for math.

### MVP 1 — Depth-tested Translate Gizmo (FOUNDATION)

Goal: demonstrate real engine-pass depth testing with a minimal transform gizmo, end-to-end.

Deliverables:

- `ecosystem/fret-gizmo`:
  - translate mode: axis picking + drag update producing `Transform3d` deltas
  - engine-facing draw output (lines + triangles) with explicit `DepthMode` (`Test/Ghost/Always`)
  - stable interaction phases: begin/update/commit/cancel
- `apps/fret-examples` demo:
  - engine pass renders:
    - a simple depth-tested scene primitive (e.g. cube/plane)
    - the gizmo lines into the same target
  - `ViewportInputEvent` routes to the gizmo tool manager
  - UI overlays (optional): show mode + snapping status text above viewport

Acceptance criteria:

- gizmo is occluded correctly by scene geometry when depth-tested,
- optional “always on top” mode works (no depth test),
- drag produces stable deltas and a single commit boundary (undo integration later).

### MVP 2 — Rotate + Scale + Universal

Goal: reach parity with the standard transform tool surface area.

Deliverables:

- rotation rings (axis rings + view-axis ring) with robust picking
- scale handles (axis + uniform)
- universal mode (translate + rotate + uniform scale)
- local/world orientation toggle

Acceptance criteria:

- interactions are stable under camera motion and varying depth,
- gizmo maintains constant on-screen size.

### MVP 3 — Snapping + Pivot + Multi-selection

Goal: editor-grade feel and predictable multi-target behavior.

Deliverables:

- snapping:
  - modifier-gated enable/disable
  - per-mode step sizes (translation/rotation/scale)
- pivot policies:
  - median pivot
  - active selection pivot (if available)
  - optional “individual origins” mode
- multiple targets update strategy (apply delta in world or local)

Acceptance criteria:

- snapping is deterministic and consistent across platforms,
- multi-selection behavior matches a documented pivot policy.

### MVP 4 — Visual Polish + Performance

Goal: “looks good” at editor scale.

Deliverables:

- occluded segments feedback (fade/desaturate, optional dashed/ghost)
- thicker lines with proper AA (engine-side)
- optional filled shapes for plane handles / ring thickness
- performance budgets and caching strategy (avoid per-frame allocations)

## TODO (Detailed)

### `ecosystem/fret-gizmo`

- Define a stable public API surface:
  - `Gizmo::update(...)` (input + camera + targets -> phase/result)
  - `Gizmo::draw(...) -> GizmoDrawList3d` (engine-pass 3D draw lists; depth mode explicit)
  - UI overlay (labels/HUD) remains an app/UI-layer concern rendered above the viewport surface (ADR 0130);
    see `apps/fret-examples/src/gizmo3d_demo.rs` for an end-to-end reference.
- Decide math types at API boundary:
  - internal `glam`; external: `mint` or Fret-owned lightweight structs
- Picking math:
  - ray construction from screen coords
  - closest points between ray and axis
  - ray/plane intersections for plane handles
  - ring intersection for rotation
- Interaction contract:
  - begin/update/commit/cancel phases
  - capture semantics: active handle lock while dragging

### Demo / Integration

- Add a new demo binary that:
  - allocates a color target + a depth texture
  - draws a depth-tested gizmo pass after scene geometry
  - uses `ViewportInputEvent` to drive the gizmo
- Keep engine work entirely in `record_engine_frame`.

## Non-goals (for early MVPs)

- Full editor undo/redo integration (tracked by ADR 0127 direction).
- GPU-agnostic “draw list” in Fret UI for 3D geometry.
- Editor plugin API for third-party tools (future).
