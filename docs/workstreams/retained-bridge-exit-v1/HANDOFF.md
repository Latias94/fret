# Retained Bridge Exit v1 Handoff

Updated: 2026-05-19

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
coverage.

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
lifecycles until declarative portal hosting replaces them. The retained canvas/editor stack still
exists inside `fret-node` behind `compat-retained-canvas`; the next M2 slice should replace
overlay/panel composition with declarative coverage or move portal subtree hosting/model lifecycles
onto a declarative host before deleting retained code.

## Next Task

Pick the next task from:

- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`

Recommended next implementation shape:

- Continue M2 by shrinking the RBX-M2-080 ledger. The sharpest independent family is now
  overlay/panel composition (blackboard, controls, minimap, toolbars, rename). The remaining portal
  family work is declarative portal subtree hosting/model lifecycle replacement. Each slice should
  first add default declarative tests, then remove or gate less retained code.
- After the ledger no longer contains behavior-only retained files, remove
  `compat-retained-canvas` / `unstable-retained-bridge` from `fret-node`.
- Keep the known independent `fret-ui` layout primitive drift
  (`chrome-container-stretch-keeps-outer-box`) separate from retained-bridge exit unless a future
  slice touches that layout path directly.

## Gates

Last run on 2026-05-19 for `RBX-M2-095`:

- `cargo nextest run -p fret-node without_retained_command_cx editor_chrome_compiles_without_retained_canvas_compat` -
  passed, 3 tests.
- `cargo check -p fret-node --no-default-features --features fret-ui` - passed.
- `cargo check -p fret-node --features compat-retained-canvas` - passed.
- `cargo nextest run -p fret-node --features compat-retained-canvas portal` - passed, 30 tests.
- `cargo nextest run -p fret-node` - passed, 331 tests.
- `cargo nextest run -p fret-node --features compat-retained-canvas` - passed, 915 tests.
- `cargo fmt --check` - passed.
- `python3 tools/check_layering.py` - passed.
- `python3 tools/check_workstream_catalog.py` - passed; validated 427 dedicated directories and 47
  standalone markdown files.
- `git diff --check` - passed.
- `out=$(git diff --check --no-index /dev/null ecosystem/fret-node/src/ui/editors/portal_command_session.rs 2>&1); test -z "$out"` -
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
