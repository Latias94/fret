# ADR 0049: Viewport Tools (Input Capture, Tool Modes, and Overlay Rendering)

Status: Deferred
Scope: Example editor layer (out of scope for the Fret framework)

## Context

An engine editor’s “feel” is largely determined by viewport tooling:

- selection/picking,
- camera navigation,
- gizmos (translate/rotate/scale),
- tool modes (move/rotate/scale/paint),
- overlay rendering (handles, bounds, selection outlines, guides).

Fret already defines the infrastructure contract for viewport embedding and input forwarding:

- `ViewportInputEvent` is data-only and effect-driven (ADR 0025).
- `ViewportInputEvent` uses window-local logical pixels ("screen px") as the UI/input source of truth
  (ADR 0017 / ADR 0132).

However, without a clear example-editor pattern for “who receives viewport input” and “who draws
over the viewport”, tool implementations will proliferate ad-hoc glue code and diverge from a
Unity/Godot-like interaction model.

Godot’s editor plugin API explicitly supports:

- forwarding viewport GUI input to tools/plugins,
- drawing overlays over the viewport (and a “force draw” variant):
  - `repo-ref/godot/editor/plugins/editor_plugin.h`

## Decision

Define an example editor-layer architecture for viewport tools:

1) a centralized **Tool Manager** that owns the active tool mode per editor context,
2) explicit **input capture** and **modal gating** rules,
3) a standard **overlay rendering** hook for tools to draw over viewports,
4) explicit **interaction phases** to support undo coalescing and smooth UX.

### 1) Tool manager is app-owned state

The editor layer owns a `ToolManager` model responsible for:

- active tool mode (e.g. `Select`, `Move`, `Rotate`, `Scale`, `PanOrbit`),
- per-viewport tool state (hovered handle, drag start, snapping toggles),
- routing `ViewportInputEvent` to the correct tool based on focus + capture rules.

This follows the app-owned model guidance (ADR 0031) and keeps policy out of the UI framework
(ADR 0027).

### 2) Input gating and capture

Routing rules:

1. Modal overlays block tool input (ADR 0020 + ADR 0011).
2. Only the focused viewport (or its focused panel) receives tool input (ADR 0025).
3. Tools may request capture on pointer down; while captured, they continue to receive move/up even
   if the pointer leaves viewport bounds.

Capture is an editor-layer concept; it should map to the UI runtime’s pointer capture where possible
(ADR 0005), but tool state ownership remains in the tool manager.

Unit conventions (recommended):

- Treat interaction thresholds (click distance, drag thresholds, hit radii) as **screen px**
  (window-local logical pixels).
- If a tool operates in render-target pixel space (e.g. it uses `ViewportInputEvent.target_px` for
  buffer-based picking), convert thresholds via `ViewportInputEvent::target_px_per_screen_px()` and
  compute cursor target coordinates via
  `ViewportInputEvent::{cursor_target_px_f32,cursor_target_px_f32_clamped}`.

### 3) Overlay rendering over viewports

Tools can contribute overlay rendering in one of two ways:

- **UI overlay**: render retained UI elements positioned in viewport coordinates (labels, buttons).
- **Scene overlay**: emit draw primitives into the UI `Scene` above the viewport surface (lines,
  rects, handles), leveraging Fret’s layer ordering semantics (ADR 0009 / ADR 0019).

The editor layer must define a stable “overlay phase” per viewport:

- `tool.draw_over_viewport(...)` runs after the viewport surface is placed, so overlays naturally
  appear above the engine render target.

### 4) Interaction phases for smooth UX and undo boundaries

Tool input handling should expose phases:

- `Begin` (capture start),
- `Update` (dragging/moving),
- `Commit` (capture end),
- `Cancel` (escape / tool abort / pointer stream cancellation).

This aligns with undo coalescing boundaries described in ADR 0024, without forcing undo policy into
the framework.

### 5) Scheduling: when to redraw

Tools may request redraw/animation frames while active:

- during hover feedback,
- during drags,
- during continuous camera navigation.

Use:

- `Effect::RequestAnimationFrame` for short-lived continuous updates (ADR 0034),
- or window continuous mode (host/editor policy) for “play mode” viewports.

## Consequences

- Viewport tooling can be built with a consistent, Unity/Godot-like mental model.
- Plugins/tools can extend viewport behavior without coupling to the renderer or platform types.
- Overlay rendering becomes a first-class, repeatable pattern rather than bespoke per-tool glue.

## Future Work

- Provide a demo tool set:
  - selection rectangle + simple picking stub,
  - translate gizmo overlay rendering (no engine integration required initially),
  - camera orbit tool with capture.
- Decide a minimal “tool API” surface for plugins (editor-layer), inspired by Godot.
- Integrate property edits + gizmo edits into a shared transaction/coalescing scheme (ADR 0024).

## Implementation Notes (Current Prototype)

- Tool manager + interaction state (example editor layer): `apps/fret-editor/src/viewport_tools.rs`
- Demo tool routing from `Effect::ViewportInput`:
  - driver handler: prototype lived in `apps/fret-demo` (entrypoints evolve; search for `viewport_input`)
  - cancel path: prototype lived in `apps/fret-demo` (Escape clears interaction + overlay)
- Prototype tool interactions:
  - selection marquee: left-drag
  - navigation stub: right-drag orbit, middle-drag pan (drag thresholded so right-click context menu still works)
    - Docking suppresses bubbling for the right-button release once the pointer moves beyond
      `DockingInteractionSettings::viewport_context_menu_drag_threshold`.
- Overlay rendering path (framework-owned hosting, editor-owned state):
  - host: `ecosystem/fret-docking/src/dock/mod.rs` (`DockViewportOverlayHooks`)
  - editor/app-owned overlay painting: `apps/fret-editor/src/viewport_overlays.rs` (e.g. `paint_viewport_marquee`)

## References

- Viewport input forwarding: `docs/adr/0025-viewport-input-forwarding.md`
- Retained UI invalidation + capture: `docs/adr/0005-retained-ui-tree.md`
- Focus + modal gating: `docs/adr/0020-focus-and-command-routing.md`, `docs/adr/0011-overlays-and-multi-root.md`
- Scheduling primitives: `docs/adr/0034-timers-animation-and-redraw-scheduling.md`
- Godot editor viewport plugin hooks:
  - `repo-ref/godot/editor/plugins/editor_plugin.h`
