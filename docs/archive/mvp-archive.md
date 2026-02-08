> Archived: this plan is kept for history; some file paths referenced here may no longer exist.

# MVP Plan Archive (Completed Stages)

This file keeps the historical, detailed MVP stage definitions that are already completed.

For current priorities and milestones, prefer:

- `docs/roadmap.md`
- `docs/shadcn-declarative-progress.md`

---

# MVP 0–6 (Archived Definitions)

These definitions reflect the intent and “definition of done” used while landing MVP 0–6.

## MVP Principles

- Keep each stage small enough to finish without redesigning contracts.
- Prefer building vertical slices that validate a hard contract end-to-end.
- Avoid “half abstraction”: if a contract is required by the stage, implement the minimal skeleton rather than bypassing it.

## Current Workspace Status

- MVP 0: done (runner scheduling skeleton; `TickId`/`FrameId`, RAF, timers)
- MVP 1: done (model store v1: `Model<T>` + `revision`)
- MVP 2: MVP done in demo (declarative elements skeleton; IDs + cross-frame state + unkeyed warning)
- MVP 3: MVP done in demo (stable `PanelKind`/`PanelKey` + `layout.json` v1 persist/restore)
- MVP 4: done (internal `DragSession` adopted by docking; DockOp emission on drop)
- MVP 5: MVP done in demo (text MVP: labels + single-line text input + IME preedit/commit + cursor area)
- MVP 6: MVP done in demo (commands + keymap MVP: bind, route, persist)

Code anchors:

- Effects/models: `crates/fret-app/src/app.rs`
- Runner scheduling: `crates/fret-launch/src/runner/mod.rs`
- Dock identity/persistence ops: `crates/fret-core/src/panels.rs`, `crates/fret-core/src/dock_layout.rs`, `crates/fret-core/src/dock_op.rs`
- Demo persistence: implemented in `apps/fret-demo` (entrypoints evolve; search for `.fret/` usage).
- Declarative elements skeleton: `crates/fret-ui/src/elements/mod.rs`
- Declarative demo widget: `apps/fret-examples/src/components_gallery.rs` (entrypoint evolves; search for declarative roots)
- IME plumbing + cursor-area effects: `crates/fret-launch/src/runner/mod.rs`
- Text system + atlas uploads: `crates/fret-render-wgpu/src/text.rs`
- `SceneOp::Text` rendering: `crates/fret-render-wgpu/src/renderer/mod.rs`
- Minimal `Text` / `TextInput` widgets: `crates/fret-ui/src/text_input/mod.rs`
- Demo property rows with labels: `apps/fret-editor/src/inspector_edit.rs`
- Keymap/when parsing + resolver: `crates/fret-app/src/keymap.rs`, `crates/fret-app/src/when_expr.rs`
- KeyDown → Command resolution (window-scoped): `crates/fret-ui/src/tree/mod.rs`
- Command dispatch (effects → runner → driver → UiTree): `crates/fret-launch/src/runner/mod.rs`
- Demo command/keymap wiring: prototype lived in `crates/fret-demo` (entrypoints evolve; search for keymap loading).
- Sample keymap file: `docs/examples/keymap.json`

## MVP 0 — Runner Scheduling Skeleton (Event-Driven + IDs)

Goal: replace ad-hoc “keep alive” redraw loops with an effects-driven scheduler that matches ADR 0034.

**Scope**

- Implement `TickId` and `FrameId` in the desktop runner.
- Add effect plumbing for:
  - `RequestAnimationFrame(window)`
  - `SetTimer(...)` / `CancelTimer(...)`
- Coalesce redraw requests per window per tick.
- Enforce bounded effects draining (`max_effect_drains_per_tick = 8`).

**Non-goals**

- Fancy animation system (easing, tweening).
- A production-grade async runtime integration.

**Definition of Done**

- No user input / no timers / no RAF requests ⇒ runner idles with `ControlFlow::Wait`.
- Dock dragging and hover feedback can request RAF and remain smooth at high refresh.
- Tracing spans can correlate `TickId` and `FrameId` (basic logging is fine; full inspector can wait).

**Demo Checklist**

- Start `fret-demo`, idle CPU stays low.
- Begin a dock drag, the UI redraws continuously until drop, then idles again.

## MVP 1 — Model Store v1 (App-Owned + Revision)

Goal: align implementation with ADR 0031 so UI can scale without borrow fights.

**Scope**

- `App::read` + `App::update` API shape.
- Per-model `revision: u64`.
- Re-entrant update protection for same model id.

**Definition of Done**

- A widget/event handler can mutate model state and request redraw/effects without `RefCell`.
- Simple caching can key off model revisions (even if no caching is implemented yet).

## MVP 2 — Declarative Elements Skeleton (IDs + Cross-Frame State)

Goal: land the minimum GPUI-style authoring/runtime skeleton (ADR 0028 / ADR 0039) early, before scaling widget count.

**Scope**

- Introduce `ElementId` / `GlobalElementId` and a window-scoped element state store keyed by `(GlobalElementId, TypeId)`.
- Implement mark/sweep with a small GC lag (`gc_lag_frames = 2`) after present.
- Support explicit keys for dynamic lists/trees and a debug warning path when keys are missing and order changes.
- Provide an authoring surface that can coexist with the current retained `UiTree`:
  - minimal `Render` or `RenderOnce` entrypoint per root,
  - a compatibility adapter so existing widgets can be hosted while the authoring model transitions.

**Non-goals**

- A full component library.
- A sophisticated caching layer (`cached` views); only reserve the hook points.

**Definition of Done**

- Demo renders a small declarative UI whose element-local state persists across frames.
- A dynamic list reorders without losing state when keyed.
- Debug builds can surface a warning when a dynamic list is rendered without keys and the sequence changes.

**Status**

- MVP done in the demo harness (entrypoints evolve; see `apps/fret-examples/src/components_gallery.rs` for declarative roots).

## MVP 3 — Docking Identity + Persistence v1

Goal: avoid later “PanelId → PanelKind” migrations by locking stable identities early.

**Scope**

- Introduce stable `PanelKind` (string key) and optional instance id.
- Persist/load a layout file v1 per ADR 0013.
- Migration module stub (even if only v1 exists).
- Docking emits `DockOp` transactions on drop (ADR 0013 / ADR 0041).

**Definition of Done**

- Restart demo/editor app and restore the dock layout.
- Plugins (future) can register panels without runtime-only ids leaking into config files.

**Status**

- MVP done in `fret-demo` (persist/restore to `./.fret/layout.json`)

## MVP 4 — Internal Drag Session (Cross-Window Ready)

Goal: unify docking drag and future widget drags (tree/table/inspector) under ADR 0041.

**Scope**

- App-scoped `DragSession` state with payload.
- Docking uses DragSession + DockOp emission (no direct graph mutation during drag).

**Definition of Done**

- Dock drag uses the shared DragSession mechanism (no per-widget bespoke drag state).

**Status**

- Implemented: `crates/fret-app/src/drag.rs` + docking migrated to use it.

## MVP 5 — Text MVP (Labels + Single-Line Text Input Skeleton)

Goal: unlock editor UI surfaces (Inspector/Hierarchy/Console) by landing a stable cross-platform text pipeline skeleton (ADR 0029) and the core editing loop (ADR 0012) early.

**Scope**

- Implement a `TextService` implementation that can:
  - shape text (initially via `cosmic-text`) and return `TextBlobId + TextMetrics`
  - manage glyph raster + atlas uploads (alpha coverage atlas; reserve RGBA path for emoji)
- Render `SceneOp::Text` in `fret-render` using the atlas.
- Add a minimal `Text` (label) widget and a minimal `TextInput` widget:
  - caret + selection state stored as element/widget state
  - consumes `Event::TextInput` and `Event::Ime(ImeEvent::{Preedit,Commit})`
  - updates candidate window position via `Effect::ImeSetCursorArea`

**Non-goals**

- Code-editor-grade text (large documents, shaping caches, bidi correctness beyond cosmic-text defaults).
- Rich clipboard formats and external drag initiation.

**Definition of Done**

- `fret-demo` can render labels in the inspector list (readable at multiple DPI scales).
- `fret-demo` has a focused `TextInput` that supports:
  - ASCII typing (via `TextInput`)
  - IME preedit/commit (inline preedit rendering can be minimal, but must exist)
  - correct `set_ime_cursor_area` updates while editing

**Status**

- MVP done in `fret-demo` (see the anchors above).

## MVP 6 — Commands + Keymap MVP (Bind + Route + Persist)

Goal: make the editor usable without bespoke per-widget key handling by landing command routing + keymap persistence (ADR 0020 / ADR 0021 / ADR 0023).

**Scope**

- Define a minimal keymap file and loader (user scope is enough for MVP).
- Bind a small set of commands (3–6) and route them through the focused node with bubbling rules.
- Show command dispatch in `fret-demo` (e.g. toggle panels, focus next, open/close overlays).

**Definition of Done**

- A keymap file can override defaults.
- Commands trigger deterministically across windows with focus-aware routing.

**Status**

- MVP done in `fret-demo`:
  - `F1`: toggles modal overlay (`demo.toggle_modal`)
  - `Ctrl+L` (when focused text input): clears text (`text.clear`)
