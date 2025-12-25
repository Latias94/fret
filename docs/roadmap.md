# Fret Roadmap (Draft)

This roadmap focuses on building an editor-grade UI framework with docking, tear-off windows, and multiple engine viewports. The goal is to keep early work aligned with long-term constraints (multi-window, wgpu/WebGPU, plugins) to minimize future rewrites.

Key contracts are captured in ADRs under `docs/adr/`.

For the short-horizon execution plan, see `docs/mvp.md` (overview) and `docs/mvp/active-plan.md` (expanded, per-MVP “definition of done”).

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
- P0: Align baseline theme semantics with gpui-component/shadcn vocabulary (semantic palette + typography), so the component ecosystem can reuse upstream conventions without inflating `fret-ui` into a UI kit. (ADR 0050 follow-up)
- P0: Adopt a component-level size/density system so Tailwind-like primitives converge on one contract (ADR 0056). (prototype implemented; MVP 47)
- P0: Adopt component-level style composition ergonomics (`StyleRefinement` + `StyledExt`) so Tailwind/shadcn-like recipes and variants can be expressed as typed, composable “style patches” without leaking UI kit semantics into `fret-ui` runtime. (MVP 45)
- P0: Tighten the **framework vs components** boundary so Tailwind/shadcn sizing/variants can converge without fighting runtime widgets:
  - keep `fret-ui` as the runtime substrate (tree, routing, layers, docking, performance primitives),
  - move the “standard UI kit surfaces” (popover/dialog/menu/tooltip/toast/command palette/menubar) to `fret-components-ui`,
  - remove hard-coded control heights/spacing from runtime primitives (notably `TextInput`) so `Size` (ADR 0056) stays component-owned. (MVP 48; in progress: `TextInput` done; overlay surfaces moved)
- P0: Adopt semantics tree + accessibility bridge boundary (A11y-ready infrastructure) (ADR 0033).
- P0: Adopt timers/animation/redraw scheduling (event-driven + continuous mode) (ADR 0034).
- P0: Adopt layout constraints + optional Flex/Grid integration boundary (ADR 0035).
- P0: Fix declarative composable sizing semantics (fit-content by default, typed size constraints, and a Taffy-backed `Flex` container) so Tailwind/shadcn component composition is viable without per-widget layout hacks. (ADR 0057)
- P0: Adopt observability strategy (tracing + inspector hooks + renderer metrics) (ADR 0036).
- P0: Adopt workspace/repo boundaries and external `fret-components` strategy (ADR 0037).
- P0: Adopt component authoring model (Render/RenderOnce + IntoElement) (ADR 0039).
- P0: Make the declarative authoring model an end-to-end, usable implementation path (not only a cross-frame state store), so virtualization and complex components can be expressed as composition rather than runtime widgets. (MVP 49/50)
- P0: Introduce a `UiHost` boundary so `fret-ui` can be embedded by third-party hosts (ADR 0052). (done; `fret-runtime` + `fret-ui-app`)
- P0: Make external file drag payload portable (no `PathBuf` in core events) (ADR 0053). (done)
- P0: Introduce runtime platform capability matrix for portability (ADR 0054). (done)
- P0: Introduce a portable system cursor boundary (cursor icon + per-window cursor routing + overlay precedence).
  - Rationale: editor-grade resizing/docking needs OS cursor affordances; without a stable boundary, each widget
    invents its own heuristics and portability will suffer.
  - Implemented as MVP 46 (prototype; see `docs/mvp/active-plan.md`).
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
- Prefer **updating** an existing ADR section over creating many micro-ADRs (keep the index usable).

When to write (or significantly update) an ADR:

- Cross-crate boundaries or dependency direction (`fret-core`/`fret-ui`/runner/platform/render).
- Long-lived file formats (layout/theme/keymap/project/scene) or their versioning/migration rules.
- Input/focus/capture/command routing semantics or window/app scope rules.
- Display list / renderer semantics (`SceneOp` meaning, ordering, blending, clipping, AA, pixels).
- Portability contracts (wasm/WebGPU capability gaps, payload shapes, “no PathBuf” style constraints).
- Caching/invalidation semantics that affect correctness, determinism, or replay/recording behavior.

MVP guidance:

- Each MVP item in `docs/mvp/active-plan.md` should link to the ADRs that define its non-negotiable contracts.
- If a prototype reveals an incorrect assumption, update the ADR first, then adjust the implementation.

## Framework Capability Promotions (From Demo/Widgets to Core)

These are recurring “editor-grade UX” needs that should be expressed as **portable framework contracts**
(`fret-core`/`fret-runtime` + runner effects), not re-implemented ad-hoc in individual widgets or demos.

The goal is a GPUI-like reuse story: third-party hosts and component ecosystems can rely on the same behavior.

P0 / near-term (schedule as MVPs/refactors):

- **System cursor + pointer feedback**: portable cursor icons, per-window cursor routing, overlay precedence, and
  reusable resize-handle primitives (planned as MVP 46 in `docs/mvp/active-plan.md`).
- **Resizable layout primitive**: a reusable split/resize container (panel group) so “dock splits”, “inspector
  sidebars”, and “data table column resize” share hit-testing and cursor behavior (pairs naturally with MVP 46).
  - Prefer modeling the divider/handle as a real child element that participates in layout (no overlap with content),
    rather than a pure overlay that can clip focus rings; see gpui-component’s `resize_handle` usage in
    `repo-ref/gpui-component/crates/ui/src/resizable/panel.rs`.
- **Pointer lock / relative motion** (viewport navigation): a portable effect boundary for “orbit/pan camera”
  interactions where OS cursor constraints differ (desktop vs web); capability-gated and opt-in.
- **Overlay hit-testing policies**: pointer-transparent overlays (click-through “empty” overlay regions) and
  deterministic layering for transient surfaces (tooltips/toasts/drag hints). (prototype implemented in runtime; used by toast)

P1 / medium-term (app polish; after primitives settle):

- **Tooltip / hover help service**: delayed tooltips, follow-cursor placement, and consistent overlay ordering
  (build on existing overlay roots and `PopoverService`).
- **Window policy hooks**: standardized “raise/focus on create”, window activation transfer, and predictable
  z-order behavior across platforms (reduce demo-specific fixes).

P2 / later (portability/product completeness):

- **Native dialogs**: open/save file dialogs, open folder, reveal-in-file-manager (effect-driven and capability-gated).
- **External integrations**: open URL, system notifications (portable subset).
- **Rich clipboard**: images/files (beyond text), with explicit portability constraints.

## Example Editor App Notes (Out of Scope for Fret Framework)

These are important for building a full engine editor, but they are *application* concerns and
should not be treated as Fret framework deliverables (see ADR 0027):

- `docs/adr/0024-undo-redo-and-edit-transactions.md`
- `docs/adr/0026-asset-database-and-import-pipeline.md`
- `docs/adr/0048-inspector-property-protocol-and-editor-registry.md`
- `docs/adr/0049-viewport-tools-input-capture-and-overlays.md`

## Example Editor Workflows (Demo-Driven, P1)

These are not framework commitments (ADR 0027), but they are essential to reaching Unity/Godot-like “feel” early and to
validate that the framework contracts are sufficient.

- Inspector P0: engine-agnostic property protocol + custom editor registry + minimal built-in editors (ADR 0048). (prototype implemented in demo; see `docs/mvp/active-plan.md` MVP 17; two-column inline rows + Alt+drag scrubbing)
- Viewport tools P0: input capture + tool routing + overlay rendering over viewport surfaces (ADR 0049, built on ADR 0025). (prototype implemented in demo; see `docs/mvp/active-plan.md` MVP 18)
- Viewport picking P0: click-to-select + selection highlight overlays (demo-driven; see `docs/mvp/active-plan.md` MVP 19).
- Gizmo P0: translate tool with explicit drag phases, capture rules, and basic polish (axis constraints + snapping stub) (prototype implemented in demo; see `docs/mvp/active-plan.md` MVP 20 + MVP 25).
- Viewport navigation P0: pan/orbit stub + wheel zoom (prototype implemented in demo; see `docs/mvp/active-plan.md` MVP 26).
- Gizmo P0: rotate tool stub (prototype implemented in demo; see `docs/mvp/active-plan.md` MVP 27).
- Multi-viewport roles P0: Scene (editor) vs Game (preview) gating patterns (prototype implemented in demo; see `docs/mvp/active-plan.md` MVP 30).
- Play mode stub P0: preview loop with RAF scheduling while a Game viewport is visible (prototype implemented in demo; see `docs/mvp/active-plan.md` MVP 31).
- Undo/redo P0: command stack + transaction coalescing boundary (ADR 0024). (prototype implemented in demo; see `docs/mvp/active-plan.md` MVP 22 + MVP 24).
- Scene documents P0: `.scene` open/new/save/save-as + unsaved changes guard (demo-driven; see `docs/mvp/active-plan.md` MVP 39–42).

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
- P0: Model observation → UI invalidation propagation (avoid manual cross-panel invalidation; align with GPUI `observe`/`notify` mental model). (ADR 0051) (prototype implemented)
- P0: Event routing: hit-test, focus, capture, bubble. (prototype implemented; see ADR 0005)
- P1: External OS file drag-and-drop routing skeleton (winit file DnD + hit-test routing). (prototype implemented; see ADR 0041)
  - Note (macOS/winit): hover/move positions are not continuously available; per-widget drop targets will require a native backend (see ADR 0041).
- P0: Multi-root overlays + z-order + modal blocking model. (see ADR 0011)
- P0: Semantics tree output (UI-only; platform bridge deferred) (ADR 0033).
- P1: Base widgets: `Root`, `Stack`, `Split`, `Clip`, `Scroll`, `Column` (non-taffy). (prototype implemented)
- P1: Scrollbar UX: draggable thumb + track clicking. (prototype implemented)
- P1: Editor-scale list widgets: `VirtualList` (runtime primitive), `TreeView` (legacy, gated; prototype implemented)
- P1: Layout contract: `layout_in(child, rect)` stores child bounds for hit-test/paint. (prototype implemented)
- P1: Optional `Flex`/`Grid` widget backed by `taffy` (defer until needed; no `UiTree` refactor).
- P0: Virtualization contract for editor-scale lists/tables/editors (no unbounded children in layout engines). (ADR 0042) (prototype implemented: `fret-ui` `VirtualList`)
- P1: Theme/tokens (typed core tokens + extensible namespaced keys for component ecosystems) (ADR 0032 / ADR 0050). (prototype implemented)
- P0: Component size/density system (Tailwind-like scales) so control heights/paddings converge before scaling the UI kit (ADR 0056). (prototype implemented; see `docs/mvp/active-plan.md` MVP 47)
- P1: Anchored overlays (popover-style) for component primitives.
  - Note: overlay surfaces now live in `fret-components-ui` (MVP 48), while `fret-ui` retains the overlay-layer mechanism.
- P1: Spin up `fret-components` repo workspace and establish a **general-purpose** component library baseline (ADR 0037):
  - `fret-components-ui` (token-driven primitives/composites; shadcn-inspired; not editor-specific) (prototype implemented in-tree),
  - `fret-components-icons` (icon registry + sets; atlas-friendly; renderer-agnostic) (prototype implemented in-tree),
  - optional later: `fret-components-editor` (editor-only patterns: inspector/table/tree wrappers).

Notes:

- Prefer “Tailwind-like primitives” as an internal authoring model: small typed tokens + recipe/variant composition (not CSS strings).
- Keep the primitive vocabulary explicit and typed (e.g. `Space`, `Radius`) backed by theme extension tokens
  (`component.space.*`, `component.radius.*`), so component APIs stay stable and theme authors can tune scales.
- `repo-ref/fret-ui-precision` is the design reference for token taxonomy and component recipes; `repo-ref/gpui-component` is the Rust ecosystem reference for themes + component ergonomics.
- `repo-ref/ui` (shadcn/ui) and `repo-ref/tailwindcss` provide upstream vocabulary for component variants and token scales.

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
- P0: Dock panel content wiring (panel kind/key → UI root per window). (MVP done in demo; `DockPanelContentService`)
- P0: Floating windows are first-class: tear-off and merge back. (MVP done in demo via `DockOp` + `WindowRequest`)
- P0: Cross-window drag state and drop target rendering. (done; app-scoped internal `DragSession` + drop overlay)
- P0: Dock persistence (layout.json v1) + stable panel identity. (MVP done in demo; persists/restores)
- P0: Layout reset + minimal layout preset save/load (“last”). (done in demo; menu-backed actions)

Remaining work (still P0, but can iterate after MVP2):

- Persist/restore window placement (monitor + DPI-aware geometry) separate from the logical dock layout. (prototype implemented; stored as `DockLayoutWindowV1.placement`)
- Improve hit-testing + drop-zone heuristics and polish the UX (snap thresholds, preview animations).
  - “Product UI” affordances to match Unity/Godot:
    - dock drag hint overlay and tab drop indicators (Godot-style),
    - dock context menu actions (float, close, move left/right, etc.),
    - debounced layout persistence (delay disk writes during interactive drags).
    - system cursor affordances for split/resize handles (MVP 46).

### M5 — Engine Viewports

- P0: Host-provided `WgpuContext` to support both editor-hosted and engine-hosted integration. (see ADR 0010)
- P0: Viewport widget that displays an engine texture/render target.
- P0: Viewport mapping + input event contract (window -> uv/px). (prototype implemented)
- P0: Engine frame hook returns command buffers + explicit render target deltas; runner applies deltas before UI render. (prototype implemented; ADR 0038)
- P1: Overlay composition primitives: UI can render overlays over the viewport; editor apps can build gizmos/selection on top.
- P1: Input forwarding and capture rules (mouse/keyboard routed to viewport when focused).

Notes:

- A minimal overlay prototype exists (viewport hover crosshair + marquee selection rectangle in dock panels).

### M6 — Text System Upgrade

- P1: Basic text for inspector/property panels (layout + glyph atlas). (MVP done in demo; see `docs/mvp-archive.md`)
- P0: Text system boundary (`TextBlobId` + metrics contract). (done; see ADR 0006)
- P2: `cosmic-text` integration for editor-grade text shaping/layout (ADR 0029). (MVP done for single-line)
- P2: Shaped-run caching + incremental atlas uploads for large documents.
- P1: IME plumbing (winit `Ime` events + `set_ime_cursor_area` feedback path). (prototype implemented; see ADR 0012)
- P2: IME composition UI (inline preedit rendering in text widgets). (MVP done for single-line)
- P2: High-performance text widgets (code editor view, large buffers).

Immediate next step:

- MVP10 is prototype implemented (text editing baseline).
- MVP11 contract is locked (ADRs 0045/0046) and a multiline validation probe is prototype implemented in `fret-demo` (see `docs/mvp/active-plan.md`).

### M7 — Portability (wasm/WebGPU)

- P2: Platform layer for web canvas surfaces.
- P2: Input/clipboard limitations documented and handled.
- P3: Mobile planning (out of scope for early phases).

### M8 — Settings, Keymap, and Persistence

- P0: File-based configuration model + strong types. (see ADR 0014)
- P0: Dock layout persistence format with versioning. (see ADR 0013)
- P1: Settings UI primitives (token-driven) for inspector + app settings.
- P0: Keymap MVP (bind + route + persist) is implemented in `fret-demo` (see `docs/mvp-archive.md` / ADR 0021 / ADR 0022).
- P0: Keymap v2 sequences + pending bindings (ADR 0043) are prototype implemented.

### M9 — Command UI + Focus + Clipboard (Editor Usability Core)

These are the “you can actually drive the editor” foundations. They are intentionally scheduled early so that
all later UI work inherits the same command/focus/clipboard semantics instead of bespoke widget logic.

- P0: Command palette overlay + minimal menu data model (ADR 0023). (MVP done in demo; see `docs/mvp/active-plan.md` MVP 7)
- P0: Focus traversal and focus scopes (Tab navigation, modal focus trap) (ADR 0020). (see `docs/mvp/active-plan.md` MVP 8)
- P0: Clipboard boundary + text editing commands (text-only first) (ADR 0041). (see `docs/mvp/active-plan.md` MVP 9)

## Module Breakdown (Crates)

### `fret-core` (P0)

- IDs and geometry (`Px`, `Rect`, `NodeId`, resource IDs).
- Dock graph data structures.
- Display list / scene primitives contract.
- Cross-platform input event types (including IME events as data-only).

### `fret-runtime` (P0)

- Host-facing runtime boundary traits and portable value types used by `fret-ui` (ADR 0052).
- Defines the “UI -> host outputs” (`Effect`) and input/command/theme/menu boundary types.

### `fret-app` (P0)

- `App` runtime: globals, models, command registry, plugin registry.
- Scheduling/executor abstraction (no hard dependency on Tokio).

### `fret-ui` (P0)

- UI runtime (retained widget tree prototype today; declarative elements planned via ADR 0028).
- Invalidation, layout widgets, hit-testing.
- Focus, capture, command routing.
- Dock UI (`DockSpace`) as a widget consuming `DockManager`.

### `fret-ui-app` (P0)

- Optional integration convenience layer that binds `fret-ui` to `fret-app::App` (type aliases + re-exports).
- Keeps first-party app/demo code ergonomic while the core UI runtime stays embeddable.

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
- Provides an engine render hook for recording command buffers, submitted by the runner before UI sampling (ADR 0038).

### `fret-render` (P0)

- wgpu device/surface setup.
- Resource registries for atlas/textures/render targets.
- Pipelines for quads/SDF/text/images.
- Supports updating registered viewport targets (resize/view updates) to keep embedded viewports crisp.

## Definition of Done (per milestone)

- Builds with `cargo check` on macOS/Linux/Windows.
- A runnable demo for user-visible milestones (M0/M2/M3/M4/M5).
- Public API changes documented in `docs/architecture.md` and/or ADR notes.
