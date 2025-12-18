# Fret Roadmap (Draft)

This roadmap focuses on building an editor-grade UI framework with docking, tear-off windows, and multiple engine viewports. The goal is to keep early work aligned with long-term constraints (multi-window, wgpu/WebGPU, plugins) to minimize future rewrites.

Key contracts are captured in ADRs under `docs/adr/`.

For a short-horizon execution plan with “definition of done”, see `docs/mvp.md`.

## Toolchain Baseline

- `wgpu 28.x` currently requires Rust `1.92+` (enforced via `rust-toolchain.toml` and `workspace.package.rust-version`).

## Status Legend (Roadmap Notation)

- `(done)`: merged on `main` and considered a stable foundation.
- `(prototype implemented)`: exists in code but not yet stabilized as a long-term contract; expect refactors.
- `(MVP done ...)`: minimal version exists; expected to evolve (quality/perf/edge cases not complete).

## Priorities

- **P0**: Foundational architecture that is hard to change later (windowing, host-provided GPU context, UI execution model, multi-root overlays, event routing, display list ordering semantics, renderer resource ownership, dock manager, persistence contracts).
- **P1**: Editor usability essentials (theme/tokens, docking UX polish, panels, menus, shortcuts, basic text).
- **P2**: Editor completeness (code editor-grade text, IME, accessibility hooks, advanced rendering effects).
- **P3**: Portability extensions (wasm/WebGPU, mobile).

## Refactors to Do Early (Avoid Big Rewrites)

These items are intentionally scheduled early because they define “hard-to-change” semantics:

- P0: Adopt the primary UI authoring/runtime model (declarative elements + externalized state) (ADR 0028).
- P0: Adopt the app-owned model store + borrow-friendly update API (ADR 0031).
- P0: Adopt typed style tokens + theme resolution rules (ADR 0032).
- P0: Adopt semantics tree + accessibility bridge boundary (A11y-ready infrastructure) (ADR 0033).
- P0: Adopt timers/animation/redraw scheduling (event-driven + continuous mode) (ADR 0034).
- P0: Adopt layout constraints + optional Flex/Grid integration boundary (ADR 0035).
- P0: Adopt observability strategy (tracing + inspector hooks + renderer metrics) (ADR 0036).
- P0: Adopt workspace/repo boundaries and external `fret-components` strategy (ADR 0037).
- P0: Adopt component authoring model (Render/RenderOnce + IntoElement) (ADR 0039).
- P0: Renderer must preserve `Scene.ops` ordering across primitive kinds (ADR 0009).
- P0: Multi-root overlays (menus, drag previews, popups, modals) must be first-class (ADR 0011).
- P0: Keyboard/IME split: physical keys for shortcuts, text input for editing (ADR 0012).
- P0: Canonical physical key representation for shortcuts + keymap persistence (ADR 0018).
- P0: Shortcut arbitration + AltGr semantics + pending bindings (avoid keymap/API breaking changes) (ADR 0043).
- P0: Focus + command routing semantics (widget/window/app scopes) are fixed early (ADR 0020).
- P0: Keymap file format + conflict/override semantics are fixed early (ADR 0021).
- P0: `when` expression model is shared by keymap + command gating (ADR 0022).
- P0: Unified command metadata powers menus + palette + shortcuts (ADR 0023).
- P0: Host-provided `WgpuContext` so both editor-hosted and engine-hosted topologies are supported (ADR 0010).
- P0: Canonical frame lifecycle + explicit engine/UI submission ordering (ADR 0015).
- P0: Color management and compositing rules are fixed early (linear compositor + viewport encoding metadata) (ADR 0040).
- P0: Dock persistence and stable panel identity (`PanelKind`) with versioned layout format (ADR 0013).
- P0: Internal drag sessions + clipboard boundary are fixed early (cross-window docking UX) (ADR 0041).
- P0: Scene state stack extension points (transform/opacity/layers) are reserved early (ADR 0019).
- P0: Resource lifetime/eviction/budgets are defined at the handle boundary (ADR 0004).
- P0: Plugin and panel boundaries are app-owned and renderer-free (ADR 0016).
- P0: Multi-window DPI semantics are explicit and portable (ADR 0017).
- P0: Viewport input forwarding contract is fixed early (ADR 0025).
- P0: Text shaping/atlas strategy is decided before shipping text-heavy widgets (ADR 0029).
- P0: Shape semantics (borders/shadows/AA rules) are defined before building docking chrome visuals (ADR 0030).
- P0: Remove layout-engine dependencies from `fret-core` (align with ADR 0035 / ADR 0037). (done)

## Decision Gates (Before Scaling Widget Count)

Before investing in a large widget library, the project should “lock in” the following.

Already locked (Accepted):

- `docs/adr/0028-declarative-elements-and-element-state.md`
- `docs/adr/0029-text-pipeline-and-atlas-strategy.md`
- `docs/adr/0030-shape-rendering-and-sdf-semantics.md`
- `docs/adr/0035-layout-constraints-and-optional-taffy-integration.md`
- `docs/adr/0038-engine-render-hook-and-submission-coordinator.md`
- `docs/adr/0040-color-management-and-compositing-contracts.md`
- `docs/adr/0041-drag-and-drop-clipboard-and-cross-window-drag-sessions.md`

Current policy:

- Treat these `Accepted` ADRs as “hard contracts”.
- If a new hard-to-change design decision appears during implementation, add a new ADR as `Status: Proposed`,
  review it, then promote to `Accepted` before expanding surface area.

## Example Editor App Notes (Out of Scope for Fret Framework)

These are important for building a full engine editor, but they are *application* concerns and
should not be treated as Fret framework deliverables (see ADR 0027):

- `docs/adr/0024-undo-redo-and-edit-transactions.md`
- `docs/adr/0026-asset-database-and-import-pipeline.md`

## Milestones

### M0 — Workspace & Boot

- P0: Cargo workspace and crate boundaries (`fret-core`, `fret-app`, `fret-ui`, `fret-platform`, `fret-render`).
- P0: Enforce crate dependency direction rules and keep contracts backend-agnostic (ADR 0037).
- P0: Minimal `fret-demo` that opens a window and presents via wgpu. (done)
- P0: Runner crate that wires `winit` + `wgpu` + renderer without pulling renderer into `fret-platform`. (done)

### M1 — App Runtime Foundations

- P0: `App` runtime with global services container (type map).
- P0: `Model<T>` store with typed handles and explicit update closures (ADR 0031).
- P0: Lease-based model updates (`App` can pass `&mut App` + `&mut T` safely) (ADR 0031).
- P0: Command registry (`CommandId`, metadata, discovery hooks).
- P0: Effects queue + fixed-point draining (redraw/window requests; effects enqueued from callbacks). (done)
- P0: Timers and animation frame requests via effects (ADR 0034).
- P1: Plugin registry scaffolding (panel factories, command registration).

### M2 — UI Runtime Core (Single Window)

- P0: Decide and implement the primary UI execution model (ADR 0028), while preserving layout/input semantics (ADR 0005). (prototype implemented: declarative element ids + cross-frame state store; retained `UiTree` still hosts most widgets)
- P0: Invalidation flags: `NeedsLayout`, `NeedsPaint`, `NeedsHitTestRebuild`.
- P0: Event routing: hit-test, focus, capture, bubble. (prototype implemented; see ADR 0005)
- P1: External OS file drag-and-drop routing skeleton (winit file DnD + hit-test routing). (prototype implemented; see ADR 0041)
  - Note (macOS/winit): hover/move positions are not continuously available; per-widget drop targets will require a native backend (see ADR 0041).
- P0: Multi-root overlays + z-order + modal blocking model. (see ADR 0011)
- P0: Semantics tree output (UI-only; platform bridge deferred) (ADR 0033).
- P1: Base widgets: `Root`, `Stack`, `Split`, `Clip`, `Scroll`, `Column` (non-taffy). (prototype implemented)
- P1: Scrollbar UX: draggable thumb + track clicking. (prototype implemented)
- P1: Layout contract: `layout_in(child, rect)` stores child bounds for hit-test/paint. (prototype implemented)
- P1: Optional `Flex`/`Grid` widget backed by `taffy` (defer until needed; no `UiTree` refactor).
- P0: Virtualization contract for editor-scale lists/tables/editors (no unbounded children in layout engines). (ADR 0042)
- P1: Theme/tokens (typed tokens; theme content app-owned) (ADR 0032).
- P1: Spin up `fret-components` repo workspace and establish the “editor kit” structure (ADR 0037):
  - `fret-components-ui` (shadcn-inspired primitives/composites),
  - `fret-components-tree`, `fret-components-inspector`, `fret-components-table`,
  - (optional early) `fret-components-charts`.

### M3 — Display List Contract + Renderer MVP

- P0: Backend-agnostic `Scene/DisplayList` contract in `fret-core`.
- P0: `Scene.ops` ordering is authoritative; renderer batching preserves order. (see ADR 0009)
- P0: Renderer consumes display list and draws:
  - instanced quads,
  - rounded rect via SDF, borders (pending),
  - clip rect stack (initially via scissor; later refine). (MVP done for quads + scissor)
- P0: Dynamic GPU buffer strategy (anticipate multi-buffering/pools for async presentation).
- P1: Atlas allocation and uploads (images, glyphs).
- P0: Render target registry for engine viewports (contract skeleton). (done)

### M4 — Docking (Multi-Window + Tear-off)

- P0: `DockManager` (App-level) owns dock graph for all windows.
- P0: Dock UX in a `DockSpace` widget (split + tabs + drag drop zones, tab insert/reorder, split handle dragging). (MVP done in demo; multi-window tear-off supported)
- P0: Floating windows are first-class: tear-off and merge back. (MVP done in demo via `DockOp` + `WindowRequest`)
- P0: Cross-window drag state and drop target rendering. (done; app-scoped internal `DragSession` + drop overlay)
- P0: Dock persistence (layout.json v1) + stable panel identity. (MVP done in demo; persists/restores)

Remaining work (still P0, but can iterate after MVP2):

- Persist/restore window placement (monitor + DPI-aware geometry) separate from the logical dock layout. (prototype implemented; stored as `DockLayoutWindowV1.placement`)
- Improve hit-testing + drop-zone heuristics and polish the UX (snap thresholds, preview animations).

### M5 — Engine Viewports

- P0: Host-provided `WgpuContext` to support both editor-hosted and engine-hosted integration. (see ADR 0010)
- P0: Viewport widget that displays an engine texture/render target.
- P0: Viewport mapping + input event contract (window -> uv/px). (prototype implemented)
- P1: Overlay composition primitives: UI can render overlays over the viewport; editor apps can build gizmos/selection on top.
- P1: Input forwarding and capture rules (mouse/keyboard routed to viewport when focused).

Notes:

- A minimal overlay prototype exists (viewport hover crosshair in dock panels).

### M6 — Text System Upgrade

- P1: Basic text for inspector/property panels (layout + glyph atlas). (MVP done in demo; see `docs/mvp-archive.md`)
- P0: Text system boundary (`TextBlobId` + metrics contract). (done; see ADR 0006)
- P2: `cosmic-text` integration for editor-grade text shaping/layout (ADR 0029). (MVP done for single-line)
- P2: Shaped-run caching + incremental atlas uploads for large documents.
- P1: IME plumbing (winit `Ime` events + `set_ime_cursor_area` feedback path). (prototype implemented; see ADR 0012)
- P2: IME composition UI (inline preedit rendering in text widgets). (MVP done for single-line)
- P2: High-performance text widgets (code editor view, large buffers).

Immediate next step:

- Move to MVP8 (focus traversal + focus scopes) now that MVP7 lands a command palette surface (see `docs/mvp.md`).

### M7 — Portability (wasm/WebGPU)

- P2: Platform layer for web canvas surfaces.
- P2: Input/clipboard limitations documented and handled.
- P3: Mobile planning (out of scope for early phases).

### M8 — Settings, Keymap, and Persistence

- P0: File-based configuration model + strong types. (see ADR 0014)
- P0: Dock layout persistence format with versioning. (see ADR 0013)
- P1: Settings UI primitives (token-driven) for inspector + app settings.
- P0: Keymap MVP (bind + route + persist) is implemented in `fret-demo` (see `docs/mvp-archive.md` / ADR 0021 / ADR 0022).

### M9 — Command UI + Focus + Clipboard (Editor Usability Core)

These are the “you can actually drive the editor” foundations. They are intentionally scheduled early so that
all later UI work inherits the same command/focus/clipboard semantics instead of bespoke widget logic.

- P0: Command palette overlay + minimal menu data model (ADR 0023). (MVP done in demo; see `docs/mvp.md` MVP 7)
- P0: Focus traversal and focus scopes (Tab navigation, modal focus trap) (ADR 0020). (see `docs/mvp.md` MVP 8)
- P0: Clipboard boundary + text editing commands (text-only first) (ADR 0041). (see `docs/mvp.md` MVP 9)

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

- UI runtime (retained widget tree prototype today; declarative elements planned via ADR 0028).
- Invalidation, layout widgets, hit-testing.
- Focus, capture, command routing.
- Dock UI (`DockSpace`) as a widget consuming `DockManager`.

### `fret-platform` (P0)

- Platform backend integration (winit today; web later) focused on IO:
  - window creation primitives,
  - event translation,
  - platform services (clipboard, IME, drag-and-drop).

### `fret-runner-winit-wgpu` (P0)

- Desktop runner that wires `winit` + `wgpu` + `fret-render` + `fret-ui` together.
- Owns the winit event loop and window lifecycle.
- Drains `App::flush_effects()` and drives redraw/raf/timers deterministically across windows.
- Owns surfaces + presentation and coordinates submission order (see ADR 0015 / ADR 0038).

### `fret-render` (P0)

- wgpu device/surface setup.
- Resource registries for atlas/textures/render targets.
- Pipelines for quads/SDF/text/images.

## Definition of Done (per milestone)

- Builds with `cargo check` on macOS/Linux/Windows.
- A runnable demo for user-visible milestones (M0/M2/M3/M4/M5).
- Public API changes documented in `docs/architecture.md` and/or ADR notes.
