# ADR 0041: Drag-and-Drop, Clipboard, and Cross-Window Drag Sessions (Editor-Grade Docking UX)


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- imgui-rs: https://github.com/imgui-rs/imgui-rs

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
Status: Accepted

## Context

Editor UX requires:

- internal drag sessions (dock tab drag, splitter resize drags, reorder, selection rectangles),
- cross-window drag (tear-off docking, dragging tabs between OS windows),
- external OS drag-and-drop (dropping files/assets into panels),
- clipboard support (text now; images/binary later),
- consistent behavior across Windows/macOS/Linux (and future wasm).

If we do not define responsibilities and event semantics early, docking and multi-window work will accumulate
platform-specific hacks that are hard to unwind.

Related contracts:

- overlays/multi-root for drag previews and drop hints (ADR 0011),
- docking operations and persistence (ADR 0013),
- multi-window/DPI and coordinate semantics (ADR 0017),
- viewport input forwarding (ADR 0025),
- scheduling/effects for platform actions (ADR 0001 / ADR 0034),
- platform boundary (ADR 0003).

## Decision

### 1) Separate “internal drag” from “external (OS) drag-and-drop”

Fret supports two distinct mechanisms:

1. **Internal drag sessions** (UI-managed):
   - used for docking/tab drag, selection, splitters, reorder, etc.
   - fully deterministic and portable.
2. **External OS drag-and-drop** (platform-managed):
   - used for dropping files/URIs/text from outside the app,
   - initiating drags to other apps is optional and can be deferred.

Internal drag is a P0 requirement for docking UX; external DnD is a P1 requirement.

### 2) Internal drag session is a first-class UI runtime concept

The UI runtime maintains a global (app-scoped) `DragSession` state:

- `DragSessionId` (monotonic),
- `payload` (app-owned, type-erased; UI treats it as opaque),
- `origin_window`, `current_window`,
- `pointer_id` (mouse/touch),
- `start_position` and current position in **logical pixels**,
- `phase`: `Starting | Dragging | Dropped | Canceled`.

The drag session:

- survives across windows (tear-off/cross-window),
- drives overlay roots for drag preview and drop-zone hints (ADR 0011),
- is updated only by platform events + UI runtime routing (not by arbitrary widget side effects).

### 3) Cross-window internal drag is supported by runner-level routing

Because pointer capture is typically window-local at the OS level, cross-window drag is implemented as:

- the runner tracks “active drag session” and continues updating it from per-window pointer events,
- when the pointer leaves one window and enters another, the runner switches `current_window` and emits
  synthetic `DragEnter/DragLeave/DragMove` style events to the UI runtime.

All coordinates remain in logical pixels; window-local coordinates are converted at the platform boundary (ADR 0017).

### 4) Docking uses internal drag payloads + explicit `DockOp`s

Docking UI must not mutate the dock graph directly.

During drag:

- the UI emits hover feedback as overlays (drop hints),
- on drop, the UI emits a high-level `DockOp` transaction (ADR 0013),
- the app applies it to the dock graph and persists layout changes.

This keeps docking behavior testable and avoids UI-layer entanglement.

### 5) External OS drag-and-drop is exposed as platform events + effects

Platform backends translate OS DnD into normalized input events:

- `ExternalDragEntered { items, position }`
- `ExternalDragMoved { position }`
- `ExternalDragExited`
- `ExternalDrop { items, position }`

`items` is a normalized list:

- file paths / URIs (primary for editors),
- optional plain text,
- optional MIME-typed bytes (deferred).

Initiating external drags (dragging from the editor to OS) is supported via effects:

- `Effect::StartExternalDrag { items }` (deferred if platform support is incomplete).

#### Hit-test routing and “hover drop target” semantics (P0)

External drops must participate in the same routing model as pointer events:

- the UI runtime performs hit-testing using the current pointer position (window logical coordinates),
- top-most overlay roots are considered first (ADR 0011),
- the computed target receives external drag events so it can:
  - provide hover feedback (`DragEnter`/`DragOver` highlight),
  - accept/reject the payload by type (e.g. only `.png` files),
  - implement “drop into slot” behaviors (asset fields, scene view, hierarchy).

This requires explicit event hooks in the UI execution model (names are illustrative):

- `ExternalDragEnter { items, position }`
- `ExternalDragOver { items, position }`
- `ExternalDragLeave`
- `ExternalDrop { items, position }`

Acceptance is widget/element-owned policy:

- The framework routes the event and provides normalized `items`.
- The editor/app layer owns the asset pipeline (import, indexing, conversion); the framework must not
  hard-code “import on drop” policy (see ADR 0027).

#### Platform note (winit)

On winit, external file DnD often arrives as:

- `WindowEvent::HoveredFile(path)` / `HoveredFileCancelled`
- `WindowEvent::DroppedFile(path)`

These events do not inherently carry cursor position. The runner must combine them with the last known
`CursorMoved` position for hit-testing.

macOS note:

- With winit today, macOS file DnD provides `HoveredFile`/`DroppedFile` but does not provide a continuous
  “drag moved” callback (no `draggingUpdated`-style event), so hover routing is inherently best-effort.
  For editor-grade per-widget drop targets on macOS, a native platform backend (or upstream winit enhancement)
  will be required.

Tracking:

- Treat “native external DnD backend (macOS/Windows) with DragOver position” as a future P0/P1 platform task
  once core editor workflows (docking, viewports, text) are solid.

### 6) Clipboard is a platform service accessed via tokenized effects

Clipboard is modeled as a platform capability (app-scoped service):

- read/write plain text (P0),
- reserve shape for images and MIME-typed bytes (P1+).

Clipboard access is effect-driven to keep UI code platform-agnostic:

#### P0 text lane (normative)

- `Effect::ClipboardWriteText { window: AppWindowId, token: ClipboardToken, text: String }`
- `Effect::ClipboardReadText { window: AppWindowId, token: ClipboardToken }`
- completion:
  - `Event::ClipboardWriteCompleted { token, outcome }`
  - `Event::ClipboardReadText { token, text }`
  - `Event::ClipboardReadFailed { token, error }`

Rules:

- Explicit clipboard requests are token-addressable and window-aware.
- Clipboard writes remain best-effort platform requests, but explicit write requests MUST complete
  with an outcome rather than remaining diagnostics-only knowledge.
- Write completion does not guarantee read-after-write will succeed across all platforms.

#### P1 rich payload lane (future seam; non-normative)

When richer clipboard representations are added, preserve the same portability rules:

- `Effect::ClipboardWritePayload { window, token, payload }`
- `Effect::ClipboardReadPayload { window, token, formats, limits }`
- completion:
  - `Event::ClipboardWriteCompleted { token, outcome }`
  - `Event::ClipboardPayload { token, payload }`
  - `Event::ClipboardReadFailed { token, error }`

Portability rules:

- no raw paths/URIs in `fret-core`,
- portable typed payloads only,
- bounded bytes for binary payloads,
- file-like references must align with token/handle semantics used elsewhere (ADR 0053 / ADR 0264).

See ADR 0266 for the mobile/privacy-oriented clipboard portability rules and the richer payload
extension seam.

### 7) Modal overlays can block drops and input beneath

Modal overlay roots (ADR 0011) can:

- block pointer events to underlying roots,
- optionally block drop targets beneath (e.g. modal dialogs).

This rule is required for predictable editor UX.

## Consequences

- Docking/tab drag remains deterministic across platforms and windows.
- External file drops integrate cleanly without leaking platform objects into UI code.
- Clipboard and DnD remain compatible with the effect scheduling model (ADR 0034).

## Future Work

- External drag initiation support for file exports and asset drags.
- Rich clipboard formats (images, MIME bytes) and per-platform nuances.
- Touch/pen drag policy and gesture integration.

## Implementation Notes (Current Prototype)

- `fret-app` provides an app-scoped internal drag state (`crates/fret-app/src/drag.rs`).
- Docking uses the shared internal drag state (no per-dock bespoke drag session struct).
- Docking drop commits (including float-to-new-window requests) are emitted as `DockOp` (via `Effect::Dock`) and applied by the app/driver during effect draining.

### ImGui-style multi-viewport routing (screen-space pointer, hovered viewport)

Dear ImGui’s docking + multi-viewport UX relies on two ideas that are worth copying:

1. **Mouse position is tracked in an absolute/screen coordinate space** (still in logical pixels).
2. **A hovered viewport/window id is reported every time the mouse moves**, so docking/drag logic can
   reason about cross-window hover without requiring OS-level pointer capture handoff.

In the pinned reference backend (`repo-ref/dear-imgui-rs` @ `a3261f5ed219`), the winit platform code:

- computes screen-space logical mouse position by adding the per-window client position to the local
  cursor coordinates (`backends/dear-imgui-winit/src/platform.rs`, `WindowEvent::CursorMoved`),
- reports the hovered viewport id via `Io::add_mouse_viewport_event` and clears it on `CursorLeft`,
- routes window-local events to the correct secondary viewport window by matching `WindowId`
  (`backends/dear-imgui-winit/src/multi_viewport.rs`, `route_event_to_viewports`).

This is a better long-term mental model than “special-case dock drags in the runner”.

#### Recommended Fret direction (P0, replaces current stopgap)

Today’s prototype includes a stopgap: while a dock-tab drag is active, the runner broadcasts synthetic
`PointerEvent::Move` to all windows so that non-focused windows can render drop hints.

The recommended direction is to replace this with an explicit internal-drag routing channel:

- Runner tracks **global pointer state** in screen-space logical pixels (derived from winit window
  position + cursor local position; ADR 0017 still owns the “logical pixels are canonical” rule).
- Runner computes the **window-under-cursor** (hovered window) from window rectangles in screen space.
- Runner emits **synthetic internal-drag hover events** to the UI runtime, per-window, even when the
  OS-level pointer is effectively captured by another window.
  - `InternalDragEnter/Over/Leave/Drop` should be routed via the same hit-test rules as `ExternalDrag`
    (overlays first, then content).
- Widgets that care about cross-window drags (DockSpace, Project panel drop targets) react to
  internal-drag events instead of relying on cross-window pointer move broadcasting.

##### Internal drag drop targets (mechanism, not policy)

To make internal-drag drop targets composable and deterministic, the UI runtime provides two
mechanism-only routing layers:

1. **Hit-test derived targets via `InternalDragRegion`**:
   - Declarative trees can wrap a subtree with an `InternalDragRegion` element.
   - When dispatching `Event::InternalDrag`, the runtime prefers the closest (deepest) enabled
     `InternalDragRegion` ancestor of the hit-tested node as the dispatch target.
   - If no `InternalDragRegion` exists in the ancestor chain, dispatch falls back to normal
     hit-testing (the hit-tested node + bubbling).

   This encourages a common “drop zone” pattern (pane surface, tab strip, list) that does not depend
   on child widgets and reduces accidental coupling between policy and hit-testing details.

2. **Explicit overrides via `InternalDragRouteService`**:
   - Some drag flows need to route to a stable anchor even when hit-testing fails or the pointer is
     outside all windows (e.g. docking tear-off follow).
   - App/policy layers can register per-window overrides keyed by `(window, drag_kind)`.
   - When the runner marks a drag session as `cross_window_hover`, internal drag events may be routed
     to that anchor node.

`InternalDragRegion` is the “normal” path for in-window drop targets; `InternalDragRouteService` is
an escape hatch for special cases that cannot rely on hit-testing.

Note on payloads:

- `DragSession.payload` is type-erased and should not be relied on for most policy-layer DnD flows.
  Prefer app-owned `Model` state keyed by pointer/session id (so action hooks remain object-safe).

Benefits:

- Removes runner-level “DockPanel” special-cases and reduces accidental side-effects on unrelated widgets.
- Matches editor-grade docking behavior patterns used by Dear ImGui (hovered viewport semantics).
- Makes it easier to support future backends (wasm single-canvas can still compute hovered logical windows).

Implementation status:

- Prototype implemented: dock drag uses `Event::InternalDrag` for cross-window hover/drop, and the runner no longer
  broadcasts `PointerEvent::Move` across windows.
- Declarative internal drag targets: `InternalDragRegion` element + handler (`crates/fret-ui/src/element.rs`,
  `crates/fret-ui/src/declarative/host_widget/event/internal_drag_region.rs`) and dispatch targeting in
  `crates/fret-ui/src/tree/dispatch.rs`.
- Runner invariant: on mouse button release, the runner ends any active **cross-window** internal drag session to
  prevent "stuck" drags if no widget consumes the `InternalDrag::Drop` (or if the drop target is missing).
- Runner tracking note: while a cross-window drag is active, the runner may update screen-space cursor position using
  `DeviceEvent::MouseMotion` deltas to avoid platform behaviors where `WindowEvent::CursorMoved` is clamped during
  OS-level mouse capture.

External OS file DnD prototype:

- Event type: `Event::ExternalDrag` (`crates/fret-core/src/input.rs`)
- Winit mapping (merges with last cursor position for routing): `crates/fret-launch/src/runner/mod.rs`
- UiTree hit-test routing support: `crates/fret-ui/src/tree/mod.rs`
- Demo/probe harness: use `apps/fret-examples/src/drag_demo.rs` (invoked via `cargo run -p fret-demo --bin drag_demo`) when adding or validating external OS file DnD behavior.
