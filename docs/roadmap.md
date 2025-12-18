# Fret Roadmap (Draft)

This roadmap focuses on building a retained-mode editor-grade UI framework with docking, tear-off windows, and multiple engine viewports. The goal is to keep early work aligned with long-term constraints (multi-window, wgpu/WebGPU, plugins) to minimize future rewrites.

Key contracts are captured in ADRs under `docs/adr/`.

## Priorities

- **P0**: Foundational architecture that is hard to change later (windowing, retained tree, event routing, display list contract, renderer resource ownership, dock manager).
- **P1**: Editor usability essentials (theme/tokens, docking UX, panels, menus, shortcuts, basic text).
- **P2**: Editor completeness (code editor-grade text, IME, accessibility hooks, advanced rendering effects).
- **P3**: Portability extensions (wasm/WebGPU, mobile).

## Milestones

### M0 — Workspace & Boot

- P0: Cargo workspace and crate boundaries (`fret-core`, `fret-app`, `fret-ui`, `fret-platform`, `fret-render`).
- P0: Minimal `fret-demo` that opens a window and presents via wgpu. (done)

### M1 — App Runtime Foundations

- P0: `App` runtime with global services container (type map).
- P0: `Model<T>` store with typed handles and explicit update closures.
- P0: Lease-based model updates (`App` can pass `&mut App` + `&mut T` safely).
- P0: Command registry (`CommandId`, metadata, discovery hooks).
- P0: Effects queue + fixed-point draining (redraw/window requests; effects enqueued from callbacks). (done)
- P1: Plugin registry scaffolding (panel factories, command registration).

### M2 — Retained UI Core (Single Window)

- P0: `Widget` trait and node tree with stable `NodeId`.
- P0: Invalidation flags: `NeedsLayout`, `NeedsPaint`, `NeedsHitTestRebuild`.
- P0: Event routing: hit-test, focus, capture, bubble. (prototype implemented; see ADR 0005)
- P1: Base widgets: `Root`, `Stack`, `Split`, `Clip`, `Scroll`, `Column` (non-taffy). (prototype implemented)
- P1: Scrollbar UX: draggable thumb + track clicking. (prototype implemented)
- P1: Layout contract: `layout_in(child, rect)` stores child bounds for hit-test/paint. (prototype implemented)
- P1: Optional `Flex`/`Grid` widget backed by `taffy` (defer until needed; no `UiTree` refactor).
- P1: Theme/tokens (shadcn-inspired typed tokens).

### M3 — Display List Contract + Renderer MVP

- P0: Backend-agnostic `Scene/DisplayList` contract in `fret-core`.
- P0: Renderer consumes display list and draws:
  - instanced quads,
  - rounded rect via SDF, borders (pending),
  - clip rect stack (initially via scissor; later refine). (MVP done for quads + scissor)
- P0: Dynamic GPU buffer strategy (anticipate multi-buffering/pools for async presentation).
- P1: Atlas allocation and uploads (images, glyphs).
- P0: Render target registry for engine viewports (contract skeleton). (done)

### M4 — Docking (Multi-Window + Tear-off)

- P0: `DockManager` (App-level) owns dock graph for all windows.
- P0: Dock UX in a `DockSpace` widget (split + tabs + drag drop zones, tab insert/reorder, split handle dragging). (prototype done for single-window)
- P0: Floating windows are first-class: tear-off and merge back. (prototype done via window-creation requests in demo)
- P0: Cross-window drag state and drop target rendering. (prototype done; natural tear-off via top chrome; empty floating windows auto-close)
- P1: Dock persistence (optional), layout versioning hook.

### M5 — Engine Viewports

- P0: Shared `wgpu::Device/Queue` integration path.
- P0: Viewport widget that displays an engine texture/render target.
- P0: Viewport mapping + input event contract (window -> uv/px). (prototype implemented)
- P1: Overlay layer (gizmo, selection rect, grid) rendered by UI over the viewport.
- P1: Input forwarding and capture rules (mouse/keyboard routed to viewport when focused).

Notes:

- A minimal overlay prototype exists (viewport hover crosshair in dock panels).

### M6 — Text System Upgrade

- P1: Basic text for inspector/property panels (layout + glyph atlas).
- P0: Text system boundary (`TextBlobId` + metrics contract). (done; see ADR 0006)
- P2: `cosmic-text` integration for editor-grade text shaping/layout.
- P2: Shaped-run caching + incremental atlas uploads for large documents.
- P2: IME events end-to-end, composition UI.
- P2: High-performance text widgets (code editor view, large buffers).

### M7 — Portability (wasm/WebGPU)

- P2: Platform layer for web canvas surfaces.
- P2: Input/clipboard limitations documented and handled.
- P3: Mobile planning (out of scope for early phases).

## Module Breakdown (Crates)

### `fret-core` (P0)

- IDs and geometry (`Px`, `Rect`, `NodeId`, resource IDs).
- Dock graph data structures.
- Display list / scene primitives contract.
- Cross-platform input event types (including IME events as data-only).

### `fret-app` (P0)

- `App` runtime: globals, models, command registry, plugin registry.
- Scheduling/executor abstraction (no hard dependency on Tokio).

### `fret-ui` (P0)

- Retained widget tree, invalidation, layout widgets, hit-testing.
- Focus, capture, command routing.
- Dock UI (`DockArea`) as a widget consuming `DockManager`.

### `fret-platform` (P0)

- Winit integration (multi-window, event loop).
- Translates platform events -> core events.
- Owns platform services: clipboard, drag-and-drop, IME plumbing.
- `WinitRunner`: drains `App::flush_effects()` and owns surfaces + redraw scheduling. (prototype implemented)

### `fret-render` (P0)

- wgpu device/surface setup.
- Resource registries for atlas/textures/render targets.
- Pipelines for quads/SDF/text/images.

## Definition of Done (per milestone)

- Builds with `cargo check` on macOS/Linux/Windows.
- A runnable demo for user-visible milestones (M0/M2/M3/M4/M5).
- Public API changes documented in `docs/architecture.md` and/or ADR notes.
