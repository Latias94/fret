# Node Graph Interaction Checklist (fret-node)

This checklist is a living conformance target for the node graph editor UI.

It contains:

1) a manual verification script (repro steps),
2) invariants that should eventually become automated conformance tests.

Roadmap / TODO tracker:

- `docs/node-graph-roadmap.md`

Authoritative contracts:

- `docs/adr/0135-node-graph-editor-and-typed-connections.md`

## Status Legend

- `(implemented)` present in code and expected to be stable.
- `(prototype)` present but not stabilized; behavior may change.
- `(todo)` not implemented yet; this checklist defines intended behavior.

## Test Harness

Recommended harness:

- `apps/fret-examples/src/node_graph_demo.rs` (use the tuning overlay to spawn stress graphs)

## A) Pan/Zoom and View Ops

### Manual script

- (prototype) Scroll wheel zooms the canvas without moving focus unexpectedly.
- (prototype) Pan/zoom wheel gating (if enabled by interaction config):
  - When `zoom_on_scroll=true` and `zoom_activation_key=ctrl_or_meta`, wheel zooms only while Ctrl/Meta is held.
  - When `pan_on_scroll=true`, wheel pans when zoom is not active.
- (prototype) Middle-drag or right-drag pans (whatever the demo binds; document the binding).
- (prototype) Pan inertia (if enabled): after releasing the pan gesture, the viewport continues with exponential decay and stops at extents.
- (prototype) “Fit view” command frames all nodes with padding.
- (prototype) “Fit selection” frames selected nodes with padding.
- (prototype) “Reset view” restores canonical pan/zoom.

### Invariants

- (prototype) `min_zoom`/`max_zoom` are enforced (`NodeGraphStyle`).
- (todo) All view ops (controls, minimap, commands) must route through the same canonical view-state mutations.

## B) Selection and Marquee

### Manual script

- (implemented) Click node selects it.
- (implemented) Click empty space clears selection.
- (prototype) Selection gating: when `elements_selectable=false`, pointer selection and marquee do nothing.
- (prototype) Shift-click adds to selection (no drag).
- (prototype) Ctrl/Cmd-click toggles selection (no drag).
- (implemented) Drag marquee selects nodes inside the rect.
  - Zoom to 0.5 and 2.0 and verify marquee threshold behavior is consistent (no “zoom makes it too sensitive”).

### Invariants

- (implemented) Selection changes are deterministic given the same pointer inputs.
- (done) Keyboard nudge and selection commands must not depend on frame timing.

## C) Node Drag, Group Drag, Snaplines

### Manual script

- (implemented) Dragging a selected node moves it.
- (prototype) Drag handle gating:
  - When `node_drag_handle_mode = header`, dragging starts only from the header area.
  - Dragging from the body does not start a node drag (but selection still works).
- (prototype) Dragging multiple selected nodes preserves relative offsets.
- (implemented) Snaplines appear and apply predictable snapping deltas.
- (prototype) Drag thresholds are screen-space:
  - Set zoom to 0.5 and try a tiny move: does not start a drag.
  - Set zoom to 2.0 and try the same tiny move: does not start a drag.
  - Increase `node_drag_threshold` in the demo tuning overlay and verify it scales in screen px (not canvas px).

### Invariants

- (implemented) Node positions are graph semantics (stored in `Graph`, not internals).
- (prototype) Snapline computation does not mutate graph until the drag is committed.

## D) Connect / Reconnect (XYHandle parity)

### Manual script

- (prototype) Start a wire drag from a port handle.
- (prototype) Hover a compatible target port and release:
  - a connection is created (or reconnection performed),
  - `GraphTransaction` is committed (undoable).
- (prototype) Connect-on-click (if enabled):
  - click a port handle to start a connection preview,
  - click another handle to attempt the connection,
  - the click-connect session ends regardless of validity (no “stuck connecting” state).
- (prototype) Keyboard click-connect:
  - focus a port handle (or hover it),
  - press `Enter` to start a connection preview,
  - focus/hover another handle and press `Enter` to attempt the connection,
  - the session ends regardless of validity.
- (prototype) Keyboard port navigation:
  - `[` / `]` cycles ports on the focused node.
  - `Alt+Arrow` moves to the nearest port in the direction (may change focused node).
- (prototype) Toggle connection mode (strict/loose) and verify targeting behavior changes.
- (prototype) Release on empty space:
  - a menu/searcher appears,
  - selecting an item either inserts a node and connects, or cancels cleanly.

Multi-connection bundle:

- (prototype) Hold Shift and hover another same-side pin to add it into the bundle.
- (prototype) Hold Ctrl/Cmd on drag start to yank existing incident edges into the bundle (if supported).

### Invariants

- (implemented) Final accept/reject is mediated by rules (`ConnectPlan`) not by direct edge mutation.
- (implemented) Single-capacity input ports disconnect existing edges when connecting a new one (per rules).
- (prototype) Reconnect preserves `EdgeId` when possible (per rules).

Strict/Loose resolution:

- (prototype) `connection_mode`:
  - `Strict`: only connect when pointer is over a compatible handle.
  - `Loose`: allow snapping to a compatible handle within `connection_radius`.
- (prototype) When multiple handles are within range, selection is deterministic:
  - closest distance first,
  - stable id ordering as tie-break.

Auto-pan:

- (prototype) While connecting, if pointer is near the canvas edge, the canvas auto-pans.
- (prototype) Auto-pan speed and edge threshold are configurable.

Drag threshold:

- (prototype) A small movement threshold prevents accidental “start connection” on click.
- (prototype) Connection drag threshold is screen-space:
  - Zoom to 0.5 and 2.0 and verify the connection-start threshold feels the same.
- (prototype) Edge reconnect drag threshold is screen-space and uses the same threshold as connection start.

## E) Portal Editors (embedded node UI)

### E1) Text editor (prototype)

- (prototype) Submit/cancel commands:
  - submit commits a `GraphTransaction` if domain accepts;
  - cancel resets to the graph-derived value.
- (prototype) Inline errors show without committing.
- (prototype) Stepper buttons apply fine/normal/coarse stepping based on modifiers.

### E2) Number editor (prototype)

Manual script:

- (prototype) Typing a number and submitting commits once and updates the node value.
- (prototype) Clicking `+/-` steps (mode depends on modifiers).
- (prototype) Dragging the `<>` handle:
  - does not start until `drag_threshold_px` is exceeded,
  - uses a mode captured on pointer down (Shift/Ctrl/Cmd),
  - updates the input buffer during drag (preview),
  - commits once on pointer up (undoable).

Invariants:

- (prototype) Portal widgets never mutate graph directly; they emit commands and the handler decides.
- (prototype) Drag sessions produce one commit (undo granularity).
- (prototype) Portal-driven auto-sizing is stable under pan/zoom (no “shrink when near viewport edge” wobble).

## F) Derived Geometry and Internals (ReactFlow internals parity)

See `docs/workstreams/fret-node-internals-m0.md` for the detailed contract checklist and the
refactor-safe conformance suite pointers.

### Invariants

- (implemented) Derived geometry stores are not serialized into graph assets.
- (prototype) Derived caches are invalidated deterministically on:
  - node size change,
  - zoom change,
  - presenter/template change.

## L) Accessibility / Semantics (prototype)

### Manual script

- (prototype) Assistive tech sees the canvas as a `Viewport` with a stable label.
- (prototype) The canvas exposes a value string that reflects:
  - zoom,
  - selection counts,
  - focused node/port/edge (when present),
  - whether a connection drag is active.
- (prototype) Optional active-descendant semantics:
  - when the canvas mounts three semantics-only children (`NodeGraphA11yFocusedPort`, `NodeGraphA11yFocusedEdge`, `NodeGraphA11yFocusedNode`) in this order,
    the canvas sets `active_descendant` to the corresponding child as focus changes.

## G) Clipboard (prototype)

### Manual script

- (prototype) Copy selection puts a payload on the system clipboard.
- (prototype) Paste creates nodes near the cursor (or last known canvas position).
- (implemented) Copy selection creates a deterministic `GraphFragment` payload.
- (implemented) Paste uses a deterministic offset strategy when repeatedly pasting at the same anchor.
- (implemented) Paste is undoable as one transaction.
- (prototype) Copy selection includes selected groups (and their child nodes) when groups are selected.

## H) Minimap and Controls (todo)

### Manual script

- (prototype) Minimap renders derived node rects + viewport rect.
- (prototype) Minimap click-to-pan and drag-to-move-viewport work.
- (prototype) Controls provide zoom in/out and fit/reset.

### Invariants

- (todo) Minimap consumes derived geometry only; no graph semantics are stored in minimap state.
- (todo) All navigation flows route through canonical view ops.

## I) Edge Rendering and Interaction (todo)

### Manual script

- (todo) Edge hit-testing uses a larger interaction width than the visual stroke width.
- (todo) Hovering an edge highlights it (without changing selection).
- (todo) Selecting an edge is deterministic and undo/redo does not affect selection state unexpectedly.
- (done) Keyboard edge focus: `Ctrl/Cmd+Tab` cycles focus/selection across edges (when enabled).
- (done) Keyboard node focus: `Tab` / `Shift+Tab` cycles focus/selection across nodes.
- (done) Keyboard port focus: `[` / `]` cycles port focus within the focused/selected node.
- (prototype) Focus anchors: focused/selected edge shows endpoint anchors (reconnect affordance).
- (prototype) Anchor reconnect: click an edge endpoint anchor and drag to reconnect.
- (prototype) Anchor hover: hovering an anchor highlights it and uses pointer cursor.
- (prototype) Anchor click without drag selects the edge (does not start reconnect).
- (prototype) Edge labels render with stable placement (and do not overlap node ports in common layouts).
- (prototype) Edge end markers (arrows) render and remain stable across zoom levels.
- (todo) Per-edge style overrides can affect color/width/label without breaking hit-testing.

### Invariants

- (todo) Edge styling does not affect graph semantics; it is derived from graph data + style/theme.
- (todo) Edge label layout is deterministic given the same inputs (node rects, ports, zoom, style).

## J) Productivity Commands (todo)

### Manual script

- (todo) `Frame all` frames all nodes with padding (same view op as “Fit view”).
- (done) Keyboard nudge moves selection by a fixed screen-space step (Shift = fast) and optionally snaps to grid.
- (done) Selection align/distribute commands exist and commit a single undo step per invocation.
- (todo) Align/distribute commands operate on selection deterministically.

### Invariants

- (todo) Command handlers commit at most one `GraphTransaction` per user-intent (undo granularity).
- (todo) Productivity commands do not depend on frame timing.

## K) Performance and Large Graphs (todo)

### Manual script

- (todo) A 5k–20k node graph remains usable (target frame time TBD).
- (prototype) Offscreen nodes do not mount portal subtrees to avoid UI tree blowup (portal culling).
- (prototype) Offscreen nodes/edges do not emit canvas paint ops to avoid scene blowup (canvas culling).

### Invariants

- (todo) Derived caches are invalidated by precise keys (layout/zoom/theme changes) and avoid full recompute when possible.
- (todo) Rendering and hit-testing support culling (viewport-based) without changing graph semantics.
