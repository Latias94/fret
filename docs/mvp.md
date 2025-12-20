# MVP Plan (Active, Short-Horizon)

This document is the **current execution plan** that complements `docs/roadmap.md`.

Completed stage definitions are archived in `docs/mvp-archive.md` to keep this file actionable.

## Current Workspace Status

- MVP 0–6: done (see `docs/mvp-archive.md`)
  - MVP 5: Text MVP landed (single-line input + IME cursor-area loop)
  - MVP 6: Commands + keymap MVP landed (bind/route/persist; `when` gating)
- MVP 7: MVP done in demo (command palette overlay; shortcut display; menu model types added)
  - Keymap v2 sequences + pending bindings are prototype implemented (ADR 0043 / ADR 0021).
- Editor-scale list widgets: prototype implemented
  - `VirtualList` (virtualization + stable keys + multi-selection) (ADR 0042 / ADR 0047)
  - `TreeView` (hierarchy-style tree over `VirtualList`)
- MVP 12: MVP done in demo (context menu overlay + submenu + keyboard nav + focus restore)
- MVP 13: MVP done in demo (Hierarchy selection model → Inspector panel)
- MVP 14: MVP done in demo (primitive inspector editing baseline)
- MVP 15: MVP done in demo (Hierarchy drag & drop: reorder + reparent)
- MVP 16: MVP done in demo (DockSpace hosts app-owned panel content via `DockPanelContentService`)
- MVP 17: prototype implemented in demo (property tree + editor registry + mixed values; bool/string/f32/vec3 editors)
- MVP 18: prototype implemented in demo (ToolManager with capture + marquee select + pan/orbit drag interactions)
- MVP 19: prototype implemented in demo (viewport click-to-select + selection highlight overlay)
- MVP 20: prototype implemented in demo (translate gizmo stub: overlay + explicit drag phases; Q/W tool switching; Esc cancel rollback)
- MVP 21: prototype implemented in demo (dock drag hints + dock tab context menu; debounced layout persistence)
- MVP 22: prototype implemented in demo (undo/redo command stack; inspector edits + translate gizmo integrate)
- MVP 23: prototype implemented in demo (Hierarchy drag & drop emits undoable commands; undo/redo restores hierarchy + selection)
- MVP 24: prototype implemented in demo (edit transactions + coalescing; viewport translate drag produces a single undo entry)
- MVP 25: prototype implemented in demo (translate axis constraints + Shift snapping stub)
- Inspector + viewport tooling boundaries: drafted as Proposed ADRs
  - ADR 0048: Inspector property protocol + custom editor registry (example editor layer)
  - ADR 0049: Viewport tools (input capture + overlay rendering) (example editor layer)

## MVP 7 — Command UI Surfaces (Palette + Menu Skeleton)

Goal: validate ADR 0023 end-to-end by exposing commands as first-class UI surfaces, not only shortcuts.

**Scope**

- Command palette overlay:
  - open/close via a command (e.g. `command_palette.toggle`)
  - search/filter and execute commands
  - show the resolved shortcut (if bound) next to each command
- Minimal menu model (data-only) that can later drive a menubar/context menus:
  - a menu is just a list of command ids + separators + optional `when`
  - rendering can be deferred; the goal is to lock the contract and wiring
- Overlay correctness:
  - palette is a multi-root overlay (ADR 0011)
  - focus enters palette while open and returns to the previous focus on close (ADR 0020)

**Non-goals**

- Perfect fuzzy matching and scoring.
- Full native menubar integration (macOS global menu, etc.).

**Definition of Done**

- `fret-demo` can open a command palette overlay and run a command by keyboard.
- Palette obeys modal gates: if a modal is open, palette behavior is deterministic (either blocked or becomes the modal itself; pick one and lock it).
- Palette lists commands with titles and (if bound) the active shortcut.

References:

- `docs/adr/0011-overlays-and-multi-root.md`
- `docs/adr/0020-focus-and-command-routing.md`
- `docs/adr/0021-keymap-file-format.md`
- `docs/adr/0022-when-expressions.md`
- `docs/adr/0023-command-metadata-menus-and-palette.md`

## MVP 8 — Focus Traversal + Focus Scopes

Goal: make large editor UIs usable with keyboard navigation, and lock focus-scope semantics before widget count explodes.

**Scope**

- Implement focus traversal (`Tab` / `Shift+Tab`) across a focus scope.
- Define focus scopes for:
  - modal overlays (trap focus)
  - dock panels (local traversal)
  - command palette (single scope root)
- Expose traversal as commands (`focus.next`, `focus.previous`) so keymaps can override.

**Definition of Done**

- `fret-demo` can traverse focus between at least:
  - a dock tab bar (or a few focusable buttons),
  - a `TextInput`,
  - a modal overlay control.
- Focus traversal never escapes a modal overlay.

Status:

- Prototype implemented in `fret-demo` (Tab / Shift+Tab bindings, focusable traversal across multiple TextInputs).

References:

- `docs/adr/0011-overlays-and-multi-root.md`
- `docs/adr/0020-focus-and-command-routing.md`

## MVP 9 — Clipboard P0 (Text)

Goal: land the platform boundary for clipboard and make `TextInput` production-usable.

**Scope**

- Platform clipboard service (text only) behind the effect boundary (ADR 0003 / ADR 0001).
- Core text commands:
  - `text.copy`, `text.cut`, `text.paste`, `text.select_all`
- `TextInput` integration and keymap defaults (platform conventions).

**Definition of Done**

- `fret-demo` `TextInput` supports copy/cut/paste/select-all across platforms.
- Clipboard plumbing is effect-driven (UI has no direct platform handle).

Status:

- Prototype implemented in `fret-demo` (Cmd/Ctrl+A/C/X/V bindings; effect-driven clipboard read/write; paste delivered as `Event::ClipboardText`).

References:

- `docs/adr/0001-app-effects.md`
- `docs/adr/0003-platform-boundary.md`
- `docs/adr/0012-keyboard-ime-and-text-input.md`
- `docs/adr/0041-drag-and-drop-clipboard-and-cross-window-drag-sessions.md`

## MVP 10 — Text Editing Baseline (Caret + Selection + Navigation)

Goal: make `TextInput` usable enough for editor UIs by locking the state model and core editing commands early.

**Scope**

- Replace the MVP “select_all bool” with a real selection model:
  - caret position (byte index or grapheme cluster index; pick one and lock it),
  - selection range (start/end), including mouse drag selection.
- Core navigation/edit commands routed via commands (not hard-coded keys):
  - `text.move_left/right`, `text.move_word_left/right`, `text.move_home/end`,
  - `text.delete_backward/forward`, `text.delete_word_backward/forward`,
  - `text.select_*` variants (shift-modified movement).
- IME correctness:
  - inline preedit rendering in the widget (already MVP for single-line),
  - `ImeSetCursorArea` updated from the actual caret rect after layout/paint (so candidate windows track the caret).

**Definition of Done**

- `fret-demo` `TextInput` supports:
  - caret movement with arrow keys,
  - selection expansion with shift+arrows,
  - copy/cut/paste over selection,
  - IME candidate window anchored at caret (macOS/Windows).

Status:

- Prototype implemented in `fret-demo` (caret + selection; mouse drag selection; arrow/word nav + delete commands; repeatable key-repeat for text editing commands; IME cursor-area follows caret).

References:

- `docs/adr/0012-keyboard-ime-and-text-input.md`
- `docs/adr/0029-text-pipeline-and-atlas-strategy.md`
- `docs/adr/0044-text-editing-state-and-commands.md`

## MVP 11 — Text Layout Contracts (Hit Test + Caret Metrics)

Goal: lock the long-term contract for “text geometry queries” that high-end widgets require (code editor, large
documents, precise IME caret positioning) before building more components on top.

This MVP is primarily a **contract / API boundary** milestone; performance work can iterate after the contract is stable.

**Scope**

- Lock the geometry query boundary for:
  - caret placement for mouse + keyboard editing,
  - selection painting geometry,
  - IME cursor-area placement (candidate window anchoring),
  - future multiline and long-document widgets.
- Define the multiline semantics that cannot change later (line breaks, caret affinity, coordinate spaces).

**Definition of Done**

- ADRs are accepted and indexed:
  - `docs/adr/0045-text-geometry-queries-hit-testing-and-caret-metrics.md`
  - `docs/adr/0046-multiline-text-layout-and-geometry-queries.md`
- A minimal validation plan exists:
  - wrapped/multiline widget demo (not a code editor yet),
  - y-aware hit testing + caret rect behavior at line breaks,
  - keyboard vertical caret movement (`text.move_up/down`, `text.select_up/down`) validated via the geometry queries,
  - IME cursor-area follows caret rect (window coordinates).

References:

- `docs/adr/0006-text-system.md`
- `docs/adr/0029-text-pipeline-and-atlas-strategy.md`
- `docs/adr/0045-text-geometry-queries-hit-testing-and-caret-metrics.md`
- `docs/adr/0046-multiline-text-layout-and-geometry-queries.md`
- Zed/GPUI text system patterns:
  - `repo-ref/zed/crates/gpui/src/text_system.rs`

## MVP 12 — Context Menus (Overlay + Menu Rendering)

Goal: make editor UIs feel “native” by supporting right-click workflows (Hierarchy/Project/Viewport) with correct
overlay + focus semantics.

**Scope**

- Context menu widget rendered as a multi-root overlay (ADR 0011):
  - click outside dismiss
  - `Escape` dismiss
  - arrow key navigation + `Enter` activate
  - submenu support (`Right` opens, `Left` closes)
- Menu content is derived from the data-only menu model (ADR 0023) and `when` gating (ADR 0022).
- Focus behavior:
  - focus enters the menu while open
  - focus restores to the previous node on close (ADR 0020)
- Basic integration points in demo:
  - `TreeView` right-click opens a Hierarchy-style menu (expand/collapse)
  - `VirtualList` right-click opens a small list menu
  - viewport surface right-click opens a viewport menu (demo-only)

**Non-goals**

- Native OS menu bars.
- Perfect sizing/positioning (multi-monitor, avoid-caret heuristics).

**Definition of Done**

- `fret-demo` shows a context menu on right-click for at least one widget surface.
- Menu supports keyboard navigation and closing rules consistently across platforms.
- Menu items execute commands through the existing command routing model.

References:

- `docs/adr/0011-overlays-and-multi-root.md`
- `docs/adr/0020-focus-and-command-routing.md`
- `docs/adr/0022-when-expressions.md`
- `docs/adr/0023-command-metadata-menus-and-palette.md`

## MVP 13 — Unity-Style Editor Shell Wiring (Hierarchy + Inspector)

Goal: validate the “editor app” programming model on top of Fret by wiring a minimal Unity-like workflow:
select in Hierarchy → inspect/edit properties → see changes reflected.

**Scope**

- Minimal editor shell wiring in `fret-demo`:
  - Hierarchy (`TreeView`) updates an app-owned selection model
  - Inspector panel renders based on that selection model
- App-owned selection model in `fret-app` model store (ADR 0031):
  - selected entity id(s) (stable identity)
  - selection changes trigger UI invalidation without widget-owned global state
- Inspector rendering is data-driven:
  - read-only text rows are sufficient for this MVP

Notes:

- Dock panel content wiring is intentionally deferred to MVP 16; this MVP validates the model/update flow first.

**Definition of Done**

- Clicking an entity in Hierarchy updates Inspector content (same frame).
- Right-click context menu works in Hierarchy without breaking selection semantics.
- Dock persistence continues to work (no regressions).

Status:

- MVP done in `fret-demo` (see `crates/fret-demo/src/editor_shell.rs`).

References:

- `docs/adr/0031-app-owned-models-and-leasing-updates.md`
- `docs/adr/0013-docking-ops-and-persistence.md`
- `docs/adr/0042-virtualization-and-large-lists.md`

## MVP 14 — Inspector Editing Baseline (Primitive Fields)

Goal: make the Inspector minimally editable so the “select → inspect → edit” loop feels real.

**Scope**

- Field editors (P0 primitives only):
  - bool checkbox/toggle,
  - number input (int/float) with parse/validation,
  - string input (single-line `TextInput`).
- Commit semantics:
  - edits apply on `Enter` / focus loss,
  - invalid input is visually indicated but does not crash.
- Command integration (no hard-coded key handling):
  - `inspector.commit`, `inspector.revert`, `inspector.increment`, `inspector.decrement` (optional).
- Focus correctness:
  - field editing behaves well with Tab traversal and context menus (ADR 0020 / MVP 8 / MVP 12).

**Non-goals**

- Undo/redo policy (editor-app scope; see ADR 0024).
- Drag-to-scrub numeric fields (Unity-style) until internal drag sessions are revisited.

**Definition of Done**

- `fret-demo` Inspector can edit at least one bool + one number + one string property for the selected entity model.
- Editing never breaks focus traversal, clipboard, or IME for text fields.

Status:

- MVP done in `fret-demo` (see `crates/fret-demo/src/editor_shell.rs` and `crates/fret-demo/src/inspector_edit.rs`).

References:

- `docs/adr/0031-app-owned-models-and-leasing-updates.md`
- `docs/adr/0020-focus-and-command-routing.md`
- `docs/adr/0044-text-editing-state-and-commands.md`

## MVP 15 — Hierarchy Drag & Drop (Reorder + Reparent)

Goal: unlock core Unity hierarchy workflows (reorder and reparent via drag & drop) and validate internal drag sessions
across dock panels.

**Scope**

- Internal drag session for tree items (ADR 0041):
  - start drag on row (thresholded),
  - hover insertion preview (above/below/into),
  - drop commits an app-owned dock/tree op (no widget-owned global state).
- UX semantics:
  - right-click still selects before opening context menu,
  - disclosure click does not change selection (Unity-style).

**Non-goals**

- External OS drag & drop (files) integration.
- Multi-window cross-process DnD.

**Definition of Done**

- `fret-demo` Hierarchy supports dragging an entity to reorder within the same parent and reparent into another entity.
- Drop preview is rendered as an overlay without breaking scene ordering constraints.

References:

- `docs/adr/0041-drag-and-drop-clipboard-and-cross-window-drag-sessions.md`
- `docs/adr/0011-overlays-and-multi-root.md`
- `docs/adr/0042-virtualization-and-large-lists.md`

## MVP 16 — Dock Panel Content Wiring (Panel → UI Root)

Goal: move “editor shell” widgets into the docking system by making dock panels host UI content, while keeping panel
identity stable across persistence and plugins.

**Scope**

- Define a `PanelKind` → “panel content builder” contract at the demo/app layer (ADR 0016):
  - a panel kind maps to a UI root node (or element root) per window
  - panel content is app-owned; `fret-ui` provides only the hosting/lifecycle hooks
- Extend `DockSpace` hosting so that each visible dock panel can mount its content node and receive input/focus.
- Ensure focus/command routing works across dock tabs and panel content without bespoke glue.

**Non-goals**

- Cross-window panel mirroring.
- Plugin loading/runtime registry (panel builder can be hard-coded in demo).

**Definition of Done**

- `fret-demo` shows Hierarchy + Inspector as real dock panels (tabs), not only in the side demo column.
- Switching dock tabs preserves per-panel UI state (selection, scroll) via stable panel identity.

Status:

- MVP done in `fret-demo` (panel content is app-owned and mounted under `DockSpace` via `DockPanelContentService`).

References:

- `docs/adr/0016-plugin-and-panel-boundaries.md`
- `docs/adr/0013-docking-ops-and-persistence.md`
- `docs/adr/0020-focus-and-command-routing.md`

## MVP 17 — Inspector P0 (Property Protocol + Minimal Editors)

Goal: define the “productivity core” workflow for a Unity/Godot-like inspector, without binding the framework to any
particular reflection/ECS implementation.

**Scope**

- Implement the editor-layer contract from ADR 0048 in `fret-demo`:
  - stable `PropertyPath` + `PropertyValue` types (data-only),
  - a `PropertyTree` builder for the demo selection model,
  - a `PropertyEdit` event emitted by inspector UI (no direct model mutation).
- Minimal built-in editors sufficient for real workflows:
  - bool, int/float, string,
  - enum (choices in metadata),
  - vec2/vec3 (as grouped leafs).
- “Mixed value” UX for multi-selection (display `Mixed`, apply edit to all selected compatible targets).
- Ensure edits follow the “command/transaction then sync-to-model” discipline (ADR 0024 / Fyrox-style).

**Non-goals**

- Full engine reflection integration (this MVP uses a demo adapter).
- Full undo/redo history UI (only the operational boundary is validated).

**Definition of Done**

- Editing a property in Inspector updates the selected entities in the demo model deterministically.
- Drag-like controls emit `Begin/Update/Commit/Cancel` so future undo coalescing is straightforward.
- Custom editor registration is demonstrated via at least one overridden editor.

References:

- `docs/adr/0048-inspector-property-protocol-and-editor-registry.md`
- `docs/adr/0027-framework-scope-and-responsibilities.md`
- `docs/adr/0024-undo-redo-and-edit-transactions.md`

## MVP 18 — Viewport Tools P0 (Input Capture + Overlay Rendering)

Goal: establish the editor-layer pattern for viewport tools so later gizmos/selection/camera controls don’t become bespoke
glue.

**Scope**

- Implement the editor-layer contract from ADR 0049 in `fret-demo`:
  - a `ToolManager` model (active tool mode + per-viewport tool state),
  - tool routing from `Effect::ViewportInput` based on focus/modal gating,
  - explicit capture during drags (consistent move/up delivery).
- Provide at least one concrete tool with overlays:
  - selection marquee OR translate gizmo stub (overlay-only is fine),
  - overlay rendered above the viewport surface.
- Scheduling rules:
  - request animation frames only while tool is active (ADR 0034).

**Non-goals**

- Engine picking/gizmo math correctness (can be stubbed).
- Advanced snapping and multi-history undo policies.

**Definition of Done**

- A focused viewport reliably receives tool input and shows interactive overlay feedback while hovering/dragging.
- Tool drags remain smooth and consistent across docking and multi-window tear-off.

References:

- `docs/adr/0049-viewport-tools-input-capture-and-overlays.md`
- `docs/adr/0025-viewport-input-forwarding.md`
- `docs/adr/0034-timers-animation-and-redraw-scheduling.md`

## MVP 19 — Viewport Picking P0 (Click-to-Select + Highlight)

Goal: make the “click in viewport → selection updates everywhere” loop feel like a real editor.

**Scope**

- Add viewport click-to-select behavior under the `Select` tool mode (ADR 0049):
  - left click selects a single entity,
  - modifier rules match Hierarchy selection semantics (Ctrl/Cmd toggle; Shift additive).
- Add a simple selection highlight overlay drawn over the viewport:
  - “picked entity marker” is sufficient (no full outline rendering required).
- Keep the picking source swappable:
  - demo can use a stub picker (e.g. “pick nearest demo entity in screen-space”),
  - engine-integrated picking can replace it later without changing UI/tool contracts.

**Non-goals**

- Correct 3D ray casting, depth-tested selection, or GPU ID buffers.
- Multi-viewport multi-camera correctness.

**Definition of Done**

- Clicking in a focused viewport updates the app-owned selection model (and thus Hierarchy + Inspector) in the same frame.
- The viewport shows an unambiguous highlight for the selected entity/entities.

Status:

- MVP done in `fret-demo` (grid-stub picking and a persistent selection highlight overlay).

References:

- `docs/adr/0031-app-owned-models-and-leasing-updates.md`
- `docs/adr/0025-viewport-input-forwarding.md`
- `docs/adr/0049-viewport-tools-input-capture-and-overlays.md`

## MVP 20 — Translate Gizmo Stub (Overlay + Drag Phases)

Goal: validate the core interaction model for editor gizmos (capture + hover + drag + commit) before committing to real
math and snapping rules.

**Scope**

- Provide a `Move`/`Translate` tool mode with a minimal gizmo:
  - overlay-only handle rendering is enough (axes lines + center handle),
  - drag updates the selected entity transform in the demo model.
- Drag phases must be explicit:
  - `Begin`/`Update`/`Commit`/`Cancel` so undo coalescing is straightforward later (ADR 0024).
- Input capture and modal gating follow the viewport tools contract (ADR 0049).

**Non-goals**

- Precise 3D manipulation, snapping, and axis constraints.
- Full undo/redo UI (only the operational boundary is validated).

**Definition of Done**

- Dragging the gizmo updates the selected entity transform smoothly and deterministically.
- Cancel returns the entity to the previous transform without leaving UI/tool state inconsistent.

Status:

- Prototype implemented in `fret-demo`:
  - `Q` / `W` tool switching (Select / Move),
  - overlay-only gizmo rendering over viewports,
  - explicit `Begin`/`Update`/`Commit`/`Cancel` phases (Esc cancels and rolls back).

References:

- `docs/adr/0049-viewport-tools-input-capture-and-overlays.md`
- `docs/adr/0024-undo-redo-and-edit-transactions.md`

## MVP 21 — Dock UX Polish (Hints + Context Menu + Debounced Persistence)

Goal: upgrade docking UX toward Unity/Godot expectations without changing core docking contracts.

**Scope**

- Godot-style dock drag hint overlay and tab drop indicators (visual guidance).
- Dock context menu actions:
  - float/dock, close, move tab left/right (subset is fine).
- Debounced layout persistence to reduce disk churn during interactive operations.

**Non-goals**

- Full workspace management UI (layouts list, reset-to-default presets).

**Definition of Done**

- Docking actions are visually guided and feel “product-like” (no invisible drop zones).
- Layout save is delayed during drags and still persists correctly at the end.

Status:

- Prototype implemented in `fret-demo`:
  - drop target hints shown while dragging dock tabs,
  - dock tab context menu actions (float + move left/right),
  - debounced `layout.json` persistence after dock ops and window placement changes.

References:

- `docs/adr/0013-docking-ops-and-persistence.md`
- `docs/adr/0011-overlays-and-multi-root.md`

## MVP 22 — Undo/Redo P0 (Command Stack + Coalescing Boundary)

Goal: establish the editor-app boundary for undo/redo early so all subsequent tools/editors can align to it.

**Scope**

- A command stack that supports:
  - undo/redo,
  - a clear transaction boundary for “drag-like continuous edits”.
- Integrate at least two edit sources:
  - inspector property edits,
  - viewport translate gizmo drags.

**Non-goals**

- Multi-document histories and advanced merge policies (Godot-style).

**Definition of Done**

- Undo/redo works for at least one inspector edit and one viewport tool commit.
- Continuous edits (e.g. gizmo drag) can be represented as a single history entry.

Status:

- Prototype implemented in `fret-demo`:
  - `edit.undo` / `edit.redo` commands with default bindings,
  - inspector `property_edit.commit` pushes undoable edit commands,
  - translate gizmo drags push a single undoable command on commit.

References:

- `docs/adr/0024-undo-redo-and-edit-transactions.md`

## MVP 23 — Undo/Redo Integration Expansion (Hierarchy DnD)

Goal: extend the undo/redo boundary to a second “editor workflow” domain that users rely on constantly.

**Scope**

- Make Hierarchy drag & drop (reorder + reparent) undoable.
- Preserve selection and UI state deterministically across undo/redo.

**Non-goals**

- Multi-document histories (per-scene stacks) and advanced merge policies.

**Definition of Done**

- Reorder and reparent operations in the Hierarchy produce undoable commands.
- Undo/redo restores both the hierarchy structure and the selection.

References:

- `docs/adr/0024-undo-redo-and-edit-transactions.md`
- `docs/adr/0016-plugin-boundaries-and-panel-ownership.md`

## MVP 24 — Edit Transactions + Coalescing Policy (Continuous Edits)

Goal: formalize transaction boundaries and coalescing rules so continuous edits (dragging, scrubbing) do not spam
history and can be cancelled deterministically.

**Scope**

- Introduce a minimal “transaction” concept in the demo editor layer:
  - begin/update/commit/cancel,
  - coalesce strategy keyed by (tool + targets + property path).
- Apply it to at least one continuous edit:
  - viewport translate drag (coalesce intermediate updates).

**Non-goals**

- Persisting history to disk.

**Definition of Done**

- A long gizmo drag produces exactly one undo history entry.
- Cancel reliably restores the pre-drag state without creating history entries.

Status:

- Prototype implemented in `fret-demo` for viewport translate drag.

References:

- `docs/adr/0024-undo-redo-and-edit-transactions.md`
- `docs/adr/0049-viewport-tools-input-capture-and-overlays.md`

## MVP 25 — Viewport Tool Polish (Axis Constraints + Snapping Stub)

Goal: improve the “Unity-like” feel of viewport manipulation without committing to final 3D math contracts yet.

**Scope**

- Axis constraint selection (X/Y) for translate gizmo.
- Optional snapping stub (grid step) controlled by a modifier key.

**Non-goals**

- Full camera-space 3D manipulation math and advanced snapping rules.

**Definition of Done**

- Users can drag along one axis deterministically.
- Snapping is predictable and does not break undo/redo semantics.

Status:

- Prototype implemented in `fret-demo` (axis constraints + Shift snapping; active-axis highlight while interacting).

References:

- `docs/adr/0049-viewport-tools-input-capture-and-overlays.md`
- `docs/adr/0024-undo-redo-and-edit-transactions.md`

## Parking Lot (Explicitly Deferred)

- External OS drag & drop hover semantics on macOS/winit (see `docs/known-issues.md`).
- Code-editor-grade text widgets (virtualized layout + incremental shaping caches).

## Code Anchors

- Command registry + effects: `crates/fret-app/src/app.rs`
- Keymap + `when`: `crates/fret-app/src/keymap.rs`, `crates/fret-app/src/when_expr.rs`
- Command palette demo widget: `crates/fret-demo/src/command_palette.rs`
- Focus + routing: `crates/fret-ui/src/tree.rs`
- Overlay/multi-root: `crates/fret-ui/src/tree.rs`, `docs/adr/0011-overlays-and-multi-root.md`
- Context menu: `crates/fret-ui/src/widgets/context_menu.rs`
- Virtualized lists: `crates/fret-ui/src/widgets/virtual_list.rs`, `crates/fret-ui/src/widgets/tree_view.rs`
- Editor shell wiring: `crates/fret-demo/src/editor_shell.rs`
- Desktop runner: `crates/fret-runner-winit-wgpu/src/runner.rs`
