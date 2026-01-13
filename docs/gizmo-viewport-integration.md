# Gizmo + Viewport Integration (Tier A)

This guide explains how to integrate `ecosystem/fret-gizmo` into an engine-owned viewport panel,
following the Tier A boundary in `docs/viewport-panels.md`.

`fret-gizmo` is **not** a `fret-ui` widget. It is viewport tool logic that the host updates from
forwarded input and renders as **engine-pass 3D geometry** (depth-tested) into the same render
target as the scene.

See also:

- Tier A viewport contract: `docs/viewport-panels.md`
- Explicit-units input contract: `docs/adr/0147-viewport-input-forwarding-explicit-units.md`
- Gizmo boundary ADR: `docs/adr/0139-viewport-gizmos-engine-pass-and-ui-overlay-boundary.md`
- End-to-end reference: `apps/fret-examples/src/gizmo3d_demo.rs`

## Recommended architecture

**UI layer (declarative):**

- Embed the engine output using `ViewportSurface` (usually via `fret-ui-kit::viewport_surface_panel(...)`).
- Forward pointer + wheel events as `Effect::ViewportInput(ViewportInputEvent)` (ADR 0147).

**Driver / engine layer:**

- Own the camera, selection, and gizmo state (`Gizmo`, `ViewGizmo`, targets).
- Consume `ViewportInputEvent` to build `GizmoInput` / `ViewGizmoInput`.
- Call `Gizmo::update(...)` to produce `GizmoUpdate` phases.
- Render `Gizmo::draw(...) -> GizmoDrawList3d` during the engine pass.

This separation keeps `fret-ui` as a portable UI runtime and avoids coupling viewport tools to
backend crates (`wgpu`, `winit`).

## Coordinate space choice (pick one and be consistent)

`fret-gizmo` takes:

- a `ViewportRect` ("pixels", caller-defined),
- and `GizmoInput.cursor_px` in the same units.

There are two viable host strategies.

### Strategy A (recommended): drive gizmos in render-target pixels

This matches typical engine cameras and avoids drift when viewport fit modes are involved.

Given `ViewportInputEvent event`:

- Set `viewport` to the render-target pixel rect:
  - `ViewportRect::new(Vec2::ZERO, Vec2::new(tw as f32, th as f32))`
  - where `(tw, th) = event.geometry.target_px_size`
- Use `event.cursor_target_px_f32()` for `GizmoInput.cursor_px` (fallback to `event.uv * target_px_size`).
- Keep gizmo sizing/picking stable by scaling config when the mapping changes:
  - `event.target_px_per_screen_px()` is the correct conversion factor for "screen px -> target px"
  - the demo uses `GizmoConfig::scale_for_cursor_units_per_screen_px(ratio)` to keep "pixel-sized"
    thresholds consistent under DPI + fit modes.

This is the most ergonomic option when gizmos are rendered into the engine target and the engine
itself already operates in target pixels.

### Strategy B: drive gizmos in window logical pixels

This is sometimes convenient if you want tooling math to live entirely in UI/input space.

- Set `viewport` to the mapped draw rect in window logical pixels:
  - `draw_rect = event.geometry.draw_rect_px` (ADR 0147)
  - convert the `Rect`/`Point`/`Px` types into `glam::Vec2` by extracting the inner numeric values
    (e.g. `event.cursor_px.x.0` in the demo).
- Use `event.cursor_px` (window logical px) as the source of `GizmoInput.cursor_px` (after converting
  to `glam::Vec2`).

If you choose Strategy B, you still must ensure your `view_projection` math is consistent with the
viewport rectangle you pass in (and you must decide how to handle fit modes and clamping).

## Building `GizmoInput` / `ViewGizmoInput`

`fret-gizmo` is intentionally host-driven: no hard-coded keybindings, no implicit capture policy.

Inputs you must provide:

- `hovered`: whether the gizmo is allowed to do hover picking this frame
- `drag_started`: "pointer went down" edge (usually left-button)
- `dragging`: "pointer is down / capture is active"
- `snap`: host-defined toggle (demo: Ctrl/Cmd)
- `precision`: host-defined multiplier (demo: Shift -> `0.2`, else `1.0`)
- `cancel`: host-defined cancel edge (commonly `Esc`)

Recommended pattern:

1. Derive a `cursor_px` (Strategy A or B).
2. Map `ViewportInputKind` to drag phases:
   - `PointerDown(left)` => `drag_started = true`, `dragging = true`
   - `PointerMove(buttons.left)` => `dragging = true`
   - `PointerUp(left)` => `dragging = false`
3. Gate `hovered` when other interactions should win (camera navigation, marquee selection, etc.).

Reference mapping is in `apps/fret-examples/src/gizmo3d_demo.rs`.

## Update, commit, and undo/redo

`Gizmo::update(...)` returns `Option<GizmoUpdate>` with:

- `phase`: `Begin` / `Update` / `Commit` / `Cancel`
- `updated_targets`: transforms computed from the drag-start snapshot + current totals
- `custom_edits`: plugin-defined property edits (opaque to the core)

Host responsibilities:

- On `Begin`: start an edit transaction / undo record (app-owned; see `ecosystem/fret-undo`).
- On `Update`: apply `updated_targets` to your scene model (often coalesced).
- On `Commit`: finalize the undo record.
- On `Cancel`: restore the drag-start snapshot (or apply the inverse, depending on your model).

## Rendering: engine-pass draw lists

Call:

- `Gizmo::draw(view_projection, viewport, active_target, targets) -> GizmoDrawList3d`

and render the returned draw list in the engine pass (typically after scene geometry), using the
explicit depth mode in the draw data. This ensures correct depth occlusion and avoids treating gizmo
geometry as UI `SceneOp`s.

## Declarative UI integration (GPUI-style mental model)

Even though gizmo logic is updated imperatively, the *hosting* can remain declarative:

- The UI declares a viewport surface for a `RenderTargetId`.
- The driver owns the gizmo/tool state and updates it from effects.
- The render hook consumes the latest tool state to record draw calls.

This mirrors common editor architectures: a declarative UI shell around an imperative engine/tooling
subsystem.

## Ergonomics roadmap (planned, not implemented yet)

ADR 0147 explicitly calls out an opportunity for helpers that bridge `ViewportInputEvent` to
`fret-gizmo` input/state without duplicating DPI + fit-mode glue.

Recommended direction:

- Keep `fret-gizmo` as a mechanism-level crate (host-driven, unit-explicit, backend-agnostic).
- Add optional policy/ergonomics helpers to `ecosystem/fret-ui-kit`, for example:
  - a small state machine that turns `ViewportInputEvent` streams into stable drag phases,
  - default modifier mappings (`snap` / `precision` / `cancel`) that apps can override,
  - composition helpers for "viewport tool arbitration" (camera vs selection vs gizmo).

Until those helpers exist, use `apps/fret-examples/src/gizmo3d_demo.rs` as the canonical reference
for integrating `fret-gizmo` correctly.

