> Archived: this plan is kept for history; prefer `docs/roadmap.md` + `docs/shadcn-declarative-progress.md` for active work.

# MVP Plan (Historical Snapshot)


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- fret-ui-precision: (internal reference; no public upstream link)
- gpui-component: https://github.com/longbridge/gpui-component
- Makepad: https://github.com/makepad/makepad
- shadcn/ui: https://github.com/shadcn-ui/ui
- Zed: https://github.com/zed-industries/zed

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This document is a historical snapshot of an older execution plan. It is no longer maintained as an active plan.

Completed stage definitions are archived in `docs/archive/mvp-archive.md` to keep this file actionable.

## ADR Policy (How MVP Work Locks Contracts)

MVP work is allowed to prototype quickly, but changes that affect “hard-to-change” semantics must be recorded in ADRs.

- MVP items should link to the ADRs that define their contracts (input/focus, display list semantics, portability, persistence).
- If an MVP uncovers a wrong assumption, update the ADR before broadening usage (more widgets, more panels, more apps).
- Prefer updating an existing ADR over creating many micro-ADRs; new ADRs are for new contract surfaces.

## Current Workspace Status

- MVP 0–6: done (see `docs/archive/mvp-archive.md`)
  - MVP 5: Text MVP landed (single-line input + IME cursor-area loop)
  - MVP 6: Commands + keymap MVP landed (bind/route/persist; `when` gating)
- MVP 7: MVP done in demo (command palette overlay; shortcut display; menu model types added)
  - Keymap v2 sequences + pending bindings are prototype implemented (ADR 0043 / ADR 0021).
- Editor-scale list widgets: prototype implemented
  - `VirtualList` (virtualization + stable keys + multi-selection) (ADR 0042 / ADR 0047)
  - `TreeView` (hierarchy-style tree over `VirtualList`)
- Theme tokens: baseline typed tokens + extensible namespaced keys for component ecosystems (ADR 0032 / ADR 0050). (prototype implemented)
- Baseline typography: `metric.font.size` + theme-driven text defaults are implemented end-to-end (framework widgets and component sizing now derive from the theme base font size).
- MVP 12: MVP done in demo (context menu overlay + submenu + keyboard nav + focus restore)
- MVP 13: MVP done in demo (Hierarchy selection model → Inspector panel; selection invalidation keeps Inspector in sync)
- MVP 14: MVP done in demo (primitive inspector editing baseline)
- MVP 15: MVP done in demo (Hierarchy drag & drop: reorder + reparent)
- MVP 16: MVP done in demo (DockSpace hosts app-owned panel content via `DockPanelContentService`)
- MVP 17: prototype implemented in demo (property tree + editor registry + mixed values; bool/string/f32/vec3 editors; two-column inline rows + per-axis Vec3 scrubbing)
- MVP 18: prototype implemented in demo (ToolManager with capture + marquee select + pan/orbit drag interactions)
- MVP 19: prototype implemented in demo (viewport click-to-select + selection highlight overlay)
- MVP 20: prototype implemented in demo (translate gizmo stub: overlay + explicit drag phases; Q/W tool switching; Esc cancel rollback)
- MVP 21: prototype implemented in demo (dock drag hints + dock tab context menu actions; debounced layout persistence)
- MVP 22: prototype implemented in demo (undo/redo command stack; inspector edits + translate gizmo integrate)
- MVP 23: prototype implemented in demo (Hierarchy drag & drop emits undoable commands; undo/redo restores hierarchy + selection)
- MVP 24: prototype implemented in demo (edit transactions + coalescing; viewport translate drag produces a single undo entry)
- MVP 25: prototype implemented in demo (translate axis constraints + Shift snapping stub)
- MVP 26: prototype implemented in demo (viewport navigation: pan/orbit stub + wheel zoom)
- MVP 11 validation: prototype implemented in demo (multiline TextArea probe panel)
- MVP 27: prototype implemented in demo (rotate gizmo stub + undo/redo)
- MVP 28: prototype implemented in demo (engine render hook + camera-driven viewport background)
- MVP 29: prototype implemented in demo (viewport render target auto-resize + registry update)
- MVP 30: prototype implemented in demo (Scene/Game viewport roles: tool gating + context menu control)
- MVP 31: prototype implemented in demo (play mode stub: RAF scheduling + animated Game viewport)
- MVP 32: prototype implemented in runner (engine frame update: render target deltas + command buffers, applied before UI render)
- Example editor layer crate: `apps/fret-editor` extracted (inspector protocol + edit services + viewport tool state)
- MVP 33: prototype implemented in demo (Project panel + `.meta` GUIDs; rename/move keep GUID stable; OS file drop imports into `Assets/Imports`; internal drag move to folders)
- MVP 34: prototype implemented in demo (dock tab bar titles + hover/active chrome baseline; overflow scroll + close tab)
- MVP 35: prototype implemented (ImGui-style multi-window internal drag routing: screen-space pointer + hovered window; dock drag now uses `Event::InternalDrag` instead of cross-window `PointerEvent` broadcasting)
- MVP 36: prototype implemented in demo (internal drag of Project asset into Hierarchy creates a new entity and selects it; cross-window supported via `Event::InternalDrag`)
- MVP 37: prototype implemented in demo (internal drag of Project asset into Scene viewport creates a new entity at drop UV using the current viewport camera; cross-window supported via `Event::InternalDrag`)
- MVP 38: prototype implemented in demo (asset drop registry: unify Hierarchy + Scene viewport drops; `.scene` drop opens the current scene stub and updates UI chrome)
- MVP 39: prototype implemented in demo (scene document P0: open selected `.scene` + load into world/hierarchy; dirty marker `*` + save writes JSON v1)
- MVP 40: prototype implemented in demo (asset open registry: Project double-click/Enter routes through `asset.open_selected`; `.scene` opens Scene doc, text-like assets open Text Probe)
- MVP 41: prototype implemented in demo (unsaved changes guard: opening a new scene or closing a window with the Scene panel prompts Save/Don't Save/Cancel; winit close requests are routed through `Event::WindowCloseRequested`)
- MVP 42: prototype implemented in demo (scene workflow P0: New Scene + Save As; File menu and Cmd/Ctrl shortcuts)
- MVP 43: prototype implemented (portability gates): runtime platform capability matrix (ADR 0054) + begin removing desktop-only payload assumptions (ADR 0053).
- MVP 44: implemented (host boundary): `UiHost` in `fret-runtime`, `fret-ui` is host-generic (no `fret-app` dependency), and `fret-ui-app` preserves demo/editor ergonomics (ADR 0052).
- MVP 46: prototype implemented (framework capability): system cursor + pointer feedback boundary (resize handles, hover cursors, per-window cursor routing).
- Inspector + viewport tooling boundaries: drafted as Proposed ADRs
  - ADR 0048: Inspector property protocol + custom editor registry (example editor layer)
  - ADR 0049: Viewport tools (input capture + overlay rendering) (example editor layer)
- Next major refactor (planned): tighten `fret-ui` vs `fret-ui-kit` boundaries so Tailwind/shadcn sizing/variants stay component-owned (see MVP 48).

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

Status:

- Contract is locked (ADRs 0045/0046) and a validation probe is prototype implemented in `fret-demo` via `TextArea` (see `PanelKey` `core.text_probe`).

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

- MVP done in the demo harness (entrypoints evolve; see `apps/fret-demo/src/main.rs` and `apps/fret-demo/src/bin/*`).
  - Note: selection changes are now propagated via model observation + UI invalidation (ADR 0051, prototype implemented); global services are still manual until service revisions are standardized (P1).

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

- MVP done in the demo harness (entrypoints evolve; see `apps/fret-demo/src/main.rs` and `apps/fret-editor/src/inspector_edit.rs`).
  - Inspector rows are two-column with inline value cells; Vec3 is shown as three inline fields and supports Alt+drag per-axis scrubbing.

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

- Full workspace management UI (named layouts list, per-project presets, rename/delete, etc.).

**Definition of Done**

- Docking actions are visually guided and feel “product-like” (no invisible drop zones).
- Layout save is delayed during drags and still persists correctly at the end.

Status:

- Prototype implemented in `fret-demo`:
  - drop target hints shown while dragging dock tabs,
  - dock tab context menu actions (float + move left/right),
  - `File -> Layout` actions:
    - reset-to-default layout,
    - save/load a minimal “last” layout preset (`./.fret/layout-presets/last.json`),
  - debounced `layout.json` persistence after dock ops and window placement changes.

References:

- `docs/adr/0013-docking-ops-and-persistence.md`
- `docs/adr/0011-overlays-and-multi-root.md`

## MVP 34 — Dock Tab Bar Readability (Titles + Hover)

Goal: make docking feel less “prototype” by rendering panel titles in the dock chrome, with minimal hover/active
styling driven by theme tokens.

**Scope**

- Render dock tab titles (panel `DockPanel.title`) in the tab bar.
- Clip title drawing to tab bounds so long titles do not spill into adjacent tabs.
- Add a close affordance:
  - hover/active tab shows a close button,
  - close emits a `DockOp` (no ad-hoc graph mutation).
- Support overflow scrolling:
  - mouse wheel over the tab bar scrolls tabs horizontally when overflowed.
- Apply minimal chrome styling:
  - hover background,
  - active underline (accent).

**Non-goals**

- Tab drag-reorder polish.
- Advanced overflow UI (scroll buttons, pinned tabs, animations).
- Ellipsis/truncation rules for long titles (clipping is enough for now).

**Definition of Done**

- Dock tabs display readable titles across the demo panels.
- Hovering a tab provides visual feedback; active tab is clearly indicated.
- Behavior is stable across theme reload and DPI/scale-factor changes.

Status:

- Prototype implemented in `fret-ui` + `fret-demo`:
  - tab titles are rendered (with per-tab clipping),
  - hover/active styling + close button,
  - title text updates are re-prepared when `DockPanel.title` changes.

References:

- `docs/adr/0013-docking-ops-and-persistence.md`
- `docs/adr/0032-style-tokens-and-theme-resolution.md`
- `docs/adr/0050-theme-config-schema-and-baseline-tokens.md`

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
- `docs/adr/0016-plugin-and-panel-boundaries.md`

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

## MVP 26 — Viewport Camera Navigation (Pan/Orbit + Wheel Zoom)

Goal: make viewport navigation feel editor-grade so selection and gizmos can be exercised without fighting the view.

**Scope**

- Middle-drag pan (2D camera translation).
- Right-drag orbit stub (2D rotation).
- Alt+left-drag orbit (Unity-style navigation shortcut).
- Mouse wheel zoom around cursor.
- Persist camera state per viewport panel.

**Non-goals**

- Full 3D camera math and projection contracts.

**Definition of Done**

- Pan/orbit/zoom updates viewport overlays and picking consistently.
- Zoom uses “zoom around cursor” so the hovered point stays stable.
- Camera state survives app restart (file persistence).

Status:

- Prototype implemented in `fret-demo`.

References:

- `docs/adr/0049-viewport-tools-input-capture-and-overlays.md`

## MVP 27 — Rotate Gizmo Stub (Overlay + Drag Phases)

Goal: extend viewport tooling with a rotation interaction so the basic Unity Q/W/E workflow is validated.

**Scope**

- Add a Rotate tool mode and a rotate gizmo overlay around the selected entity.
- Drag-to-rotate updates `transform.rotation_y` and participates in undo/redo coalescing.
- Optional modifier snapping (Shift) for predictable steps.

**Non-goals**

- Full 3D camera-space rotation math.
- Axis/plane constraints beyond a single stub axis.

**Definition of Done**

- `E` switches to Rotate mode.
- Dragging the rotate gizmo updates the selected entity rotation and produces one undo entry.
- Cancel is deterministic (`Esc` rolls back without history).
- Rotation is computed from the cursor angle around the gizmo center (atan2) and supports hover highlight.

Status:

- Prototype implemented in `fret-demo`.

References:

- `docs/adr/0049-viewport-tools-input-capture-and-overlays.md`
- `docs/adr/0024-undo-redo-and-edit-transactions.md`

## MVP 28 — Engine Render Hook P0 (Recorded Commands + Central Submit)

Goal: validate the mainline engine integration contract (ADR 0038) by rendering viewport targets via recorded command
buffers that are submitted by the runner *before* UI sampling/presentation (ADR 0015).

**Scope**

- Add a wgpu-facing engine render hook in the desktop runner:
  - driver records `wgpu::CommandBuffer`s for a frame,
  - runner submits engine command buffers first, then the UI command buffer.
- Demo exercises the contract end-to-end:
  - a small “engine pass” renders a grid background into the Scene render target,
  - the grid is driven by the per-panel viewport camera (pan/orbit/zoom),
  - UI samples it via `ViewportSurface` in the same frame.

**Non-goals**

- Full engine render graph integration (multi-encoder graphs are supported, but the demo keeps a single pass).
- Render target resize/update deltas (tracked in ADR 0038 but can land later).

**Definition of Done**

- Moving the viewport camera updates the sampled viewport background (not only overlays/picking).
- No code outside the runner submits work directly to `wgpu::Queue` for frame-participating rendering.

Status:

- Prototype implemented in `fret-demo` + `fret-launch`.

References:

- `docs/adr/0038-engine-render-hook-and-submission-coordinator.md`
- `docs/adr/0015-frame-lifecycle-and-submission-order.md`

## MVP 29 — Viewport Target Lifecycle P0 (Auto-Resize + Registry Updates)

Goal: make embedded viewports “editor-grade” by ensuring render target sizes match the on-screen viewport content size
(no stretching blur) while keeping input mapping (uv/target_px) consistent.

**Scope**

- Track the on-screen viewport content rect for each active viewport target in `DockManager`.
- Engine hook uses the content rect + `scale_factor` to derive the desired physical pixel size.
- Demo resizes the scene render target when the panel size changes:
  - recreate the texture,
  - `renderer.update_render_target(...)` updates the registry view + size,
  - dock panels update `ViewportPanel.target_px_size` for consistent mapping.
- Resize policy uses a small “bucket” (e.g. 64px) to reduce reallocation churn during interactive dock resizing.

**Non-goals**

- Debounced resize policies (can land later; correctness first).
- Multi-sampled resolve targets.

**Definition of Done**

- Resizing the Scene dock panel keeps the viewport crisp (target matches panel pixel size).
- Viewport input mapping stays stable (picking/overlays do not drift after resize).

Status:

- Prototype implemented in `fret-demo` + `fret-ui`.

References:

- `docs/adr/0007-viewport-surfaces.md`
- `docs/adr/0038-engine-render-hook-and-submission-coordinator.md`

## MVP 30 — Scene/Game Viewport Roles P0 (Editor Tool Gating)

Goal: make multi-viewport editors feel Unity-like by establishing “roles” for viewports:

- Scene view is an editor viewport (selection/gizmos/overlays).
- Game view is a preview viewport (no editor gizmos/selection overlays by default).

**Scope**

- Demo defines a simple role policy keyed by `PanelKey` (`core.scene` vs `core.game`).
- Game view default policy:
  - editor overlays are suppressed,
  - editor selection/gizmo interactions are not started from viewport input,
  - viewport context menu is disabled so right-click can be reserved for navigation.

**Non-goals**

- Full play mode/runtime input integration (engine-level).
- Per-plugin viewport role registration.

**Definition of Done**

- Switching between Scene/Game tabs preserves independent cameras and render targets.
- Game view does not show editor selection/gizmos and does not change selection on click.

Status:

- Prototype implemented in `fret-demo` + `fret-ui` (`ViewportPanel.context_menu_enabled`).

## MVP 31 — Play Mode Stub P0 (Preview Loop + RAF Scheduling)

Goal: validate an editor-grade “preview loop” without committing the framework to continuous rendering by default.

**Scope**

- Add a demo command to toggle play mode.
- When play mode is enabled and the Game viewport is visible:
  - request animation frames (ADR 0034) to keep the preview updating,
  - render a simple time-based animation in the Game viewport background (no engine simulation yet).

**Non-goals**

- Full runtime input mapping and simulation stepping.
- Editor/engine state synchronization (app-specific).

**Definition of Done**

- Toggling play mode makes the Game viewport visibly animate while Scene remains static.
- Disabling play mode stops continuous redraw.

Status:

- Prototype implemented in `fret-demo` (`demo.play.toggle`).

## MVP 43 — Platform Capabilities P0 (Runtime Matrix + Gating)

Goal: make the wasm/WebGPU path a **contracted portability target** rather than a future rewrite by introducing a
runtime capability matrix that drives `when` gating and platform IO boundaries.

**Scope**

- Add a `PlatformCapabilities` data model at the platform boundary (ADR 0054).
- Thread capabilities into the command routing context so menus/palette/shortcuts can gate consistently (ADR 0022 / ADR 0023).
- Use capabilities to disable unsupported features rather than relying on ad-hoc `cfg` branches:
  - multi-window tear-off docking (ADR 0013 / ADR 0017),
  - external file drag payload shape (ADR 0053),
  - clipboard/text-only fallbacks.
- Expose capabilities in debug diagnostics (HUD/inspector) (ADR 0036).

**Non-goals**

- Shipping a web runner (this MVP only locks the portability contract and wires the data through).
- Solving all web permission/policy UX (future).

**Definition of Done**

- A single source of truth for platform feature availability exists at runtime (`PlatformCapabilities`).
- `when` expressions can gate on capability keys, and menus/palette reflect the same gating.
- Demo can simulate “single-window mode” by forcing capabilities and shows tear-off commands disabled.

Status:

- Prototype implemented in the runner + demo (capabilities threaded into `InputContext` and `when`).

References:

- `docs/adr/0054-platform-capabilities-and-portability-matrix.md`
- `docs/adr/0053-external-drag-payload-portability.md`
- Makepad’s wasm entrypoint message pump (reference posture, not an API dependency):
  - `repo-ref/makepad/platform/src/os/web/web.rs`

## MVP 44 — UiHost Boundary P0 (Embeddable UI Runtime)

Goal: keep the UI runtime embeddable for third-party engines/editors (GPUI-like adoption) without forcing them to adopt the full `fret-app` runtime.

**Scope**

- Introduce a minimal host trait used by `fret-ui` (`UiHost`):
  - globals, model store access, command registry access,
  - request redraw + push effects,
  - `tick_id`/`frame_id` and timer token allocation,
  - internal drag session access (for docking/DnD UX).
- Implement the trait for `fret_app::App` so the demo behavior remains unchanged.
- Keep `fret-ui` host-generic (no dependency on `fret-app`).
- Preserve demo/editor ergonomics via an integration convenience layer (`fret-ui-app`).

**Non-goals**

- A fully host-agnostic API surface (host-facing types may still be refined over time).
- A production-ready third-party runner/backend (this MVP locks the core trait boundary).

**Definition of Done**

- `cargo test --workspace` passes with no demo behavior changes required.
- `fret-ui` compiles without depending on `fret-app`.
- `UiHost` (and portable boundary value types) live in `fret-runtime`.
- `fret-demo` compiles without being forced into host generics everywhere (via `fret-ui-app`).

Status:

- Implemented:
  - `UiHost` moved to `fret-runtime`
  - `fret-app::App` implements `UiHost`
  - `fret-ui` is host-generic and no longer depends on `fret-app`
  - `fret-ui-app` provides `fret-app`-bound type aliases for demo/editor ergonomics

References:

- `docs/adr/0052-ui-host-runtime-boundary.md`

## MVP 45 — Component Primitives P0 (Token-Driven Recipes + Variants)

Goal: establish a **general-purpose** component library baseline (GPUI-component-like) that is:

- token-driven (ADR 0032 / ADR 0050),
- ergonomic for applications (not editor-only),
- extensible via namespaced theme keys and component variants.

**Scope**

- Define a minimal “primitives + recipes” set suitable for building real apps:
  - `Button`, `IconButton`, `Checkbox`, `TextField`, `Select` (subset is fine),
  - `Tabs` + `Toolbar` baseline (enough to replace ad-hoc chrome in the demo).
- Establish a consistent variants model (typed):
  - `size` (`sm/md/lg`), `intent` (`default/primary/danger`), `state` (hover/active/disabled).
- Drive visuals exclusively from theme tokens:
  - baseline tokens in `fret-ui`,
  - component-specific tokens via namespaced dotted keys (e.g. `component.button.*`).
- Validate with a small demo surface that uses the new primitives to ensure the API composes.

**Non-goals**

- CSS-like selector strings or a runtime “utility class” parser.
- Advanced effects (blur/glow/shadow) unless a renderer semantics ADR lands first.

**Definition of Done**

- A minimal component crate exists (may be incubated in-tree first, then extracted to a separate `fret-components` repo).
- The demo replaces at least one ad-hoc UI region with these primitives without losing behavior.
- Theme customization works by editing `theme.json` (no code changes needed for simple recolors/spacing tweaks).

Status:

  - Prototype implemented (incubated in this repo):
  - `ecosystem/fret-ui-kit`: token-driven components + Tailwind-like typed style refinements (`Button`, `IconButton`, `TextField`, `Select`, `Checkbox`, `Switch`, `Separator`, `Tabs`, `Toolbar`).
    - Tailwind-like primitive vocabulary is now explicit and reusable: `Space` + `Radius` (typed) backed by theme extension tokens (`component.space.*`, `component.radius.*`).
    - Component authoring ergonomics are now GPUI-component-like: `StyleRefinement` is a composable “style patch” and `StyledExt` provides `.styled().px_3().py_2().rounded_md()...` chains; any component can opt in by implementing `RefineStyle`.
    - Recipes (P0): component-level “recipes” provide shared, token-driven chrome contracts. Input-family controls (TextField/Select/Combobox/TextAreaField) share a single `resolve_input_chrome(...)` resolver that enforces a stable override priority (callsite refinement → component tokens → shared input tokens → size/baseline fallbacks).
    - List rows (P0): `recipes::list_row` provides a single, Tailwind-aligned `VirtualListStyle` + row height contract shared by `ListView`, `CommandList`, and the UI Kit rich `VirtualList` demo (multi-line rows).
    - Menu lists (P0): `recipes::menu_list` provides a shared row chrome contract for overlay menus (`ContextMenu`/`Popover`) to eliminate per-widget padding/row height magic numbers and keep menu sizing consistent.
    - Themes (P0 bridge): component recipes preferentially query gpui/shadcn semantic keys (e.g. `background`, `foreground`, `list.*`, `popover.*`) via `Theme::color_by_key`, relying on the framework alias layer (ADR 0050 §1.1) to keep existing Fret themes working.
    - `command_palette::install_command_palette` provides a one-call subtree install pattern (input + list + keyboard nav) for app ergonomics.
    - `ResizablePanelGroup` provides a component-level naming surface for the resizable split primitive (shadcn-style vocabulary).
    - `Combobox` provides a minimal typeahead + anchored list interaction (focus stays in input; list is `Popover`-backed).
    - `sonner::toast(...)` provides a shadcn-style facade for transient notifications.
  - Overlay “surfaces” have been migrated to `ecosystem/fret-ui-kit` as part of MVP 48 boundary tightening:
    - `ContextMenu`, `Popover`, `DialogOverlay`, `CommandPaletteOverlay`, `AppMenuBar`, `ToastOverlay`, `TooltipOverlay`.
    - The runtime still owns the overlay-layer mechanism (`UiTree` layers) and the menu request store (`ContextMenuService`).
  - `ecosystem/fret-icons`: renderer-agnostic icon registry + small builtin glyph fallback set.
  - `fret-demo`: adds a `components_gallery` panel (`PanelKey` `core.components_gallery`) to validate composition and theme-driven styling.
  - `fret-demo --bin components_gallery`: standalone shadcn gallery window (no docking/editor shell) to validate component ergonomics and overlays in isolation.

References:

- `docs/adr/0037-workspace-boundaries-and-components-repository.md`
- `docs/adr/0032-style-tokens-and-theme-resolution.md`
- `docs/adr/0050-theme-config-schema-and-baseline-tokens.md`
- `repo-ref/gpui-component/crates/ui`
- `repo-ref/fret-ui-precision/docs/`
  - `repo-ref/ui/apps/v4/registry/new-york-v4/ui/popover.tsx`

## MVP 46 — Pointer Feedback P0 (System Cursor + Resize Handles)

Goal: make “editor-grade pointer affordances” a **framework capability**, not ad-hoc per-widget behavior.

This closes a UX gap observed in the demo: resize handles and split dividers can be dragged, but the OS cursor
does not reflect the affordance (e.g. column/row resize) because there is no portable cursor API boundary yet.

**Scope**

- Add a portable “system cursor icon” contract to the UI→host boundary:
  - define a `CursorIcon` type (portable subset),
  - add an effect to request cursor changes per window (e.g. `Effect::CursorSetIcon { window, icon }`),
  - gate the feature via `PlatformCapabilities` (for future wasm/mobile).
- Define a deterministic cursor resolution model for retained UI:
  - a per-window “cursor intent” channel derived from hit-testing (topmost wins),
  - stable behavior across multiple roots/overlays (context menus, popovers, drag previews),
  - reset-to-default when leaving all cursor areas.
- Provide a general-purpose `ResizeHandle` primitive in the component ecosystem:
  - consistent hit target size + visual styling,
  - emits resize drag events and requests the correct cursor icon,
  - reusable by docking splits, resizable panels, data tables, etc.
- Adopt the primitive in `DockSpace` split handles (demo-visible validation).

**Non-goals**

- Pixel-perfect parity with every platform cursor set (desktop-first; wasm/mobile can be partial/no-op).
- Advanced “cursor regions” like text I-beam selection shaping beyond a portable enum.

**Definition of Done**

- Hovering a `DockSpace` split handle shows an OS resize cursor (col/row resize) and restores on exit.
- Dragging a split handle keeps the resize cursor during the drag, even across window boundaries.
- The behavior is deterministic with overlays:
  - context menus/popovers can override cursors when hovered,
  - drag preview windows do not “trap” the cursor resolution for docking back.
- The cursor API is documented and linked to the relevant ADRs (update, don’t add a micro-ADR unless needed):
  - `docs/adr/0001-app-effects.md`
  - `docs/adr/0003-platform-boundary.md`
  - `docs/adr/0054-platform-capabilities-and-portability-matrix.md`

Status:

- Prototype implemented (desktop runner + dock split handles; `TextInput`/`TextArea` and common component widgets set hover cursors; `fret_ui::ResizeHandle` draws a hairline divider with a thick hit target, without reserving layout gap).

References:

- `repo-ref/zed/crates/workspace/src/dock.rs` (resize handle as element + cursor)
- `repo-ref/gpui-component/crates/ui/src/resizable/resize_handle.rs` (cursor + occlude + drag behavior)
- `docs/adr/0011-overlays-and-multi-root.md`
- `docs/adr/0041-drag-and-drop-clipboard-and-cross-window-drag-sessions.md`

## MVP 47 — Component Size/Density System P0 (Tailwind-like Scales, GPUI-Inspired)

Goal: stop doing per-widget spacing/height fixes by locking a **component-level sizing contract** that
matches the ergonomics of **gpui-component** and the vocabulary of **Tailwind/shadcn**.

This MVP is specifically about producing a *clean, reusable* component ecosystem foundation (for third-party
apps), not about editor-only UI.

**Scope**

- Introduce a component-level `Size` (xs/sm/md/lg) and a small set of derived “control metrics”:
  - `input_h`, `input_px/py`, `list_px/py`, `row_gap_y`, `icon_size`, etc.
- Provide a unified sizing API for components (GPUI-style):
  - a `Sizable` trait (or equivalent) so `Button/TextField/ListView/Select/...` share the same `.with_size(...)`.
  - a `StyleSized` helper surface (or equivalent) so “tailwind-like primitives” can be applied consistently
    (e.g. `.list_px(size)`, `.input_h(size)`), similar to `repo-ref/gpui-component/crates/ui/src/styled.rs`.
- Tie the sizing system to theme tokens:
  - baseline metrics come from typed tokens (ADR 0050),
  - component ecosystems can override via namespaced dotted keys (ADR 0050 §5.1),
  - list-specific tokens (e.g. `metric.list.*`) remain valid, but are consumed through the sizing layer.
- Migrate existing `ecosystem/fret-ui-kit` components to the new sizing system and remove ad-hoc per-component
  “magic numbers” where possible.

**Non-goals**

- A CSS utility-class parser or runtime Tailwind interpreter.
- Committing to advanced renderer-visible effects (shadow/blur/glow) unless a shape semantics ADR update lands first
  (ADR 0030).

**Definition of Done**

- `Button`, `TextField`, `Select`, and list-like components (`ListView`/`ScrollArea` wrappers) support `Size` and
  render with consistent paddings/heights across the UI kit.
- Size/density tuning happens in one place (the sizing layer), not by patching individual widgets.
- The UI Kit demo includes a small “size matrix” surface to validate the contract visually.

Status:

- Prototype implemented:
  - `ecosystem/fret-ui-kit/src/sizing.rs` defines `Size` + `Sizable`.
    - `Size::control_text_px` now derives from the theme’s base typography (`metric.font.size`, alias `font.size`) by default, so a theme can scale the entire component ecosystem consistently.
  - Core UI kit components adopt `.with_size(...)` and derive control metrics from `Size`.
  - `fret-ui::VirtualList` exposes `set_style` / `set_row_height` for size-aware list wrappers.
  - List-like components share a single list style mapping to avoid “per-widget spacing patching”.
  - `fret-demo --bin components_gallery` includes a small size matrix surface.

References:

- `docs/adr/0056-component-size-and-density-system.md`
- `repo-ref/gpui-component/crates/ui/src/styled.rs` (search `Size`, `StyleSized`, `list_px/list_py`, `input_h`)

## MVP 48 — Runtime/Components Boundary Tightening (shadcn-ready)

Goal: make the **Tailwind/shadcn sizing + variants system** (ADR 0056) the single source of truth by
removing “UI kit opinions” from the runtime crate (`fret-ui`) and concentrating shadcn-like surfaces
in the component crate (`ecosystem/fret-ui-kit`), closer to the GPUI vs gpui-component split.

This is a deliberate, “no fear” refactor MVP to prevent slow drift and perpetual per-widget patches.

**Scope**

- Clarify and enforce the boundary (ADR 0037):
  - `fret-ui`: runtime substrate (tree, routing, focus/capture, layers, docking, perf primitives like `VirtualList`/`Scroll`).
  - `ecosystem/fret-ui-kit`: shadcn-like surfaces and policies (popover/dialog/menu/tooltip/toast/command palette/menubar),
    and all sizing/variants/token recipes.
- Remove hard-coded control heights/spacing from runtime primitives:
  - `TextInput`/`TextArea` stop deciding “control height”; component wrappers (`TextField`, etc.) own the chrome sizing via `Size`.
- Migrate “standard overlays kit” out of `fret-ui`:
  - move `WindowOverlays` and overlay widgets/services out of runtime, or re-export them only from the component crate,
    while keeping the **overlay layer mechanism** in `fret-ui`.
- Token drift mitigation:
  - document and implement a single fallback rule so `Space`/`Radius` can safely fall back to baseline `metric.*`
    tokens when `component.*` is not provided (avoid theme value duplication drift).

**Non-goals**

- A runtime Tailwind class parser.
- Major visual redesign or new effects (shadow/blur/glow); those require renderer semantics decisions.

**Definition of Done**

- `fret-demo` and `fret-demo --bin components_gallery` no longer depend on `fret_ui::WindowOverlays` or runtime overlay widgets directly;
  they use `ecosystem/fret-ui-kit` surfaces instead.
- Runtime primitives have no “opinionated” shadcn sizing baked in (notably `TextInput` height).
- `cargo test --workspace` passes and the UI kit still works.

Status:

- MVP done (with a remaining follow-up tracked by MVP 51):
  - `WindowOverlays` moved from `fret-ui` into `ecosystem/fret-ui-kit` to keep overlay policy/component ergonomics out of the runtime crate.
  - `TextInput` no longer hard-codes control height; `Size` (ADR 0056) stays component-owned.
  - Overlay widgets moved from `fret-ui` into `ecosystem/fret-ui-kit` (context menu, popover, dialog, command palette shell, menubar, toast, tooltip).
  - `EventCx::open_context_menu*` centralizes “open menu” wiring so runtime widgets (e.g. docking) don’t depend on component surfaces.
  - Token drift mitigation: `Space` and `Radius` fall back to baseline `metric.*` when `component.*` is missing.
  - Follow-up: some UI-kit-shaped runtime widgets still exist (e.g. `TreeView`); migrating them cleanly depends on landing the declarative authoring model (MVP 49/50).

References:

- `docs/adr/0037-workspace-boundaries-and-components-repository.md`
- `docs/adr/0056-component-size-and-density-system.md`
- `repo-ref/gpui-component/crates/ui` (styled + component split)

## Parking Lot (Explicitly Deferred)

- External OS drag & drop hover semantics on macOS/winit (see `docs/known-issues.md`).
- Code-editor-grade text widgets (virtualized layout + incremental shaping caches).

## Code Anchors

- Command registry + effects: `crates/fret-app/src/app.rs`
- Keymap + `when`: `crates/fret-app/src/keymap.rs`, `crates/fret-app/src/when_expr.rs`
- Command palette surface: `ecosystem/fret-ui-shadcn/src/command.rs` (demo: `apps/fret-demo/src/bin/components_gallery.rs`)
- Focus + routing: `crates/fret-ui/src/tree/mod.rs`
- Overlay/multi-root: `crates/fret-ui/src/tree/mod.rs`, `docs/adr/0011-overlays-and-multi-root.md`
- Context menu: `ecosystem/fret-ui-kit/src/primitives/context_menu.rs`
- Virtualized lists: `crates/fret-ui/src/virtual_list.rs` (tree wrapper: `ecosystem/fret-ui-kit/src/declarative/tree.rs`)
- Demo shell wiring: `apps/fret-demo/src/main.rs`
- Desktop runner: `crates/fret-launch/src/runner/mod.rs`
