# `fret-node` Fearless Refactor (v1) - Milestones

This file defines what must be true before each milestone can be considered complete. The goal is
not to maximize activity; the goal is to reduce ambiguity and make the landing path reviewable.

Execution companion: `design.md` (surface map + next worktree order).

## Global success criteria

Every milestone should improve one of these axes without regressing the others, or explicitly
document the tradeoff:

- **Architecture clarity**
  - public authoring posture is unambiguous,
  - state boundaries match intent,
  - mechanism vs policy boundaries remain aligned.
- **Editor correctness**
  - committed edits remain transaction-safe,
  - controlled sync does not silently bypass history/diagnostics contracts,
  - undo/redo semantics stay coherent.
- **Declarative viability**
  - declarative surfaces can host editor-grade behavior without leaking retained authoring,
  - portal/overlay anchoring stays deterministic,
  - transaction-safe declarative paths remain testable.
- **Regression protection**
  - existing useful gates remain green,
  - new architecture claims add new evidence, not just prose.

## M0 - Decision gates and baseline seam map

Status target: short, reviewable, mostly-documentation closure

### Goal

Lock the decisions that will constrain the refactor and capture the current state in a way that can
be reviewed without re-reading the whole crate.

### Deliverables

- A reframed workstream README that states:
  - the public authoring posture,
  - the retained compatibility posture,
  - the state/controller problems to solve,
  - the target architecture.
- A milestone plan and TODO tracker aligned to those decisions.
- A short current-hazards section with evidence anchors for the active architectural risks.
- A short reviewer checklist so posture regressions are easy to spot in review.
- A minimal seam map of the current "best available" surfaces:
  - paint-only declarative surface,
  - compat-retained declarative surface,
  - retained engine,
  - store/queue/lookups/controller gap.

### Done criteria

- Reviewers can answer these questions from the docs alone:
  - What should app authors use today?
  - What is the long-term target posture?
  - What is still compatibility-only?
  - What are the next architectural slices to land?

### Evidence anchors

- `docs/workstreams/fret-node-declarative-fearless-refactor-v1/README.md`
- `docs/workstreams/fret-node-declarative-fearless-refactor-v1/design.md`
- `docs/workstreams/fret-node-declarative-fearless-refactor-v1/milestones.md`
- `docs/workstreams/fret-node-declarative-fearless-refactor-v1/todo.md`

## M1 - Public posture and API closure

Status target: documentation + API-shape convergence

### Goal

Remove ambiguity from the public story. `fret-node` should teach one coherent authoring posture and
make compatibility paths explicit instead of accidental.

### Deliverables

- One canonical recommendation for shipping editor-grade usage today.
- One canonical target posture for the final declarative-first architecture.
- One bounded description of when `compat-retained-canvas` is justified.
- A surface plan for converging toward a single high-level declarative entrypoint.
- A wording audit for examples and internal guides so retained demos stay explicitly scoped.
- Diagnostics-driven example hosts stay aligned with the current `UiDiagnosticsService`
  script-driving contract when shared framework surfaces evolve on `main`.

### Done criteria

- Public docs stop reading like there are two equally blessed downstream authoring models.
- Retained constructors are no longer implied as the normal ecosystem path.
- New examples and cookbook-style guidance prefer declarative composition by default.
- Diagnostics-enabled example hosts do not silently drift behind the active service signature.

### Evidence anchors

- `ecosystem/fret-node/src/ui/declarative/mod.rs`
- `ecosystem/fret-node/src/ui/declarative/compat_retained.rs`
- `ecosystem/fret-node/src/ui/binding.rs`
- `apps/fret-examples/src/node_graph_demo.rs`
- `apps/fret-examples/src/node_graph_legacy_demo.rs`
- `apps/fret-examples/src/imui_node_graph_demo.rs`
- `docs/workstreams/xyflow-gap-analysis.md`
- `docs/workstreams/crate-audits/fret-node.l0.md`

## M2 - State boundary split

Status target: architectural refactor with compatibility plan

### Goal

Shrink `NodeGraphViewState` back to true view state and explicitly separate interaction policy from
runtime tuning.

### Deliverables

- A concrete split plan for:
  - `NodeGraphViewState` (viewport + selection + draw order),
  - interaction configuration,
  - runtime tuning.
- A serialization compatibility plan for existing persisted data.
- Store/runtime wiring updated to use the new boundaries without breaking the editor contract.
- First landed slice: `NodeGraphViewState` persists `NodeGraphInteractionConfig` +
  `NodeGraphRuntimeTuning`, while widget/runtime snapshots still resolve a combined
  `NodeGraphInteractionState` for compatibility.
- Persistence ownership is now explicit: the file wrapper writes pure view-state under `state`, with
  `interaction` / `runtime_tuning` stored as wrapper-owned fields in `state_version = 2`.

### Done criteria

- Reviewers can point to one place for persisted view state, one place for interaction policy, and
  one place for runtime tuning.
- The resulting shapes make it harder to persist accidental performance knobs as if they were view
  semantics.
- Controlled sync and diagnostics still have a stable data contract.

### Required regression protection

- focused `cargo nextest run -p fret-node` coverage for view-state migration and store behavior
- at least one diag or integration gate proving the split does not regress viewport/selection flows

### Evidence anchors

- `ecosystem/fret-node/src/io/mod.rs`
- `ecosystem/fret-node/src/runtime/store.rs`
- `ecosystem/fret-node/src/runtime/tests.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/view_state/sync.rs`
- `ecosystem/fret-node/src/ui/declarative/paint_only.rs`
- `ecosystem/fret-node/src/runtime/store.rs`
- `ecosystem/fret-node/src/runtime/tests.rs`

## M3 - Controller/instance facade and transaction-safe declarative commits

Status target: architectural + API landing milestone

### Goal

Make the declarative path editor-grade by routing committed edits through store/controller entry
points rather than direct graph mutation.

### Progress note (2026-03-07)

- First landing slices are complete:
  - `node_graph_surface` node-drag commit now builds a `GraphTransaction`.
  - `NodeGraphSurfaceProps` now takes a single `NodeGraphSurfaceBinding`, so the
    store-backed controller contract is bundled as one declarative surface input and the
    store/no-controller fallback path stays removed.
  - The declarative path now dispatches committed edits through `NodeGraphController` and syncs
    graph / view models back from store.
  - `ecosystem/fret-node/src/ui/controller.rs` now provides a first minimal `NodeGraphController`
    facade over store + optional view queue.
  - The controller now also exposes XyFlow-style connection queries via
    `node_connections` / `port_connections`, so app code can query node/handle adjacency
    without reaching into store lookups directly.
  - The controller now also covers the first bounds-aware viewport helpers:
    `set_center_in_bounds*` and `fit_view_nodes_in_bounds*`.
  - Retained glue now starts consuming controller-owned viewport transport instead of teaching raw
    queue mutation first: `NodeGraphCanvas::with_controller`, `NodeGraphMiniMapOverlay::with_controller`,
    and the gallery workflow snippet controls now route common viewport actions through the binding-first facade.
  - Those helpers now have a real store fallback when no `view_queue` exists, and still route
    through queued `SetViewport` requests when a queue is present.
  - The controller now also owns an optional edit transport and queue-aware submission helpers
    (`submit_transaction*`, `submit_transaction_and_sync_*`), so app-facing code no longer needs to
    choose between raw queue mutation and direct store dispatch first.
  - Retained edit glue now also converges on the controller-first path:
    `NodeGraphCanvas::with_controller` carries optional edit/view queues,
    `NodeGraphPortalHost::with_controller` and `NodeGraphOverlayHost::new(...).with_controller(...)` prefer
    controller-owned transaction submission, `NodeGraphBlackboardOverlay::new(...).with_controller(...)`
    now gives retained symbol actions the same controller-first path, and `compat_retained` now
    takes a controller binding directly instead of exposing public queue transport props.
  - `NodeGraphViewportHelper` is now bounded to the explicit advanced transport seam only:
    `NodeGraphViewportHelper::new(view_state, view_queue)` remains available for retained-only
    integrations, while controller-first app-facing composition now calls
    `NodeGraphController::{set_viewport*, set_center_in_bounds*, fit_view_nodes*, fit_view_nodes_in_bounds*}`
    directly without a second wrapper surface.
  - Raw queue / viewport transport exports now live under the explicit `fret_node::ui::advanced::*`
    namespace; queue-bound controller construction is also demoted behind `NodeGraphControllerTransportExt`, and the old root `fret_node::ui::*` queue/helper aliases are removed from the
    public surface.
  - The retained-backed domain demo and the workflow gallery snippet now also import those raw queue
    surfaces from `advanced::*`, so the sample code no longer teaches root `ui::*` queue imports.
  - Crate-internal retained/test callers now also use explicit module paths instead of the old root
    queue aliases, completing the in-tree cleanup for this transport seam.
  - Declarative keyboard zoom / wheel zoom / pinch zoom / drag-pan updates now start converging on
    controller/store-backed view-state replacement instead of only mutating the external
    `NodeGraphViewState` model.
  - The deferred `fit-to-portals` viewport apply path in the render/portal pass now also uses the
    same controller/store-backed view-state replacement path.
  - Declarative click selection / marquee preview / cancel restore also now start converging on
    controller/store-backed selection helpers instead of only mutating the external
    `NodeGraphViewState` selection fields.
  - Declarative marquee preview no longer churns store selection on pointer move: previewed nodes now
    stay in local reducer state, pointer-up commits through the controller/store-backed selection
    seam, and escape/pointer-cancel simply drop the transient marquee state.
  - Declarative hit-node click selection and empty-space clear no longer write store selection on
    pointer-down: they stay in local transient state until pointer-up, while node-drag threshold
    crossing commits the pending selection before the drag transaction path takes over.
  - Declarative node drag now uses explicit `Armed` / `Active` / `Canceled` phases, so threshold
    activation, selection-only release, and cancel-drop behavior no longer depend on paired boolean
    flags.
  - Declarative escape cancel now also clears pending-selection-only sessions, while helper-backed
    gates cover selection-only release and pointer-cancel transient cleanup.
  - Declarative left-button pointer release now routes through a dedicated helper that arbitrates
    node-drag vs pending-selection vs marquee completion, with focused tests covering pending-only,
    inactive-toggle-marquee, and no-state releases.
  - Declarative Escape / pointer-cancel cleanup now shares a mode-aware transient reducer, while
    pointer post-event invalidation/notify/redraw bookkeeping goes through dedicated helpers, with
    focused tests covering the already-canceled node-drag divergence.
  - Declarative keydown capture now dispatches through explicit diag/zoom action helpers, with
    focused tests covering diag-key parsing, diag view presets, portal-disable cleanup, zoom
    reset, and paint-override toggling.
  - Declarative left-button pointer-down now dispatches through explicit snapshot/reducer
    helpers, with focused tests covering pan-start cleanup plus hit-node, marquee, and
    empty-space-clear branches.
  - Declarative pointer-move now dispatches through explicit node-drag, marquee, and hover
    helpers, with focused tests covering drag activation, canceled drag cleanup, marquee
    preview/cancel, and hover hit updates.
  - The local-vs-store boundary is now explicit for selection paint/layout: active marquee preview
    overrides pending click-selection preview, which overrides committed store-backed selection;
    pan/node/marquee/hover sessions remain local until commit/cancel time.
  - Declarative left-button pointer-up now dispatches through explicit node-drag,
    pending-selection, and marquee release helpers, with focused tests covering branch
    cleanup and commit semantics.
  - Declarative pointer-up / pointer-cancel event closures now dispatch through
    explicit session helpers, with focused tests covering left-release finish, non-left ignore,
    pan-release cleanup, and cancel-finish semantics.
  - Declarative paint-only tests now share small controller/store and pointer-session
    fixtures, so follow-up reducer/session gates stop duplicating large setup blocks.
  - Declarative paint-only release/cancel/session-host helpers now live under the
    first private submodule split, `paint_only/pointer_session.rs`, so the main surface file
    keeps orchestration responsibility while this interaction slice gains a named boundary.
  - Declarative paint-only pointer-move helpers/outcomes now live under the second
    private submodule split, `paint_only/pointer_move.rs`, so drag/marquee/hover move handling
    no longer expands the main surface file.
  - Diagnostics-only `Digit3/4/5` graph tweaks now build transactions from `graph_diff` and commit
    through the same controller/store transaction path instead of mutating `Graph` in place.
  - `apps/fret-examples/src/node_graph_demo.rs` now builds a `NodeGraphSurfaceBinding` for the
    declarative surface so the recommended demo path exercises the transaction-safe commit
    architecture without teaching raw graph/view/controller triplets.
  - `apps/fret-examples/src/node_graph_domain_demo.rs` now acts as the retained-backed best-practice
    sample for controller-first canvas / overlay / portal composition.
  - `apps/fret-examples/src/node_graph_legacy_demo.rs` now also routes retained canvas / rename
    overlay / blackboard / portal / minimap glue through the controller-first path and no longer
    keeps a demo-owned `NodeGraphEditQueue`, so the legacy demo stops teaching raw edit queue
    mutation or queue ownership for those core surfaces.
  - Feature-gated retained coverage now also includes a blackboard controller-first gate proving
    symbol creation prefers controller/store commit over raw queue transport.
  - `NodeGraphController` now also owns undo/redo sync helpers for the default store-backed
    app-facing surface, and focused declarative coverage now proves node-drag commit history can be
    undone/redone while graph/view mirrors stay in sync.
  - `NodeGraphSurfaceBinding` now acts as the instance-style app-facing facade for common queries,
    viewport actions, graph adjacency queries, and controlled-sync helpers (`replace_graph`,
    `replace_view_state`, `set_selection`, `outgoers`, `incomers`, `connected_edges`,
    `port_connections`, `node_connections`, `undo`, `redo`), while `binding.controller()` stays available as the advanced
    escape hatch.
  - Controlled mode now has an explicit full-replace-first policy: replacing the authoritative graph
    document is treated as a reset + re-sync operation, while diff-first replace helpers remain a
    deferred follow-up rather than the default contract.
  - That policy now has a named app-facing helper: `NodeGraphSurfaceBinding::replace_document(...)`
    and `NodeGraphController::replace_document_and_sync_models(...)` replace graph + view-state,
    clear undo/redo history, and re-sync external mirrors in one step; the legacy demo no longer
    resets the store by calling `NodeGraphStore` directly.
  - Declarative surface now tracks the authoritative graph/selection boundary: graph-document
    replacement clears local pan / node-drag / marquee / pending-selection / hover / portal
    transient state, while selection-only authoritative changes clear selection-scoped previews
    without flushing pan/hover caches; focused tests lock this real editor-flow parity gate.
  - Paint cache discipline is now explicit as a gate: selection-only authoritative updates keep
    grid / derived / node / edge cache keys stable, while graph replacement invalidates only the
    graph-dependent caches instead of churning viewport-only paint caches.
  - Declarative overlay/portal anchoring now has an explicit correctness seam: hover tooltip
    positioning prefers hosted portal bounds when available, but deterministically falls back to the
    local hover-anchor store when portals are disabled or unavailable; focused tests lock both
    precedence paths.
  - Declarative portal hosting now has named seams for both visible-subset selection and subtree
    bounds publication: `collect_portal_label_infos_for_visible_subset(...)` keeps draw-order/cap
    semantics deterministic and culls against dragged rects, while
    `sync_portal_canvas_bounds_in_models(...)` makes `LayoutQueryRegion` bounds harvest
    epsilon-filtered and reviewable.
  - Callback layering is now explicit: `NodeGraphCommitCallbacks` owns committed graph diffs,
    `NodeGraphViewCallbacks` owns viewport/selection synchronization, and
    `NodeGraphGestureCallbacks` is reserved for retained/editor gesture lifecycle hooks, while
    `install_callbacks(...)` / `NodeGraphCanvas::with_callbacks(...)` keep the composite seam.
- Remaining M3 scope is still substantial: we still need broader advanced-controller coverage,
  more declarative commit coverage, and additional controlled-mode / parity gates.

### Deliverables

- The `NodeGraphSurfaceBinding` + `NodeGraphController` pair that together provide an instance-style facade and unify:
  - viewport manipulation,
  - common graph queries,
  - canonical update/edit entry points,
  - controlled synchronization helpers.
- Declarative interaction paths that commit through controller/store/transactions.
- A clearer separation between headless/store callbacks and UI gesture callbacks, with view-state callbacks called out as their own middle layer.

### Done criteria

- The recommended declarative editor-grade path no longer commits by mutating `Graph` directly.
- Apps do not need to stitch together store/lookups/view queue/commands manually to get basic
  instance-style ergonomics.
- Undo/redo remains coherent across the declarative editor path.

### Required regression protection

- transaction/undo/redo tests for the new controller path
- at least one declarative drag or marquee gate that proves commit goes through the transaction-safe
  path (landed for controller-backed declarative node drag plus selection/marquee callback paths in
  `paint_only.rs`)
- cancel/release gates for selection-only release, escape cancel, and pointer-cancel transient
  cleanup in the declarative path
- left-release reducer gates for node-drag, pending-only, inactive-toggle-marquee, and no-state
  releases
- pointer-session event gates for left-release finish, non-left ignore, pan-release
  cleanup, and cancel-finish semantics
- shared declarative test-fixture helpers for controller/store callback setup and
  pointer-session finish assertions
- first private `paint_only/pointer_session.rs` module split for release/cancel/session
  host helpers
- second private `paint_only/pointer_move.rs` module split for drag/marquee/hover move
  helpers and outcomes
- cancel reducer + pointer session helper gates for Escape-vs-pointer-cancel divergence around
  already-canceled node drags
- keydown dispatch gates for diag-key parsing, diag view presets, portal-disable cleanup, zoom
  reset, and paint-override toggling
- pointer-down reducer gates for pan-start cleanup plus hit-node, marquee, and empty-space
  clear branches
- pointer-move reducer gates for drag activation, canceled drag cleanup, marquee
  preview/cancel, and hover hit updates
- controlled-mode regression coverage for replace/diff behavior

### Evidence anchors

- `ecosystem/fret-node/src/runtime/store.rs`
- `ecosystem/fret-node/src/runtime/changes.rs`
- `ecosystem/fret-node/src/runtime/lookups.rs`
- `ecosystem/fret-node/src/ui/controller.rs`
- `ecosystem/fret-node/src/ui/binding.rs`
- `ecosystem/fret-node/src/ui/declarative/paint_only.rs`
- `ecosystem/fret-node/src/ui/declarative/paint_only/overlay_elements.rs`
- `ecosystem/fret-node/src/ui/portal.rs`
- `ecosystem/fret-node/src/ui/overlays/group_rename.rs`
- `ecosystem/fret-node/src/ui/overlays/blackboard.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/tests/overlay_blackboard_conformance.rs`
- `ecosystem/fret-node/src/ui/declarative/compat_retained.rs`
- `apps/fret-examples/src/node_graph_demo.rs`
- `apps/fret-examples/src/node_graph_domain_demo.rs`
- `apps/fret-examples/src/node_graph_legacy_demo.rs`

## M4 - Declarative editor-grade interaction and portal closure

Status target: behavior convergence milestone

### Goal

Bring the declarative path much closer to the retained engine on the behaviors that matter most for
real editors.

### Progress note (2026-03-06)

- Retained portal + rename overlay glue now has a controller-first path
  (`NodeGraphPortalHost::with_controller`, `NodeGraphOverlayHost::new(...).with_controller(...)`),
  and `compat_retained` now relies on a controller binding instead of public queue transport props.
- `node_graph_domain_demo` and `compat_retained` now exercise that path, reducing how often new
  app-facing examples need to teach raw `edit_queue` mutation.
- The retained widget test harness is back in sync with the latest `fret-ui` retained bridge
  contracts, so controller-first rename / portal retained conformance gates run again.
- Declarative portal measurement now has a contract-shaped bridge into derived geometry:
  `NodeGraphSurfaceProps.measured_geometry` exposes the shared `MeasuredGeometryStore` seam,
  `record_portal_measured_node_size_in_state(...)` /
  `flush_portal_measured_geometry_state(...)` stage `LayoutQueryRegion` measurements, and derived
  geometry cache keys now include presenter revision so measured node-size publication rebuilds
  geometry/index caches deterministically.
- Declarative motion anchoring now has explicit gates on both sides of the portal/overlay seam:
  dragged hovered nodes update `hover_anchor_store` using drag-adjusted rects, and tooltip anchor
  resolution still prefers dragged portal bounds over stale hover anchors when both are available.
- Declarative portal/hover seams now have dedicated private modules,
  `paint_only/portal_measurement.rs` and `paint_only/hover_anchor.rs`, so the main paint-only
  surface file keeps orchestration responsibility while these contracts stay reviewable in named
  seams.
- Diagnostics-only tooltip/marquee overlay composition now also has a named private seam,
  `paint_only/overlay_elements.rs`, so paint-only overlay rendering keeps policy-shaped element
  assembly out of the main surface file while the next menu/toolbar boundary work can build on
  focused flip/clamp gates instead of large inline branches.
- Compat-retained screen-space overlay placement now also has a shared private seam,
  `ui/screen_space_placement.rs`, so panel / toolbar / rename / blackboard / controls / minimap
  geometry all reuse the same clamp and anchor-placement math while higher-level policy stays in
  the owning overlay widgets.
- Canvas menu/searcher session policy now also has a named private seam,
  `canvas/widget/menu_session.rs`, and `SearcherState` now records explicit `SearcherRowsMode`
  policy so flat-vs-catalog presentation is no longer inferred indirectly from
  `ContextMenuTarget` variants when opening or rebuilding overlay state.
- Insert-node candidate sourcing now also has a named private seam,
  `canvas/widget/insert_candidates.rs`, so background / connection / edge pickers all share the
  same `Reroute` prepend contract and candidate-to-context-menu mapping instead of keeping those
  list-building rules scattered across searcher and edge-insert openers.
- Insert-node execution policy now also has a named private seam,
  `canvas/widget/insert_execution.rs`, so background insert, connection insert, drag-drop fallback,
  and reroute selection flows reuse one `Reroute` create-op branch plus one inserted-node selection
  reducer instead of duplicating post-commit focus and draw-order updates across widget entrypoints.
- Split-edge reroute execution now also has a named private seam,
  `canvas/widget/split_edge_execution.rs`, so edge context actions, double-click gestures, and
  command-open flows all share one reroute split planner, one rejection-toast fallback, and one
  post-commit selection path instead of keeping that edge transaction wiring repeated inline.
- The private insert execution seam now also owns split-edge candidate preview/plan helpers,
  `canvas/widget/insert_execution.rs`, so edge-insert direct actions plus insert-node drag preview
  and drop flows reuse one candidate-aware split planner and one rejection-toast fallback instead of
  re-deriving reroute positions and `plan_split_edge_candidate` branches per entrypoint.
- Connection picker activation now also has a named private seam,
  `canvas/widget/context_menu/connection_execution.rs`, so connection-insert and conversion picker
  actions now share picker activation, planner/result helpers, and suspended-wire resume policy
  instead of keeping that orchestration embedded in `context_menu/activate.rs`.
- Group target selection and edge-target context actions now also have named private seams,
  `canvas/widget/context_menu/{target_selection,edge_execution}.rs`, so group selection sync and
  edge action execution no longer stay duplicated between right-click setup and activation dispatch
  branches.
- Background insert picker activation now also has a named private seam,
  `canvas/widget/context_menu/background_execution.rs`, so background insert planning,
  commit/selection, and rejection-toast handling no longer stay embedded in
  `context_menu/activate.rs`.
- Edge-insert picker activation now also routes through the `canvas/widget/edge_insert` seam, so
  `context_menu/activate.rs` no longer owns candidate lookup plus handoff for the edge-insert
  searcher target.
- Keyboard and pointer menu-item activation now also have a named private seam,
  `canvas/widget/context_menu/selection_activation.rs`, so enabled-item lookup and activation
  payload cloning no longer stay duplicated between `context_menu/input.rs` and
  `context_menu/pointer.rs`.
- Keyboard context-menu navigation and typeahead now also have a named private seam,
  `canvas/widget/context_menu/key_navigation.rs`, so enabled-item stepping and typeahead fallback
  rules no longer stay embedded in `context_menu/input.rs`.
- Group bring-to-front / send-to-back command reducers now also have a named private seam,
  `canvas/widget/group_draw_order.rs`, so selected-group ordering and missing-group merge rules no
  longer stay duplicated inside `command_open.rs`.
- Right-click menu presentation and edge-target selection now also have named private seams,
  `canvas/widget/context_menu/opening.rs` and `context_menu/target_selection.rs`, so menu state
  presentation and edge selection sync no longer stay duplicated in `right_click.rs`.
- Static group/background/edge context-menu items now also have a named private seam,
  `canvas/widget/context_menu/item_builders.rs`, so command-item construction no longer stays
  duplicated between `right_click.rs` and `context_menu/opening.rs`.
- Split-edge reroute outcome handling now also routes through the private
  `canvas/widget/split_edge_execution.rs` seam, so command-open, double-click, and edge context
  actions no longer duplicate the same outcome/toast/application branches.
- Right-click group/edge target hit testing now also routes through the private
  `canvas/widget/context_menu/target_hit.rs` seam, so `right_click.rs` no longer owns the raw
  group-header/group-resize/edge hit-test traversal inline.
- Command-open UI orchestration now also routes through the private
  `canvas/widget/command_ui.rs` seam, so transient dismissal, invoked-at fallback, and common
  paint invalidation no longer stay repeated across `command_open.rs` entrypoints.
- Searcher overlay UI orchestration now also routes through the private
  `canvas/widget/searcher_ui.rs` seam, so overlay install/open/dismiss handling and shared
  event-finish paint invalidation no longer stay repeated across `searcher.rs`,
  `searcher_logic.rs`, `command_open.rs`, and `edge_insert/picker.rs`.
- Searcher row activation and pending-drag arming now also route through the private
  `canvas/widget/searcher_activation.rs` seam, so pointer hit resolution, active-row sync, and
  pointer-up activation/dismiss fallback no longer stay repeated inline in `searcher.rs`.
- Searcher keyboard navigation and query mutation now also route through the private
  `canvas/widget/searcher_input.rs` seam, so active-row stepping, Enter activation handoff, and
  query-edit rebuild triggers no longer stay embedded in `searcher.rs`.
- Searcher pointer hover feedback and wheel scroll state now also route through the private
  `canvas/widget/searcher_pointer.rs` seam, so hovered-row sync, hover-driven active-row
  promotion, and wheel scroll clamping no longer stay embedded in `searcher.rs`.
- `searcher.rs` now also acts as a thin retained event router, while escape / key /
  pointer-down / pointer-up / pointer-move / wheel behavior each delegate to their owning
  private seam instead of keeping event glue in one file.
- Context-menu event glue now also routes through the private
  `canvas/widget/context_menu/ui.rs` seam, so dismiss / restore / paint invalidation rules no
  longer stay duplicated between `context_menu/input.rs` and `context_menu/pointer.rs`.
- `context_menu/input.rs` and `context_menu/pointer.rs` now also act as thin retained event
  routers, delegating key and pointer behavior to `key_navigation.rs` and
  `selection_activation.rs` instead of keeping event glue inline.
- `right_click.rs` now also acts as a thin retained event router, while pending right-click
  click-vs-drag threshold checks are shared between `event_pointer_move.rs` and
  `event_pointer_up.rs`, and context-menu opening delegates into `context_menu/opening.rs`.
- `left_click/handlers.rs` now also routes group-resize / group-header / background branches
  through the private `canvas/widget/left_click/group_background.rs` seam, so group selection
  sync, pending drag/resize arming, and background marquee/pan fallback no longer stay embedded
  in the main hit-dispatch match.
- `left_click/handlers.rs` now also routes port / edge-anchor branches through the private
  `canvas/widget/left_click/connection_hits.rs` seam, so connect-on-click resolution, reconnect
  drag arming, and edge-anchor selection sync no longer stay embedded in the main hit-dispatch
  match.
- `left_click/handlers.rs` now also routes resize / node / edge branches through the private
  `canvas/widget/left_click/element_hits.rs` seam, so node selection sync, drag-handle gating,
  resize arming, and edge alt-insert arming no longer stay embedded in the main hit-dispatch
  match; the file now behaves as a thin retained hit router.
- `pointer_up.rs` now also routes node-resize / group-resize / group-drag / node-drag commit
  branches through the private `canvas/widget/pointer_up_commit.rs` seam, so graph-op commit
  assembly and drag-end outcome labeling no longer stay embedded in the retained pointer release
  router.
- `pointer_up.rs` now also routes pending group / node / wire release branches through the private
  `canvas/widget/pointer_up_pending.rs` seam, so click-distance selection toggles, click-connect
  wire re-arming, and pointer-capture release cleanup no longer stay embedded in the retained
  pointer release router.
- `event_pointer_down.rs` now also routes background-zoom / edge-insert / reroute double-click
  branches through the private `canvas/widget/pointer_down_double_click.rs` seam, so repeated
  edge/background hit filtering and double-click orchestration no longer stay embedded in the
  retained pointer-down router.
- `event_pointer_down.rs` now also routes close-button dispatch, pending right-click arming,
  sticky-wire activation, and pan-start branches through the private
  `canvas/widget/pointer_down_gesture_start.rs` seam, so gesture-start ordering no longer stays
  embedded in the retained pointer-down router.
- `focus_nav.rs` now also routes edge / node / port traversal through the private
  `canvas/widget/focus_nav_traversal.rs` seam, so selection/focus cycling order and auto-pan on
  node focus no longer stay embedded in the port-hint / activation file.
- `focus_nav.rs` now also routes focused-port validity refresh, canvas-center lookup, and
  focused-port activation through the private `canvas/widget/focus_nav_ports.rs` seam, so
  connect-preview simulation and click-connect activation no longer stay embedded in the thin
  focus-nav router.
- `event_pointer_move.rs` now also routes missing pointer-up inference, pending right-click pan
  threshold checks, and retained move-handler dispatch through the private
  `canvas/widget/pointer_move_release.rs` and `canvas/widget/pointer_move_dispatch.rs` seams, so
  release synthesis and move arbitration no longer stay embedded in one retained pointer-move
  router.
- `event_pointer_wheel.rs` now also routes wheel zoom / pan and pinch viewport motion
  through the private `canvas/widget/pointer_wheel_viewport.rs` seam, so viewport-motion
  cancellation, wheel pan math, and pinch zoom math no longer stay embedded in the retained
  wheel router.
- `event_timer.rs` now routes timer-driven viewport and auto-pan motion
  through the private `canvas/widget/timer_motion.rs` seam, so pan inertia, viewport
  animation, auto-pan replay, and move-end debounce no longer stay embedded in the
  retained timer router.
- `event_keyboard.rs` now routes escape / overlay / modifier shortcut / tab /
  nudge / delete handling through the private `canvas/widget/keyboard_shortcuts.rs` seam,
  so key-driven command dispatch and overlay-aware keyboard exits no longer stay embedded in
  the retained keyboard router.
- `event_keyboard.rs` now also routes pan-activation hold/release through the private
  `canvas/widget/keyboard_pan_activation.rs` seam, so space-to-pan arming, release, and
  paint invalidation no longer stay embedded in the retained keyboard router.
- `retained_widget.rs` now routes semantics / layout / prepaint through the private
  `canvas/widget/retained_widget_frame.rs` seam, so viewport semantics value assembly,
  diagnostics-anchor child layout, queue drain-on-layout, and cull-window tracking no
  longer stay embedded in the main retained widget trait router.
- `retained_widget.rs` now also routes command / event / paint runtime bridge
  work through the private `canvas/widget/retained_widget_runtime.rs` seam, so
  style/skin/paint-override sync, text-input command deferral, middleware handoff, and
  middleware-handled redraw/invalidation no longer stay embedded in the main trait router.
- `retained_widget.rs` now also routes command availability through the private
  `canvas/widget/retained_widget_command_availability.rs` seam, so clipboard capability
  gating and selection/content availability checks no longer stay embedded in the main
  retained widget trait router.
- `node_drag.rs` now routes snapline arbitration and preview planning through
  the private `canvas/widget/node_drag_snap.rs` and
  `canvas/widget/node_drag_preview.rs` seams, so snap-guides math and drag-preview
  node/group projection no longer stay embedded in the retained drag router.
- `node_drag.rs` now also routes anchor clamp / extent union / multi-drag
  extent clamp math through the private `canvas/widget/node_drag_constraints.rs` seam,
  so node/group constraint math no longer stays embedded in the retained drag router.

### Deliverables

- declarative selection/marquee reducers with explicit commit/cancel behavior
- declarative node portal hosting for the visible subset
- overlay/tooling composition that uses the right policy surfaces instead of ad-hoc local logic
- deterministic anchoring between canvas, portals, and overlays

### Done criteria

- The declarative path is not just paint-only; it is a believable editor-grade integration path.
- Portal-based node content and overlay anchoring no longer feel like separate experiments.
- Existing retained behavior can be compared against declarative gates without hand-wavy criteria.

### Required regression protection

- keep the promoted paint-only suite green
- add at least one overlay/portal correctness gate
- add at least one declarative interaction parity gate that is meaningful to real editor usage

### Evidence anchors

- `ecosystem/fret-node/src/ui/declarative/paint_only.rs`
- `ecosystem/fret-node/src/ui/screen_space_placement.rs`
- `ecosystem/fret-node/src/ui/canvas/state.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/menu_session.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/insert_candidates.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/insert_execution.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/split_edge_execution.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/connection_execution.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/background_execution.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/edge_execution.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/key_navigation.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/item_builders.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/opening.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/target_hit.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/command_ui.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/searcher_ui.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/searcher_activation.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/searcher_input.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/searcher_pointer.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/searcher.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/ui.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/right_click.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/left_click/group_background.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/left_click/connection_hits.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/left_click/element_hits.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_commit.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_pending.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_down_double_click.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_down_gesture_start.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/focus_nav_traversal.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/focus_nav_ports.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/event_pointer_move.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_move_release.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_move_dispatch.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/event_pointer_wheel.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_wheel_viewport.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/event_timer.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/timer_motion.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/event_keyboard.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/keyboard_shortcuts.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/keyboard_pan_activation.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/retained_widget.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/retained_widget_frame.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/retained_widget_runtime.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/retained_widget_command_availability.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/node_drag.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/node_drag_snap.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/node_drag_preview.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/node_drag_constraints.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/selection_activation.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/target_selection.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/group_draw_order.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/edge_insert/insert.rs`
- `ecosystem/fret-node/src/ui/portal.rs`
- `ecosystem/fret-node/src/ui/overlays/group_rename.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/tests/overlay_group_rename_conformance.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/tests/portal_lifecycle_conformance.rs`
- `apps/fret-examples/src/node_graph_domain_demo.rs`
- `apps/fret-examples/src/node_graph_legacy_demo.rs`
- `tools/diag-scripts/node-graph/`

## M5 - Compatibility retained convergence and deletion gate

Status target: governance milestone, not necessarily immediate code deletion

### Goal

Define the conditions under which the retained compatibility path can stop growing and eventually be
removed or permanently demoted.

### Deliverables

- Explicit retention policy for `compat-retained-canvas`:
  - what it still covers,
  - what new work must not be added there,
  - what declarative parity conditions are required before deletion.
- A stable gate matrix comparing compatibility retained vs declarative behavior where it matters.

### Done criteria

- The compatibility path has a bounded role rather than an open-ended future.
- New editor work does not default to "just add it to retained first" without justification.
- Reviewers can tell whether a retained-only addition is acceptable, temporary, or out of scope.

### Evidence anchors

- `ecosystem/fret-node/Cargo.toml`
- `ecosystem/fret-node/src/ui/declarative/compat_retained.rs`
- `apps/fret-examples/src/node_graph_demo.rs`
- `apps/fret-examples/src/node_graph_legacy_demo.rs`
- `docs/node-graph-controlled-mode.md`
- `docs/workstreams/fret-node-declarative-fearless-refactor-v1/README.md`

## Suggested milestone order

Land in this order unless a blocking bug forces a smaller detour:

1. `M0` docs and decision gates
2. `M1` public posture closure
3. `M2` state boundary split
4. `M3` controller + transaction-safe declarative commits
5. `M4` declarative interaction/portal closure
6. `M5` retained compatibility convergence
