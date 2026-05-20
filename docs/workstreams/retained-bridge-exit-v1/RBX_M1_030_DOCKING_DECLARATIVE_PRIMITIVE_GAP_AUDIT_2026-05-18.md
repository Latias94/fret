# RBX-M1-030 Docking Declarative Primitive Gap Audit

Date: 2026-05-18
Status: Done
Workstream: `retained-bridge-exit-v1`
Task: `RBX-M1-030`

## Purpose

Identify the minimum declarative primitives or runtime seams that docking still needs before
`DockSpace` can stop being authored through the retained bridge.

This is an audit slice only. It does not remove `DockSpace`, `RetainedSubtreeProps`, or the public
retained docking helpers.

## Conclusion

Docking is not blocked by panel content authoring. Panel UI roots already have a declarative path:

- `ecosystem/fret-docking/src/dock/panel_registry.rs:21` renders a dock panel root through
  `declarative::render_root(...)`.
- `ecosystem/fret-docking/src/dock/panel_registry.rs:32` wraps panel content in `ViewCache`.
- `ecosystem/fret-docking/src/dock/panel_registry.rs:200` defines `DockPanelRegistry` as the
  app-owned panel UI seam.
- `ecosystem/fret-docking/src/dock/panel_registry.rs:299` renders and binds panel roots every frame.
- `ecosystem/fret-docking/src/dock/panel_registry.rs:350` attaches those roots as `DockSpace`
  children.

The blocker is the host surface. `DockSpace` is still a retained, policy-heavy managed surface that
combines long-lived interaction state, raw event arbitration, child-root placement, prepaint
liveness, command routing, custom chrome painting, and controlled child painting order.

The next implementation step should therefore not be "rewrite DockSpace as elements" in one jump.
First split `DockSpace` into a reusable controller/state machine plus retained adapter. Then either
reuse existing declarative primitives or add one narrow mechanism-level managed-surface primitive
only for the remaining lifecycle gaps.

## Current Retained Entry Points

Public surface still exposes retained docking construction:

- `ecosystem/fret-docking/src/lib.rs:16` re-exports `DockSpace`, `DockSpaceMount`, and retained
  mounting helpers.
- `ecosystem/fret-docking/src/dock/mod.rs:203` creates a retained `DockSpace` node.
- `ecosystem/fret-docking/src/dock/mod.rs:215` creates a retained `DockSpace` node with a test id.
- `ecosystem/fret-docking/src/dock/mod.rs:234` mounts the retained dock space as a root.

The imui adapter still embeds docking by creating a retained subtree:

- `ecosystem/fret-docking/src/imui.rs:72` builds `RetainedSubtreeProps`.
- `ecosystem/fret-docking/src/imui.rs:74` builds `RetainedSubtreeFactory`.
- `ecosystem/fret-docking/src/imui.rs:101` starts the retained dock host construction path.
- `ecosystem/fret-docking/src/imui.rs:109` creates the retained `DockSpace`.
- `ecosystem/fret-docking/src/imui.rs:112` creates the retained `DockHostRoot`.
- `ecosystem/fret-docking/src/imui.rs:152` runs per-frame configure + panel binding during retained
  layout.
- `ecosystem/fret-docking/src/imui.rs:195` paints the retained dock space child.

`DockSpace` itself remains the central retained widget:

- `ecosystem/fret-docking/src/dock/space.rs:310` stores cross-frame docking host state directly on
  `DockSpace`.
- `ecosystem/fret-docking/src/dock/space.rs:319` through `:329` include active splitter,
  floating-drag, pending drag, panel content, size, and viewport capture state.
- `ecosystem/fret-docking/src/dock/space.rs:330` through `:372` include tab title caches,
  hover/press/menu/scroll state, glyph/SVG resources, theme revisions, and active tab state.
- `ecosystem/fret-docking/src/dock/space.rs:2565` implements `Widget<H> for DockSpace`.

## What Existing Declarative Primitives Already Cover

`crates/fret-ui` already has several mechanism-level pieces that a declarative docking host should
reuse where possible:

- `PointerRegionProps` supports mechanism-only pointer listening and capture-phase pointer moves
  (`crates/fret-ui/src/element.rs:325`).
- `InternalDragRegionProps` supports internal drag listener regions and an optional route kind
  (`crates/fret-ui/src/element.rs:376`).
- `SemanticsProps` covers role/test-id/focusable/relationship metadata
  (`crates/fret-ui/src/element.rs:1123`).
- `LayoutQueryRegionProps` records queryable bounds snapshots
  (`crates/fret-ui/src/element.rs:1247`).
- `ViewCacheProps` provides an experimental declarative cache boundary
  (`crates/fret-ui/src/element.rs:1455`).
- `PressableProps` covers normal button-like activation/focus behavior
  (`crates/fret-ui/src/element.rs:1611`).
- `ViewportSurfaceProps` and `CanvasProps` cover viewport and custom leaf painting use cases
  (`crates/fret-ui/src/element.rs:2262`, `crates/fret-ui/src/element.rs:2283`).

These are useful building blocks, but they do not yet replace `DockSpace` as a whole.

## Missing Or Insufficient Capabilities

### 1. Externalized Dock Host Controller State

Declarative authoring wants stable identity with state outside the rebuilt element tree. Today the
state is embedded in `DockSpace` as retained widget fields:

- drag/session state: `divider_drag`, `split_fraction_motion`, `floating_drag`,
  `pending_dock_drags`, `pending_dock_tabs_drags`
- panel/viewport state: `panel_content`, `panel_last_sizes`, `viewport_capture`
- chrome state: hover/press/menu/scroll/title/glyph caches

Next requirement:

- Extract a `DockSpaceController` or equivalent docking-owned state object that can be driven by a
  retained adapter now and a declarative adapter later.

### 2. Declarative Child-Root Placement For Managed Panel Content

`DockSpace` does not simply lay out normal children through the layout engine. It computes docking
geometry and then places each rendered panel root at a selected rect:

- `ecosystem/fret-docking/src/dock/space.rs:6932` owns the retained layout pass.
- `ecosystem/fret-docking/src/dock/space.rs:7123` computes active/floating dock layout maps.
- `ecosystem/fret-docking/src/dock/space.rs:7205` gathers bound panel roots.
- `ecosystem/fret-docking/src/dock/space.rs:7220` calls `layout_viewport_root(...)` for each active
  panel root.
- `crates/fret-ui/src/widget.rs:713` shows `layout_viewport_root(...)` also registers viewport root
  bounds and element root bounds.

Next requirement:

- Either expose a declarative mechanism that can position an arbitrary set of child roots from a
  host-computed layout map, or extract docking so an existing declarative composition can express the
  same root placement without reaching for `LayoutCx::layout_viewport_root`.

### 3. Prepaint Liveness Under View-Cache Reuse

Docking must refresh drag routes and diagnostics even when no event occurred and even when cache
replay could skip painting:

- `ecosystem/fret-docking/src/dock/space.rs:2609` starts the retained prepaint hook.
- `ecosystem/fret-docking/src/dock/space.rs:2615` and `:2621` refresh internal drag routes for dock
  panel and dock-tabs drags.
- `ecosystem/fret-docking/src/dock/space.rs:2631` records diagnostics in prepaint so scripts can
  rely on snapshots even when paint is replay-cached.
- `ecosystem/fret-docking/src/dock/space.rs:2791` through `:2810` keeps paint/layout invalidated
  while drag/capture/split motion needs frame progression.
- `crates/fret-ui/src/widget.rs:923` defines the retained `PrepaintCx` lifecycle.
- `crates/fret-ui/src/widget.rs:1035` supports prepaint animation-frame requests.

`InternalDragRegionProps.route_kind` is useful, but docking currently refreshes multiple route kinds
and ties liveness to diagnostics and animation invalidation. A declarative host must preserve those
outcomes explicitly.

### 4. Raw Event Arbitration With Capture, Focus, Cursor, Effects, And Propagation Control

The retained event path is a large state machine, not a simple pressable:

- `ecosystem/fret-docking/src/dock/space.rs:2813` starts `DockSpace::event(...)`.
- `ecosystem/fret-docking/src/dock/space.rs:6787` captures or releases the pointer.
- `ecosystem/fret-docking/src/dock/space.rs:6884` requests focus.
- `ecosystem/fret-docking/src/dock/space.rs:6888` sets cursor icons.
- `ecosystem/fret-docking/src/dock/space.rs:6901` pushes runtime effects.
- `ecosystem/fret-docking/src/dock/space.rs:6904` stops propagation.
- `crates/fret-ui/src/widget.rs:290` through `:309` expose the retained focus/capture/propagation
  controls used by the widget.
- `crates/fret-ui/src/widget.rs:358` exposes cursor icon requests.

Next requirement:

- Keep event policy in `fret-docking`, but provide a declarative adapter path that can drive the same
  controller decisions and emit the same runtime actions without embedding the policy in `fret-ui`.

### 5. Custom Chrome Painting With Controlled Child Paint Order

`DockSpace` paints dock chrome, floating chrome, split handles, drop hints, overlays, drag ghosts, and
then paints active panel child roots in controlled order:

- `ecosystem/fret-docking/src/dock/space.rs:7234` starts `DockSpace::paint(...)`.
- `ecosystem/fret-docking/src/dock/space.rs:7682` paints the root dock chrome.
- `ecosystem/fret-docking/src/dock/space.rs:7804` paints floating dock chrome.
- `ecosystem/fret-docking/src/dock/space.rs:7833` paints split handles.
- `ecosystem/fret-docking/src/dock/space.rs:7888` paints drop hints.
- `ecosystem/fret-docking/src/dock/space.rs:8006` paints drop overlays.
- `ecosystem/fret-docking/src/dock/space.rs:8022` paints the drag payload ghost.
- `ecosystem/fret-docking/src/dock/space.rs:8034` through `:8055` resolves panel nodes and calls
  `cx.paint(...)` for each panel.
- `crates/fret-ui/src/widget.rs:1340` shows retained widgets can imperatively paint a child node at a
  selected rect.

`Canvas` is a declarative leaf paint primitive. It does not by itself provide "paint custom chrome,
then paint these arbitrary child roots" semantics.

### 6. Command Routing And Dock Focus Requests

Docking still needs command participation:

- `ecosystem/fret-docking/src/dock/space.rs:6908` handles retained commands.
- `ecosystem/fret-docking/src/dock/space.rs:6910` handles `dock.focus_requested_panel`.
- `ecosystem/fret-docking/src/dock/space.rs:6920` focuses the requested panel node.
- `ecosystem/fret-docking/src/dock/space.rs:6925` stops command propagation.

Next requirement:

- A declarative docking host must have a command routing/focus request seam, or command handling must
  move into a docking runtime service that can target declarative element/node identity.

## Recommended Next Slices

### RBX-M1-040 Extract `DockSpaceController`

Scope:

- Extract `DockSpace`'s cross-frame state into a docking-owned controller/state type.
- Keep the retained `Widget` adapter as the only caller.
- Do not change public docking authoring APIs in this slice.

Done when:

- `DockSpace` no longer directly owns the bulky policy state fields, or it owns a single controller
  that owns them.
- Existing docking tests still pass.
- The retained adapter delegates event/layout/prepaint/paint decisions to controller methods where
  practical.

Gates:

- `cargo fmt --check`
- `cargo nextest run -p fret-docking`
- `python3 tools/check_layering.py`

### RBX-M1-050 Extract Layout/Paint Snapshots

Scope:

- Extract a `DockSpaceLayoutSnapshot` / `DockSpaceFrame` that contains root layout, floating layouts,
  active panel bounds, viewport layouts, hover/drop-hint context, and paint inputs.
- Reduce layout/paint recomputation before attempting a declarative adapter.

Done when:

- Layout produces a reusable snapshot consumed by paint.
- The snapshot has focused unit coverage for panel bounds and floating layout ordering.

Gates:

- `cargo nextest run -p fret-docking`
- targeted snapshot/unit tests for split, floating, viewport, and drop-hint cases
- `python3 tools/check_layering.py`

### RBX-M1-060 Decide The Declarative Host Mechanism

Scope:

- Try to express docking host behavior with existing primitives first:
  `InternalDragRegion`, `PointerRegion`, `LayoutQueryRegion`, `ViewCache`, `Canvas`, and
  `ViewportSurface`.
- If that is insufficient, add one narrow mechanism-level managed-surface primitive in `fret-ui` that
  exposes lifecycle hooks without owning docking policy.

Done when:

- The decision is recorded with code evidence.
- If a new primitive is added, its API is mechanism-only and has no docking-specific policy.
- A proof-of-life declarative docking host can mount and place at least one panel root.

Gates:

- `cargo nextest run -p fret-ui -p fret-docking`
- `python3 tools/check_layering.py`
- a small docking diagnostics or layout-sidecar proof for panel root placement

### RBX-M1-070 Replace Public Retained Docking Entry Points

Scope:

- Add the declarative docking host API.
- Migrate `imui.rs`, first-party demos, and tests to the declarative host.
- Delete `create_dock_space_node`, `mount_dock_space`, and `RetainedSubtreeProps` usage from
  `fret-docking` when no users remain.

Done when:

- `ecosystem/fret-docking` no longer depends on `fret-ui/unstable-retained-bridge`.
- `tools/check_layering.py` removes `fret-docking` from the retained bridge allowlist.
- Docking drag, tear-off, viewport, focus, and diagnostics gates remain green.

## Evidence Commands

Commands used for this audit:

- `python3 tools/audit_crate.py --crate fret-docking`
- `rg -n "retained_bridge|UiTreeRetainedExt|create_node_retained|RetainedSubtree|impl<.*Widget|impl Widget|Widget<" ecosystem/fret-docking/src ecosystem/fret-docking/tests -g '*.rs'`
- source reads of:
  - `ecosystem/fret-docking/src/lib.rs`
  - `ecosystem/fret-docking/src/dock/mod.rs`
  - `ecosystem/fret-docking/src/dock/panel_registry.rs`
  - `ecosystem/fret-docking/src/imui.rs`
  - `ecosystem/fret-docking/src/dock/space.rs`
  - `crates/fret-ui/src/element.rs`
  - `crates/fret-ui/src/widget.rs`

Validation run for the documentation update is recorded in `EVIDENCE_AND_GATES.md`.
