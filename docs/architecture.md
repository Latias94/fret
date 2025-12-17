# Fret Architecture (Draft)

Fret is a Rust GUI framework aimed at building a game editor with a **Unity/Unreal/Godot-like** workflow: docking, tear-off windows, multiple viewports, and layered rendering. The long-term goal is **Windows/macOS/Linux first**, then **wasm (WebGPU)**, and eventually mobile.

This document intentionally focuses on decisions that minimize future rewrites.

## Architecture Decision Records (ADRs)

Key cross-crate contracts are tracked as ADRs:

- `docs/adr/0001-app-effects.md`
- `docs/adr/0002-display-list.md`
- `docs/adr/0003-platform-boundary.md`
- `docs/adr/0004-resource-handles.md`

## Goals

- Retained-mode UI core suitable for large editor applications.
- Docking + tear-off windows (Imgui viewports-style UX).
- Multiple engine viewports in one window (and across windows).
- WGPU-based rendering pipeline, compatible with future WebGPU/wasm.
- Clear separation between platform (window/events), UI core (tree/layout/input), and renderer (GPU).

## Non-goals (for the first iterations)

- Full accessibility stack.
- Perfect text fidelity/IME on day one.
- Mobile support.

## High-Level Layering

1. **Platform layer**: OS windows, event loop, input events, clipboard, IME, drag-and-drop.
2. **UI core**: retained widget tree, layout, hit-testing, focus/keyboard routing, docking model, display list.
3. **Renderer**: translates the display list to GPU work (wgpu), manages atlas/resources, presents to surfaces.

The renderer must be able to run in two hosting modes:

- **Editor-hosted** (recommended default): Fret creates `wgpu::Instance/Adapter/Device/Queue` and the game engine uses the shared GPU context.
- **Engine-hosted**: the engine provides `Device/Queue` (and possibly `Instance/Adapter`), and Fret attaches surfaces + UI rendering on top.

## Crate Layout (Workspace)

- `crates/fret-core`: platform-agnostic core (IDs, geometry, docking model, layout/input contracts).
- `crates/fret-app`: app runtime (global services, models/entities, scheduling, command/action dispatch).
- `crates/fret-platform`: winit-based platform implementation (multi-window, event loop).
- `crates/fret-render`: wgpu-based renderer building blocks (context/device bootstrap, rendering backends).
- `crates/fret-ui`: retained UI runtime (widget tree, layout, hit-testing, focus routing, display list builder).
- `crates/fret`: public facade crate (re-exports).
- `crates/fret-demo`: minimal runnable demo (winit + wgpu surface clear).

## Versioning Notes (Upstream References)

Winit and wgpu evolve quickly, and API changes can make it easy to read the “wrong” source.

- Prefer reading the exact version used by this workspace (via `Cargo.lock`), or check out the matching git tag upstream (e.g. `winit v0.30.12`).
- The platform layer should treat winit as an implementation detail to keep upgrades localized.

## Core Model: Retained Tree + Invalidation

### Why retained-mode

Editors are long-lived, complex, and stateful. Retained mode provides:

- stable widget identity (important for docking, drag state, selection),
- predictable performance via incremental updates and caching,
- easier cross-window coordination.

### Proposed structure

- The UI is a tree of nodes with stable IDs.
- Each node has:
  - layout state (rect, constraints, last measured size),
  - event state (hover, focus, capture),
  - paint cache (optional display list fragments).
- Changes are propagated via **invalidation flags**:
  - `NeedsLayout`
  - `NeedsPaint`
  - `NeedsHitTestRebuild`

## App Runtime: Globals, Models, and Commands

Large editors need a predictable data-flow model to avoid coupling UI widgets directly to each other.

### Global services

Fret provides an `App`-level service container (similar to GPUI’s `Global`) for:

- theme and style tokens,
- dock manager,
- keymap / command registry,
- selection / inspector context,
- asset loaders and caches.

### Models (entities)

Shared application state lives in **models** (a.k.a. entities). Widgets subscribe to model changes and invalidate themselves.

#### App-owned entities and typed handles (Zed/GPUI-inspired)

One key constraint in Rust UI runtimes is avoiding borrow conflicts between `&mut App` and `&mut ModelState`.

We adopt an **App-owned entity** approach:

- All model state is owned by `App` (single owner).
- Callers hold a typed, cloneable handle `Model<T>` that does not directly own the state.
- Mutations happen via explicit update closures, and can emit effects (redraw, commands, window requests).

To keep this ergonomic without pervasive `Rc<RefCell<...>>`, the update path should support **leasing**:

- Temporarily move the model state out of the store while the closure runs.
- This allows passing `&mut App` and `&mut T` to the same closure without aliasing.
- A dropped/poisoned lease must not permanently remove the model; restore on unwind.

Guideline:

- Put long-lived shared state in models (selection, dock graph, settings, project state).
- Put small and short-lived UI interaction state inside the widget tree (hover, local focus helpers, transient form state).

### Commands / Actions

Editors are command-driven (menus, shortcuts, palette, toolbar). Commands must be first-class:

- `CommandId` identifies a logical operation (`dock.toggle_zoom`, `edit.undo`, `scene.play`).
- Input maps to commands via a keymap.
- Commands are routed through focus and can be handled by the focused widget, bubble to parents, or fall back to global handlers.

## Docking: Multi-Window, Tear-Off, and Cross-Window Drag

Docking is managed by a **global Dock Manager** (app-level service), not by a single window.

### Dock data model

- A Dock Graph describes splits and tab stacks.
  - `Split`: axis + children + fractional sizes.
  - `Tabs`: ordered `PanelId`s + an active index.
- **Floating/tear-off** is represented as additional `AppWindowId` roots in the same `DockGraph` (each OS window has a root node). Window geometry/persistence lives outside the core graph.

### Interaction model (inspired by Godot)

- Drag state is global (lives in the app-level `DockManager`).
- Each window renders drop targets/hints during drag.
- On drop:
  - either attach to a drop zone within a window,
  - or create/merge into a floating window.

Platform actions (such as creating a new OS window for tear-off) should be requested by the dock UI and executed by the platform layer. The current direction is an `App`-owned effects queue (e.g. `Effect::Window(WindowRequest::Create(...))`) so platform code can drain and execute requests in one place.

Reference: Godot’s dock windows are explicitly created and managed (`editor/docks/editor_dock_manager.cpp`).

Practical note:

- `DockSpace` reserves a top "chrome" region (future menu/toolbar area). Dropping a dragged tab onto this region triggers a tear-off window.

## Rendering: Scene/Display List + SDF Quads

The renderer consumes a **display list** (or “Scene”) built from the retained tree.

Current state:

- `fret-render` implements a minimal quad renderer (instanced quads + SDF rounded corners) and a scissor-based clip stack.
- `fret-demo` builds a `Scene` via `UiTree` and presents it via wgpu.

### SDF usage

- Use shader-based SDF for:
  - rounded rect panels,
  - borders,
  - splitters,
  - soft clipping for UI chrome.

This matches the proven approach in GPUI’s shaders, where many UI primitives are expressed as quads with SDF evaluation in the fragment stage.

### Atlas

- Single/multi-texture atlas for:
  - glyph bitmaps,
  - icons,
  - small UI images.
- Avoid per-item texture binds; use instancing + atlas tiles.

## Windowing & Surfaces

Each OS window owns a presentable surface (swapchain). Multi-window is required for tear-off docks.

- Winit is treated as an implementation detail behind `fret-platform`.
- For `wgpu`, prefer owning the window handle (e.g. `Arc<Window>`) when creating a `Surface`, so the surface can be stored with a `'static` lifetime and managed safely.
- Web/wasm requires creating surfaces from canvases; this should live in the platform layer.

## Resource Handles & Engine Viewports

The core UI must never depend on `wgpu` types directly.

- `fret-core` defines stable IDs: `ImageId`, `FontId`, `RenderTargetId`, etc.
- `fret-render` owns the actual GPU resources and resolves IDs to `wgpu` resources.
- Engine viewports integrate by registering external textures/render targets into the renderer, returning a stable handle that UI widgets can paint.

Primary integration path:

- shared `Device/Queue` between editor UI and engine renderer (zero-copy presentation into a viewport panel).

Fallback only:

- non-shared device with explicit copies (higher cost, more complexity).

## Layout Engine

- Use `taffy` for flex/grid in general UI.
- Dock splits and pane sizing are specialized and should not depend on flex rules.

## Theme & Styling (shadcn-inspired)

We use design tokens instead of a CSS parser:

- typed token structs (`Color`, `Radius`, `Spacing`, `TypographyScale`),
- a theme registry (optional JSON import/export),
- builder helpers for ergonomics (similar to Tailwind/shadcn style APIs), without coupling the core to a stringly-typed system.

This mirrors the strengths of `gpui-component`’s theme approach while keeping the core stable.

## Text Strategy

- Short-term: simplified text for property panels.
- Long-term: integrate `cosmic-text` for editor-grade text shaping/layout; keep the rendering backend decoupled so we can iterate on subpixel/gamma and caching later.
- Performance stance (Zed-style): cache shaped glyph runs and atlas uploads; treat text and UI primitives as GPU-driven “game-like” rendering.

## Async & Scheduling

Fret should not hard-require a specific async runtime.

- Provide a small `Executor` abstraction in `fret-app`.
- Default implementation can be a lightweight executor and `EventLoopProxy` wakeups.
- Optional feature flags can enable integration with Tokio/async-std if needed.

### Frame scheduling (120fps-inspired)

Avoid unconditional continuous redraw. The platform layer should:

- redraw only when there are **dirty windows** (state changed, animation tick, or input),
- optionally keep presenting for a short “burst” after user input (e.g. ~1s) to avoid variable refresh-rate downshifts,
- handle per-platform sync differences (e.g. “scheduled” vs “completed” semantics on macOS/Metal).

On the renderer side, anticipate **multi-buffering** for dynamic GPU resources (instance buffers, staging uploads) to avoid frame N / N+1 races when presentation becomes more asynchronous.

### Effects flush loop (ownership/handles)

Model updates and widget events should enqueue side effects instead of performing platform operations directly.

`App::flush_effects()` should be the single place that:

- drains window create/close requests,
- drains redraw requests,
- resolves dropped handles (refcounts reaching zero),
- triggers platform wakeups/redraw.

This keeps borrow scopes small and makes multi-window coordination predictable.

## Settings & Configuration (settings-ui-inspired)

For editor-grade products, settings quickly become a cross-cutting concern. Prefer:

- centralized, strongly typed `UserSettings` / `ProjectSettings` models,
- “files as the organizing principle” (both in code and in UI),
- avoiding macro glue that entangles “pre-UI” crates with UI component crates.

## Plugin & Component Boundaries

To support an editor ecosystem, plugins must be able to register:

- panels (dockable views),
- commands and key bindings,
- menus/toolbars,
- inspector editors for engine types.

Recommended shape:

- `Plugin` registers into an `AppRegistry` during startup.
- Panels are created via factories and referenced by stable `PanelId` / `PanelKind`.
- All plugin integration points are owned by `fret-app` and `fret-ui`, never by `fret-render`.

## Proposed Public API (Sketch)

The goal is a stable retained-mode core with optional sugar layers.

### Widgets

```rust
pub trait Widget {
    fn event(&mut self, cx: &mut EventCx, event: &Event);
    fn layout(&mut self, cx: &mut LayoutCx) -> Size;
    fn paint(&mut self, cx: &mut PaintCx);
}
```

Key properties:

- Widgets have stable identity via `NodeId`.
- Containers layout and paint their children via context helpers (not by direct borrowing).

### Contexts

- `EventCx`: focus, capture, request redraw, dispatch commands, open/close windows (via services).
- `LayoutCx`: query constraints, call `layout_child`, measure text, access scale factor and theme metrics.
- `PaintCx`: push primitives into a `Scene` (`fill_rounded_rect`, `draw_text`, `push_clip`, `draw_image`, ...).

### Models

```rust
let selection: Model<Selection> = app.models().insert(Selection::default());
selection.update(&mut app, |sel, _cx| sel.set_active(entity_id));
```

## Early Architecture Decisions (ADR-style)

1. **Rendering backend**: `wgpu` (portable, WebGPU/wasm-friendly).
2. **UI model**: retained tree with invalidation + caching.
3. **Docking**: global Dock Manager + floating windows as first-class nodes.
4. **Layout**: `taffy` for general UI; custom layout for dock splits.
5. **Clipping**: prefer shader-based clipping/SDF over stencil-heavy approaches (simpler cross-backend behavior).
6. **Viewport integration**: shared `Device/Queue` as the primary path; non-shared as fallback only.

## Open Questions (Deferred, but interfaces should anticipate them)

- Vector paths: triangulation (e.g. lyon) vs GPU rasterization; keep `Scene` primitives flexible.
- IME and text input: represent composition events in the input layer; do not bake IME into widgets.
- Accessibility/semantics: keep a parallel semantics tree hook in `fret-ui`.
- SVG rendering: likely `resvg` on CPU into atlas; keep it as an optional module.

## Roadmap (Execution Order)

1. Multi-window platform shell (winit) + redraw scheduling.
2. Core retained tree + hit-testing + focus/capture.
3. Dock graph + drag/drop + tear-off windows.
4. Renderer: instanced quads + SDF rounding/borders + rectangular clipping.
5. Atlas + basic text/icon rendering.
6. Engine viewport embedding (texture panels) + overlay/gizmos.
