# ADR 0041: Drag-and-Drop, Clipboard, and Cross-Window Drag Sessions (Editor-Grade Docking UX)

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

### 6) Clipboard is a platform service accessed via effects

Clipboard is modeled as a platform capability (app-scoped service):

- read/write plain text (P0),
- reserve shape for images and MIME-typed bytes (P1+).

Clipboard access is effect-driven to keep UI code platform-agnostic:

- `Effect::ClipboardSetText { text: String }`
- `Effect::ClipboardGetText { window: AppWindowId }` (delivers `Event::ClipboardText(String)` back into the window event stream)

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

External OS file DnD prototype:

- Event type: `Event::ExternalDrag` (`crates/fret-core/src/input.rs`)
- Winit mapping (merges with last cursor position for routing): `crates/fret-runner-winit-wgpu/src/runner.rs`
- UiTree hit-test routing support: `crates/fret-ui/src/tree.rs`
- Demo probe widget (logs hover/drop): `crates/fret-demo/src/dnd_probe.rs`
