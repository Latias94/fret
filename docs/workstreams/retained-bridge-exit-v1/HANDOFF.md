# Retained Bridge Exit v1 Handoff

Updated: 2026-05-20

## Current State

`RBX-M1-010`, `RBX-M1-020`, `RBX-M1-021`, `RBX-M1-030`, `RBX-M1-040`, `RBX-M1-050`,
`RBX-M1-060`, `RBX-M1-070`, `RBX-M1-075`, `RBX-M1-080`, and `RBX-M1-085` are complete. `RBX-M1-080` removed
`fret-docking`'s `fret-ui/unstable-retained-bridge` dependency, deleted the retained docking
adapter and retained public entry points, removed `fret-docking` from the retained-bridge
allowlist, and mapped the deleted retained test files to public declarative or mechanism-level
coverage. `RBX-M1-085` then migrated first-party docking demos and the cookbook docking example to
the public declarative dock-space entry points and added policy coverage that prevents those
examples from teaching the deleted retained public APIs. The docking retained bridge audit is
recorded in:

- `docs/workstreams/retained-bridge-exit-v1/RBX_M1_010_DOCKING_RETAINED_BRIDGE_AUDIT_2026-05-18.md`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M1_030_DOCKING_DECLARATIVE_PRIMITIVE_GAP_AUDIT_2026-05-18.md`

The first implementation slice was:

- `RBX-M1-020` - Extract docking split geometry and handle painting from
  `fret_ui::retained_bridge`.

Readiness note:

- `docs/workstreams/retained-bridge-exit-v1/RBX_M1_020_READINESS_NOTE_2026-05-18.md`

## Key Finding

Docking proved the retained bridge can be removed only after the public declarative dock-space host
owns the same editor-grade host responsibilities: child-root placement, prepaint liveness, event
arbitration, command/focus, diagnostics, viewport mapping/input, chrome painting, drag/drop,
floating windows, split handles, tab overflow/scroll/hover/drag, and tear-off debounce. Those
responsibilities now live in `fret-docking` policy code backed by `fret-ui`'s mechanism-only
`ManagedSurface`; retained bridge types are no longer part of `fret-docking`'s dependency or public
surface.

M2 is in progress. `RBX-M2-010` narrowed `fret-node` retained bridge access to the explicit
`compat-retained-canvas` island. `RBX-M2-020` removed UI Gallery's dependency on that island by
migrating the node graph cull torture page and AI workflow node graph demo to
`NodeGraphSurfaceBinding` plus declarative `node_graph_surface(...)`; workflow viewport controls now
use `LayoutQueryRegion` for stage bounds instead of a retained `BoundsRecorder`. `RBX-M2-030`
removed the first-party legacy retained node graph demo feature/bins/modules, leaving
`node_graph_demo` behind `node-graph-demos` as the supported first-party node graph demo path.
`RBX-M2-040` removed the public declarative retained-subtree shim
(`node_graph_surface_compat_retained(...)` / `NodeGraphSurfaceCompatRetainedProps`) while keeping
the lower-level `compat-retained-canvas` feature compiling for the remaining retained canvas/editor
implementation island. `RBX-M2-050` then moved retained node graph widget/editor/overlay/panel/
portal modules and root exports behind a crate-private/test-only compatibility island, keeping the
retained behavior matrix available while removing the public retained widget authoring surface.
`RBX-M2-060` moved pure overlay/panel/screen-space policy and layout modules onto the default
declarative `fret-ui` gate while keeping retained overlay widgets/paint behind
`compat-retained-canvas`. `RBX-M2-070` moved declarative portal editor chrome tests onto the
default gate while leaving retained portal text/number editor command handlers gated. `RBX-M2-080`
recorded the remaining retained node graph capability ledger and added a source-policy gate that
keeps code-level retained bridge usage on the explicit migration ledger. `RBX-M2-085` moved the
portal submit/cancel/step command protocol onto the default declarative `fret-ui` gate while the
retained portal host and retained portal text/number command handlers continue to consume it from
the explicit compatibility island. `RBX-M2-090` then moved portal text/number submit/cancel/step
decision policy onto the default editor gate. `RBX-M2-095` moved portal text/number session command
application onto the default editor gate; retained portal text/number command handlers now provide
model/session I/O adapters around default command policy and session application. `RBX-M2-100`
added a default-gated declarative controls overlay composition that builds the controls panel,
six-button roster, labels, a11y labels/test IDs, enabled/disabled command binding state, and
activation command dispatch hooks without constructing the retained controls widget. The retained
controls widget remains behind `compat-retained-canvas` as the oracle for pointer, keyboard, hover,
focus, and retained paint behavior until those interaction families have default declarative
coverage. `RBX-M2-105` then added a default-gated declarative blackboard overlay composition that
builds the blackboard panel, header, sorted symbol rows, labels, a11y labels/test IDs, and a
mechanism-only action hook without constructing the retained blackboard widget. The retained
blackboard widget remains behind `compat-retained-canvas` as the oracle for transaction submission,
rename sessions, keyboard/focus navigation, pointer hover/press state, and retained paint behavior.
`RBX-M2-106` then added a default-gated declarative minimap overlay composition that builds a
panel-root element, declarative canvas child, `node_graph.minimap` semantics/test ID, and a paint
plan for the minimap panel, projected node markers, and viewport marker without constructing the
retained minimap widget. The retained minimap widget remains behind `compat-retained-canvas` as the
oracle for keyboard pan/zoom/focus, pointer drag panning, focus/capture propagation, retained hit
testing, and store/controller viewport updates.
`RBX-M2-107` then added a default-gated declarative toolbar overlay composition that builds
absolute node/edge toolbar containers, preserves retained-compatible placement planning and
visibility rules, and stamps toolbar semantics/test IDs without constructing the retained toolbar
widgets. The retained toolbar widgets remain behind `compat-retained-canvas` as the oracle for
child measurement, retained child-root layout/paint, hit testing, and model/internals-driven
target resolution.
`RBX-M2-108` then added a default-gated declarative rename overlay composition that consumes the
shared rename host layout plan, builds an absolute panel plus declarative text input, preserves the
caller-owned bound text model, stamps rename root/input semantics and stable test IDs, and wires
submit/cancel commands without constructing the retained `NodeGraphOverlayHost` widget. The
retained rename host remains behind `compat-retained-canvas` as the oracle for seed-text ownership,
focus-loss close, focus request/restore, keyboard submit/cancel event routing, graph/edit queue
transaction submission, blackboard rename handoff, and retained paint/hit testing.
`RBX-M2-109` then moved rename submit/cancel command parsing, keyboard submit/cancel decision, and
active-session application onto the default overlay gate in `rename_command.rs`. The retained
`NodeGraphOverlayHost` now delegates close/commit decisions to that default policy and remains as a
model I/O plus controller/edit-queue submission adapter. It still stays behind
`compat-retained-canvas` as the oracle for seed-text ownership during layout, focus request/restore,
focus-loss close integration, blackboard rename handoff, and retained paint/hit testing.
`RBX-M2-110` then moved rename lifecycle planning onto the default overlay gate in
`rename_lifecycle.rs`. Default tests now cover group/symbol seed text, first-open focus request,
no reseed/refocus for already-open sessions, focus-loss close without stealing the new focus owner,
and focus restoration when a hidden rename input still owns focus. The retained
`NodeGraphOverlayHost` now applies that default lifecycle plan as a model/tree I/O adapter.
`RBX-M2-111` then moved minimap keyboard/pointer interaction planning onto the default overlay gate
in `minimap_interaction_policy.rs`. Default tests now cover keyboard pan/zoom/focus/ignore
decisions, pointer down focus/capture/stop-propagation/repaint planning, non-left/outside pointer
rejection, and pointer-up capture release/finish gating. The retained minimap widget now applies
that default interaction plan as a store/view-state I/O and retained event side-effect adapter.
`RBX-M2-112` then moved toolbar visible-target filtering, node/edge child rect planning,
empty-size hiding, and child-bound hit testing onto the default overlay gate in
`toolbar_layout_policy.rs`. Retained toolbar widgets now apply that default layout/hit-test plan as
internals-target, retained child measurement, retained `layout_in`, and child-root paint adapters,
while declarative toolbar composition reuses the same default rect planning.
`RBX-M2-113` then moved controls overlay keyboard select/activate/focus-canvas planning plus
pointer hover/down/up focus/capture/repaint/activation planning onto the default overlay gate in
`controls_interaction_policy.rs`. The retained controls widget now applies that default
interaction plan as a retained side-effect adapter for focus, cursor, capture, repaint completion,
and command dispatch.
`RBX-M2-114` then moved blackboard overlay keyboard select/activate/focus-canvas planning plus
pointer hover/down/up focus/capture/repaint/activation planning onto the default overlay gate in
`blackboard_interaction_policy.rs`. The retained blackboard widget now applies that default
interaction plan as a retained side-effect adapter for focus, cursor, capture, repaint, and
transaction/rename dispatch.
`RBX-M2-115` then moved blackboard panel/button/label paint ordering, text constraints,
active-action backgrounds, and missing-symbol fallback onto the default overlay gate in
`blackboard_paint_plan.rs`. The retained blackboard paint module now applies that default paint
plan as a retained `PaintCx`/text-blob/scene-op adapter.
`RBX-M2-116` then moved controls panel/button paint ordering, text constraints, connection-mode
labels, pressed/hovered/keyboard-active backgrounds, and focus-gated keyboard highlight rules onto
the default overlay gate in `controls_paint_plan.rs`. The retained controls paint path now applies
that default paint plan as a retained `PaintCx`/text-blob/scene-op adapter.
`RBX-M2-117` then moved controls panel hit-testing and pointer-down host side-effect planning onto
the default overlay gate in `controls_host_policy.rs`. The retained controls widget now consumes
that default host plan for panel hit-testing plus pointer-down focus, propagation, capture, and
repaint decisions. The declarative controls composition now wraps its panel in a `PointerRegion`
so blank panel pointer-downs focus/stop propagation without dispatching controls commands, while
button descendants keep their `Pressable` activation path.
`RBX-M2-118` then proved the declarative controls button pointer-up/capture/command completion path
without constructing the retained controls widget. The default declarative controls test now covers
pointer-down capture, no early command dispatch, pointer-up capture release, focus transfer to the
activated button, command dispatch on in-bounds release, and capture completion without command
dispatch on out-of-bounds release through the existing `Pressable` mechanism.
`RBX-M2-119` then added a narrow `ManagedSurfaceLayoutCx::measure_child(...)` mechanism and proved
toolbar Auto child measurement plus child-root layout/paint placement on the default declarative
path. Node and edge toolbar declarative hosts can now measure a child, compute the retained
placement-policy rect, layout the child there, and paint it without retained toolbar widgets. The
retained toolbar widgets remain as the oracle for deletion logistics and compatibility gating.
`RBX-M2-120` then moved node/edge toolbar model/internals-driven target resolution onto the default
`toolbar_policy.rs` gate. Retained toolbar widgets and declarative toolbar helpers now share the
same selected fallback/requested target plus `NodeGraphInternalsStore` window-geometry resolver.
Default tests cover selected fallback, requested selected/unselected targets, and missing internals
geometry for node and edge toolbars; compat retained oracle tests remain green after the extraction.
`RBX-M2-121` then deleted the retained node/edge toolbar widgets, retained toolbar layout adapter,
retained toolbar conformance test module, test-only exports, and retained bridge source-policy
allowlist entries. Toolbar behavior now lives in default `toolbar_policy.rs`,
`toolbar_layout_policy.rs`, and `toolbars_declarative.rs`; both default and
`compat-retained-canvas` package gates pass after deletion.
`RBX-M2-122` then added a focus-capable `Pressable` activation hook and used it in declarative
controls to prove pointer and keyboard activation can dispatch the bound command and restore focus
to a node graph surface/canvas target. The retained controls widget remains behind
`compat-retained-canvas` as the oracle for deletion logistics and any remaining retained-only
behavior families; the focus-restore gap is now covered on the default declarative path.
`RBX-M2-123` then moved retained controls root keyboard semantics and Escape behavior onto the
default declarative controls path. The declarative controls root now exposes the
`node_graph.controls` panel semantics node, retained-compatible active value fallback, pointer-down
active value promotion, root keyboard navigation/activation, and Escape focus return without command
dispatch. Default controls tests cover those behavior families, and the compat retained
`overlay_minimap_controls_conformance` oracle remains green for the old retained controls/minimap
behavior.
`RBX-M2-124` then backfilled default declarative controls overlay/surface integration coverage
before deleting the retained controls widget. The new default tests prove pointer-down outside the
controls panel falls through to the surface, blank pointer-down inside the panel blocks surface
input and focuses the controls root, focus traversal reaches controls from a focusable surface, and
Escape returns focus to that surface without dispatching commands. The compat retained
`overlay_minimap_controls_conformance` oracle remains green, and retained controls were deliberately
left in place for a narrow deletion slice that must preserve retained minimap coverage.
`RBX-M2-125` then deleted the retained controls widget, removed the retained controls module and
test-only exports, removed `src/ui/overlays/controls.rs` from the retained bridge source migration
ledger, and trimmed the combined retained `overlay_minimap_controls_conformance` oracle to
minimap-only coverage. Default declarative controls tests remain green, and retained minimap
pointer/keyboard/store/semantics coverage remains green under `compat-retained-canvas`.
`RBX-M2-126` then upgraded the default declarative minimap from paint-only composition to a
managed-host proof for retained minimap side effects. The declarative minimap now has a focusable
`node_graph.minimap` semantics root, a `ManagedSurface` host with minimap-only hit testing, pointer
focus return/capture/release, drag pan updates, keyboard pan/zoom, Escape focus return, redraw, and
notify coverage. The retained minimap widget remains behind `compat-retained-canvas` as the oracle
for the next narrow deletion slice.
`RBX-M2-127` then deleted the retained minimap widget, removed the retained minimap module/export
and retained oracle test module, and removed `src/ui/overlays/minimap.rs` from the retained bridge
source migration ledger. A deletion-preflight compat retained oracle run proved the old minimap
pointer/drag/keyboard/controller behavior in the current worktree before the retained source was
removed; default declarative minimap tests now carry the behavior contract.
`RBX-M2-128` then upgraded the default declarative blackboard from composition/action-hook coverage
to a focusable host proof for retained blackboard side effects. The declarative blackboard now has
a `node_graph.blackboard` semantics root with active action value, panel pointer blocking, outside
pointer fallthrough, pressable capture/up completion, pointer/keyboard action dispatch, keyboard
navigation, and Escape focus return coverage. The retained blackboard widget remains behind
`compat-retained-canvas` as the oracle because graph/controller transaction submission and
symbol-rename handoff still need default declarative integration before deletion.
`RBX-M2-129` then wired the declarative blackboard action path to `NodeGraphSurfaceBinding` and
`NodeGraphOverlayState`. Default tests now prove Add Symbol, Insert Symbol Ref, and Delete Symbol
commit through the store/controller binding path, while Rename opens `symbol_rename` overlay state
without queueing a graph transaction. The retained blackboard oracle remains green under
`compat-retained-canvas`, so blackboard is ready for a deletion-preflight oracle run and narrow
retained-source deletion slice.
`RBX-M2-130` then ran the deletion-preflight retained blackboard oracle and deleted the retained
blackboard widget, retained blackboard paint adapter, and retained blackboard conformance module.
The retained bridge source ledger no longer allows `blackboard.rs` / `blackboard_paint.rs`; default
declarative blackboard tests now carry the behavior contract.

## Completed Implementation

`RBX-M1-020` added a docking-private split geometry helper and migrated `fret-docking` source/tests
off the retained bridge split helpers and `retained_bridge::ResizeHandle`.

`RBX-M1-021` migrated `apps/fret-examples/src/docking_arbitration_demo.rs` diagnostics split
geometry off `fret_ui::retained_bridge::resizable_panel_group` and deleted the remaining retained
bridge resizable helper module/export after repo-wide no-user proof.

There are no remaining Rust source users of:

- `retained_bridge::resizable_panel_group`
- `retained_bridge::ResizablePanelGroupLayout`
- `resizable::compute_layout`

`RBX-M1-030` found that panel content is declarative-capable through panel-root registries. The
missing piece was the host lifecycle around
`DockSpace`: externalized controller state, child-root placement, prepaint liveness, raw event
arbitration, command/focus routing, and custom chrome/child paint ordering.

`RBX-M1-040` added `DockSpaceController` as the docking-owned cross-frame host state object and kept
the retained `DockSpace` widget as the adapter. The extraction is intentionally behavior-preserving:
methods still live on `DockSpace` for now, with a transitional `Deref` / `DerefMut` shim delegating
state field access to the controller.

`RBX-M1-050` added a private `DockSpaceLayoutSnapshot`. `DockSpace::layout` stores a snapshot for
the current frame, and `DockSpace::paint` reuses it when bounds/frame/split settings match. Paint
still has a fallback rebuild path so retained host behavior remains unchanged if paint runs without a
matching layout snapshot. The snapshot and builder are `pub(super)` internal surfaces so a future
declarative dock host adapter can reuse the same frame decision object without making it public API.

`RBX-M1-060` added a narrow mechanism-only `ManagedSurface` primitive in `fret-ui` and proved it with
targeted `fret-ui` tests for child-root layout, prepaint, and controlled child-root paint. It also
extracted `panel_root_placements_for_snapshot(...)` in `fret-docking` so retained `DockSpace::layout`
and future declarative dock hosts share the same panel placement decision. A docking test now proves
that a declarative managed surface can consume `DockSpaceLayoutSnapshot` for panel-root layout and
paint without `RetainedSubtreeProps`. `DockSpaceLayoutSnapshot::paint_panel_bounds` is graph-order
stable and no longer depends on `HashMap` iteration order.

`RBX-M1-070` added public declarative docking entry points in
`ecosystem/fret-docking/src/dock/declarative.rs`: `dock_space_element(...)`,
`dock_space_element_from_registry(...)`, `dock_panel_element(...)`, and
`DockPanelElementRegistryService`. These APIs render and host `AnyElement` panel roots through
`ManagedSurface` without `RetainedSubtreeProps`. It also added an imui declarative wrapper
(`dock_space_declarative*`) and documented the old retained `create_dock_space_node*` /
`mount_dock_space*` helpers as legacy.

`RBX-M1-075` first slice extended `ManagedSurface` with event, command, and command availability
hooks, then used the command hook in `dock_space_element(...)` to handle
`dock.focus_requested_panel`. A new public declarative dock-space test proves
`DockManager::request_activate_panel(..., focus: true)` can focus the requested panel root through
the declarative host path.

`RBX-M1-075` second slice exposed current host `node()` through `ManagedSurface`
layout/prepaint/paint contexts and used it to keep dock panel/tabs internal-drag routes alive from
the public declarative dock-space host. The same path now registers the declarative host as the
window dock-space node. A new test proves declarative route-anchor/node registration without
creating retained `DockSpace`.

`RBX-M1-075` third slice moved common docking diagnostics publication into
`ecosystem/fret-docking/src/dock/diagnostics.rs` and wired the public declarative dock-space
prepaint hook to publish `WindowInteractionDiagnosticsStore` snapshots. It records active dock-drag
diagnostics, dock graph stats, and dock graph signatures, and requests animation frames while an
active dock drag affects the host window. A new public declarative dock-space test proves the
diagnostics/liveness path without creating retained `DockSpace`.

`RBX-M1-075` fourth slice extended `ManagedSurfacePaintCx` with `scale_factor()`, `services()`, and
`child_bounds(...)`. The public declarative dock-space host now paints panel roots using actual
child bounds with the snapshot rect as a fallback, matching retained `DockSpace::paint` fallback
semantics more closely. A `fret-ui` managed-surface test locks those paint context capabilities.

`RBX-M1-075` fifth slice syncs declarative viewport layouts from `DockSpaceLayoutSnapshot` into
`DockManager::sync_viewport_layouts_for_window(...)` during layout and prepaint. A public
declarative dock-space test proves viewport mapping publication and stale-layout cleanup without
retained `DockSpace::paint`.

`RBX-M1-075` sixth slice extracted split-handle paint inputs from
`ecosystem/fret-docking/src/dock/paint.rs` and made the public declarative dock-space host carry
those inputs in its per-frame output. The managed-surface paint hook now paints split handles
without borrowing retained `DockSpace`. The public declarative dock-space panel-root test now also
asserts that split-handle chrome is painted.

`RBX-M1-075` seventh slice extracted viewport-surface paint inputs from
`ecosystem/fret-docking/src/dock/paint.rs` and made the public declarative dock-space host carry
those inputs in its per-frame output. The managed-surface paint hook now paints
`SceneOp::ViewportSurface` and viewport overlay hooks without borrowing retained `DockSpace`. A
public declarative dock-space test proves pure viewport panels render through the declarative host.

`RBX-M1-075` eighth slice extracted floating container chrome paint inputs from
`ecosystem/fret-docking/src/dock/paint.rs` and made the public declarative dock-space host carry
those inputs in its per-frame output. The managed-surface paint hook now paints in-window floating
outer/title-bar chrome without borrowing retained `DockSpace`. A public declarative dock-space test
proves floating chrome renders through the declarative host.

`RBX-M1-075` ninth slice extracted active dock drag ghost snapshot selection into
`ecosystem/fret-docking/src/dock/diagnostics.rs` and made the public declarative dock-space host
carry those snapshots in its per-frame output. The managed-surface paint hook now prepares the
dragged panel title through `ManagedSurfacePaintCx::services()` and paints drag payload ghosts
without borrowing retained `DockSpace`. A public declarative dock-space test proves drag ghost
rendering through the declarative host.

`RBX-M1-075` tenth slice extracted basic float/empty/center drop-overlay painting into
`paint_basic_drop_overlay(...)` and made the public declarative dock-space host carry
`DockManager::hover` plus `DockSpaceLayoutSnapshot::layout_all` in its per-frame output. The
managed-surface paint hook now paints center drop overlays without borrowing retained `DockSpace`.
A public declarative dock-space test proves the center drop-overlay path.

`RBX-M1-075` eleventh slice made the public declarative dock-space host derive `DockDropHints` from
the resolved `DockManager::hover` target and carry the hints in its per-frame output. The
managed-surface paint hook now reuses `paint_drop_hints(...)` to paint the drop-hint plate and pads
without borrowing retained `DockSpace`. A public declarative dock-space test proves the drop-hint
pad path.

`RBX-M1-075` twelfth slice extracted structural tab chrome quads into reusable
`TabChromePaintInput` / `paint_tab_chrome_inputs(...)` helpers. Retained `paint_dock(...)` now
delegates panel background, tab bar, active/hover tab plate, and active underline painting through
that helper, while still owning tab title, close button, overflow, and viewport fill details. The
public declarative dock-space host carries those tab chrome inputs in its per-frame output and
paints tab bar chrome before panel roots. A public declarative dock-space test proves the tab chrome
path.

`RBX-M1-075` thirteenth slice extracted non-text complex drop-overlay geometry into reusable
`ComplexDropOverlayPaintInput` / `paint_complex_drop_overlay_inputs(...)` helpers. Retained
`paint_drop_overlay(...)` now delegates tab insert markers and edge split-slot preview overlays
through the shared helper while still owning tab-title preview text. The public declarative
dock-space host carries those complex overlay inputs in its per-frame output and paints edge
split-slot previews plus tab insert markers from its managed-surface paint hook. Public declarative
dock-space tests prove both paths.

`RBX-M1-075` fourteenth slice extracted tab-insert preview title painting into reusable
`paint_tab_insert_preview_title(...)`. Retained `paint_drop_overlay(...)` now delegates the preview
title through the shared helper, and the public declarative dock-space host prepares and paints the
preview title from its managed-surface paint hook. `ManagedSurfacePaintCx` now exposes
`release_text_blob_on_next_paint(...)` so paint-time transient text blobs remain valid for the scene
that references them and are released on the next managed-surface repaint or cleanup. A public
declarative dock-space test proves the tab-insert preview title path, and a `fret-ui` managed
surface test proves the deferred text release contract.

`RBX-M1-075` fifteenth slice extracted tab title, active-tab close affordance, overflow button, and
overflow menu painting into reusable `TabDetailPaintInput` / `paint_tab_detail_inputs(...)`
helpers. Retained `paint_dock(...)` now delegates tab detail painting through the shared helper,
while the public declarative dock-space host prepares transient tab title/close/overflow text
resources and paints those tab details from its managed-surface paint hook. A public declarative
dock-space test proves tab title, active close glyph, and overflow glyph rendering through the
declarative host path.

`RBX-M1-075` sixteenth slice moved the active-tab close affordance `PointerDown` / `PointerUp` path
onto the public declarative dock-space host. `ManagedSurfaceEventCx` remains mechanism-only and does
not expose prepaint-output reads; the docking host rebuilds a temporary
`DockSpaceLayoutSnapshot` from the current bounds and `DockManager` for close hit-testing. The
declarative host tracks pressed tab-close state, captures/releases the pointer, and emits
`Effect::Dock(DockOp::ClosePanel { ... })` on release over the same close affordance or within
click slop. A public declarative dock-space test proves the close effect path.

`RBX-M1-075` seventeenth slice moved the tab overflow-menu close click path onto the public
declarative dock-space host. The declarative host now owns overflow-menu state in `fret-docking`,
opens the menu from the overflow button, feeds that state back into shared
`TabDetailPaintInput` / `paint_tab_detail_inputs(...)`, and emits
`Effect::Dock(DockOp::ClosePanel { ... })` for overflow-menu row close without also activating the
tab. It also emits `Effect::Dock(DockOp::SetActiveTab { ... })` for overflow-menu row activation
without closing a tab. Public declarative dock-space tests prove the menu paint, close-effect, and
activation paths.

`RBX-M1-075` eighteenth slice moved the in-window floating close `PointerDown` / `PointerUp` path
onto the public declarative dock-space host. The declarative host now owns pressed-floating-close
state in `fret-docking`, reuses `DockSpaceLayoutSnapshot` floating chrome geometry for hit-testing
and pressed close painting, emits `Effect::Dock(DockOp::RaiseFloating { ... })` on close press, and
emits `Effect::Dock(DockOp::MergeFloatingInto { ... })` when release lands on the same close
affordance. A public declarative dock-space test proves the floating close effect path.

`RBX-M1-075` nineteenth slice moved the in-window floating title-bar drag move-rect path onto the
public declarative dock-space host. The declarative host now owns floating-drag state in
`fret-docking`, reuses `DockSpaceLayoutSnapshot` floating chrome geometry for title-bar hit-testing,
emits `Effect::Dock(DockOp::RaiseFloating { ... })` on title-bar press, and emits
`Effect::Dock(DockOp::SetFloatingRect { ... })` while the title bar is dragged. This deliberately
does not yet move dock-preview/merge-on-release arbitration for floating title-bar drags. A public
declarative dock-space test proves the move-rect path.

`RBX-M1-075` twentieth slice moved overflow-menu wheel scrolling onto the public declarative
dock-space host. The declarative host reuses the retained adapter's overflow-menu geometry and
scroll formula, updates `TabOverflowMenuState` inside `fret-docking`, consumes wheel events only
inside the menu rect, and keeps docking policy/state out of `fret-ui`. A public declarative
dock-space test proves that wheel scrolling exposes and activates the expected row without creating
retained `DockSpace`.

`RBX-M1-075` twenty-first slice moved tab-strip wheel scrolling onto the public declarative
dock-space host. The declarative host now owns tab-scroll state in `fret-docking`, keyed by window
and tabs node, and feeds that state into tab chrome/detail paint inputs, tab close hit-testing,
overflow-menu opening, and tab-insert preview painting. It reuses the retained adapter's wheel
formula and consumes wheel events inside overflowing tab bars without creating retained
`DockSpace`. A public declarative dock-space test proves that wheel scrolling makes the expected
tab close hit-testable.

`RBX-M1-075` twenty-second slice moved tab hover, tab overflow button hover, and overflow menu row
hover state onto the public declarative dock-space host. The declarative host now owns tab-hover
state in `fret-docking` and refreshes transient tab interaction paint state from the latest docking
service state at paint time, avoiding stale hover/menu visuals from older layout/prepaint frame
outputs. Public declarative dock-space tests prove ordinary tab hover, overflow button hover, and
overflow menu row hover without creating retained `DockSpace`.

`RBX-M1-075` twenty-third slice moved the narrow panel-tab drag activation path onto the public
declarative dock-space host. The declarative host now owns pending dock-drag state in
`fret-docking`, starts a `DRAG_KIND_DOCK_PANEL` runtime drag after the configured threshold is met,
reuses the retained runtime drag startup helper, preserves tab-local grab offsets and dock-preview
inversion policy, releases pointer capture on runtime drag start, and respects
`DockingPolicy::allow_panel_drag`. Public declarative dock-space tests prove activation, threshold
gating, and panel drag policy gating without creating retained `DockSpace`.

`RBX-M1-075` twenty-fourth slice moved tabs-group drag activation from empty tab-bar space onto the
public declarative dock-space host. The declarative host now owns pending tabs-drag state in
`fret-docking`, starts a `DRAG_KIND_DOCK_TABS` runtime drag after the configured threshold is met,
reuses the retained runtime drag startup helper, preserves tab-bar-local grab offsets and
dock-preview inversion policy, releases pointer capture on runtime drag start, and respects
`DockingPolicy::allow_tabs_group_drag`. Public declarative dock-space tests prove activation and
tabs-group drag policy gating without creating retained `DockSpace`.

`RBX-M1-075` twenty-fifth slice moved the floating title-bar drag dock-preview and center
merge-on-release path onto the public declarative dock-space host. The declarative host now owns
floating-drag activation state in `fret-docking`, latches dock-preview inversion policy at
threshold activation, resolves `DockManager::hover` over the root dock layout while the activated
floating title-bar drag moves, and emits `DockOp::MergeFloatingInto` on center drop release. A
public declarative dock-space test proves hover resolution and release-time merge without creating
retained `DockSpace`.

`RBX-M1-075` twenty-sixth slice moved left-button viewport pointer capture onto the public
declarative dock-space host. The declarative host now owns viewport capture state in `fret-docking`,
forwards `ViewportInputKind::PointerDown`, clamped captured `PointerMove`, `PointerUp`, and
`PointerCancel` effects through shared viewport helpers, and requests/releases pointer capture on
the managed-surface host without adding docking policy to `fret-ui`. Public declarative dock-space
tests prove captured moves outside the draw rect stay on the original viewport and pointer cancel
releases capture without creating retained `DockSpace`.

`RBX-M1-075` twenty-seventh slice moved floating close/title-bar hover visual state onto the public
declarative dock-space host. The declarative host now owns floating hover state in `fret-docking`,
updates it from `PointerMove` hit-tests, applies it at paint time so visuals use the latest event
state rather than a stale layout/prepaint frame, and preserves retained cursor hints for floating
close/title-bar hover. A public declarative dock-space test proves title-bar hover background and
close hover affordance painting without creating retained `DockSpace`.

`RBX-M1-075` twenty-eighth slice moved the stale-hover cleanup part of raw `InternalDrag`
arbitration onto the public declarative dock-space host. The declarative host now clears
`DockManager::hover` for `InternalDragKind::{Drop, Leave, Cancel}` and requests redraw only when a
hover was actually cleared, preserving the retained robustness behavior for drops that arrive after
the active drag session is already gone. A public declarative dock-space test proves the cleanup
path without creating retained `DockSpace`.

`RBX-M1-075` twenty-ninth slice moved drop-target resolution out of the retained `DockSpace::event`
local function set and into docking-private `dock/drop_resolve.rs`. Retained `DockSpace` now uses
that shared resolver, and the public declarative dock-space host uses it for
`InternalDragKind::{Enter, Over}` hover resolution. A public declarative dock-space test proves
that internal-drag `Over` can resolve the root split outer-left hint rect without creating retained
`DockSpace`. `ManagedSurfaceEventCx` now exposes the existing window-local pointer position helper
so ecosystem declarative hosts do not need retained `EventCx`.

`RBX-M1-075` thirtieth slice moved drop-intent resolution/application out of the retained
`DockSpace::event` local function set and into docking-private `dock/drop_resolve.rs`. Retained
`DockSpace` now uses those shared intent helpers, and the public declarative dock-space host handles
`InternalDragKind::Drop` by resolving through the shared target resolver, applying the shared
`DockDropIntent` into `Effect::Dock(...)`, clearing hover, invalidating layout when needed, and
ending the active dock drag session. A public declarative dock-space test proves an inner-left
hint-rect drop emits `DockOp::MovePanel`, applies cleanly to split the tabs node, and cancels the
active drag session without creating retained `DockSpace`.

`RBX-M1-075` thirty-first slice moved tab-bar drag auto-scroll onto the public declarative
dock-space host. The declarative host now caches tab widths measured during managed-surface paint
in docking-owned interaction state, then uses those measured widths for event-side hit-testing,
drop-target resolution, and auto-scroll. The first frame still has an approximate-width fallback
until paint has measured titles. A public declarative dock-space test proves repeated
`InternalDragKind::Over` events near the right tab-bar edge advance the insert index without
creating retained `DockSpace`; the retained auto-scroll comparison test remains green.

`RBX-M1-075` thirty-second slice moved stable out-of-bounds tear-off debounce mutation onto the
public declarative dock-space host. The declarative host now reads
`PlatformCapabilities.ui.window_tear_off`, gates requests through `DockingPolicy`, preserves the
conservative multi-window default through `DockSpaceElementOptions::allow_multi_window_tear_off`,
mutates drag payload tear-off debounce fields, and emits `DockOp::RequestTearOffPanel` /
`DockOp::RequestTearOffTabs` only after a stable second OOB frame. A public declarative
dock-space test proves the tear-off request path without creating retained `DockSpace`; retained
tear-off comparison tests remain green.

`RBX-M1-080` removed `fret-docking` from the retained bridge. The public surface now exports
declarative docking entry points (`dock_space_element`, `dock_space_element_from_registry`, and
`dock_panel_element`) and policy tests reject the old retained entry points. The retained
`DockSpace` adapter, retained panel registry/prelude, and retained-only tests were deleted after
their covered capabilities were mapped to compiling public declarative or mechanism-level tests in
`EVIDENCE_AND_GATES.md`.

Follow-up parity backfill after deletion added public declarative tests for cross-window element
bounds scoping, cross-window overlay-anchor lookup, registry-provided viewport panel child event
reachability, and missing non-viewport panel fallback UI. The viewport child reachability test uses
a declarative `PointerRegion` that actively requests focus on pointer down, matching the old
retained `FocusOnDown` test behavior.

A follow-up anchored layout-order hardening fixed a `fret-ui` mechanism issue found while verifying
the docking deletion: wrappers that mix layout-engine static children with manually positioned
absolute children now commit child layout side effects in author order. This lets a later `Anchored`
sibling resolve a preceding absolute anchor element in the same layout pass, without adding a broad
future-sibling/current-bounds fallback. The targeted anchored regression and `fret-docking` package
gate are green. The broader `fret-ui` layout primitive harness still has an independent
`chrome-container-stretch-keeps-outer-box` flex/chrome drift that is not anchored- or
docking-specific.

`RBX-M2-010` started the node-graph migration by narrowing `fret-node`'s retained bridge entry:
the redundant `compat-retained-bridge` feature alias was deleted, and
`compat-retained-canvas` is now the only `fret-node` feature that enables
`fret-ui/unstable-retained-bridge`. `RBX-M2-020` moved UI Gallery's node graph pages to
`NodeGraphSurfaceBinding` plus declarative `node_graph_surface(...)`. `RBX-M2-030` removed the
first-party legacy retained node graph demo feature/bins/modules and locked that with
`fret-node` policy tests. `RBX-M2-040` deleted the declarative retained-subtree compatibility module
and removed its public re-exports, then tightened the
`retained_compatibility_surface_stays_declarative_only` policy test so the symbols cannot return.
`RBX-M2-050` made the retained widget/editor/overlay/panel/portal modules crate-private and removed
their public root re-exports, with test-only crate-private access retained for the conformance
matrix. `RBX-M2-060` made `overlays` and `screen_space_placement` compile in the default
declarative `fret-ui` path, expanding default `fret-node` nextest coverage from 269 to 319 tests
with overlay/panel/minimap/toolbar/blackboard/rename/screen-space policy coverage. Retained overlay
widgets and retained paint helpers remain gated behind `compat-retained-canvas`. `RBX-M2-070`
then made `editors/chrome.rs` compile in the default declarative path, expanding default
`fret-node` nextest coverage from 319 to 324 tests, while retained portal text/number editor
command handlers remain gated. `RBX-M2-080` added
`surface_policy_tests::retained_bridge_source_usage_stays_on_the_migration_ledger`, recorded the
remaining retained capability families in
`RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`, and verified both default
declarative `fret-node` coverage and the full retained compatibility oracle. The headless graph
surface, default declarative UI surface, and explicit retained canvas compatibility island still
compile. The retained canvas/editor stack still exists inside `fret-node` behind
`compat-retained-canvas`. `RBX-M2-085` added `ui/portal_commands.rs` as a default-gated protocol
module for portal text command builders/parsers, made `ui/portal.rs` re-export that protocol for
retained compatibility consumers, and verified both default protocol tests and the full retained
oracle. `RBX-M2-090` added `ui/editors/portal_command_policy.rs` as a default-gated policy module,
moved portal text/number edit specs and submit result types into that module, and converted retained
`portal_text.rs` / `portal_number.rs` handlers to consume policy plans instead of owning
submit/cancel/step command decisions. `RBX-M2-095` then added
`ui/editors/portal_command_session.rs` as a default-gated session adapter, added default tests for
text/number session command application without retained `CommandCx`, and reduced retained
`PortalTextEditHandler` / `PortalNumberEditHandler` command paths to model/session I/O adapters
around that default logic. The retained portal editor modules still carry portal rendering/model
lifecycles until declarative portal hosting replaces them. `RBX-M2-100` added default-gated
declarative controls overlay composition and action command dispatch coverage. `RBX-M2-105` added
default-gated declarative blackboard overlay composition and action-hook activation coverage. The
retained canvas/editor stack still exists inside `fret-node` behind `compat-retained-canvas`.
`RBX-M2-106` added default-gated declarative minimap overlay composition and paint-plan coverage.
`RBX-M2-107` added default-gated declarative toolbar overlay composition and placement coverage.
`RBX-M2-108` added default-gated declarative rename overlay composition and submit/cancel command
protocol coverage. `RBX-M2-109` added default rename command/session application and left retained
rename host code as a model I/O adapter. `RBX-M2-110` added default rename lifecycle planning and
made the retained host consume it for seed/focus/focus-loss behavior. `RBX-M2-111` added default
minimap keyboard/pointer interaction planning and made the retained minimap widget consume it for
focus/capture/repaint/viewport-update side effects. `RBX-M2-112` added default toolbar
layout/hit-test planning and made retained toolbar widgets consume it for child layout and hit-test
decisions. `RBX-M2-113` added default controls interaction planning and made the retained controls
widget consume it for keyboard/pointer/focus/capture/repaint decisions. `RBX-M2-114` added default
blackboard interaction planning and made the retained blackboard widget consume it for
keyboard/pointer/focus/capture/repaint/action-dispatch decisions. `RBX-M2-115` added default
blackboard paint planning and made retained blackboard paint consume it for panel, action button,
label, active-state, and missing-symbol paint decisions. `RBX-M2-116` added default controls paint
planning and made retained controls paint consume it for panel, button, label, active-state, and
focus-gated keyboard highlight decisions. `RBX-M2-117` added default controls host planning and
made retained controls hit-testing/pointer-down side effects consume it for panel blocking,
focus/propagation, button capture, and repaint decisions; declarative controls now also covers
blank panel pointer-down focus/no-command behavior without stealing button pressable activation.
`RBX-M2-118` added default declarative controls pointer-up/capture completion coverage, proving
the existing `Pressable` path handles capture release, focus, command dispatch timing, and
outside-release cancellation for controls buttons.
`RBX-M2-119` added a mechanism-level managed-surface child measurement hook and default toolbar
managed-host tests for Auto child measurement plus child layout/paint placement. Later slices
completed toolbar target resolution/deletion, controls retained-widget deletion, minimap
managed-host side-effect parity/deletion, blackboard declarative host/action integration parity,
retained blackboard deletion, and declarative rename managed-host parity. `RBX-M2-131` proved
default rename seed/focus, submit/cancel focus restore, focus-loss close, graph/store transaction
submission, and hit-test masking without constructing the retained rename host. `RBX-M2-132` then
used that proof plus a deletion-preflight retained group/symbol rename oracle to delete the
retained rename host, retained rename event adapter, and retained rename oracle tests. Rename is now
a default declarative managed-host path; `group_rename.rs` only carries overlay state. `RBX-M2-133`
deleted the no-user retained diagnostics anchor widgets plus their dead retained canvas
diagnostics-anchor layout plumbing. `RBX-M2-134` proved active-descendant semantics on the default
declarative node graph binding/surface for focused port, edge, and node, preserved the retained
active-descendant priority order of port before edge before node, verified both
`NodeGraphSurfaceBinding::surface_props()` and `NodeGraphSurfaceProps::new(...)`, suppressed stale
active descendants for missing graph items, then deleted retained `a11y.rs` and its retained oracle
test after deletion-preflight retained/default coverage. `RBX-M2-135` then moved the retained portal
subtree lifecycle key contract onto the default visible-subset portal path by keying portal labels
with node id plus node kind plus node kind version. Default tests now prove stable subtree identity
across frames, reset on node kind/kind_version changes, growth-only measured node-size hints, and
removed-node measured-geometry cleanup. The retained portal lifecycle/measured-geometry/internals
oracle remains green under `compat-retained-canvas`. `RBX-M2-140` then moved arbitrary per-kind
portal renderer hosting onto the default declarative surface by adding
`NodeGraphDeclarativePortalRenderer`, wiring `NodeGraphNodeTypes` through
`node_graph_surface_with_portal_renderer(...)`, preserving the `(node id, kind, kind_version)`
lifecycle key, proving custom renderer fallback and registry hosting, and proving custom subtree
measurements publish into `MeasuredGeometryStore`. Retained portal files are now kept for the
remaining portal command-adapter/deletion-preflight oracle work, not because arbitrary per-kind
renderer hosting still requires retained code. `RBX-M2-145` then moved portal command routing onto
the default declarative surface by adding `NodeGraphDeclarativePortalCommandHandler`,
`NodeGraphSurfaceProps::portal_command_handler`, default re-exports for the portal command protocol,
and a surface-root command hook that parses portal commands, routes handled commits through
`NodeGraphSurfaceBinding`, restores focus to the surface, requests redraw/notify, and leaves
unclaimed portal commands unhandled. `RBX-M2-150` then moved first-party portal text/number editor
handlers onto that default declarative command seam, converted their session state from host globals
to cloneable model-backed editor state, deleted the retained `CommandCx` adapters from
`portal_text.rs` / `portal_number.rs`, and removed those files from the retained bridge source
ledger after pre-delete and post-delete retained portal oracle runs. `RBX-M2-160` then deleted the
retained `NodeGraphPortalHost`, retained portal command-handler traits/adapters, and retained
portal lifecycle/measured-geometry/measured-internals oracle modules after the default declarative
path already covered portal subtree lifecycle keys, measured-geometry cleanup/publishing,
arbitrary per-kind renderer hosting, portal command routing, and first-party text/number editor
command submission. `RBX-M2-170` then deleted the no-user retained `NodeGraphEditor` and
`NodeGraphPanel` composition wrappers, removed their compat module entries and retained-source
ledger allowlist entries, and kept the actual panel placement contract on default
`screen_space_placement::rect_in_bounds`. `RBX-M2-180` then deleted no-user retained helper
modules (`retained_submit.rs`, `retained_event_tail.rs`, `panel_button_paint.rs`), removed their
module entries, deleted the retained-only `begin_panel_press(...)` adapter, and left
`panel_pointer_policy.rs` as default-only hover/release policy shared by controls and blackboard.
`RBX-M2-190` then removed the retained middleware event/command hook surface:
`NodeGraphCanvasMiddleware` no longer names retained `EventCx` / `CommandCx`, retained runtime
command/event dispatch no longer calls middleware hooks, and the remaining middleware shape is a
`before_commit` transaction guard used only inside the retained canvas island. `RBX-M2-200` then
isolated retained canvas widget tail actions: `widget_tail.rs` now owns retained-agnostic redraw,
paint-invalidation, and handled-event side-effect traits; retained `EventCx` / `CommandCx` /
`LayoutCx` / `PaintCx` implementations live in `retained_widget_tail.rs`; and
`paint_invalidation.rs` / `redraw_request.rs` are locked by a default source-policy test so they
cannot re-import retained bridge Cx types. `RBX-M2-210` then applied the same pattern to
wire-drag commit side effects: `wire_drag/commit_cx.rs` is now retained-agnostic, while retained
`EventCx` / `CommandCx` impls live in `wire_drag/retained_commit_cx.rs`. `RBX-M2-220` then moved
pointer-up finish release-capture plus paint invalidation behind `PointerCaptureReleaseCx`; retained
`EventCx` implements that seam in `retained_widget_tail.rs`, and the pointer-up finish helper files
are source-policy gated against retained Cx imports. `RBX-M2-230` then moved sticky-wire
pointer-down finish release-capture, propagation stop, and paint invalidation behind
`HandledPointerCaptureReleaseCx`; `sticky_wire_connect/finish.rs` is now source-policy gated as
retained-bridge-free support. `RBX-M2-240` then moved edge-insert drag move finish paint
invalidation behind `WidgetPaintInvalidationCx`; `edge_insert_drag/drag/tail.rs` is now
source-policy gated as retained-bridge-free support. `RBX-M2-250` then moved cancel cleanup finish
release-capture, optional propagation stop, and paint invalidation behind
`HandledPointerCaptureReleaseCx`; retained `cx.app` timer I/O remains in `cancel.rs`, and
`cancel_cleanup.rs` is now source-policy gated as retained-bridge-free support. `RBX-M2-260` then
moved sticky-wire target picker host/window access plus handled-event finish behavior behind
`StickyWireTargetPickerCx`; retained `EventCx` implements that seam in
`sticky_wire_targets/retained_picker_cx.rs`, and `sticky_wire_targets/picker.rs` is now
source-policy gated as retained-bridge-free support. `RBX-M2-270` then moved group drag/resize
preview tail paint invalidation behind `WidgetPaintInvalidationCx`; retained `cx.app` auto-pan
view-state I/O remains in the retained event callers, and `group_drag/tail.rs` /
`group_resize/tail.rs` are now source-policy gated as retained-bridge-free support. `RBX-M2-280`
then moved group drag/resize move handler host/bounds access behind `GroupPreviewMoveCx`; retained
`EventCx` implements that seam in `group_preview_move_retained_cx.rs`, and `group_drag.rs` /
`group_resize.rs` are now source-policy gated as retained-bridge-free support. `RBX-M2-290` then
moved pending group drag activation host access behind `PendingGroupActivationCx`; retained
`EventCx` implements that seam in `pending_group_activation_retained_cx.rs`, pending group resize
activation no longer takes an unused retained Cx parameter, and `pending_group_drag.rs` /
`pending_group_resize.rs` are now source-policy gated as retained-bridge-free support.
`RBX-M2-300` then moved pending group drag, pending group resize, and pending node resize
pointer-up release tail actions behind `PointerCaptureReleaseCx`; retained `EventCx` already
implements that seam in `retained_widget_tail.rs`, and the pending release helper files are now
source-policy gated as retained-bridge-free support.
`RBX-M2-310` then moved pending wire drag pointer-up release/promotion tail actions behind
`PointerCaptureReleaseCx`; `pointer_up_pending/wire_drag.rs` is now source-policy gated as
retained-bridge-free support.
`RBX-M2-320` then moved pending node drag click-select release view-state I/O plus pointer-up tail
actions behind `PendingNodeDragReleaseCx`; retained `EventCx` implements that seam in
`pending_node_drag_release_retained_cx.rs`, and `pointer_up_pending/click_select.rs` is now
source-policy gated as retained-bridge-free support.
`RBX-M2-330` then moved group drag, group resize, and node resize pointer-up commit host/window I/O
plus pointer-up tail actions behind `PointerUpCommitCx`; retained `EventCx` implements that seam
in `pointer_up_commit_retained_cx.rs`, and the pointer-up commit helper files are now
source-policy gated as retained-bridge-free support.
`RBX-M2-340` then moved node drag move tail auto-pan host I/O and paint invalidation behind
`NodeDragMoveTailCx`; retained `EventCx` implements that seam in
`node_drag_move_tail_retained_cx.rs`, and `node_drag/tail.rs` is now source-policy gated as
retained-bridge-free support.
`RBX-M2-350` then moved marquee begin capture/paint invalidation and marquee finish view-state
I/O/release tail actions behind `MarqueeCx`; retained `EventCx` implements that seam in
`marquee_retained_cx.rs`, and `marquee_begin.rs` / `marquee_finish.rs` are now source-policy gated
as retained-bridge-free support.
`RBX-M2-360` then moved node drag preview host/graph-read I/O behind `NodeDragPreviewCx`; retained
`EventCx` implements that seam in `node_drag_preview_retained_cx.rs`, and
`node_drag_preview.rs` / `node_drag_preview/compute.rs` are now source-policy gated as
retained-bridge-free support.
`RBX-M2-370` then moved node drag snapline geometry reads and multi-drag extent geometry reads
behind `NodeDragGeometryCx`; retained `EventCx` implements that seam in
`node_drag_geometry_retained_cx.rs`, and `node_drag_snap.rs` /
`node_drag_constraints_extent.rs` are now source-policy gated as retained-bridge-free support.
`RBX-M2-380` then moved keyboard pan activation paint invalidation and stop-propagation side
effects behind existing `widget_tail` seams; retained `EventCx` already implements those seams in
`retained_widget_tail.rs`, and `keyboard_pan_activation.rs` is now source-policy gated as
retained-bridge-free support.
`RBX-M2-390` then moved clipboard feedback host/window/paint invalidation and timer-motion paint
invalidation behind retained-agnostic seams. Retained `EventCx` implements clipboard feedback in
`event_clipboard_feedback_retained_cx.rs` and already implements motion paint invalidation through
`retained_widget_tail.rs`; `event_clipboard_feedback.rs`, `event_clipboard_feedback_cx.rs`, and
`timer_motion_shared.rs` are now source-policy gated as retained-bridge-free support. Clipboard
unavailable feedback now has behavior tests proving matching tokens still clear pending paste,
show the info toast, schedule the toast timer, request redraw, and invalidate paint while stale
tokens remain side-effect free.
`RBX-M2-400` then moved expired-toast timer paint invalidation behind the existing
retained-agnostic widget tail paint seam. `event_timer_toast.rs` is now source-policy gated as
retained-bridge-free support, with behavior tests proving matching toast timers clear the toast and
invalidate paint while stale timers leave toast state and feedback side effects untouched.
`RBX-M2-410` then deleted the unused retained `EventCx` parameter from pending node resize move
handling. `pending_resize.rs` is now source-policy gated as retained-bridge-free support, with
handler tests proving below-threshold moves stay pending and above-threshold moves activate node
resize.
`RBX-M2-420` then moved edge double-click finish stop-propagation plus paint invalidation behind
the existing retained-agnostic `WidgetHandledCx` seam. `pointer_down_double_click_edge/finish.rs`
is now source-policy gated as retained-bridge-free support, with a local tail test proving finish
still stops propagation, requests redraw, and invalidates paint; existing reroute/picker
double-click gesture tests remain green.
`RBX-M2-430` then moved searcher dismiss release-capture, handled finish, and paint invalidation
tails behind retained-agnostic `widget_tail` seams. `searcher_activation_state/clear.rs`,
`searcher_ui.rs`, and `searcher_ui/event.rs` are now source-policy gated as retained-bridge-free
support, with focused tests proving dismiss clears overlay/pending-drag state and releases capture
without adding paint side effects at that layer, while finish/invalidate still request redraw/paint
invalidation and stop propagation where appropriate.
`RBX-M2-440` then moved searcher row-drag release activation/dismiss coordination behind a
retained-agnostic `SearcherReleaseCx` seam. `searcher_activation_state/release.rs` is now
source-policy gated as retained-bridge-free support, retained row activation lives in
`searcher_activation_state/release_retained_cx.rs`, and focused tests prove no-pending release is
side-effect free, row release activates and finishes, and outside release dismisses and finishes.
`RBX-M2-450` then moved searcher row-drag arming pointer id, tick id, and pointer capture access
behind a retained-agnostic `SearcherArmCx` seam. `searcher_activation_state/arm.rs` is now
source-policy gated as retained-bridge-free support, retained pointer/timer/capture access lives in
`searcher_activation_state/arm_retained_cx.rs`, and focused tests prove unselectable rows are
side-effect free while selectable rows record pending insert-node drag state and capture the
pointer.
`RBX-M2-460` then moved searcher pointer-down routing behind a retained-agnostic
`SearcherPointerDownCx` capability composed from the searcher arm seam and widget-tail
dismiss/finish seams. `searcher_activation/pointer_down.rs` is now source-policy gated as
retained-bridge-free support, with focused tests proving no-searcher side-effect-free behavior,
row arm/finish, outside dismiss/finish, and secondary-button dismiss/finish.
`RBX-M2-470` then moved searcher pointer-up routing behind the retained-agnostic
`SearcherReleaseCx` seam. `searcher_activation/pointer_up.rs` is now source-policy gated as
retained-bridge-free support, with focused tests proving non-left button ignore, no-searcher
pending-drag cleanup, row activation/finish, and outside dismiss/finish.
`RBX-M2-480` then moved the outer searcher activation pointer-down/up wrappers behind
`SearcherPointerDownCx` and `SearcherReleaseCx`. `searcher_activation.rs` is now source-policy
gated as retained-bridge-free support; the retained `EventCx` call site remains in `searcher.rs`
until the higher-level searcher event route is replaced or narrowed.
`RBX-M2-490` then moved searcher pointer move/wheel routing behind the retained-agnostic
`WidgetPaintInvalidationCx` seam. `searcher_pointer.rs`, `searcher_pointer/move_event.rs`, and
`searcher_pointer/wheel_event.rs` are now source-policy gated as retained-bridge-free support, with
focused tests proving no-searcher move/wheel no-op behavior, hover paint invalidation, repeated
hover no-op behavior, wheel scroll paint invalidation, boundary wheel consumption without paint,
and Ctrl-wheel pass-through.
`RBX-M2-500` then moved searcher key-down routing behind the retained-agnostic `SearcherInputCx`
seam. `searcher_input.rs`, `searcher_input/dispatch.rs`, and `searcher_input_query.rs` are now
source-policy gated as retained-bridge-free support; retained row activation I/O lives in the
adapter-only `searcher_input/activation_retained_cx.rs`. Focused tests prove Enter activation,
ArrowDown navigation, query update, Ctrl text pass-through, and no-searcher no-op behavior.
`RBX-M2-510` then moved the top-level searcher escape/key/pointer/wheel route wrapper behind the
retained-agnostic `SearcherCx` capability composed from the existing searcher pointer-down,
release, and input seams. `searcher.rs` is now source-policy gated as retained-bridge-free support,
while retained pointer/timer/capture, row activation, and widget-tail I/O stay in their existing
adapter-only implementations. Focused top-level route tests prove Escape dismiss/finish, Enter row
activation, and pointer-down row-drag arming without retained Cx types.
`RBX-M2-520` then moved context menu UI open/restore/dismiss/finish/invalidate helpers behind
retained-agnostic widget-tail seams plus a narrow `ContextMenuFocusCx` seam. `context_menu/ui.rs`
and `context_menu/ui/event.rs` are now source-policy gated as retained-bridge-free support;
retained focus-self I/O lives in `context_menu/ui/event_retained_cx.rs`. Focused tests prove
open/focus/finish, restore/finish without focus, dismiss/finish, no-menu dismiss no-op behavior,
and paint invalidation.
`RBX-M2-530` then moved context menu pointer-move routing behind the retained-agnostic
`WidgetPaintInvalidationCx` seam. `context_menu/key_navigation/pointer_move.rs` is now
source-policy gated as retained-bridge-free support, with focused tests proving no-menu no-op
behavior, hover update paint invalidation, and repeated-hover no-op invalidation behavior.
`RBX-M2-540` then moved context menu key-down routing behind the retained-agnostic
`ContextMenuKeyDownCx` seam. `context_menu/key_navigation.rs` and
`context_menu/key_navigation/key_down.rs` are now source-policy gated as retained-bridge-free
support; retained active-selection activation I/O lives in
the shared `context_menu/selection_activation/retained_cx.rs` adapter after `RBX-M2-550`.
Focused tests prove no-menu no-op behavior, ArrowDown navigation/finish, Enter activation/close,
Enter keep-open restore, typeahead, and Backspace typeahead pop behavior.
`RBX-M2-550` then moved shared context menu selection activation and pointer-down routing behind
retained-agnostic `ContextMenuSelectionActivationCx` / `ContextMenuPointerDownCx` seams.
`context_menu/selection_activation.rs` and `context_menu/selection_activation/pointer_down.rs` are
now source-policy gated as retained-bridge-free support; retained item execution I/O lives in
`context_menu/selection_activation/retained_cx.rs`. Focused tests prove no-menu no-op behavior,
left enabled-item activation and close, left disabled-item restore, left outside-menu close, and
right-button replacement-menu pass-through behavior.

## Next Task

Pick the next task from:

- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`

Recommended next implementation shape:

- Continue M2 by shrinking the RBX-M2-080 ledger. The retained controls widget is now gone; the
  retained toolbar widgets are gone; retained minimap is gone; retained blackboard is gone; retained
  rename host is gone; retained diagnostics anchors are gone; retained a11y active-descendant
  anchors are gone; retained portal host/oracle code is gone; retained editor/panel composition
  wrappers are gone; retained overlay helper tails are gone; retained middleware event/command
  hooks are gone; retained canvas tail actions now have a retained-agnostic seam; the wire-drag
  commit Cx seam is retained-agnostic; pointer-up finish cleanup now uses the same tail seam; and
  sticky-wire finish now uses the handled release-capture tail seam; edge-insert drag move finish
  now uses the paint invalidation tail seam; cancel cleanup finish now uses the handled
  release-capture tail seam; sticky-wire target picker now uses a retained-agnostic host/window
  seam; group drag/resize preview move handlers now use a retained-agnostic host/bounds seam;
  pending group drag activation now uses a retained-agnostic host seam, while pending group resize
  activation no longer carries a Cx parameter; pending group/node release helpers now use the
  retained-agnostic release-capture tail seam; pending wire release now uses the same
  retained-agnostic release-capture tail seam; pending node drag click-select release now uses a
  retained-agnostic host/release tail seam; group drag/group resize/node resize pointer-up commit
  now uses a retained-agnostic host/window/release seam; node drag move tail now uses a
  retained-agnostic host/paint invalidation seam; marquee begin/finish now use a retained-agnostic
  host/capture/release seam; node drag preview compute now uses a retained-agnostic host/graph-read
  seam; node drag snaplines and multi-drag extent helpers now use a retained-agnostic
  host/geometry-read seam; keyboard pan activation now uses retained-agnostic widget tail seams;
  clipboard feedback, toast timer, and timer-motion helpers now use retained-agnostic
  feedback/paint seams; pending node resize move no longer takes a retained Cx parameter; edge
  double-click finish now uses the retained-agnostic handled tail seam; searcher dismiss release,
  finish, and paint invalidation now use retained-agnostic widget tail seams; searcher row-drag
  release coordination now uses a retained-agnostic release seam; searcher row-drag arming now
  uses a retained-agnostic arm seam; searcher pointer-down routing now uses a retained-agnostic
  pointer-down seam; searcher pointer-up routing now uses the retained-agnostic release seam; the
  outer searcher activation pointer-down/up wrapper now uses those retained-agnostic seams too;
  searcher pointer move/wheel routing now uses the retained-agnostic paint invalidation seam;
  searcher key-down routing now uses the retained-agnostic input seam; the top-level searcher route
  wrapper now composes those seams through `SearcherCx`; context menu UI open/restore/dismiss/
  finish/invalidate tails now use retained-agnostic widget-tail seams plus `ContextMenuFocusCx`;
  context menu pointer-move routing now uses the retained-agnostic paint invalidation seam;
  context menu key-down routing now uses the retained-agnostic key-down activation seam; context
  menu selection activation and pointer-down routing now use retained-agnostic selection activation
  seams.
  The remaining retained bridge source ledger is still the canvas widget root and `canvas/widget/**`,
  but `canvas/middleware.rs`, `widget_tail.rs`, `paint_invalidation.rs`, `redraw_request.rs`,
  `wire_drag/commit_cx.rs`, `pointer_up_finish.rs`, `pointer_up_session/cleanup.rs`,
  `pointer_up_session/release.rs`, `pointer_up_pending/release.rs`,
  `pointer_up_pending/release/group.rs`, `pointer_up_pending/release/node.rs`,
  `pointer_up_pending/wire_drag.rs`, `pointer_up_pending/click_select.rs`,
  `pointer_up_commit_cx.rs`, `pointer_up_commit/group_drag.rs`, `pointer_up_commit/resize.rs`,
  `pointer_up_commit/resize/group.rs`, `pointer_up_commit/resize/node.rs`,
  `node_drag_constraints.rs`, `node_drag_constraints_extent.rs`, `node_drag_geometry_cx.rs`,
  `node_drag_snap.rs`,
  `node_drag/tail.rs`, `node_drag_move_tail_cx.rs`,
  `node_drag_preview.rs`, `node_drag_preview/compute.rs`, `node_drag_preview_cx.rs`,
  `marquee_begin.rs`, `marquee_cx.rs`, `marquee_finish.rs`,
  `keyboard_pan_activation.rs`,
  `event_clipboard_feedback.rs`, `event_clipboard_feedback_cx.rs`, `event_timer_toast.rs`,
  `timer_motion_shared.rs`,
  `pending_resize.rs`,
  `pointer_down_double_click_edge/finish.rs`,
  `searcher_activation.rs`, `searcher_activation/pointer_down.rs`,
  `searcher_activation/pointer_up.rs`, `searcher_activation_state/arm.rs`,
  `searcher_activation_state/clear.rs`, `searcher_activation_state/release.rs`,
  `searcher.rs`,
  `searcher_input.rs`, `searcher_input/dispatch.rs`, `searcher_input_query.rs`,
  `searcher_pointer.rs`, `searcher_pointer/move_event.rs`, `searcher_pointer/wheel_event.rs`,
  `searcher_ui.rs`, `searcher_ui/event.rs`,
  `context_menu/ui.rs`, `context_menu/ui/event.rs`,
  `context_menu/key_navigation.rs`, `context_menu/key_navigation/key_down.rs`,
  `context_menu/key_navigation/pointer_move.rs`,
  `context_menu/selection_activation.rs`, `context_menu/selection_activation/pointer_down.rs`,
  `sticky_wire_connect/finish.rs`, `edge_insert_drag/drag/tail.rs`, `cancel_cleanup.rs`,
  `sticky_wire_targets/picker.rs`, `group_drag/tail.rs`, `group_resize/tail.rs`,
  `group_preview_move_cx.rs`, `group_drag.rs`, `group_resize.rs`,
  `pending_group_activation_cx.rs`, `pending_node_drag_release_cx.rs`, `pending_group_drag.rs`, and
  `pending_group_resize.rs` are compat-gated retained-bridge-free support. The remaining
  canvas interaction families still need default-path tests before their retained widget/event code
  can be deleted. Each slice should first add default declarative tests or retained-agnostic seams,
  then remove or gate less retained code.
- After the ledger no longer contains behavior-only retained files, remove
  `compat-retained-canvas` / `unstable-retained-bridge` from `fret-node`.
- Keep the known independent `fret-ui` layout primitive drift
  (`chrome-container-stretch-keeps-outer-box`) separate from retained-bridge exit unless a future
  slice touches that layout path directly.

## Gates

Last run on 2026-05-21 for `RBX-M2-550`:

- `cargo check -p fret-node --features compat-retained-canvas` - passed with the pre-existing
  `fret-ui` `current_effective_opacity` dead-code warning.
- `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(context_menu_selection_activation_route_stays_off_retained_bridge) | test(context_menu_key_down_route_stays_off_retained_bridge) | test(pointer_down_without_context_menu_is_side_effect_free) | test(pointer_down_left_inside_enabled_item_activates_and_closes_menu) | test(pointer_down_left_disabled_item_restores_menu_and_finishes) | test(pointer_down_left_outside_menu_closes_menu_and_finishes) | test(pointer_down_right_button_leaves_menu_taken_and_unfinished) | test(key_down_enter_activates_active_item_and_closes_menu) | test(key_down_enter_keep_open_restores_menu_and_finishes) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'` -
  passed, 10 tests.
- `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/context_menu/selection_activation.rs ecosystem/fret-node/src/ui/canvas/widget/context_menu/selection_activation/pointer_down.rs ecosystem/fret-node/src/ui/canvas/widget/context_menu/key_navigation.rs ecosystem/fret-node/src/ui/canvas/widget/context_menu/key_navigation/key_down.rs` -
  no matches.
- `cargo fmt --check` - passed.
- `python3 tools/check_layering.py` - passed.
- `python3 tools/check_workstream_catalog.py` - passed; validated 428 dedicated directories and 47
  standalone markdown files.
- `git diff --check` - passed.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-550` is a narrow adapter-boundary slice in `fret-node`'s retained canvas
    widget. The compat compile gate, targeted compat nextest gate, source-policy scan, formatting,
    layering, catalog, and whitespace checks cover the changed surface.

Previous run on 2026-05-20 for `RBX-M2-370`:

- `cargo check -p fret-node --features compat-retained-canvas` - passed with the pre-existing
  `fret-ui` `current_effective_opacity` dead-code warning.
- `cargo nextest run -p fret-node --features compat-retained-canvas node_drag_geometry_helpers_stay_off_retained_bridge node_drag_move_emits_on_node_drag node_drag_respects_per_node_extent_rect multi_node_drag_clamps_by_selection_bounds_in_node_extent_rect child_node_drag_is_clamped_to_group_when_expand_parent_is_false child_node_drag_expands_group_when_expand_parent_is_true snap_delta_for_rects_snaps_left_edge snap_delta_for_rects_snaps_center_y retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound` -
  passed, 10 tests.
- `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/node_drag_constraints.rs ecosystem/fret-node/src/ui/canvas/widget/node_drag_constraints_extent.rs ecosystem/fret-node/src/ui/canvas/widget/node_drag_geometry_cx.rs ecosystem/fret-node/src/ui/canvas/widget/node_drag_snap.rs` -
  no matches.
- `cargo fmt --check` - passed.
- `python3 tools/check_layering.py` - passed.
- `python3 tools/check_workstream_catalog.py` - passed; validated 428 dedicated directories and 47
  standalone markdown files.
- `git diff --check` - passed.

Previous run on 2026-05-20 for `RBX-M2-360`:

- `cargo check -p fret-node --features compat-retained-canvas` - passed with the pre-existing
  `fret-ui` `current_effective_opacity` dead-code warning.
- `cargo nextest run -p fret-node --features compat-retained-canvas node_drag_preview_compute_stays_off_retained_bridge node_drag_move_emits_on_node_drag child_node_drag_is_clamped_to_group_when_expand_parent_is_false child_node_drag_expands_group_when_expand_parent_is_true node_drag_records_single_history_entry_for_multi_node_move retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound` -
  passed, 7 tests.
- `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/node_drag_preview.rs ecosystem/fret-node/src/ui/canvas/widget/node_drag_preview/compute.rs ecosystem/fret-node/src/ui/canvas/widget/node_drag_preview_cx.rs` -
  no matches.
- `cargo fmt --check` - passed.
- `python3 tools/check_layering.py` - passed.
- `python3 tools/check_workstream_catalog.py` - passed; validated 428 dedicated directories and 47
  standalone markdown files.
- `git diff --check` - passed.

Previous run on 2026-05-20 for `RBX-M2-350`:

- `cargo check -p fret-node --features compat-retained-canvas` - passed with the pre-existing
  `fret-ui` `current_effective_opacity` dead-code warning.
- `cargo nextest run -p fret-node --features compat-retained-canvas marquee_begin_finish_stays_off_retained_bridge background_click_starts_pending_marquee_and_clears_selection_on_up marquee_replace_mode_replaces_selection_even_with_ctrl_pressed marquee_selects_connected_edges_for_selected_nodes marquee_selects_connected_edges_for_selected_nodes_with_store retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound` -
  passed, 7 tests.
- `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/marquee_begin.rs ecosystem/fret-node/src/ui/canvas/widget/marquee_cx.rs ecosystem/fret-node/src/ui/canvas/widget/marquee_finish.rs` -
  no matches.
- `cargo fmt --check` - passed.
- `python3 tools/check_layering.py` - passed.
- `python3 tools/check_workstream_catalog.py` - passed; validated 428 dedicated directories and 47
  standalone markdown files.
- `git diff --check` - passed.

Previous run on 2026-05-20 for `RBX-M2-340`:

- `cargo check -p fret-node --features compat-retained-canvas` - passed with the pre-existing
  `fret-ui` `current_effective_opacity` dead-code warning.
- `cargo nextest run -p fret-node --features compat-retained-canvas node_drag_move_tail_stays_off_retained_bridge node_drag_move_emits_on_node_drag child_node_drag_is_clamped_to_group_when_expand_parent_is_false child_node_drag_expands_group_when_expand_parent_is_true node_drag_records_single_history_entry_for_multi_node_move retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound` -
  passed, 7 tests.
- `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/node_drag/tail.rs ecosystem/fret-node/src/ui/canvas/widget/node_drag_move_tail_cx.rs` -
  no matches.
- `cargo fmt --check` - passed.
- `python3 tools/check_layering.py` - passed.
- `python3 tools/check_workstream_catalog.py` - passed; validated 428 dedicated directories and 47
  standalone markdown files.
- `git diff --check` - passed.

Previous run on 2026-05-20 for `RBX-M2-330`:

- `cargo check -p fret-node --features compat-retained-canvas` - passed with the pre-existing
  `fret-ui` `current_effective_opacity` dead-code warning.
- `cargo nextest run -p fret-node --features compat-retained-canvas pointer_up_commit_handlers_stay_off_retained_bridge build_group_drag_ops_includes_group_and_moved_nodes_only build_node_resize_ops_collects_node_and_group_changes node_resize_expands_group_when_expand_parent_is_true group_resize_is_previewed_and_committed_on_pointer_up group_resize_clamps_to_children retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound` -
  passed, 8 tests.
- `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/pointer_up_commit_cx.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_up_commit/group_drag.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_up_commit/resize.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_up_commit/resize/group.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_up_commit/resize/node.rs` -
  no matches.
- `cargo fmt --check` - passed.
- `python3 tools/check_layering.py` - passed.
- `python3 tools/check_workstream_catalog.py` - passed; validated 428 dedicated directories and 47
  standalone markdown files.
- `git diff --check` - passed.

Previous run on 2026-05-20 for `RBX-M2-320`:

- `cargo check -p fret-node --features compat-retained-canvas` - passed with the pre-existing
  `fret-ui` `current_effective_opacity` dead-code warning.
- `cargo nextest run -p fret-node --features compat-retained-canvas pending_node_drag_release_handlers_stay_off_retained_bridge apply_pending_node_selection_toggles_selection_and_keeps_node_last_in_draw_order shift_clicking_a_node_does_not_clear_selection node_click_does_not_select_node_when_node_selectable_is_false retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound` -
  passed, 6 tests.
- `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/pointer_up_pending/click_select.rs ecosystem/fret-node/src/ui/canvas/widget/pending_node_drag_release_cx.rs` -
  no matches.
- `cargo fmt --check` - passed.
- `python3 tools/check_layering.py` - passed.
- `python3 tools/check_workstream_catalog.py` - passed; validated 428 dedicated directories and 47
  standalone markdown files.
- `git diff --check` - passed.

Previous run on 2026-05-20 for `RBX-M2-310`:

- `cargo check -p fret-node --features compat-retained-canvas` - passed with the pre-existing
  `fret-ui` `current_effective_opacity` dead-code warning.
- `cargo nextest run -p fret-node --features compat-retained-canvas retained_canvas_tail_policy_helpers_stay_off_retained_bridge should_promote_pending_wire_drag_requires_click_connect_and_new_drag click_connect_target_port_click_commits_wire_and_clears_click_connect_state retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound` -
  passed, 5 tests.
- `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/pointer_up_pending/wire_drag.rs` -
  no matches.
- `cargo fmt --check` - passed.
- `python3 tools/check_layering.py` - passed.
- `python3 tools/check_workstream_catalog.py` - passed; validated 428 dedicated directories and 47
  standalone markdown files.
- `git diff --check` - passed.

Previous run on 2026-05-20 for `RBX-M2-300`:

- `cargo check -p fret-node --features compat-retained-canvas` - passed with the pre-existing
  `fret-ui` `current_effective_opacity` dead-code warning.
- `cargo nextest run -p fret-node --features compat-retained-canvas retained_canvas_tail_policy_helpers_stay_off_retained_bridge pending_group_drag_release_clears_session_without_committing pending_group_resize_release_clears_session_without_committing pending_group_activation_handlers_stay_off_retained_bridge retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound` -
  passed, 6 tests.
- `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/pointer_up_session/release.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_up_pending/release.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_up_pending/release/group.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_up_pending/release/node.rs` -
  no matches.
- `cargo fmt --check` - passed.
- `python3 tools/check_layering.py` - passed.
- `python3 tools/check_workstream_catalog.py` - passed; validated 428 dedicated directories and 47
  standalone markdown files.
- `git diff --check` - passed.

Previous run on 2026-05-20 for `RBX-M2-290`:

- `cargo check -p fret-node --features compat-retained-canvas` - passed with the pre-existing
  `fret-ui` `current_effective_opacity` dead-code warning.
- `cargo nextest run -p fret-node --features compat-retained-canvas pending_group_activation_handlers_stay_off_retained_bridge group_preview_move_handlers_stay_off_retained_bridge pending_group_drag_release_clears_session_without_committing pending_group_resize_release_clears_session_without_committing group_header_click_selects_group_and_arms_pending_group_drag group_resize_is_previewed_and_committed_on_pointer_up group_resize_clamps_to_children retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound` -
  passed, 9 tests.
- `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/pending_group_drag.rs ecosystem/fret-node/src/ui/canvas/widget/pending_group_resize.rs ecosystem/fret-node/src/ui/canvas/widget/pending_group_activation_cx.rs` -
  no matches.
- `cargo fmt -p fret-node` - passed.
- `cargo fmt --check` - passed.
- `python3 tools/check_layering.py` - passed.
- `python3 tools/check_workstream_catalog.py` - passed; validated 428 dedicated directories and 47
  standalone markdown files.
- `git diff --check` - passed.

Previous run on 2026-05-20 for `RBX-M2-280`:

- `cargo check -p fret-node --features compat-retained-canvas` - passed with the pre-existing
  `fret-ui` `current_effective_opacity` dead-code warning.
- `cargo nextest run -p fret-node --features compat-retained-canvas group_preview_move_handlers_stay_off_retained_bridge update_drag_preview_state update_resize_preview_state group_resize_is_previewed_and_committed_on_pointer_up group_resize_clamps_to_children retained_canvas_tail_policy_helpers_stay_off_retained_bridge retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound` -
  passed, 10 tests.
- `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/group_drag.rs ecosystem/fret-node/src/ui/canvas/widget/group_resize.rs ecosystem/fret-node/src/ui/canvas/widget/group_preview_move_cx.rs` -
  no matches.
- `cargo fmt -p fret-node` - passed.
- `cargo fmt --check` - passed.
- `python3 tools/check_layering.py` - passed.
- `python3 tools/check_workstream_catalog.py` - passed; validated 428 dedicated directories and 47
  standalone markdown files.
- `git diff --check` - passed.

Previous run on 2026-05-20 for `RBX-M2-270`:

- `cargo check -p fret-node --features compat-retained-canvas` - passed with the pre-existing
  `fret-ui` `current_effective_opacity` dead-code warning.
- `cargo nextest run -p fret-node --features compat-retained-canvas update_drag_preview_state update_resize_preview_state group_resize_is_previewed_and_committed_on_pointer_up group_resize_clamps_to_children retained_canvas_tail_policy_helpers_stay_off_retained_bridge retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound` -
  passed, 9 tests.
- `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/paint_invalidation.rs ecosystem/fret-node/src/ui/canvas/widget/redraw_request.rs ecosystem/fret-node/src/ui/canvas/widget/widget_tail.rs ecosystem/fret-node/src/ui/canvas/widget/wire_drag/commit_cx.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_up_finish.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_up_session/cleanup.rs ecosystem/fret-node/src/ui/canvas/widget/sticky_wire_connect/finish.rs ecosystem/fret-node/src/ui/canvas/widget/edge_insert_drag/drag/tail.rs ecosystem/fret-node/src/ui/canvas/widget/cancel_cleanup.rs ecosystem/fret-node/src/ui/canvas/widget/sticky_wire_targets/picker.rs ecosystem/fret-node/src/ui/canvas/widget/group_drag/tail.rs ecosystem/fret-node/src/ui/canvas/widget/group_resize/tail.rs` -
  no matches.
- `cargo fmt -p fret-node` - passed.
- `cargo fmt --check` - passed.
- `python3 tools/check_layering.py` - passed.
- `python3 tools/check_workstream_catalog.py` - passed; validated 428 dedicated directories and 47
  standalone markdown files.
- `git diff --check` - passed.

Previous run on 2026-05-20 for `RBX-M2-260`:

- `cargo check -p fret-node --features compat-retained-canvas` - passed with the pre-existing
  `fret-ui` `current_effective_opacity` dead-code warning.
- `cargo nextest run -p fret-node --features compat-retained-canvas finish_sticky_wire_target_picker_stops_and_invalidates_paint retained_canvas_tail_policy_helpers_stay_off_retained_bridge retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound` -
  passed, 4 tests.
- `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/paint_invalidation.rs ecosystem/fret-node/src/ui/canvas/widget/redraw_request.rs ecosystem/fret-node/src/ui/canvas/widget/widget_tail.rs ecosystem/fret-node/src/ui/canvas/widget/wire_drag/commit_cx.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_up_finish.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_up_session/cleanup.rs ecosystem/fret-node/src/ui/canvas/widget/sticky_wire_connect/finish.rs ecosystem/fret-node/src/ui/canvas/widget/edge_insert_drag/drag/tail.rs ecosystem/fret-node/src/ui/canvas/widget/cancel_cleanup.rs ecosystem/fret-node/src/ui/canvas/widget/sticky_wire_targets/picker.rs` -
  no matches.
- `cargo fmt -p fret-node` - passed.
- `cargo fmt --check` - passed.
- `python3 tools/check_layering.py` - passed.
- `python3 tools/check_workstream_catalog.py` - passed; validated 428 dedicated directories and 47
  standalone markdown files.
- `git diff --check` - passed.

Previous run on 2026-05-20 for `RBX-M2-250`:

- `cargo check -p fret-node --features compat-retained-canvas` - passed with the pre-existing
  `fret-ui` `current_effective_opacity` dead-code warning.
- `cargo nextest run -p fret-node --features compat-retained-canvas finish_cancel retained_canvas_tail_policy_helpers_stay_off_retained_bridge escape_cancel_releases_pointer_capture_during_panning escape_cancel_emits_connect_end_canceled escape_cancel_panning_emits_move_end_canceled node_drag_start_and_escape_cancel_emits_node_drag_end_canceled retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound` -
  passed, 9 tests.
- `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/paint_invalidation.rs ecosystem/fret-node/src/ui/canvas/widget/redraw_request.rs ecosystem/fret-node/src/ui/canvas/widget/widget_tail.rs ecosystem/fret-node/src/ui/canvas/widget/wire_drag/commit_cx.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_up_finish.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_up_session/cleanup.rs ecosystem/fret-node/src/ui/canvas/widget/sticky_wire_connect/finish.rs ecosystem/fret-node/src/ui/canvas/widget/edge_insert_drag/drag/tail.rs ecosystem/fret-node/src/ui/canvas/widget/cancel_cleanup.rs` -
  no matches.
- `cargo fmt -p fret-node` - passed.
- `cargo fmt --check` - passed.
- `python3 tools/check_layering.py` - passed.
- `python3 tools/check_workstream_catalog.py` - passed; validated 428 dedicated directories and 47
  standalone markdown files.
- `git diff --check` - passed.

Previous run on 2026-05-20 for `RBX-M2-240`:

- `cargo check -p fret-node --features compat-retained-canvas` - passed with the pre-existing
  `fret-ui` `current_effective_opacity` dead-code warning.
- `cargo nextest run -p fret-node --features compat-retained-canvas retained_canvas_tail_policy_helpers_stay_off_retained_bridge finish_edge_insert_drag_move_invalidates_paint retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound` -
  passed, 4 tests.
- `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/paint_invalidation.rs ecosystem/fret-node/src/ui/canvas/widget/redraw_request.rs ecosystem/fret-node/src/ui/canvas/widget/widget_tail.rs ecosystem/fret-node/src/ui/canvas/widget/wire_drag/commit_cx.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_up_finish.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_up_session/cleanup.rs ecosystem/fret-node/src/ui/canvas/widget/sticky_wire_connect/finish.rs ecosystem/fret-node/src/ui/canvas/widget/edge_insert_drag/drag/tail.rs` -
  no matches.
- `cargo fmt -p fret-node` - passed.
- `cargo fmt --check` - passed.
- `python3 tools/check_layering.py` - passed.
- `python3 tools/check_workstream_catalog.py` - passed; validated 428 dedicated directories and 47
  standalone markdown files.
- `git diff --check` - passed.

Previous run on 2026-05-20 for `RBX-M2-230`:

- `cargo check -p fret-node --features compat-retained-canvas` - passed with the pre-existing
  `fret-ui` `current_effective_opacity` dead-code warning.
- `cargo nextest run -p fret-node --features compat-retained-canvas retained_canvas_tail_policy_helpers_stay_off_retained_bridge widget_tail retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound` -
  passed, 7 tests.
- `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/paint_invalidation.rs ecosystem/fret-node/src/ui/canvas/widget/redraw_request.rs ecosystem/fret-node/src/ui/canvas/widget/widget_tail.rs ecosystem/fret-node/src/ui/canvas/widget/wire_drag/commit_cx.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_up_finish.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_up_session/cleanup.rs ecosystem/fret-node/src/ui/canvas/widget/sticky_wire_connect/finish.rs` -
  no matches.
- `cargo fmt -p fret-node` - passed.
- `cargo fmt --check` - passed.
- `python3 tools/check_layering.py` - passed.
- `python3 tools/check_workstream_catalog.py` - passed; validated 428 dedicated directories and 47
  standalone markdown files.
- `git diff --check` - passed.

Previous run on 2026-05-20 for `RBX-M2-220`:

- `cargo check -p fret-node --features compat-retained-canvas` - passed with the pre-existing
  `fret-ui` `current_effective_opacity` dead-code warning.
- `cargo nextest run -p fret-node --features compat-retained-canvas retained_canvas_tail_policy_helpers_stay_off_retained_bridge widget_tail retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound` -
  passed, 6 tests.
- `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/paint_invalidation.rs ecosystem/fret-node/src/ui/canvas/widget/redraw_request.rs ecosystem/fret-node/src/ui/canvas/widget/widget_tail.rs ecosystem/fret-node/src/ui/canvas/widget/wire_drag/commit_cx.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_up_finish.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_up_session/cleanup.rs` -
  no matches.
- `cargo fmt -p fret-node` - passed.
- `cargo fmt --check` - passed.
- `python3 tools/check_layering.py` - passed.
- `python3 tools/check_workstream_catalog.py` - passed; validated 428 dedicated directories and 47
  standalone markdown files.
- `git diff --check` - passed.

Previous run on 2026-05-20 for `RBX-M2-210`:

- `cargo check -p fret-node --features compat-retained-canvas` - passed with the pre-existing
  `fret-ui` `current_effective_opacity` dead-code warning.
- `cargo nextest run -p fret-node --features compat-retained-canvas retained_canvas_tail_policy_helpers_stay_off_retained_bridge widget_tail commit_cx retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound` -
  passed, 6 tests.
- `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/paint_invalidation.rs ecosystem/fret-node/src/ui/canvas/widget/redraw_request.rs ecosystem/fret-node/src/ui/canvas/widget/widget_tail.rs ecosystem/fret-node/src/ui/canvas/widget/wire_drag/commit_cx.rs` -
  no matches.
- `cargo fmt -p fret-node` - passed.
- `cargo fmt --check` - passed.
- `python3 tools/check_layering.py` - passed.
- `python3 tools/check_workstream_catalog.py` - passed; validated 428 dedicated directories and 47
  standalone markdown files.
- `git diff --check` - passed.

Previous run on 2026-05-20 for `RBX-M2-200`:

- `cargo check -p fret-node --features compat-retained-canvas` - passed with the pre-existing
  `fret-ui` `current_effective_opacity` dead-code warning.
- `cargo nextest run -p fret-node retained_canvas_tail_policy_helpers_stay_off_retained_bridge retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound` -
  passed, 3 tests.
- `cargo nextest run -p fret-node --features compat-retained-canvas widget_tail retained_canvas_tail_policy_helpers_stay_off_retained_bridge retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound` -
  passed, 5 tests.
- `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/paint_invalidation.rs ecosystem/fret-node/src/ui/canvas/widget/redraw_request.rs ecosystem/fret-node/src/ui/canvas/widget/widget_tail.rs` -
  no matches.
- `cargo fmt -p fret-node` - passed.
- `cargo fmt --check` - passed.
- `python3 tools/check_layering.py` - passed.
- `python3 tools/check_workstream_catalog.py` - passed; validated 428 dedicated directories and 47
  standalone markdown files.
- `git diff --check` - passed.

Previous run on 2026-05-20 for `RBX-M2-190`:

- pre-delete `cargo nextest run -p fret-node --features compat-retained-canvas middleware_can_override_select_all_command middleware_can_reject_commits_before_apply retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound` -
  passed, 4 tests.
- post-delete `cargo check -p fret-node --features compat-retained-canvas` - passed with the
  pre-existing `fret-ui` `current_effective_opacity` dead-code warning.
- post-delete `cargo nextest run -p fret-node retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound` -
  passed, 2 tests.
- post-delete `cargo nextest run -p fret-node --features compat-retained-canvas retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound middleware_can_reject_commits_before_apply` -
  passed, 3 tests.
- post-delete `rg -n "retained_bridge|CommandCx|EventCx|NodeGraphCanvasCommandOutcome|NodeGraphCanvasEventOutcome|handle_event\\(|handle_command\\(" ecosystem/fret-node/src/ui/canvas/middleware.rs ecosystem/fret-node/src/ui/canvas/middleware -g '*.rs'` -
  no matches.

Previous run on 2026-05-20 for `RBX-M2-180`:

- pre-delete `cargo nextest run -p fret-node --features compat-retained-canvas sync_panel_hover_only_reports_real_changes release_panel_press_only_activates_on_matching_release_target centered_text_origin_centers_within_button_rect leading_text_origin_keeps_padding_and_vertical_centering retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound` -
  passed, 6 tests.
- pre-delete `rg -n "\b(retained_submit|submit_graph_transaction|submit_graph_and_view_transaction|retained_event_tail|request_paint_repaint|finish_paint_event|focus_canvas_and_finish_paint_event|focus_canvas_and_finish_layout_event|finish_portal_command|begin_panel_press|paint_panel_button|paint_panel_label|centered_text_origin|leading_text_origin)\b" ecosystem/fret-node/src apps crates ecosystem tools --glob '!target/**' --glob '!ecosystem/fret-node/src/lib.rs' --glob '!ecosystem/fret-node/src/ui/retained_submit.rs' --glob '!ecosystem/fret-node/src/ui/retained_event_tail.rs' --glob '!ecosystem/fret-node/src/ui/overlays/panel_button_paint.rs' --glob '!ecosystem/fret-node/src/ui/overlays/panel_pointer_policy.rs'` -
  no live consumers outside module entries and deleted/self files.
- post-delete `cargo check -p fret-node --features compat-retained-canvas` - passed with the
  pre-existing `fret-ui` `current_effective_opacity` dead-code warning.
- post-delete `cargo nextest run -p fret-node sync_panel_hover_only_reports_real_changes release_panel_press_only_activates_on_matching_release_target retained_bridge_source_usage_stays_on_the_migration_ledger default_overlay_policy_surfaces_stay_off_retained_bridge retained_widget_compat_island_stays_crate_private_and_controller_bound` -
  passed, 5 tests.
- post-delete `cargo nextest run -p fret-node --features compat-retained-canvas sync_panel_hover_only_reports_real_changes release_panel_press_only_activates_on_matching_release_target retained_bridge_source_usage_stays_on_the_migration_ledger default_overlay_policy_surfaces_stay_off_retained_bridge retained_widget_compat_island_stays_crate_private_and_controller_bound` -
  passed, 5 tests.
- post-delete `rg -n "retained_bridge|UiTreeRetainedExt|RetainedSubtreeProps|use fret_ui::retained_bridge|fret_ui::retained_bridge::" ecosystem/fret-node/src/ui/overlays -g '*.rs'` -
  no matches.

Previous run on 2026-05-20 for `RBX-M2-170`:

- no-user `rg -n "\b(NodeGraphEditor|NodeGraphPanel|NodeGraphPanelPosition|NodeGraphPanelSize)\b" ecosystem/fret-node/src apps crates ecosystem tools --glob '!target/**' --glob '!ecosystem/fret-node/src/lib.rs'` -
  no matches.
- pre-delete `cargo nextest run -p fret-node --features compat-retained-canvas positioned_rect_top_right_respects_margin rect_in_bounds_top_right_respects_margin retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound` -
  passed, 4 tests.
- post-delete `cargo nextest run -p fret-node rect_in_bounds_top_right_respects_margin retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound` -
  passed, 3 tests.
- post-delete `cargo nextest run -p fret-node --features compat-retained-canvas retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound` -
  passed, 2 tests.
- post-delete `cargo check -p fret-node --features compat-retained-canvas` - passed with the
  pre-existing `fret-ui` `current_effective_opacity` dead-code warning.
- `cargo fmt -p fret-node` - passed.
- `cargo fmt --check` - passed.
- `python3 tools/check_layering.py` - passed.
- `python3 tools/check_workstream_catalog.py` - passed; validated 428 dedicated directories and 47
  standalone markdown files.
- `git diff --check` - passed.

Previous run on 2026-05-20 for `RBX-M2-160`:

- pre-delete `cargo nextest run -p fret-node --features compat-retained-canvas portal_lifecycle_conformance portal_measured_geometry_conformance portal_measured_internals_conformance declarative_visible_subset_portal_identity_persists_and_resets_on_kind_or_version_change flush_portal_measured_geometry_state_keeps_growth_only_and_removes_missing_nodes declarative_portal_renderer_hosts_custom_subtrees_by_node_kind_with_default_fallback declarative_surface_hosts_node_type_registry_without_retained_portal_host declarative_portal_renderer_publishes_custom_subtree_measurements declarative_portal_command_host_submits_transactions_without_retained_portal_host declarative_portal_text_editor_handler_submits_transactions_without_retained_command_cx declarative_portal_number_editor_handler_submits_transactions_without_retained_command_cx` -
  passed, 12 tests.
- post-delete `cargo check -p fret-node --features compat-retained-canvas` - passed with the
  pre-existing `fret-ui` `current_effective_opacity` dead-code warning.
- post-delete `cargo nextest run -p fret-node retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound` -
  passed, 2 tests.
- post-delete `cargo nextest run -p fret-node --features compat-retained-canvas retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound` -
  passed, 2 tests.
- post-delete `cargo nextest run -p fret-node declarative_visible_subset_portal_identity_persists_and_resets_on_kind_or_version_change flush_portal_measured_geometry_state_keeps_growth_only_and_removes_missing_nodes declarative_portal_renderer_hosts_custom_subtrees_by_node_kind_with_default_fallback declarative_surface_hosts_node_type_registry_without_retained_portal_host declarative_portal_renderer_publishes_custom_subtree_measurements declarative_portal_command_host_submits_transactions_without_retained_portal_host declarative_portal_text_editor_handler_submits_transactions_without_retained_command_cx declarative_portal_number_editor_handler_submits_transactions_without_retained_command_cx` -
  passed, 8 tests.
- `cargo fmt -p fret-node` - passed.
- `cargo fmt --check` - passed.
- `python3 tools/check_layering.py` - passed.
- `python3 tools/check_workstream_catalog.py` - passed; validated 428 dedicated directories and 47
  standalone markdown files.
- `git diff --check` - passed.

Previous run on 2026-05-20 for `RBX-M2-140`:

- `cargo nextest run -p fret-node declarative_portal_renderer_hosts_custom_subtrees_by_node_kind_with_default_fallback declarative_surface_hosts_node_type_registry_without_retained_portal_host declarative_portal_renderer_publishes_custom_subtree_measurements` -
  passed, 3 tests.
- `cargo nextest run -p fret-node --features compat-retained-canvas portal_lifecycle_conformance portal_measured_geometry_conformance portal_measured_internals_conformance declarative_portal_renderer_hosts_custom_subtrees_by_node_kind_with_default_fallback declarative_surface_hosts_node_type_registry_without_retained_portal_host declarative_portal_renderer_publishes_custom_subtree_measurements` -
  passed, 7 tests.

Previous run on 2026-05-20 for `RBX-M2-135`:

- `cargo nextest run -p fret-node declarative_visible_subset_portal_identity_persists_and_resets_on_kind_or_version_change flush_portal_measured_geometry_state_keeps_growth_only_and_removes_missing_nodes` -
  passed, 2 tests.
- `cargo nextest run -p fret-node --features compat-retained-canvas portal_lifecycle_conformance portal_measured_geometry_conformance portal_measured_internals_conformance declarative_visible_subset_portal_identity_persists_and_resets_on_kind_or_version_change flush_portal_measured_geometry_state_keeps_growth_only_and_removes_missing_nodes` -
  passed, 6 tests.

Previous run on 2026-05-20 for `RBX-M2-134`:

- deletion-preflight `cargo nextest run -p fret-node node_graph_surface_active_descendant` -
  passed, 2 tests.
- deletion-preflight `cargo nextest run -p fret-node --features compat-retained-canvas a11y_active_descendant_conformance node_graph_surface_active_descendant` -
  passed, 4 tests.
- post-delete `cargo nextest run -p fret-node node_graph_surface_active_descendant` -
  passed, 5 tests.
- post-delete `cargo nextest run -p fret-node node_graph_surface_active_descendant retained_bridge_source_usage_stays_on_the_migration_ledger` -
  passed, 6 tests.
- post-delete `cargo check -p fret-node --features compat-retained-canvas` - passed.
- post-delete `cargo nextest run -p fret-node --features compat-retained-canvas retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound node_graph_surface_active_descendant` -
  passed, 7 tests.
- `cargo fmt -p fret-node` - passed.
- `cargo fmt --check` - passed.
- `python3 tools/check_layering.py` - passed.
- `python3 tools/check_workstream_catalog.py` - passed; validated 427 dedicated directories and 47
  standalone markdown files.
- `git diff --check` - passed.
- `rg -n "^\\s*(pub\\s+)?mod a11y;|NodeGraphA11yActiveDescendant|NodeGraphA11yFocused|a11y_active_descendant_conformance" ecosystem/fret-node/src -S` -
  no matches.

Previous run on 2026-05-20 for `RBX-M2-133`:

- `cargo check -p fret-node --features compat-retained-canvas` - passed.
- `cargo nextest run -p fret-node --features compat-retained-canvas retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound` -
  passed, 2 tests.
- `rg -n "DiagnosticsAnchorPorts|diagnostics_anchor_ports|with_diagnostics_anchor_ports|retained_widget_layout_publish|publish_diagnostics_derived_outputs|NodeGraphDiagAnchor|NodeGraphDiagConnectingFlag|diag_anchors" ecosystem/fret-node/src docs/ui-diagnostics-and-scripted-tests.md docs/workstreams/retained-bridge-exit-v1 -g '*.rs' -g '*.md'` -
  only historical workstream references remain.

Earlier run on 2026-05-20 for `RBX-M2-132`:

- deletion-preflight `cargo nextest run -p fret-node --features compat-retained-canvas overlay_group_rename_conformance overlay_symbol_rename_conformance rename_declarative rename_lifecycle rename_command` -
  passed, 26 tests.
- deletion-preflight `cargo nextest run -p fret-node rename_declarative rename_lifecycle rename_command rename_host_layout` -
  passed, 19 tests.
- post-delete `cargo nextest run -p fret-node rename_declarative rename_lifecycle rename_command rename_host_layout` -
  passed, 19 tests.
- post-delete `cargo check -p fret-node --features compat-retained-canvas` - passed.
- post-delete `cargo nextest run -p fret-node --features compat-retained-canvas rename_declarative rename_lifecycle rename_command retained_bridge_source_usage_stays_on_the_migration_ledger default_overlay_policy_surfaces_stay_off_retained_bridge overlay_policy_modules_compile_without_retained_canvas_compat` -
  passed, 19 tests.
- post-delete `cargo nextest run -p fret-ui managed_surface` - passed, 9 tests.
- post-delete `cargo check -p fret-node --no-default-features --features fret-ui` - passed.
- `cargo fmt -p fret-node` - passed.
- `cargo fmt --check` - passed.
- `python3 tools/check_layering.py` - passed.
- `python3 tools/check_workstream_catalog.py` - passed; validated 427 dedicated directories and 47
  standalone markdown files.
- `git diff --check` - passed.
- `rg -n "NodeGraphOverlayHost|rename_host_event|overlay_group_rename_conformance|overlay_symbol_rename_conformance|layout_hidden_child_and_release_focus|src/ui/overlays/group_rename\\.rs" ecosystem/fret-node/src -g '*.rs'` -
  no matches.

Earlier run on 2026-05-20 for `RBX-M2-131`:

- `cargo nextest run -p fret-node rename_declarative rename_lifecycle rename_command rename_host_layout` -
  passed, 19 tests.
- `cargo nextest run -p fret-ui managed_surface` - passed, 9 tests.
- `cargo nextest run -p fret-node --features compat-retained-canvas overlay_group_rename_conformance overlay_symbol_rename_conformance rename_declarative rename_lifecycle rename_command` -
  passed, 26 tests.
- `cargo fmt --check` - passed.
- `git diff --check -- crates/fret-ui/src/managed_surface.rs ecosystem/fret-node/src/ui/overlays/rename_declarative.rs ecosystem/fret-node/src/ui/overlays/mod.rs ecosystem/fret-node/src/ui/overlays/rename_host_layout.rs ecosystem/fret-node/src/ui/overlays/rename_policy.rs` -
  passed.

Earlier run on 2026-05-20 for `RBX-M2-130`:

- deletion-preflight `cargo nextest run -p fret-node --features compat-retained-canvas overlay_blackboard_conformance blackboard_declarative blackboard_interaction_policy blackboard_paint_plan` -
  passed, 27 tests.
- `cargo nextest run -p fret-node blackboard_declarative blackboard_interaction_policy blackboard_paint_plan overlay_policy_modules_compile_without_retained_canvas_compat default_overlay_policy_surfaces_stay_off_retained_bridge retained_bridge_source_usage_stays_on_the_migration_ledger` -
  passed, 20 tests.
- `cargo nextest run -p fret-node --features compat-retained-canvas blackboard_declarative blackboard_interaction_policy blackboard_paint_plan retained_bridge_source_usage_stays_on_the_migration_ledger default_overlay_policy_surfaces_stay_off_retained_bridge` -
  passed, 19 tests.

Earlier run on 2026-05-20 for `RBX-M2-127`:

- `cargo check -p fret-node --features compat-retained-canvas` - passed.
- `cargo check -p fret-node --no-default-features --features fret-ui` - passed.
- deletion-preflight `cargo nextest run -p fret-node --features compat-retained-canvas minimap_declarative minimap_interaction_policy overlay_minimap_controls_conformance retained_bridge_source_usage_stays_on_the_migration_ledger default_overlay_policy_surfaces_stay_off_retained_bridge` -
  passed, 16 tests.
- `cargo nextest run -p fret-node controls_declarative controls_host_policy controls_interaction_policy minimap_declarative minimap_interaction_policy retained_bridge_source_usage_stays_on_the_migration_ledger default_overlay_policy_surfaces_stay_off_retained_bridge` -
  passed, 31 tests.
- `cargo nextest run -p fret-node --features compat-retained-canvas controls_declarative controls_host_policy controls_interaction_policy minimap_declarative minimap_interaction_policy retained_bridge_source_usage_stays_on_the_migration_ledger default_overlay_policy_surfaces_stay_off_retained_bridge` -
  passed, 31 tests.
- `rg -n "\\bNodeGraphMiniMapOverlay\\b|overlay_minimap_controls_conformance|src/ui/overlays/minimap\\.rs|include_str!\\(\\\"ui/overlays/minimap\\.rs\\\"\\)|mod minimap;|pub use minimap|MINIMAP_RS|minimap_navigation_surface_stays" ecosystem/fret-node/src -g '*.rs'` -
  no retained minimap widget/module/export/oracle matches; only declarative
  `NodeGraphMiniMapOverlayElementProps` names remain.
- `cargo fmt -p fret-node` - passed.
- `cargo fmt --check` - passed.
- `python3 tools/check_layering.py` - passed.
- `python3 tools/check_workstream_catalog.py` - passed; validated 427 dedicated directories and 47
  standalone markdown files.
- `git diff --check` - passed.

Earlier run on 2026-05-20 for `RBX-M2-126`:

- `cargo check -p fret-node --features compat-retained-canvas` - passed.
- `cargo check -p fret-node --no-default-features --features fret-ui` - passed.
- `cargo nextest run -p fret-node minimap_declarative` - passed, 5 tests.
- `cargo nextest run -p fret-node controls_declarative controls_host_policy controls_interaction_policy minimap_declarative minimap_interaction_policy retained_bridge_source_usage_stays_on_the_migration_ledger default_overlay_policy_surfaces_stay_off_retained_bridge` -
  passed, 31 tests.
- `cargo nextest run -p fret-node --features compat-retained-canvas controls_declarative controls_host_policy controls_interaction_policy minimap_declarative minimap_interaction_policy overlay_minimap_controls_conformance retained_bridge_source_usage_stays_on_the_migration_ledger default_overlay_policy_surfaces_stay_off_retained_bridge` -
  passed, 36 tests.
- `cargo fmt -p fret-ui -p fret-node` - passed.
- `cargo fmt --check` - passed.
- `python3 tools/check_layering.py` - passed.
- `python3 tools/check_workstream_catalog.py` - passed; validated 427 dedicated directories and 47
  standalone markdown files.
- `git diff --check` - passed.

Earlier run on 2026-05-20 for `RBX-M2-125`:

- `cargo check -p fret-node --features compat-retained-canvas` - passed.
- `cargo check -p fret-node --no-default-features --features fret-ui` - passed.
- `cargo nextest run -p fret-node controls_declarative controls_host_policy controls_interaction_policy retained_bridge_source_usage_stays_on_the_migration_ledger default_overlay_policy_surfaces_stay_off_retained_bridge` -
  passed, 22 tests.
- `cargo nextest run -p fret-node --features compat-retained-canvas controls_declarative controls_host_policy controls_interaction_policy overlay_minimap_controls_conformance retained_bridge_source_usage_stays_on_the_migration_ledger default_overlay_policy_surfaces_stay_off_retained_bridge` -
  passed, 27 tests.
- `rg -n "\\bNodeGraphControlsOverlay\\b|src/ui/overlays/controls\\.rs|include_str!\\(\\\"ui/overlays/controls\\.rs\\\"\\)|mod controls;|pub use controls::|controls_overlay_requires_explicit_editor_config_model|controls_overlay_" ecosystem/fret-node/src -g '*.rs'` -
  no retained controls widget/module/export matches; only declarative `node_graph_controls_overlay_element(...)` names remain.
- `cargo fmt -p fret-node` - passed.
- `cargo fmt --check` - passed.
- `python3 tools/check_layering.py` - passed.
- `python3 tools/check_workstream_catalog.py` - passed; validated 427 dedicated directories and 47
  standalone markdown files.
- `git diff --check` - passed.

Earlier run on 2026-05-20 for `RBX-M2-124`:

- `cargo nextest run -p fret-node controls_declarative_pointer_events_fall_through_outside_panel_to_surface controls_declarative_blocks_surface_input_within_panel_even_off_button controls_declarative_focus_traversal_reaches_controls_from_surface` -
  passed, 3 tests.
- `cargo nextest run -p fret-node controls_declarative controls_host_policy controls_interaction_policy overlay_minimap_controls_conformance` -
  passed, 20 tests.
- `cargo nextest run -p fret-node --features compat-retained-canvas controls_declarative controls_host_policy controls_interaction_policy overlay_minimap_controls_conformance` -
  passed, 35 tests.
- `cargo fmt -p fret-node` - passed.
- `cargo fmt --check` - passed.
- `python3 tools/check_layering.py` - passed.
- `python3 tools/check_workstream_catalog.py` - passed; validated 427 dedicated directories and 47
  standalone markdown files.
- `git diff --check` - passed.

Earlier run on 2026-05-20 for `RBX-M2-119`:

- `cargo nextest run -p fret-ui managed_surface` - passed, 7 tests.
- `cargo nextest run -p fret-node node_toolbar_declarative_host_auto_measures_and_places_child_without_retained_widget edge_toolbar_declarative_host_auto_measures_and_hides_child_without_retained_widget toolbars_declarative toolbar_layout_policy toolbar_policy overlay_policy_modules_compile_without_retained_canvas_compat` -
  passed, 16 tests.
- `cargo nextest run -p fret-node --features compat-retained-canvas node_toolbar_declarative_host_auto_measures_and_places_child_without_retained_widget edge_toolbar_declarative_host_auto_measures_and_hides_child_without_retained_widget toolbars_declarative toolbar_layout_policy overlay_toolbars_conformance` -
  passed, 13 tests.
- `cargo fmt` - passed.

Earlier run on 2026-05-20 for `RBX-M2-118`:

- `cargo nextest run -p fret-node controls_declarative_button_pointer_up_completes_capture_focus_and_command_dispatch controls_declarative_activation_dispatches_commands_and_honors_disabled_bindings controls_host_policy controls_interaction_policy` -
  passed, 9 tests.
- `cargo nextest run -p fret-node --features compat-retained-canvas controls_declarative_button_pointer_up_completes_capture_focus_and_command_dispatch controls_host_policy controls_interaction_policy controls_declarative overlay_minimap_controls_conformance` -
  passed, 27 tests.
- `cargo check -p fret-node --no-default-features --features fret-ui` - passed.
- `cargo check -p fret-node --features compat-retained-canvas` - passed.
- `cargo nextest run -p fret-node` - passed, 382 tests.
- `cargo nextest run -p fret-node --features compat-retained-canvas` - passed, 958 tests.
- `cargo fmt --check` - passed.
- `python3 tools/check_layering.py` - passed.

Earlier run on 2026-05-20 for `RBX-M2-117`:

- `cargo nextest run -p fret-node controls_declarative_panel_blank_pointer_down_focuses_overlay_without_command controls_host_policy controls_interaction_policy controls_declarative overlay_policy_modules_compile_without_retained_canvas_compat controls_overlay_requires_explicit_editor_config_model` -
  passed, 13 tests.
- `cargo nextest run -p fret-node --features compat-retained-canvas controls_host_policy controls_interaction_policy controls_declarative overlay_minimap_controls_conformance` -
  passed, 26 tests.
- `cargo check -p fret-node --no-default-features --features fret-ui` - passed.
- `cargo check -p fret-node --features compat-retained-canvas` - passed.
- `cargo nextest run -p fret-node` - passed, 381 tests.
- `cargo nextest run -p fret-node --features compat-retained-canvas` - passed, 957 tests.
- `cargo fmt --check` - passed.
- `python3 tools/check_layering.py` - passed.
- `python3 tools/check_workstream_catalog.py` - passed; validated 427 dedicated directories and 47
  standalone markdown files.
- `git diff --check` - passed.
- `out=$(git diff --check --no-index /dev/null ecosystem/fret-node/src/ui/overlays/controls_host_policy.rs 2>&1); test -z "$out"` -
  passed.

Earlier run on 2026-05-20 for `RBX-M2-116`:

- `cargo nextest run -p fret-node controls_paint_plan controls_layout controls_policy panel_item_state overlay_policy_modules_compile_without_retained_canvas_compat controls_overlay_requires_explicit_editor_config_model` -
  passed, 16 tests.
- `cargo nextest run -p fret-node --features compat-retained-canvas controls_paint_plan controls_declarative overlay_minimap_controls_conformance` -
  passed, 21 tests.
- `cargo check -p fret-node --no-default-features --features fret-ui` - passed.
- `cargo check -p fret-node --features compat-retained-canvas` - passed.
- `cargo nextest run -p fret-node` - passed, 377 tests.
- `cargo nextest run -p fret-node --features compat-retained-canvas` - passed, 953 tests.
- `cargo fmt --check` - passed.
- `python3 tools/check_layering.py` - passed.
- `python3 tools/check_workstream_catalog.py` - passed; validated 427 dedicated directories and 47
  standalone markdown files.
- `git diff --check` - passed.
- `out=$(git diff --check --no-index /dev/null ecosystem/fret-node/src/ui/overlays/controls_paint_plan.rs 2>&1); test -z "$out"` -
  passed.

Earlier run on 2026-05-20 for `RBX-M2-115`:

- `cargo nextest run -p fret-node blackboard_paint_plan blackboard_layout blackboard_policy panel_item_state overlay_policy_modules_compile_without_retained_canvas_compat` -
  passed, 14 tests.
- `cargo nextest run -p fret-node --features compat-retained-canvas blackboard_paint_plan blackboard_declarative overlay_blackboard_conformance blackboard_paint` -
  passed, 16 tests.
- `cargo check -p fret-node --no-default-features --features fret-ui` - passed.
- `cargo check -p fret-node --features compat-retained-canvas` - passed.
- `cargo nextest run -p fret-node` - passed, 374 tests.
- `cargo nextest run -p fret-node --features compat-retained-canvas` - passed, 950 tests.
- `cargo fmt --check` - passed.
- `python3 tools/check_layering.py` - passed.
- `python3 tools/check_workstream_catalog.py` - passed; validated 427 dedicated directories and 47
  standalone markdown files.
- `git diff --check` - passed.
- `out=$(git diff --check --no-index /dev/null ecosystem/fret-node/src/ui/overlays/blackboard_paint_plan.rs 2>&1); test -z "$out"` -
  passed.

Earlier run on 2026-05-20 for `RBX-M2-114`:

- `cargo nextest run -p fret-node blackboard_interaction_policy blackboard_declarative blackboard_layout blackboard_policy panel_navigation_policy panel_pointer_policy panel_item_state overlay_policy_modules_compile_without_retained_canvas_compat` -
  passed, 23 tests.
- `cargo nextest run -p fret-node --features compat-retained-canvas blackboard_interaction_policy blackboard_declarative overlay_blackboard_conformance` -
  passed, 17 tests.
- `cargo check -p fret-node --no-default-features --features fret-ui` - passed.
- `cargo check -p fret-node --features compat-retained-canvas` - passed.
- `cargo nextest run -p fret-node` - passed, 371 tests.
- `cargo nextest run -p fret-node --features compat-retained-canvas` - passed, 950 tests.
- `cargo fmt --check` - passed.
- `python3 tools/check_layering.py` - passed.
- `python3 tools/check_workstream_catalog.py` - passed; validated 427 dedicated directories and 47
  standalone markdown files.
- `git diff --check` - passed.
- `out=$(git diff --check --no-index /dev/null ecosystem/fret-node/src/ui/overlays/blackboard_interaction_policy.rs 2>&1); test -z "$out"` -
  passed.

Earlier run on 2026-05-19 for `RBX-M2-112`:

- `cargo nextest run -p fret-node toolbar_layout_policy toolbars_declarative toolbar_policy overlay_policy_modules_compile_without_retained_canvas_compat controls_overlay_requires_explicit_editor_config_model` -
  passed, 15 tests.
- `cargo nextest run -p fret-node --features compat-retained-canvas toolbar_layout_policy toolbars_declarative overlay_toolbars_conformance` -
  passed, 11 tests.
- `cargo check -p fret-node --no-default-features --features fret-ui` - passed.
- `cargo check -p fret-node --features compat-retained-canvas` - passed.
- `cargo nextest run -p fret-node` - passed, 363 tests.
- `cargo nextest run -p fret-node --features compat-retained-canvas` - passed, 942 tests.
- `cargo fmt --check` - passed.
- `python3 tools/check_layering.py` - passed.
- `python3 tools/check_workstream_catalog.py` - passed; validated 427 dedicated directories and 47
  standalone markdown files.
- `git diff --check` - passed.
- `out=$(git diff --check --no-index /dev/null ecosystem/fret-node/src/ui/overlays/toolbar_layout_policy.rs 2>&1); test -z "$out"` -
  passed.

Earlier run on 2026-05-19 for `RBX-M2-090`:

- `cargo nextest run -p fret-node portal_command_policy editor_chrome_compiles_without_retained_canvas_compat` -
  passed, 3 tests.
- `cargo check -p fret-node --no-default-features --features fret-ui` - passed.
- `cargo check -p fret-node --features compat-retained-canvas` - passed.
- `cargo nextest run -p fret-node --features compat-retained-canvas portal` - passed, 28 tests.
- `cargo nextest run -p fret-node` - passed, 329 tests.
- `cargo nextest run -p fret-node --features compat-retained-canvas` - passed, 913 tests.
- `cargo fmt --check` - passed.
- `python3 tools/check_layering.py` - passed.
- `python3 tools/check_workstream_catalog.py` - passed; validated 427 dedicated directories and 47
  standalone markdown files.
- `git diff --check` - passed.
- `out=$(git diff --check --no-index /dev/null ecosystem/fret-node/src/ui/editors/portal_command_policy.rs 2>&1); test -z "$out"` -
  passed.

Earlier run on 2026-05-19 for `RBX-M2-085`:

- `cargo nextest run -p fret-node portal_text_command_protocol` - passed, 2 tests.
- `cargo check -p fret-node --no-default-features --features fret-ui` - passed.
- `cargo check -p fret-node --features compat-retained-canvas` - passed.
- `cargo nextest run -p fret-node` - passed, 327 tests.
- `cargo nextest run -p fret-node --features compat-retained-canvas` - passed, 911 tests.
- `cargo fmt --check` - passed.
- `python3 tools/check_layering.py` - passed.
- `python3 tools/check_workstream_catalog.py` - passed; validated 427 dedicated directories and 47
  standalone markdown files.
- `git diff --check` - passed.

Earlier run on 2026-05-19 for `RBX-M2-080`:

- `cargo nextest run -p fret-node retained_bridge_source_usage_stays_on_the_migration_ledger` -
  passed, 1 test.
- `cargo check -p fret-node --no-default-features --features fret-ui` - passed.
- `cargo check -p fret-node --features compat-retained-canvas` - passed.
- `cargo nextest run -p fret-node` - passed, 325 tests.
- `cargo nextest run -p fret-node --features compat-retained-canvas` - passed, 909 tests.
- `cargo fmt --check` - passed.
- `python3 tools/check_layering.py` - passed.
- `python3 tools/check_workstream_catalog.py` - passed; validated 427 dedicated directories and 47
  standalone markdown files.
- `git diff --check` - passed.
- `rg -l "use fret_ui::retained_bridge|use fret_ui::\\{UiHost, retained_bridge|fret_ui::retained_bridge::|RetainedSubtreeProps|UiTreeRetainedExt" ecosystem/fret-node/src/ui -g '*.rs' | sort | wc -l` -
  reported 175 retained-ledger source files.

Earlier run on 2026-05-19 for `RBX-M1-080` completion:

- `cargo nextest run -p fret-ui anchored_can_resolve_preceding_absolute_anchor_element_in_same_frame mechanism_harness_anchored_layout_invalidation_matches_oracles` -
  passed, 2 tests.
- `cargo fmt --check -p fret-docking` - passed.
- `cargo fmt --check` - passed.
- `cargo check -p fret-docking` - passed.
- `cargo check -p fret-docking --features imui` - passed.
- `cargo clippy -p fret-docking --all-targets --features imui --no-deps -- -D warnings` -
  passed.
- `cargo clippy -p fret-docking --all-targets --no-deps -- -D warnings` - passed during the
  follow-up parity backfill.
- `cargo nextest run -p fret-ui managed_surface` - passed, 6 tests.
- `cargo nextest run -p fret-docking` - passed, 89 tests during the follow-up parity backfill.
- `python3 tools/check_layering.py` - passed.
- `python3 tools/check_workstream_catalog.py` - passed; validated 427 dedicated directories and 47
  standalone markdown files.
- `git diff --check` - passed.
- `cargo nextest run -p fret-ui declarative::tests::layout::mechanism_harness::mechanism_harness_layout_primitives_match_oracles` -
  failed on independent `chrome-container-stretch-keeps-outer-box` flex/chrome layout drift.
- `python3 tools/audit_crate.py --crate fret-node` - passed.
- `cargo check -p fret-node --no-default-features --features headless` - passed.
- `cargo check -p fret-node --no-default-features --features fret-ui` - passed.
- `cargo check -p fret-node --features compat-retained-canvas` - passed.
- `cargo nextest run -p fret-node retained_compatibility_surface_stays_declarative_only` -
  passed, 1 test.
- `rg -n "DockSpace::|create_node_retained|retained_bridge|UiTreeRetainedExt|RetainedSubtree|DockPanelRegistry|with_panel_content|unstable-retained-bridge" ecosystem/fret-docking/src ecosystem/fret-docking/tests ecosystem/fret-docking/Cargo.toml -g '*.rs' -g 'Cargo.toml'` -
  only public-surface policy negative assertion strings matched.

Earlier task-local parity checks in the current session:

- `cargo nextest run -p fret-docking public_declarative_dock_space_entry_point_tab_drop public_declarative_dock_space_entry_point_viewport_capture_ignores_other_pointer_move_and_up` -
  passed, 4 tests.
- `cargo nextest run -p fret-docking public_declarative_dock_space_entry_point_records_panel_root_bounds_for_element_queries` -
  passed, 1 test.
- `cargo check -p fret-docking` - passed warning-clean after retained wrapper cleanup.

Last run on 2026-05-18:

- `cargo fmt --check` - passed.
- `cargo check -p fret-demo --bin docking_arbitration_demo` - passed.
- `cargo clippy -p fret-demo --bin docking_arbitration_demo --no-deps -- -D warnings` - passed.
- `python3 tools/check_layering.py` - passed.
- `python3 tools/check_workstream_catalog.py` - passed.
- `git diff --check` - passed.
- `rg -n "retained_bridge::resizable_panel_group|retained_bridge::ResizablePanelGroupLayout|resizable::compute_layout" crates ecosystem apps -g '*.rs'`
  - no matches.

`RBX-M1-030` audit commands:

- `python3 tools/audit_crate.py --crate fret-docking` - passed.
- `rg -n "retained_bridge|UiTreeRetainedExt|create_node_retained|RetainedSubtree|impl<.*Widget|impl Widget|Widget<" ecosystem/fret-docking/src ecosystem/fret-docking/tests -g '*.rs'`
  - found retained host/lifecycle usage in docking source and tests.
- `cargo fmt --check` - passed.
- `python3 tools/check_layering.py` - passed.
- `python3 tools/check_workstream_catalog.py` - passed.
- `git diff --check` - passed.

`RBX-M1-040` gates:

- `cargo check -p fret-docking` - passed.
- `cargo fmt --check` - passed.
- `cargo nextest run -p fret-docking` - passed, 111 tests.
- `python3 tools/check_layering.py` - passed.
- `python3 tools/check_workstream_catalog.py` - passed.
- `git diff --check` - passed.

`RBX-M1-050` gates:

- `cargo check -p fret-docking` - passed.
- `cargo fmt --check` - passed.
- `cargo nextest run -p fret-docking` - passed, 111 tests.
- `python3 tools/check_layering.py` - passed.
- `python3 tools/check_workstream_catalog.py` - passed.
- `git diff --check` - passed.

`RBX-M1-060` gates:

- `cargo check -p fret-ui` - passed.
- `cargo check -p fret-docking` - passed.
- `cargo fmt --check` - passed.
- `cargo nextest run -p fret-ui managed_surface` - passed, 3 tests.
- `cargo nextest run -p fret-docking` - passed, 112 tests.
- `python3 tools/check_layering.py` - passed.
- `cargo nextest run -p fret-ui -p fret-docking` - failed on existing/independent
  `fret-ui declarative::tests::anchored_layout_invalidation_harness::mechanism_harness_anchored_layout_invalidation_matches_oracles`.
  A targeted repeat of that harness also fails with the same `first-panel` layout-bounds mismatch.

`RBX-M1-070` gates:

- `cargo check -p fret-docking` - passed.
- `cargo check -p fret-docking --features imui` - passed.
- `cargo nextest run -p fret-docking public_declarative_dock_space_entry_point_hosts_registry_panel_roots` - passed.
- `cargo nextest run -p fret-docking public_docking_surface_prefers_declarative_entry_points retained_docking_entry_points_are_documented_as_legacy` - passed.
- `cargo nextest run -p fret-docking` - passed, 115 tests.
- `cargo nextest run -p fret-ui managed_surface` - passed, 3 tests.
- `cargo fmt --check` - passed.
- `cargo check -p fret-demo --bin todo_demo` - passed.
- `cargo check -p fret-cookbook --example docking_basics --features cookbook-docking` - passed.
- `python3 tools/check_layering.py` - passed.
- `python3 tools/check_workstream_catalog.py` - passed.
- `git diff --check` - passed.

`RBX-M1-075` first-slice gates:

- `cargo nextest run -p fret-ui managed_surface_dispatches_event_command_and_availability_hooks` -
  passed.
- `cargo nextest run -p fret-ui managed_surface` - passed, 4 tests.
- `cargo check -p fret-docking` - passed.
- `cargo nextest run -p fret-docking public_declarative_dock_space_entry_point_handles_focus_requested_panel_command` -
  passed.
- `cargo nextest run -p fret-docking public_declarative_dock_space_entry_point` - passed, 2 tests.

`RBX-M1-075` second-slice gates:

- `cargo nextest run -p fret-docking public_declarative_dock_space_entry_point_installs_internal_drag_route_anchor` -
  passed.
- `cargo nextest run -p fret-docking public_declarative_dock_space_entry_point` - passed, 3 tests.
- `cargo nextest run -p fret-docking` - passed, 117 tests.
- `cargo fmt --check` - passed.
- `python3 tools/check_layering.py` - passed.
- `python3 tools/check_workstream_catalog.py` - passed.
- `git diff --check` - passed.

`RBX-M1-075` third-slice gates:

- `cargo check -p fret-docking` - passed.
- `cargo nextest run -p fret-docking public_declarative_dock_space_entry_point_publishes_diagnostics_and_liveness` -
  passed.
- `cargo nextest run -p fret-ui managed_surface` - passed, 4 tests.
- `cargo nextest run -p fret-docking public_declarative_dock_space_entry_point` - passed, 4 tests.
- `cargo nextest run -p fret-docking` - passed, 118 tests.
- `cargo check -p fret-docking --features imui` - passed.
- `cargo check -p fret-demo --bin docking_demo --bin container_queries_docking_demo --bin imui_editor_proof_demo` -
  passed.
- `cargo check -p fret-cookbook --example docking_basics --features cookbook-docking` - passed.
- `cargo fmt --check` - passed.
- `python3 tools/check_layering.py` - passed.
- `python3 tools/check_workstream_catalog.py` - passed; validated 427 dedicated directories and 47
  standalone markdown files.
- `git diff --check` - passed.

`RBX-M1-075` fourth-slice gates:

- `cargo nextest run -p fret-ui managed_surface` - passed, 5 tests.
- `cargo nextest run -p fret-docking public_declarative_dock_space_entry_point` - passed, 4 tests.
- `cargo nextest run -p fret-docking` - passed, 118 tests.
- `cargo fmt --check` - passed.
- `python3 tools/check_layering.py` - passed.
- `python3 tools/check_workstream_catalog.py` - passed.
- `git diff --check` - passed.

`RBX-M1-075` fifth-slice gates:

- `cargo nextest run -p fret-docking public_declarative_dock_space_entry_point_syncs_viewport_layouts` -
  passed, 1 test.
- `cargo check -p fret-docking` - passed.
- `cargo nextest run -p fret-docking public_declarative_dock_space_entry_point` - passed, 5 tests.
- `cargo nextest run -p fret-ui managed_surface` - passed, 5 tests.
- `cargo nextest run -p fret-docking` - passed, 119 tests.
- `cargo fmt --check` - passed.
- `python3 tools/check_layering.py` - passed.
- `python3 tools/check_workstream_catalog.py` - passed; validated 427 dedicated directories and 47
  standalone markdown files.
- `git diff --check` - passed.

`RBX-M1-075` sixth-slice gates:

- `cargo check -p fret-docking` - passed.
- `cargo nextest run -p fret-docking public_declarative_dock_space_entry_point` - passed, 5 tests.
- `cargo nextest run -p fret-ui managed_surface` - passed, 5 tests.
- `cargo nextest run -p fret-docking` - passed, 119 tests.
- `cargo fmt --check` - passed.
- `python3 tools/check_layering.py` - passed.
- `python3 tools/check_workstream_catalog.py` - passed; validated 427 dedicated directories and 47
  standalone markdown files.
- `git diff --check` - passed.

`RBX-M1-075` seventh-slice gates:

- `cargo check -p fret-docking` - passed.
- `cargo nextest run -p fret-docking public_declarative_dock_space_entry_point` - passed, 6 tests.
- `cargo nextest run -p fret-docking` - passed, 120 tests.
- `cargo nextest run -p fret-ui managed_surface` - passed, 5 tests.
- `cargo fmt --check` - passed.

`RBX-M1-075` eighth-slice gates:

- `cargo check -p fret-docking` - passed.
- `cargo nextest run -p fret-docking public_declarative_dock_space_entry_point_paints_floating_chrome` -
  passed, 1 test.
- `cargo nextest run -p fret-docking public_declarative_dock_space_entry_point` - passed, 7 tests.
- `cargo nextest run -p fret-docking` - passed, 121 tests.
- `cargo nextest run -p fret-ui managed_surface` - passed, 5 tests.
- `cargo fmt --check` - passed.

`RBX-M1-075` ninth-slice gates:

- `cargo check -p fret-docking` - passed.
- `cargo nextest run -p fret-docking public_declarative_dock_space_entry_point_paints_drag_payload_ghost` -
  passed, 1 test.
- `cargo nextest run -p fret-docking public_declarative_dock_space_entry_point` - passed, 8 tests.
- `cargo nextest run -p fret-docking` - passed, 122 tests.
- `cargo nextest run -p fret-ui managed_surface` - passed, 5 tests.
- `cargo fmt --check` - passed.

`RBX-M1-075` tenth-slice gates:

- `cargo check -p fret-docking` - passed.
- `cargo nextest run -p fret-docking public_declarative_dock_space_entry_point_paints_center_drop_overlay` -
  passed, 1 test.
- `cargo nextest run -p fret-docking public_declarative_dock_space_entry_point` - passed, 9 tests.
- `cargo nextest run -p fret-docking` - passed, 123 tests.
- `cargo nextest run -p fret-ui managed_surface` - passed, 5 tests.
- `cargo fmt --check` - passed.

Previous `RBX-M1-020` gates:

- `cargo nextest run -p fret-docking` - passed, 111 tests.
- `cargo check -p fret-demo --bin docking_arbitration_demo` - passed.
- `python3 tools/check_layering.py` - passed.
- `python3 tools/check_workstream_catalog.py` - passed.
- `git diff --check` - passed.
