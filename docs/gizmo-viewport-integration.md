# Gizmo + Viewport Integration (Tier A)

This guide explains how to integrate `ecosystem/fret-gizmo` into an engine-owned viewport panel,
following the Tier A boundary in `docs/viewport-panels.md`.

`fret-gizmo` is **not** a `fret-ui` widget. It is viewport tool logic that the host updates from
forwarded input and renders as **engine-pass 3D geometry** (depth-tested) into the same render
target as the scene.

See also:

- Tier A viewport contract: `docs/viewport-panels.md`
- Explicit-units input contract: `docs/adr/0132-viewport-input-forwarding-explicit-units.md`
- Gizmo boundary ADR: `docs/adr/0130-viewport-gizmos-engine-pass-and-ui-overlay-boundary.md`
- End-to-end reference: `apps/fret-examples/src/gizmo3d_demo.rs`

## Recommended architecture

**UI layer (declarative):**

- Embed the engine output using `ViewportSurface` (usually via `fret-ui-kit::viewport_surface_panel(...)`).
- Forward pointer + wheel events as `Effect::ViewportInput(ViewportInputEvent)` (ADR 0132).

**Driver / engine layer:**

- Own the camera, selection, and gizmo state (`Gizmo`, `ViewGizmo`, targets).
- If you host multiple gizmos/plugins (recommended), own a `GizmoPluginManager` and register plugins
  (e.g. `TransformGizmoPlugin`, `LightRadiusGizmoPlugin`).
- Consume `ViewportInputEvent` to build `GizmoInput` / `ViewGizmoInput`.
- Call `GizmoPluginManager::update(...)` (or `Gizmo::update(...)` for a single built-in gizmo) to
  produce `GizmoUpdate` phases.
- Render `GizmoPluginManager::draw(...) -> GizmoDrawList3d` (or `Gizmo::draw(...)`) during the
  engine pass.

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

#### Helper: `ViewportToolInput` (recommended)

To reduce boilerplate and keep unit conversions consistent, use:

- `fret_gizmo::ViewportToolInput::from_viewport_input_target_px(&event, MouseButton::Left)`

Note:

- `ViewportToolInput` is shared infrastructure (ADR 0153). It lives in `ecosystem/fret-viewport-tooling`
  and is re-exported by `fret-gizmo` for convenience.

It derives:

- `viewport: ViewportRect` in render-target pixels,
- `cursor_px` in render-target pixels (float),
- `drag_started` / `dragging` for the chosen button,
- `cursor_units_per_screen_px` (a conservative `target_px_per_screen_px`), useful for scaling
  pixel-sized thresholds via `GizmoConfig::scale_for_cursor_units_per_screen_px(...)`.

The end-to-end example uses this helper: `apps/fret-examples/src/gizmo3d_demo.rs`.

#### Optional: `f64-math` for large-world picking stability

`fret-gizmo` is f32-first, but you can enable `fret-gizmo/f64-math` to use f64 internally for
projection/unprojection and ray construction (picking-critical math). This helps when your world
coordinates are very large (or very far from the origin) and f32 loses near/far separation.

Note: most gizmo update math still runs in f32; long-term "large world" support likely needs a
broader scene-units/rebasing policy in the host.

### Strategy B: drive gizmos in window logical pixels

This is sometimes convenient if you want tooling math to live entirely in UI/input space.

- Set `viewport` to the mapped draw rect in window logical pixels:
  - `draw_rect = event.geometry.draw_rect_px` (ADR 0132)
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

## Tool arbitration (recommended): route gizmos as viewport tools

Most editor viewports need multiple concurrent tools:

- camera navigation (orbit/pan/zoom),
- selection (click + marquee),
- view gizmo (camera cube),
- transform gizmo (translate/rotate/scale),
- plus domain-specific gizmos.

To keep these boundaries explicit and reusable across ecosystem crates, treat each interaction as a
**viewport tool** and route them through the shared host helper (ADR 0153).

Recommended host pattern:

- Keep camera navigation as an explicit host policy (e.g. RMB/MMB drag wins, mouse wheel zoom).
- Route everything else (view gizmo / transform gizmo / selection tools) through
  `fret_ui_kit::viewport_tooling::route_viewport_tools(...)`.
- Use `ViewportToolRouterState` (`hot` / `active` / captured button) as the stable “tool session”
  state you store in your model.
- If you need a pure “am I over a handle?” check for routing, prefer the side-effect-free pick helper:
  - `GizmoPluginManager::pick_hovered_handle(...)` (no state updates; see ADR 0153 hit-test rule)
- When handling `Esc` / cancel commands, cancel the active tool session via the routing helpers:
  - callback router: `cancel_active_viewport_tools(...)`
  - trait-object router: `ViewportToolArbitrator::cancel_active_and_clear_hot()`

End-to-end reference:

- `apps/fret-examples/src/gizmo3d_demo.rs` (multiple tools routed through `route_viewport_tools`)

## Update, commit, and undo/redo

`GizmoPluginManager::update(...)` returns `Option<GizmoUpdate>` with:

- `phase`: `Begin` / `Update` / `Commit` / `Cancel`
- `updated_targets`: transforms computed from the drag-start snapshot + current totals
- `custom_edits`: plugin-defined property edits (opaque to the core)

Host responsibilities:

- On `Begin`: start an edit transaction / undo record (app-owned; see `ecosystem/fret-undo`).
- On `Update`: apply `updated_targets` to your scene model (often coalesced).
- On `Commit`: finalize the undo record.
- On `Cancel`: restore the drag-start snapshot (or apply the inverse, depending on your model).

## Host properties for custom plugins (read-only)

Custom gizmo plugins often need to read host domain values (to draw correct readouts, and to capture
drag-start snapshots) without maintaining a host-driven push cache.

`fret-gizmo` supports this via a host-implemented, read-only contract:

- `fret_gizmo::GizmoPropertySource` (`read_scalar(target, key) -> Option<f32>`)

Thread it into the plugin manager:

- pass `Some(&your_source)` to `GizmoPluginManager::{update,draw}`

Reference:

- `apps/fret-examples/src/gizmo3d_demo.rs` (`DemoGizmoPropertySource`)

## Rendering: engine-pass draw lists

Call:

- `GizmoPluginManager::draw(view_projection, viewport, depth_range, active_target, targets, input, properties) -> GizmoDrawList3d`

and render the returned draw list in the engine pass (typically after scene geometry), using the
explicit depth mode in the draw data. This ensures correct depth occlusion and avoids treating gizmo
geometry as UI `SceneOp`s.

### Engine-pass overlay hook wiring (recommended)

The runner provides an engine-pass hook point (`ViewportOverlay3dHooksService`) that the host can
use to record gizmos/debug overlays into an existing viewport render pass.

To reduce boilerplate for Tier A integrations, prefer the shared immediate overlay helpers in
`fret-launch`:

- install once (e.g. in `WinitAppDriver::init`): `install_viewport_overlay_3d_immediate(app)`
- upload per frame: `upload_viewport_overlay_3d_immediate(...) -> Overlay3dPipelines`
- record inside the pass: `record_viewport_overlay_3d(app, window, target_id, &mut pass, &ctx)`

## Declarative UI integration (GPUI-style mental model)

Even though gizmo logic is updated imperatively, the *hosting* can remain declarative:

- The UI declares a viewport surface for a `RenderTargetId`.
- The driver owns the gizmo/tool state and updates it from effects.
- The render hook consumes the latest tool state to record draw calls.

This mirrors common editor architectures: a declarative UI shell around an imperative engine/tooling
subsystem.

## Ergonomics helpers (available today)

The core boundary remains host-driven (ADR 0130 / ADR 0132), but common glue is now shared:

- Portable input mapping + tool protocol: `ecosystem/fret-viewport-tooling`
  - `ViewportToolInput`, `ViewportRect`, `ViewportTool{Id,Priority,Result}`, `ViewportToolCx`
- Default host routing/arbitration helpers: `ecosystem/fret-ui-kit/src/viewport_tooling.rs`
  - `ViewportToolArbitrator` (trait-object tools)
  - `ViewportToolRouterState` + `route_viewport_tools` (callback router, easy for demos/apps)
- Engine-pass overlay wiring helpers: `crates/fret-launch`
  - `install_viewport_overlay_3d_immediate`, `upload_viewport_overlay_3d_immediate`

`fret-gizmo` re-exports the tool protocol types so apps that only use gizmos can depend on a single
crate in v1.

### Remaining gaps (v1)

- Multi-pointer / touch tooling sessions (router is single-pointer-first in v1).
