# `fret-node` Fearless Refactor (v1) - Design Map

Status: execution-oriented companion (last updated 2026-04-07)
Scope: `ecosystem/fret-node` only

This file is the shortest possible answer to:

- what surfaces should app authors use,
- what surfaces are still advanced-only,
- what retained seams are intentionally still internal,
- what the next worktree should change first.

Use this file together with:

- `README.md` for rationale and architecture intent,
- `todo.md` for the actionable task backlog,
- `milestones.md` for done criteria and regression expectations.

## Surface map

### App-facing default surface

Use these surfaces for new app code and new examples unless a specific retained-only gate requires
something else.

- `NodeGraphController`
  - owns the recommended app-facing runtime facade,
  - owns viewport operations directly (`set_viewport*`, `set_center_in_bounds*`,
    `fit_view_nodes_in_bounds*`, `fit_canvas_rect_in_bounds*`, `screen_to_canvas`,
    `canvas_to_screen`),
  - keeps public viewport options store-first (`min/max zoom`, `padding`, `include_hidden_nodes`)
    instead of exposing retained queue animation overrides,
  - owns transaction-safe graph commits and query helpers.
- `NodeGraphSurfaceBinding`
  - remains the instance-style app-facing bundle for declarative surfaces,
  - now mirrors the full common store-first viewport helper family (`set_viewport*`,
    `set_center_in_bounds*`, `fit_view_nodes_in_bounds*`, `fit_canvas_rect_in_bounds*`,
    plus `screen_to_canvas` / `canvas_to_screen`, including action-host and option-bearing
    variants where applicable),
  - now also mirrors routine bound-store edit/sync/history helpers (`dispatch_transaction*`,
    `submit_transaction*`, `replace_*_action_host`, `set_selection_action_host`,
    `undo_action_host`, `redo_action_host`),
  - is now also the routine declarative `paint_only` action/UiHost seam for commit/selection/diag
    flows and fit-to-portals viewport updates, so internal declarative orchestration stops
    re-introducing controller-only triplets for ordinary bound-surface work,
  - should be enough for routine app-facing viewport authoring without dropping to
    explicit controller construction for normal instance-style hooks.
- `node_graph_surface(...)`
  - remains the recommended lightweight declarative authoring surface,
  - should keep converging toward the editor-grade path rather than staying a "lite demo only"
    surface forever.
- `node_graph_surface_compat_retained(...)`
  - is still allowed as a compatibility path,
  - but should be taught as compatibility-only, not as the long-term public design center.

### XYFlow alignment note

XYFlow’s core affordance is “nodes as component subtrees in a panning/zooming world layer”. Fret can
support that outcome, but the long-term home for the generic world-layer mechanism is
`ecosystem/fret-canvas` (see `docs/workstreams/xyflow-gap-analysis.md`). `fret-node` should remain
focused on the editor-grade substrate + transaction-safe interaction model, and use portals/world
layer techniques only when they directly serve the node editor’s contracts (portals, overlays,
anchoring, diagnostics).

### Crate-internal retained compatibility seams

These still exist because retained conformance coverage and compatibility glue still need them, but
they should not read like public downstream authoring APIs.

- `NodeGraphCanvas::with_view_queue`
- `NodeGraphController::bind_edit_queue_transport`
- `ui/compat_transport.rs` (`NodeGraphEditQueue`)
- `NodeGraphPortalHost::with_edit_queue`
- `NodeGraphOverlayHost::with_edit_queue`
- `NodeGraphBlackboardOverlay::with_edit_queue`

These methods/types are crate-private on purpose.

### Deleted or demoted surfaces already landed

These are no longer part of the recommended public story.

- Root `fret_node::ui::*` queue/helper aliases are removed.
- Public `fret_node::ui::advanced::*` edit transport is removed.
- `NodeGraphViewportHelper` is removed.
- Raw edit/view queue transport is crate-internal only.
- Public viewport option types are now split from retained queue animation options; the queue-only
  motion overrides stay crate-internal.
- Direct public retained queue-binding methods are demoted to crate-private compatibility seams.
- Unused direct `NodeGraphCanvas::with_edit_queue(...)` is removed outright.

## Recommended integration recipes

### Recipe A - new editor-grade app code

Use this unless you have a specific retained-only requirement.

1. Create `NodeGraphStore`.
2. Create `NodeGraphController` from the store.
3. Pass the controller into declarative composition.
4. Route committed edits through controller/store transactions.
5. Keep app-facing viewport operations on the controller surface.
6. In declarative button/action hooks, prefer `NodeGraphSurfaceBinding::*_action_host(...)` instead
   of teaching raw queue ownership.

### Recipe B - compatibility retained shell

Use this when a retained-only harness still needs the legacy canvas / overlay stack.

1. Create `NodeGraphStore`.
2. Create `NodeGraphController`.
3. Pass the controller explicitly to retained widgets via `with_controller(...)`; this is the only
   public retained binding seam.
4. Let retained internals fall back to crate-private queue seams only where the compatibility path
   still needs them.

### Recipe C - retained transport compatibility internals

This is not a downstream recipe. It exists only for in-tree retained compatibility harnesses and
focused tests that still need queue transport while the retained stack is being collapsed.

1. Keep raw edit/view queues inside `fret-node` internals.
2. Bind them only through crate-private retained/controller glue.
3. Do not re-export or teach this path in examples, cookbook docs, or app-facing guides.

## Next worktree order

Status note (2026-04-03):

- Slice 1 is now landed for the current controller-facing viewport/XyFlow mapping.
- Focused `controller.rs` + `binding.rs` gates now cover viewport read/projection,
  `set_viewport*`, `set_center_in_bounds*`, `fit_view_nodes_in_bounds*`, and
  `fit_canvas_rect_in_bounds*`.
- Unless fresh evidence reveals another controller helper gap, the next default execution surface
  should start at Slice 2 rather than reopening controller breadth by habit.

### Slice 1 - remaining controller surface breadth

Why first:

- the public viewport story is now intentionally store-first, so the remaining ambiguity is no
  longer queue transport but which broader imperative helpers still belong on the controller,
- the `Controller` vs `Binding` naming/ownership story is now locked, so the remaining work is
  helper breadth, internal organization, and the remaining XyFlow-style mapping work,
- it can land without reopening the already-finished transport cleanup.

What should be true after landing:

- reviewers can explain which imperative graph/viewport/query helpers belong on the controller
  surface without reaching for retained compatibility seams,
- controller helper breadth stops drifting every time a new helper is added,
- app code keeps one obvious controller/binding-first story for runtime actions.

First landing in this worktree:

- Public `NodeGraphFitViewOptions` / `NodeGraphSetViewportOptions` now live in a dedicated
  `viewport_options.rs` module and only expose fields the store-first controller path really
  consumes.
- Retained `view_queue.rs` keeps its richer animation options as crate-internal transport-only
  types, so public app code no longer sees queue-era motion knobs that are no-ops on the
  controller/binding path.
- `update_node*` / `update_edge*` now use `NodeGraphNodeUpdate` / `NodeGraphEdgeUpdate` drafts,
  so structural node-port edits and edge endpoint rewires are not representable through the
  ergonomic helper surface and must stay on explicit transactions.
- The next follow-up after this slice should decide which remaining imperative helpers or transport
  ownership changes still belong on the controller surface before widening it further.
- `NodeGraphSurfaceBinding` is now already split into focused companion modules
  (`binding_queries.rs`, `binding_store_sync.rs`, `binding_viewport.rs`), so future breadth work
  should preserve responsibility boundaries instead of re-growing a single surface file.

### Slice 2 - declarative transaction closure

Status note (2026-04-03):

- The selection/marquee/pointer-session reducer split is now landed.
- Local-vs-store interaction boundaries are explicit: transient drag/marquee/pending-selection/
  hover state stays local until commit/cancel, while committed selection/graph edits route through
  the binding/controller seams.
- Declarative `paint_only` runtime source ownership is now locked: runtime files must treat
  `binding.store_model()` as the authoritative graph/view/editor-config source instead of
  consulting bound graph/view/config mirrors directly.
- Declarative graph-edit commit authority is now also centralized in `paint_only/transactions.rs`:
  runtime files must not replace graph/document directly or dispatch/submit transactions outside
  that seam.
- The retained compatibility runtime no longer keeps a stale `cfg(test)` editor-config
  reconstruction fallback; retained runtime, retained tests, and `--all-features` builds now all
  use the same explicit editor-config seam.
- The next narrow follow-up inside Slice 2 should focus on any still-missing transaction-backed
  declarative graph-edit path, not on reopening either the reducer split or the store-first source
  boundary that now has focused gates.

Why next:

- this is still the highest-risk correctness gap,
- it directly determines whether declarative composition can really be the editor-grade default.

What should be true after landing:

- committed declarative edits no longer bypass transaction/store architecture,
- drag/marquee/selection commits have clear transaction-safe gates,
- undo/redo semantics remain coherent from the declarative path.

First landing in this worktree:

- marquee preview selection now stays in local declarative reducer state instead of churning store
  selection on every pointer move,
- hit-node click selection and empty-space clear now also stay local until pointer-up (or node-drag
  activation), instead of writing store selection on pointer-down,
- pointer-up commits the previewed selection through the same controller/store-backed selection seam,
- left-button pointer release now routes through a dedicated helper that arbitrates node-drag vs
  pending-selection vs marquee completion before the shared pointer-session cleanup,
- node-drag threshold crossing commits any pending hit selection before the drag transaction path
  takes over,
- node drag local state now uses explicit `Armed` / `Active` / `Canceled` phases so threshold
  activation, selection-only release, and cancel-drop semantics stay reviewable,
- escape now also clears pending-selection-only sessions (not just marquee / node-drag sessions),
- escape/pointer-cancel now drop transient marquee/click-selection state instead of issuing
  selection restore writes, now via a shared cancel reducer that keeps the Escape-vs-pointer-cancel
  node-drag semantics explicit,
- pointer-driven layout/paint follow-up effects (`invalidate` / `notify` / `request_redraw`) now
  route through dedicated helpers so reducer extraction does not duplicate host-side bookkeeping,
- keyboard capture now parses explicit declarative diag/zoom actions, so `Escape`, diagnostics
  digits, and keyboard zoom no longer depend on one monolithic closure branch,
- left-button pointer-down now also routes through explicit snapshot/reducer helpers, so pan
  start, hit-node preview, marquee arming, and empty-space clear stop competing inside one
  oversized event closure,
- pointer-move now also dispatches through dedicated node-drag, marquee, and hover helpers,
  so activation/preview/paint-only hover updates no longer live in one monolithic branch.
- left-button pointer-up now also dispatches through dedicated node-drag,
  pending-selection, and marquee release helpers, so release commit ordering and transient
  cleanup no longer depend on one mixed branch.
- pointer-up / pointer-cancel event closures now also route through explicit session
  helpers, so release/cancel finish semantics and host-side invalidate/notify/redraw effects
  are testable outside the closure bodies.
- declarative paint-only tests now share small controller/store and pointer-session
  fixtures, so new reducer/session slices can land focused gates without duplicating large setup
  blocks.
- the first private paint-only submodule split is landed: release/cancel/session-host
  helpers now live in `paint_only/pointer_session.rs`, so the main surface file keeps orchestration
  responsibilities while this interaction slice gets a named boundary.
- the second private paint-only submodule split is landed: pointer-move helpers and
  outcomes now live in `paint_only/pointer_move.rs`, so drag/marquee/hover move handling stops
  expanding the main surface file.

### Slice 3 - portal and overlay closure

Status note (2026-04-03):

- Visible-subset portal hosting is now a declared public seam: `NodeGraphSurfaceProps` carries
  `NodeGraphVisibleSubsetPortalConfig` instead of loose portal booleans, so the editor-facing
  surface now names the visible-subset hosting contract directly.
- `paint_only/portals.rs` now consumes that config seam for visible-subset hosting and
  fit-to-portals replay, while focused gates keep draw-order/cap semantics and dragged-rect
  visibility behavior locked.
- Overlay tooltip orchestration no longer depends on the portal-hosting module for node label/port
  summaries; that shared lookup now lives in the neutral `paint_only/surface_support.rs` seam.
- Declarative diagnostics policy is now also explicit on the surface: `NodeGraphSurfaceProps`
  carries `NodeGraphDiagnosticsConfig`, while the demo chooses whether `FRET_DIAG` enables
  diagnostics instead of the mechanism layer reading process env directly.
- Root `fret_node::ui::*` now also re-exports `NodeGraphDiagnosticsConfig` and
  `NodeGraphVisibleSubsetPortalConfig`, so app-facing declarative authoring does not have to mix
  `ui` and `ui::declarative` import paths just to configure `NodeGraphSurfaceProps`.
- Retained toolbar target-selection and visibility policy now routes through the private
  `ui/overlays/toolbar_policy.rs` seam, so node and edge toolbar widgets stop carrying duplicated
  "selected target vs explicit target" fallback rules inline.
- The public toolbar policy types now also live with that seam:
  `NodeGraphToolbarVisibility` / `NodeGraphToolbarPosition` / `NodeGraphToolbarAlign` /
  `NodeGraphToolbarSize` are no longer declared inside `toolbars.rs`, keeping the widget file
  focused on anchor layout/measurement rather than public policy type ownership.
- Retained rename overlays now also route active-session selection, seed-text loading, focus-loss
  cancel policy, and commit-transaction planning through the private
  `ui/overlays/rename_policy.rs` seam, so `NodeGraphOverlayHost` no longer duplicates group-vs-
  symbol rename branches or commits a hidden second session.
- Retained controls overlays now also route button roster order, default command mapping,
  override-resolution, a11y labels, and display labels through the private
  `ui/overlays/controls_policy.rs` seam, so layout, keyboard navigation, and activation all consume
  one authority table instead of keeping repeated button lists inside `controls.rs`.
- The public controls binding types now also live with that policy seam:
  `NodeGraphControlsCommandBinding` / `NodeGraphControlsBindings` are no longer declared inside
  `controls.rs`, keeping the widget file focused on implementation rather than public policy type
  ownership.
- Retained blackboard overlays now also route action roster order, keyboard navigation policy,
  action labels, default symbol naming, transaction planning, and symbol-rename opening through the
  private `ui/overlays/blackboard_policy.rs` seam, so `blackboard.rs` keeps layout/paint/event
  orchestration while action policy no longer sprawls through the widget body.
- Retained minimap overlays now also route keyboard action mapping, pan/zoom step policy, zoom
  clamp, and center-based zoom planning through the private `ui/overlays/minimap_policy.rs` seam,
  so `minimap.rs` now keeps internals sampling, pointer drag handling, and viewport application
  while keyboard navigation policy stops living inline in the widget event branch.
- Retained minimap overlays now also route viewport-update ownership and zoom normalization through
  the private `ui/overlays/minimap_navigation_policy.rs` seam, so controller/store/default
  navigation binding resolution no longer stays embedded in `minimap.rs`.
- The public minimap binding types now also live with that policy seam:
  `NodeGraphMiniMapNavigationBinding` / `NodeGraphMiniMapBindings` are no longer declared inside
  `minimap.rs`, keeping the widget file focused on implementation rather than public policy type
  ownership.
- Canvas menu/searcher overlay-session policy types now also live on a dedicated private seam:
  `ui/canvas/state/state_overlay_policy.rs` now owns `ContextMenuTarget` and
  `SearcherRowsMode`, so `state_overlay_sessions.rs` keeps session-container ownership instead of
  remaining the implicit home for menu/searcher policy enums.
- Searcher picker opener policy now also lives on one request seam:
  `ui/canvas/widget/searcher_picker/request.rs` owns `SearcherPickerRequest` including
  `rows_mode`, so background/connection insert pickers, edge-insert pickers, and conversion
  pickers stop re-embedding `Catalog` vs `Flat` request policy at each opener.
- Searcher row activation now also reuses the insert-candidate menu authority:
  `ui/canvas/widget/insert_candidates/menu.rs` now owns single candidate-to-menu-item synthesis,
  so `searcher_row_activation` no longer hand-assembles `InsertNodeCandidate(...)` actions outside
  the same seam used by context-menu candidate lists.
- Searcher row activation now also reuses selectable-row policy from `searcher_rows`,
  so activation gating no longer keeps a second implicit "candidate + enabled" rule separate from
  active-row selection and keyboard navigation.
- Context-menu target dispatch now also routes non-command activation through a named private seam:
  `ui/canvas/widget/context_menu/activate/target.rs` now owns the target-to-executor route enum,
  so `activate.rs` keeps command-vs-target action-kind dispatch while background/connection/edge/
  conversion activation routing stops living as an unowned inline match.
- Command context-menu activation now also routes target-scoped selection side effects through a
  named private seam: `ui/canvas/widget/context_menu/activate/command.rs` now owns the
  group-selection-vs-ignore policy, so command dispatch no longer keeps a hidden "only group
  targets sync selection before dispatch" branch inline.
- Edge context-menu activation now also routes edge action planning through a named private seam:
  `ui/canvas/widget/context_menu/edge_execution.rs` now owns the edge-action route enum, so
  insert-picker / reroute / delete / custom edge actions no longer stay as an unowned inline match
  before delegating to their executor modules.
- Right-click context-menu opening now also routes target-hit priority through a named private seam:
  `ui/canvas/widget/context_menu/opening.rs` now owns the group-vs-edge-vs-background opening
  route, so opening priority stops living as an inline `if` chain while the target-specific
  openers keep only the already-resolved target presentation work.
- Context-menu presentation now also routes open-event state effects through a named private seam:
  `ui/canvas/widget/context_menu/ui.rs` now owns menu install plus hover-edge cleanup policy and
  event-finish focus/invalidation, while `opening.rs` only builds the menu state and passes an
  explicit hover-edge policy instead of a boolean flag.
- Context-menu presentation lifecycle now also mirrors the searcher split:
  `ui/canvas/widget/context_menu/ui/overlay.rs` owns state install/restore/take/clear plus
  hover-edge cleanup policy, `ui/canvas/widget/context_menu/ui/event.rs` owns event-finish and
  open/restore/dismiss tails, and `ui/canvas/widget/context_menu/ui.rs` now acts as a thin wrapper
  surface instead of a mixed state-and-event file.
- Searcher overlay install now also has an explicit replacement seam:
  `ui/canvas/widget/searcher_ui/overlay.rs` now owns the "clear context menu, then install or
  replace searcher state" rule through a dedicated state helper, so this overlay-replacement policy
  no longer stays hidden inside the root install function.
- Context-menu/searcher event tails now also share the retained widget runtime finish helper:
  `ui/canvas/widget/context_menu/ui/event.rs` and `ui/canvas/widget/searcher_ui/event.rs` both use
  `retained_widget_runtime_shared` for stop-propagation plus paint invalidation, so overlay event
  tails stop re-embedding the same low-level redraw/invalidation steps.
- Active menu-session occupancy now also routes through the private
  `ui/canvas/widget/menu_session.rs` seam, so window-focus deferral, space-to-pan gating,
  Tab-navigation suppression, edge double-click preflight, motion/auto-pan tick guards, and
  retained `view_interacting(...)` all reuse one `context_menu || searcher` authority instead of
  re-embedding that overlay-session policy inline.
- Retained portal/overlay transaction fallback now also routes through the private
  `ui/retained_submit.rs` seam, so portal command commits plus blackboard/group-rename overlays
  share one controller-first vs edit-queue fallback policy instead of duplicating that
  compatibility branch inline.
- Retained action-panel keyboard routing now also routes through the private
  `ui/overlays/panel_navigation_policy.rs` seam, so controls and blackboard overlays share one
  Arrow/Home/End/Enter/Escape navigation authority instead of each embedding the same keyboard
  roster policy inline.
- Retained toolbar child layout lifecycle now also routes through the private
  `ui/overlays/toolbars_layout.rs` seam, so node and edge toolbars share one child measurement,
  hide-and-release-focus, and child paint authority while the root widget file keeps only
  target-specific anchor resolution.
- Retained overlay/portal handled-event tails now also route through the private
  `ui/retained_event_tail.rs` seam, so portal commands plus controls/blackboard/minimap/group-
  rename overlays share one authority for focus-to-canvas, stop-propagation, redraw, and
  paint/layout invalidation tails instead of duplicating those handled-event endings inline.
- Retained action-panel pointer state now also routes through the private
  `ui/overlays/panel_pointer_policy.rs` seam, so controls and blackboard overlays share one hover
  sync plus press-on-down / activate-on-matching-up authority instead of each re-embedding that
  pointer-state policy inline.
- Retained minimap projection math now also routes through the private
  `ui/overlays/minimap_projection.rs` seam, so world-bounds union, project/unproject transforms,
  and center-pan math live behind one focused authority instead of staying embedded in the overlay
  widget file.
- Retained blackboard layout and hit-testing now also route through the private
  `ui/overlays/blackboard_layout.rs` seam, so panel/header/row geometry plus action hit detection
  live behind one focused authority instead of staying embedded in the overlay widget file.
- Retained controls layout and hit-testing now also route through the private
  `ui/overlays/controls_layout.rs` seam, so panel geometry plus button hit detection live behind
  one focused authority instead of staying embedded in the overlay widget file.
- Retained action-panel item state now also routes through the private
  `ui/overlays/panel_item_state.rs` seam, so controls and blackboard overlays share one authority
  for keyboard selection resets, pointer-to-keyboard promotion, and visible item-state evaluation
  instead of each re-embedding that panel-state policy inline.
- Retained rename host layout lifecycle now also routes through the private
  `ui/overlays/rename_host_layout.rs` seam, so hidden/cancelled/active overlay layout planning
  lives behind one focused authority instead of staying embedded in the rename host widget file.
- Retained rename host key handling now also routes through the private
  `ui/overlays/rename_host_event.rs` seam, so Enter/Escape commit-vs-close routing plus
  controller-first submit/close ordering live behind one focused authority instead of staying
  embedded in the rename host widget file.
- Retained minimap drag planning now also routes through the private
  `ui/overlays/minimap_drag_policy.rs` seam, so pointer-down recentering and drag-pan delta
  planning live behind one focused authority instead of staying embedded in the overlay widget
  file.
- Retained overlay panel button paint now also routes through the private
  `ui/overlays/panel_button_paint.rs` seam, so controls and blackboard overlays share one
  authority for centered button-label placement and left-aligned panel text placement instead of
  each re-embedding that paint-side text geometry inline.
- Retained blackboard paint orchestration now also routes through the private
  `ui/overlays/blackboard_paint.rs` seam, so header/title paint ordering, action-button highlight
  resolution, and missing-symbol label fallback live behind one focused authority instead of
  staying embedded in the overlay widget file.
- Context-menu open-state replacement now also routes edge-insert submenu reopening through the
  private `ui/canvas/widget/context_menu/ui/overlay.rs` seam, so hover-edge preserve-vs-clear
  policy stops being bypassed by direct `interaction.context_menu = Some(...)` writes in
  `edge_insert/context_menu.rs`.
- The next narrow follow-up inside Slice 3 should keep focusing on the remaining overlay/menu
  policy placement, not on reopening visible-subset portal hosting or the now-aligned
  toolbar/controls/minimap/menu-session/searcher-picker policy ownership as unowned experiments.

Why after callback/commit cleanup:

- portal and overlay seams are now much easier to simplify once commit ownership is unambiguous.

What should be true after landing:

- retained overlays no longer need ad-hoc queue fallback outside the crate-private compatibility
  seams,
- controller-first retained composition remains the default teaching posture,
- focus / portal / rename / blackboard behavior keeps its current conformance coverage.

### Slice 4 - compatibility retained convergence gate

Why last:

- this is where deletion decisions become credible rather than speculative.

What should be true after landing:

- `compat-retained-canvas` has a clearly bounded role,
- every surviving retained-only seam has an explicit justification,
- removal criteria are written down instead of implied.

## Worktree starter checklist

Before opening the next worktree, confirm these assumptions:

- The change belongs to `ecosystem/fret-node`, not `crates/fret-ui`.
- The public story should stay controller-first and declarative-first.
- New raw queue surfaces should not be added to root `fret_node::ui::*`.
- If a retained-only seam survives, it should either be crate-private or explicitly advanced-only.
- Any behavior change should leave behind at least one focused regression gate.

## Fast reviewer checklist

A reviewer should be able to answer "yes" to all of these:

- Does the change strengthen `NodeGraphController` as the default app-facing surface?
- Does the change keep raw queue ownership explicit instead of accidental?
- Does the change avoid teaching retained queue seams as public app APIs?
- Does the change preserve or improve the existing conformance gates?
- Does the change leave the workstream easier to continue from a fresh worktree?

## Commands to keep using

- `cargo fmt -p fret-node`
- `cargo check -p fret-node --features compat-retained-canvas --tests`
- `cargo nextest run -p fret-node <targeted-tests> --features compat-retained-canvas`
- `python tools/check_layering.py`
