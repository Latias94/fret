# ADR 0130: Viewport Gizmos — Engine-pass Rendering and UI Overlay Boundary

Status: Proposed

## Context

Fret targets editor-grade workflows with embedded engine viewports (`SceneOp::ViewportSurface`, ADR 0007),
multi-window docking, and overlay-heavy UI composition. Viewport tooling (selection, gizmos, snapping, camera
navigation) is explicitly editor/app scope (ADR 0027 / ADR 0049), but Fret must provide infrastructure
contracts that make Unity/Godot/Unreal-class gizmos feasible without embedding tool policy into `fret-ui`.

The key requirement for transform gizmos in a game engine editor is **true 3D rendering with depth testing**
(occlusion, depth bias, MSAA/TAA integration), not an approximation using 2D UI primitives.

At the same time, editor UIs routinely need *screen-space* affordances anchored to the viewport (labels,
tooltips, HUD readouts, selection rectangles). These must remain composable with docking and overlays
(ADR 0011 / ADR 0009).

References:

- Viewport embedding: `docs/adr/0007-viewport-surfaces.md`
- Viewport input forwarding: `docs/adr/0025-viewport-input-forwarding.md`
- Frame lifecycle and engine integration points: `docs/adr/0015-frame-lifecycle-and-submission-order.md`,
  `docs/adr/0038-engine-render-hook-and-submission-coordinator.md`
- Renderer ordering contract: `docs/adr/0002-display-list.md`, `docs/adr/0009-renderer-ordering-and-batching.md`
- Canvas/interactive surfaces guidance: `docs/adr/0128-canvas-widgets-and-interactive-surfaces.md`
- 3D gizmo design reference (update/draw split + visuals vocabulary): optional `repo-ref/transform-gizmo`
  (not always present; see `docs/repo-ref.md`).
- Godot editor viewport hooks (input + draw-over-viewport + explicit overlay update): optional `repo-ref/godot`
  checkout (not always present; see `docs/repo-ref.md`):
  `repo-ref/godot/editor/plugins/editor_plugin.h`

## Goals

1. Make **depth-tested 3D transform gizmos** a first-class, portable integration outcome.
2. Keep Fret’s core/runtime crates mechanism-only; keep gizmo/tool policy ecosystem/app-owned (ADR 0027).
3. Preserve strict scene ordering semantics for viewport + UI overlays (ADR 0009).
4. Support both:
   - engine-pass gizmo rendering (3D, depth tested), and
   - UI overlay affordances (2D, screen-space, optional).
5. Enable a “good-looking, easy-to-use” baseline aligned with mature editor expectations (Unity/Godot-class),
   without committing those policies to `fret-ui`.

## Non-goals

- Standardize gizmo behavior policy in `fret-ui` (hotkeys, snapping rules, axis selection heuristics).
- Add a 3D renderer to `fret-render` or encode 3D depth semantics into `SceneOp`.
- Require a specific engine architecture (ECS, scene graph, selection model).

## Decision

### 1) 3D gizmos are rendered in the engine pass (depth tested), not as UI `SceneOp` primitives

Transform gizmo geometry that must participate in depth testing is rendered by the engine into the same render
target that backs the viewport surface.

Fret continues to treat viewport content as an opaque engine-owned output via:

- `SceneOp::ViewportSurface { target: RenderTargetId, ... }`

This ensures:

- depth testing and occlusion are handled by the engine’s render pipeline,
- gizmo rendering quality matches the engine (MSAA/TAA, HDR/tonemapping, postprocessing),
- Fret’s renderer remains 2D/UI-focused and backend-agnostic (ADR 0002 / ADR 0030).

### 2) Input stays effect-driven and uses viewport-mapped coordinates

Viewport tool interactions (including gizmos) consume `ViewportInputEvent` forwarded via effects (ADR 0025).
The event includes:

- `uv` and `target_px` derived from the viewport mapping.

Editor/app tooling uses these together with the engine camera (view/projection) to compute:

- pick rays,
- axis/plane/ring intersections,
- constraint resolution and snapping,
- edit transactions (ADR 0127 / ADR 0024 direction).

### 3) Engine integration uses runner hooks, not direct widget-to-engine calls

Engine rendering and render target updates flow through the runner boundary, so that:

- the engine provides viewport targets (color textures) and submits GPU work for the frame,
- Fret composites the viewport output as an opaque surface in the ordered `Scene`.

This keeps platform separation intact (ADR 0003) and avoids coupling widgets to engine objects.

### 4) UI overlay affordances remain supported as 2D scene ops (optional)

UI-layer affordances that do not require depth testing (labels, HUD text, 2D selection rectangles) may be
rendered as regular scene operations *above* the viewport surface, preserving ordering semantics (ADR 0009).

Guidance:

- Prefer `SceneOp::Path` for strokes/lines when possible.
- Do not add a separate `Polyline2D` scene primitive while `PathStyle::Stroke` is the canonical stroke contract
  (ADR 0080), to avoid duplicate semantics and backend drift.

### 5) No new `SceneOp` primitives are introduced specifically for gizmos in v1

Adding generic mesh/polylines to the display list is a large contract expansion and must be justified by
cross-cutting UI needs (not only gizmos).

If, in the future, the UI ecosystem needs arbitrary 2D meshes (e.g. complex plot markers, vector effects):

- introduce a dedicated prepared-geometry handle + service (analogous to `PathService`),
- define strict ordering/clip/transform semantics (ADR 0009 / ADR 0078),
- and add renderer conformance tests for the new primitive.

This avoids:

- inflating `Scene` with large inline payloads,
- duplicating existing `Path` stroke semantics,
- accidental drift across backends.

## Consequences

Pros:

- Enables Unity/Godot/Unreal-class gizmos (depth tested) without making `fret-render` a 3D renderer.
- Keeps policy in ecosystem/app crates while keeping the runtime substrate stable and portable.
- Preserves strict layering: viewport content remains an opaque surface; overlays remain ordered ops above it.

Cons / Costs:

- Requires an engine-side gizmo render pass (and associated GPU resources) as part of the viewport pipeline.
- Requires an explicit engine integration surface for passing gizmo state and rendering commands.

## Implementation Notes (Non-normative)

- Ecosystem direction: create `ecosystem/fret-gizmo` as a policy-heavy editor/tool crate (ADR 0027).
  - Core API should separate `update(...)` (interaction/math) from engine-facing draw outputs, similar in spirit
    to `transform-gizmo` (optional `repo-ref/transform-gizmo` checkout), but without inheriting framework-specific types.
- Renderer substrate: provide a minimal engine-pass overlay hook in `fret-render` so engines/runners can draw
  depth-tested gizmo geometry inside the viewport render pass without adding new `SceneOp` primitives.
  - `fret_render::viewport_overlay::{ViewportOverlay3dContext, ViewportOverlay3d, run_overlays}`
- Runner/app boundary: expose an app-owned hook service so engine integrations can consistently “draw overlays
  after scene” inside the viewport pass without wiring bespoke callbacks in every demo/runner.
  - `fret_launch::{ViewportOverlay3dHooks, ViewportOverlay3dHooksService}`
- Docking overlay hooks (`DockViewportOverlayHooks`) remain appropriate for 2D overlay affordances (HUD/labels),
  but the depth-tested 3D gizmo geometry should be drawn inside the engine-owned viewport render target.

## UX Baseline (Recommended, Editor-layer)

This section is intentionally **recommended defaults** for ecosystem/editor code. It is not a `fret-ui`
framework commitment (ADR 0027), but it serves as a shared “mature editor” alignment target.

### Visual quality targets

- **Screen-space constant size**: gizmo appears at a stable pixel size across camera distances (auto-scaled).
- **Depth-tested by default**: occlusion is correct; the gizmo is part of the viewport render pipeline.
- **Optional “on top” mode**: allow a mode that draws the gizmo without depth testing (Unity-style toggle).
- **Occluded feedback**: when depth-tested, consider fading/desaturating occluded segments instead of hard cut.
- **AA quality**: rely on engine pipeline AA (MSAA/TAA); avoid duplicating complex AA in UI `SceneOp`.

### Interaction and usability targets

- Modes: `Translate`, `Rotate`, `Scale`, and a combined `Universal` mode.
- Space: `Local` vs `World` orientation toggle.
- Snapping: modifier-gated snapping with configurable step sizes (translation/rotation/scale).
- Pivot policy: multi-selection pivot choices (e.g. median vs individual vs active selection).
- Robustness: hover highlight, active handle lock during drag, drag thresholding, and Escape cancel.
- Edit boundaries: drag interactions should map cleanly to begin/update/commit/cancel phases (ADR 0127 direction).
