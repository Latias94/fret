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
- `ecosystem/fret-node/src/ui/controller_queries.rs`
- `ecosystem/fret-node/src/ui/controller_viewport.rs`
- `ecosystem/fret-node/src/ui/controller_store_sync.rs`
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
- Declarative controller-backed transaction/view-state helpers and selection preview/commit
  reducers now also live under the private `paint_only/transactions.rs` and
  `paint_only/selection.rs` seams, so the main paint-only surface keeps orchestration
  responsibility instead of re-embedding transaction plumbing and selection-state writes inline.
- Declarative left-pointer down snapshot/arming helpers now also live under the private
  `paint_only/pointer_down.rs` seam, so the main paint-only surface stops re-embedding pan arming,
  hit snapshot reads, and selection/marquee arming branches inline.
- Declarative diagnostics hotkeys, preset application, and keyboard zoom reducers now also live
  under the private `paint_only/diag.rs` seam, so the main paint-only surface stops re-embedding
  keyboard/diagnostics branching inline.
- Declarative grid/derived/nodes/edges cache warmers now also live under the private
  `paint_only/cache.rs` seam, so the main paint-only surface stops re-embedding cache rebuild
  sequencing inline while keeping the same cache-key contracts.
- Declarative local uncontrolled-model/bootstrap wiring now also lives under the private
  `paint_only/surface_models.rs` seam, so the main paint-only surface stops re-embedding local
  state bundle construction inline.
- Declarative visible-subset portal hosting and deferred `fit-to-portals` viewport application now
  also live under the private `paint_only/portals.rs` seam, so the main paint-only surface stops
  re-embedding portal subtree hosting, bounds-store pruning, and pending-fit orchestration inline
  while keeping the same dragged-rect visibility and portal-bounds contracts.
- Declarative diagnostics hover-tooltip overlay orchestration now also lives under the private
  `paint_only/overlays.rs` seam, so the main paint-only surface stops re-embedding hover-anchor
  reads, portal-bounds fallback, and tooltip element wiring inline while keeping the same
  portal-vs-hover anchor precedence contract.
- Declarative marquee overlay append and final overlay-layer wrapping now also live under the
  private `paint_only/overlays.rs` seam, so the main paint-only surface stops re-embedding
  overlay child flush/wrap plumbing inline and keeps the overlay stack reviewable from one seam.
- Declarative edge/portal diagnostics aggregation and semantics value assembly now also live under
  the private `paint_only/semantics.rs` seam, so the main paint-only surface stops re-embedding
  observability counters and long semantics formatting inline while keeping the same diagnostics
  contract for script gates.
- Declarative key/pointer/wheel/pinch handler construction now also lives under the private
  `paint_only/input_handlers.rs` seam, so the main paint-only surface stops re-embedding the full
  event closure builder set inline while keeping the same reducer/effect contracts.
- Declarative grid/derived/node/edge cache key generation, draw model construction, and canvas
  paint helpers now also live under the private `paint_only/cache.rs` seam, so the main
  paint-only surface stops re-embedding retained-like cache/paint implementation blocks inline
  while keeping the same invalidation and diagnostics contracts.
- Declarative surface state snapshots, authoritative-boundary sync, portal measured-geometry
  flush, cache refresh, and semantics preparation now also live under the private
  `paint_only/surface_frame.rs` seam, so the main paint-only surface stops re-embedding the full
  pre-render context preparation block inline while keeping the same invalidation and semantics
  contracts.
- Declarative canvas paint closure wiring, portal hosting, hover-anchor sync, hover tooltip
  append, fit-to-portals replay, marquee overlay append, and final overlay flush now also live
  under the private `paint_only/surface_content.rs` seam, so the main paint-only surface stops
  re-embedding the full post-handler render/output block inline while keeping the same portal and
  overlay contracts.
- Declarative bounds sync, keyboard/pointer gesture handler wiring, and pointer-region shell
  assembly now also live under the private `paint_only/surface_shell.rs` seam, so the main
  paint-only surface stops re-embedding the full `semantics_with_id(..., move |cx, element| { ...
  })` shell block inline while keeping the same focus/input contracts.
- Declarative geometry quantization, rectangle helpers, marquee math, node-drag delta/commit
  helpers, and point hit-testing now also live under the private `paint_only/surface_math.rs`
  seam, so the main paint-only surface stops re-embedding the shared geometry and gesture math
  helper set inline while keeping the same drag and hit-test contracts.
- Declarative uncontrolled-model bootstrap, mouse-button/hash helpers, and authoritative
  surface-boundary snapshot/sync now also live under the private `paint_only/surface_support.rs`
  seam, while diagnostic visible-node transaction builders now live beside the rest of the
  diagnostic policy in `paint_only/diag.rs`, so the main paint-only surface stops re-embedding
  these support and diagnostic helper blocks inline while keeping the same model-bootstrap,
  invalidation, and diag contracts.
- `ui/controller.rs` now also routes controller queries, viewport/fit-view helpers, and
  store-sync/replace/selection transport through the private `controller_queries.rs`,
  `controller_viewport.rs`, and `controller_store_sync.rs` seams, so the app-facing
  `NodeGraphController` surface stops re-embedding the full query + viewport + queue/store
  orchestration implementation inline while keeping the same binding/controller contract.
- `ui/canvas/widget.rs` now also routes the retained canvas surface impl through the private
  `canvas/widget/widget_surface.rs` seam, so the root widget module stops re-embedding constructor,
  config/style-sync, fit-on-mount, and shared cull/render helper orchestration inline while
  keeping the same retained canvas contract and module map.
- repeated `InteractionState` gate predicates for cursor/hover/edge-cache/pan-inertia now also
  route through the private `canvas/widget/interaction_gate.rs` seam, so widget submodules stop
  re-embedding the same busy/idle interaction checks inline while keeping the same behavior.
- repeated focus-session mutations for focused edge/node/port transitions and selection-only sync
  now also route through the private `canvas/widget/focus_session.rs` seam, so focus navigation
  helpers stop re-embedding the same focus-reset and selection-update blocks inline while keeping
  the same behavior.
- cancel-path residual cleanup, hover/focus reset, and pan-drag reset now also route through the
  private `canvas/widget/cancel_session.rs` seam, so cancel/pointer-up helpers stop re-embedding
  the same interaction cleanup blocks inline while keeping the same behavior.
- left-click hit routes, pan-zoom start, marquee selection, and wire-commit cleanup now also reuse
  the expanded private `canvas/widget/focus_session.rs` seam, so pointer-down helpers stop
  re-embedding the same edge-focus and hover-port hint resets inline while keeping the same
  behavior.
- left-click pointer-down preparation and pan-start competing-session cleanup now also route
  through the private `canvas/widget/press_session.rs` seam, so retained widget hit handlers stop
  re-embedding the same pending-drag / marquee / edge-insert reset blocks inline while keeping the
  same behavior.
- pending pointer-up release finish helpers and node-drag release residual cleanup now also route
  through the private `canvas/widget/pointer_up_session.rs` seam, so pointer-up handlers stop
  re-embedding the same pending-slot finish and snap-guide cleanup blocks inline while keeping the
  same behavior.
- pointer-up commit releases, marquee finish cleanup, and pending resize activation now also route
  through the private `canvas/widget/pointer_up_session.rs` and
  `canvas/widget/pending_resize_session.rs` seams, so release/activation helpers stop
  re-embedding the same companion-slot clearing and resize-activation blocks inline while keeping
  the same behavior.
- pending node/group drag activation and early-abort cleanup now also route through the private
  `canvas/widget/pending_drag_session.rs` seam, so pending drag helpers stop re-embedding the same
  pending-slot abort and activation-state construction blocks inline while keeping the same
  behavior.
- pending insert-node drag abort/finish and pending wire/edge-insert activation now also route
  through the private `canvas/widget/insert_node_drag/session.rs` and
  `canvas/widget/pending_connection_session.rs` seams, so insert/connection pending helpers stop
  re-embedding the same capture-release and pending-to-active construction blocks inline while
  keeping the same behavior.
- insert-node drag internal enter/leave/drop handling now also routes preview invalidation and
  drag-event finish through the private `canvas/widget/insert_node_drag/session.rs` seam, so
  `internal_move.rs`, `internal_drop.rs`, and `insert_node_drag/mod.rs` stop re-embedding the same
  preview repaint and propagation-stop tail blocks inline while keeping the same behavior.
- searcher overlay dismissal and row-drag release now also route through the private
  `canvas/widget/searcher_activation_state.rs` seam, so `searcher_activation.rs` and
  `searcher_ui.rs` stop re-embedding the same pending insert-drag clearing and capture-release
  state transitions inline while keeping the same behavior.
- command-driven transient dismissal now also routes searcher close through the private
  `canvas/widget/searcher_activation_state.rs` seam, so `command_ui.rs` stops clearing the
  searcher overlay without also clearing pending searcher row-drag state while keeping the same
  behavior.
- cancel gesture cleanup for insert-node drag now also routes through the private
  `canvas/widget/insert_node_drag/session.rs` seam, so `cancel_gesture_state.rs` stops
  re-embedding pending-insert and preview-slot clearing inline while keeping the same behavior.
- context-menu close/restore state now also routes through the private
  `canvas/widget/context_menu/ui.rs` seam, so `command_ui.rs`, `searcher_ui.rs`,
  `context_menu/activate.rs`, and conversion-picker handoff in `wire_drag/commit/new_wire.rs`
  stop re-embedding the same context-menu slot clearing inline while keeping the same behavior.
- context-menu slot take/restore now also routes through the private
  `canvas/widget/context_menu/ui.rs` seam, so `context_menu/opening.rs`,
  `context_menu/selection_activation.rs`, and `context_menu/key_navigation.rs` stop
  re-embedding the same menu-slot mutation inline while keeping the same behavior.
- command redraw tails now also route through the private `canvas/widget/command_ui.rs` seam, so
  `command_history.rs`, `command_mode.rs`, `command_selection.rs`, `command_view.rs`,
  `command_move.rs`, `command_edit.rs`, `command_edit_remove.rs`, `command_focus_cycle.rs`, and
  `command_focus_port.rs` stop re-embedding the same redraw-plus-paint-invalidation tail blocks
  inline while keeping the same behavior.
- event/timer paint invalidation now also routes through the private
  `canvas/widget/paint_invalidation.rs` seam, so `event_clipboard_feedback.rs`,
  `event_timer_toast.rs`, `timer_motion_shared.rs`, `keyboard_pan_activation.rs`,
  `pointer_wheel_pan.rs`, and `pointer_wheel_zoom.rs` stop re-embedding the same event-scope
  redraw-plus-paint-invalidation tail blocks inline while keeping the same behavior.
- edge-drag / edge-insert drag / double-click / pointer-up event tails now also route through the
  same private `canvas/widget/paint_invalidation.rs` seam, so `edge_drag/move_start.rs`,
  `edge_drag/pointer_up.rs`, `edge_insert_drag/drag.rs`, `edge_insert_drag/pending.rs`,
  `pointer_down_double_click_background.rs`, `pointer_down_double_click_edge.rs`,
  `pointer_down_gesture_start.rs`, and `pointer_up_finish.rs` stop re-embedding the same
  event-scope redraw-plus-paint-invalidation tail blocks inline while keeping the same behavior.
- pan/marquee/group-drag/hover wire-drag event tails now also route through the same private
  `canvas/widget/paint_invalidation.rs` seam, so `pan_zoom_begin.rs`, `pan_zoom_move.rs`,
  `marquee_begin.rs`, `group_drag.rs`, `group_resize.rs`, `hover.rs`, and `wire_drag_helpers.rs`
  stop re-embedding the same event-scope redraw-plus-paint-invalidation tail blocks inline while
  keeping the same behavior.
- pointer-up / left-click / marquee-selection / node-drag / sticky-wire event tails now also route
  through the same private `canvas/widget/paint_invalidation.rs` seam, so
  `pointer_up_left_route.rs`, `pointer_up_state.rs`, `left_click/group_background.rs`,
  `left_click/connection_hits.rs`, `left_click/element_hits.rs`, `marquee_selection.rs`,
  `node_drag.rs`, `node_resize/move_update.rs`, `sticky_wire_connect.rs`, and
  `sticky_wire_targets.rs` stop re-embedding the same event-scope redraw-plus-paint-invalidation
  tail blocks inline while keeping the same behavior.
- cancel / context-menu / searcher / insert-node-drag event tails now also route through the same
  private `canvas/widget/paint_invalidation.rs` seam, so `cancel_cleanup.rs`,
  `context_menu/ui.rs`, `context_menu/opening.rs`, `searcher_ui.rs`, and
  `insert_node_drag/session.rs` stop re-embedding the same event-scope
  redraw-plus-paint-invalidation tail blocks inline while keeping the same behavior.
- command / retained-runtime / wire-commit paint tails now also route through small private helper
  seams, so `command_ui.rs`, `retained_widget_runtime_shared.rs`, `wire_drag/commit_cx.rs`,
  `wire_drag/commit/mod.rs`, and `wire_drag/move_update/mod.rs` stop re-embedding the same
  redraw-plus-paint-invalidation tail blocks inline while keeping the same behavior.
- paint/layout redraw requests now also route through the private
  `canvas/widget/redraw_request.rs` seam, so `paint_grid_cache.rs`, `paint_edges/main.rs`,
  `paint_root/cached_edges/single_rect.rs`, `paint_root/cached_edges/tile_path.rs`,
  `retained_widget_layout_drain.rs`, and `wire_drag/commit_cx.rs` stop re-embedding the same
  next-frame redraw request blocks inline while keeping the same behavior.
- `paint_edges/main.rs` now also routes hash/glow-bounds helper logic through the private
  `canvas/widget/paint_edges/support.rs` seam, so the root edge-paint surface stops re-embedding
  the full helper set for stable cache keys and glow bounds math inline while keeping the same
  behavior.
- `paint_edges/main.rs` now also routes drop-marker drawing and wire-drag preview style/path
  emission through the private `canvas/widget/paint_edges/preview.rs` seam, so the root
  edge-paint surface stops re-embedding preview marker geometry and preview wire paint
  orchestration inline while keeping the same behavior.
- `paint_edges/main.rs` now also routes selected/base outline paint, selected glow effect setup,
  and selected/hovered highlight resolution through the private
  `canvas/widget/paint_edges/chrome.rs` seam, so the root edge-paint surface stops re-embedding
  edge chrome orchestration inline while keeping the same wire/marker draw behavior.
- `paint_edges/main.rs` now also routes edge paint batch preparation plus edge-insert /
  insert-node-drop marker projection through the private `canvas/widget/paint_edges/prepare.rs`
  seam, so the root edge-paint surface stops re-embedding edge width classification and marker
  projection setup inline.
- `paint_edges/main.rs` now also routes static edge-label drawing plus label/marker budget stats
  publication through the private `canvas/widget/paint_edges/labels.rs` seam, so the root
  edge-paint surface stops re-embedding label tail orchestration and budget registry publication
  inline.
- `paint_edges/main.rs` now also routes the main edge wire/marker paint pass plus paint-budget
  bookkeeping through the private `canvas/widget/paint_edges/pass.rs` seam, so the root
  edge-paint surface stops re-embedding the full edge iteration loop and redraw-budget
  bookkeeping inline.
- cached-edge single-rect/tiled label replay and single-rect label build orchestration now also
  route through the private `canvas/widget/paint_root/cached_edges/labels.rs` seam, so
  `paint_root/cached_edges/single_rect.rs` and `paint_root/cached_edges/tile_path.rs` stop
  re-embedding the same label-cache replay closure or the single-rect label build tail inline.
- command / retained-runtime / wire-commit paint tails now also route through small private helper
  seams, so `command_ui.rs`, `retained_widget_runtime_shared.rs`, `wire_drag/commit_cx.rs`,
  `wire_drag/commit/mod.rs`, and `wire_drag/move_update/mod.rs` stop re-embedding the same
  redraw-plus-paint-invalidation tail blocks inline while keeping the same behavior.
- `ui/canvas/paint.rs` now also routes wire-path prep, port-shape factories, edge-marker
  factories, and text cache helpers through the private `canvas/paint/paint_wire.rs`,
  `canvas/paint/paint_ports.rs`, `canvas/paint/paint_markers.rs`, and
  `canvas/paint/paint_text.rs` seams, so the root paint module stops re-embedding path-factory and
  text-cache implementation blocks inline while keeping the same cache contract and lifecycle
  surface.
- `ui/canvas/spatial.rs` now also routes coarse index construction, port-edge adjacency, and
  derived spatial wrapper helpers through the private `canvas/spatial/spatial_index.rs`,
  `canvas/spatial/spatial_adjacency.rs`, and `canvas/spatial/spatial_derived.rs` seams, so the
  root spatial module stops re-embedding index/adjacency/derived implementation blocks inline while
  keeping the same spatial cache contract and test surface.
- `ui/canvas/state.rs` now also routes paste-series stepping, viewport easing, and geometry-cache
  preview/key helpers through the private `canvas/state/state_paste_series.rs`,
  `canvas/state/state_viewport_animation.rs`, and `canvas/state/state_geometry_cache.rs` seams, so
  the root state module stops re-embedding isolated helper impl blocks inline while keeping the
  same shared state contract and tests.
- `ui/canvas/state.rs` now also routes menu/searcher/toast/paste session structs through the
  private `canvas/state/state_overlay_sessions.rs` seam, and derived geometry cache key / preview
  cache structs through the private `canvas/state/state_preview_cache.rs` seam, so the root state
  module stops re-embedding pure data clusters inline while keeping the same state paths and tests.
- `ui/canvas/state.rs` now also routes insert/node/group/marquee/wire/edge drag session structs
  through the private `canvas/state/state_drag_sessions.rs` seam, so the root state module stops
  re-embedding the drag-session data inventory inline while keeping the same state paths and tests.
- `ui/canvas/workflow.rs` now also routes wire-drop insert planning through the private
  `canvas/workflow/workflow_insert.rs` seam, so the root workflow module stops re-embedding the
  insert/autoconnect planner inline while keeping the same root export and tests.
- `ui/canvas/searcher.rs` now also routes query scoring/normalization and row builders through the
  private `canvas/searcher/searcher_score.rs` and `canvas/searcher/searcher_build.rs` seams, so the
  root searcher module stops re-embedding scoring/catalog assembly helpers inline while keeping the
  same row types, constants, and root exports.
- `ui/canvas/middleware.rs` now also routes middleware chaining and transaction validation adapters
  through the private `canvas/middleware/middleware_chain.rs` and
  `canvas/middleware/middleware_validation.rs` seams, so the root middleware module stops
  re-embedding chain/validation implementations inline while keeping the same trait/context
  contracts and root exports.
- `ui/canvas/route_math.rs` now also routes curve primitives and route tangent helpers through the
  private `canvas/route_math/route_math_curve.rs` and
  `canvas/route_math/route_math_tangent.rs` seams, while `ui/canvas/conversion.rs` now routes
  conversion candidate building and insert-plan helpers through the private
  `canvas/conversion/conversion_candidates.rs` and
  `canvas/conversion/conversion_plan.rs` seams, so both root modules stop re-embedding small pure
  helper blocks inline while keeping the same root exports.
- `ui/canvas/snaplines.rs` now also routes snap-anchor extraction and best-guide delta selection
  through the private `canvas/snaplines/snaplines_align.rs` seam, so the root snaplines module
  stops re-embedding small pure alignment helpers inline while keeping the same result contract and
  tests.
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
- `pointer_up.rs` now routes pointer-up state sync, sticky-wire
  ignore handling, pan-release unwind, and left-button release ordering through the
  private `canvas/widget/pointer_up_state.rs` and
  `canvas/widget/pointer_up_left_route.rs` seams, so retained pointer release routing
  no longer keeps state sync and left-tail fallback ordering embedded in one surface.
- `event_pointer_down.rs` now also routes close-button dispatch, pending right-click arming,
  sticky-wire activation, and pan-start branches through the private
  `canvas/widget/pointer_down_gesture_start.rs` seam, so gesture-start ordering no longer stays
  embedded in the retained pointer-down router.
- `event_pointer_down.rs` now routes pointer-down interaction priming
  and final left/right/ignore tail dispatch through the private
  `canvas/widget/event_pointer_down_state.rs` and
  `canvas/widget/event_pointer_down_route.rs` seams, so retained pointer-down routing
  no longer keeps timer-stop/state-sync setup and tail button fallback embedded in one surface.
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
- `event_pointer_move.rs` now routes modifier/multi-select state sync,
  last-pointer seeding, and cursor/auto-pan tail work through the private
  `canvas/widget/event_pointer_move_state.rs` and
  `canvas/widget/event_pointer_move_tail.rs` seams, so retained pointer-move routing no
  longer keeps move-state priming and tail post-dispatch sync embedded in one surface.
- `event_pointer_wheel.rs` now also routes wheel zoom / pan and pinch viewport motion
  through the private `canvas/widget/pointer_wheel_viewport.rs` seam, so viewport-motion
  cancellation, wheel pan math, and pinch zoom math no longer stay embedded in the retained
  wheel router.
- `event_pointer_wheel.rs` now routes wheel modifier state sync
  and scroll/pinch event dispatch through the private
  `canvas/widget/event_pointer_wheel_state.rs` and
  `canvas/widget/event_pointer_wheel_route.rs` seams, so retained wheel routing no
  longer keeps modifier priming and scroll/pinch entry dispatch embedded in one surface.
- `event_timer.rs` now routes timer-driven viewport and auto-pan motion
  through the private `canvas/widget/timer_motion.rs` seam, so pan inertia, viewport
  animation, auto-pan replay, and move-end debounce no longer stay embedded in the
  retained timer router.
- `event_timer.rs` now also routes toast expiry cleanup and timer-motion sequencing
  through the private `canvas/widget/event_timer_toast.rs` and
  `canvas/widget/event_timer_route.rs` seams, so retained timer handling no longer keeps
  toast dismissal and motion/debounce dispatch ordering embedded in one surface.
- `event_router.rs` now routes non-pointer lifecycle dispatch and pointer-variant
  dispatch through the private `canvas/widget/event_router_system.rs` and
  `canvas/widget/event_router_pointer.rs` seams, so clipboard/focus cancel,
  internal-drag/timer/keyboard routing, and pointer-variant branching no longer stay
  embedded in one retained event router surface.
- `event_router_system.rs` now routes lifecycle/system events and keyboard input
  dispatch through the private `canvas/widget/event_router_system_lifecycle.rs` and
  `canvas/widget/event_router_system_input.rs` seams, so retained non-pointer routing no
  longer keeps clipboard/focus/timer/internal-drag handling and keyboard dispatch in
  one surface.
- `event_router_pointer.rs` now routes button-pointer dispatch and wheel/pinch
  dispatch through the private `canvas/widget/event_router_pointer_button.rs` and
  `canvas/widget/event_router_pointer_wheel.rs` seams, so retained pointer routing no
  longer keeps down/move/up branching and wheel/pinch branching in one surface.
- `event_clipboard.rs` now routes pending-paste token resolution and clipboard
  feedback side effects through the private
  `canvas/widget/event_clipboard_pending.rs` and
  `canvas/widget/event_clipboard_feedback.rs` seams, so retained clipboard event
  handling no longer keeps token matching/requeue logic and toast/redraw feedback
  embedded in one surface.
- `event_keyboard.rs` now routes escape / overlay / modifier shortcut / tab /
  nudge / delete handling through the private `canvas/widget/keyboard_shortcuts.rs` seam,
  so key-driven command dispatch and overlay-aware keyboard exits no longer stay embedded in
  the retained keyboard router.
- `event_keyboard.rs` now also routes pan-activation hold/release through the private
  `canvas/widget/keyboard_pan_activation.rs` seam, so space-to-pan arming, release, and
  paint invalidation no longer stay embedded in the retained keyboard router.
- `event_keyboard.rs` now routes text-input gating, multi-selection
  modifier sync, and keydown/up dispatch ordering through the private
  `canvas/widget/event_keyboard_state.rs` and
  `canvas/widget/event_keyboard_route.rs` seams, so retained keyboard entry handling no
  longer keeps state priming and top-level key routing embedded in one surface.
  paint invalidation no longer stay embedded in the retained keyboard router.
- `retained_widget.rs` now routes semantics / layout / prepaint through the private
  `canvas/widget/retained_widget_frame.rs` seam, so viewport semantics value assembly,
  diagnostics-anchor child layout, queue drain-on-layout, and cull-window tracking no
  longer stay embedded in the main retained widget trait router.
- `retained_widget.rs` now also routes command / event / paint runtime bridge
  work through the private `canvas/widget/retained_widget_runtime.rs` seam, so
  style/skin/paint-override sync, text-input command deferral, middleware handoff, and
  middleware-handled redraw/invalidation no longer stay embedded in the main trait router.
- `retained_widget_runtime.rs` now further routes retained command / event / paint
  bridge work through the private
  `canvas/widget/retained_widget_runtime_command.rs`,
  `canvas/widget/retained_widget_runtime_event.rs`,
  `canvas/widget/retained_widget_runtime_paint.rs`, and
  `canvas/widget/retained_widget_runtime_shared.rs` seams, so runtime theme sync,
  middleware context assembly, text-input command deferral, and handled invalidation no
  longer stay embedded in one retained runtime surface.
- `retained_widget.rs` now also routes command availability through the private
  `canvas/widget/retained_widget_command_availability.rs` seam, so clipboard capability
  gating and selection/content availability checks no longer stay embedded in the main
  retained widget trait router.
- `retained_widget_command_availability.rs` now routes focus/clipboard gating and
  graph/view-state availability queries through the private
  `canvas/widget/retained_widget_command_availability_gate.rs` and
  `canvas/widget/retained_widget_command_availability_query.rs` seams, so retained edit
  command availability no longer keeps capability checks and selection/content reads
  embedded in one surface.
- `node_drag.rs` now routes snapline arbitration and preview planning through
  the private `canvas/widget/node_drag_snap.rs` and
  `canvas/widget/node_drag_preview.rs` seams, so snap-guides math and drag-preview
  node/group projection no longer stay embedded in the retained drag router.
- `node_drag.rs` now also routes anchor clamp / extent union / multi-drag
  extent clamp math through the private `canvas/widget/node_drag_constraints.rs` seam,
  so node/group constraint math no longer stays embedded in the retained drag router.
- `paint_grid.rs` now routes tile scene-op generation through the private
  `canvas/widget/paint_grid_tiles.rs` seam, so grid line/dot/cross emission and
  focused pattern tests no longer stay embedded in the retained grid cache/router surface.
- `pointer_up_commit.rs` now routes node-drag release commit
  assembly through the private `canvas/widget/pointer_up_node_drag.rs` seam and shares
  pointer-capture teardown via `canvas/widget/pointer_up_finish.rs`, so retained
  pointer-up finalize logic no longer stays duplicated across commit/pending reducers.
- `focus_nav_traversal.rs` now routes edge/node/port cycle
  traversal through the private `canvas/widget/focus_nav_traversal_edge.rs`,
  `canvas/widget/focus_nav_traversal_node.rs`, and
  `canvas/widget/focus_nav_traversal_port.rs` seams, so retained focus-cycle reducers
  no longer stay embedded in a single traversal surface.
- `focus.rs` now routes focused-edge repair, draw-order fingerprinting, and
  directional port navigation through the private `canvas/widget/focus_edge_repair.rs`,
  `canvas/widget/focus_draw_order.rs`, and `canvas/widget/focus_port_direction.rs` seams,
  so retained focus-maintenance helpers no longer stay embedded in one mixed utility surface.
- `callbacks.rs` now routes graph commit/delete dispatch, connect lifecycle
  callbacks, and viewport/node-drag/view-change emissions through the private
  `canvas/widget/callbacks_graph.rs`, `canvas/widget/callbacks_connect.rs`, and
  `canvas/widget/callbacks_view.rs` seams, so retained callback orchestration no longer stays
  embedded in one mixed surface.
- `clipboard.rs` now routes paste-anchor math, clipboard host effects, and
  paste/duplicate transaction assembly through the private
  `canvas/widget/clipboard_anchor.rs`, `canvas/widget/clipboard_transfer.rs`, and
  `canvas/widget/clipboard_paste.rs` seams, so retained clipboard reducers no longer stay
  embedded in one mixed surface.
- `marquee.rs` now routes background-marquee arming, active selection
  updates, threshold/pan arbitration, and pointer-up cleanup through the private
  `canvas/widget/marquee_begin.rs`, `canvas/widget/marquee_selection.rs`,
  `canvas/widget/marquee_pending.rs`, and `canvas/widget/marquee_finish.rs` seams,
  so retained marquee reducers no longer stay embedded in one mixed surface.
- `cancel.rs` now routes gesture-state teardown, viewport-motion cancellation,
  and final cleanup through the private `canvas/widget/cancel_gesture_state.rs`,
  `canvas/widget/cancel_viewport_state.rs`, and `canvas/widget/cancel_cleanup.rs` seams,
  so retained cancel reducers no longer stay embedded in one mixed surface.
- `pan_zoom.rs` now routes zoom cache mutation, pan-start arbitration, and
  pan-move velocity updates through the private `canvas/widget/pan_zoom_zoom.rs`,
  `canvas/widget/pan_zoom_begin.rs`, and `canvas/widget/pan_zoom_move.rs` seams,
  so retained pan/zoom reducers no longer stay embedded in one mixed surface.
- `insert_execution.rs` now routes candidate point resolution, graph-op
  planning, and insertion feedback through the private
  `canvas/widget/insert_execution_point.rs`, `canvas/widget/insert_execution_plan.rs`, and
  `canvas/widget/insert_execution_feedback.rs` seams, so retained insert execution reducers no
  longer stay embedded in one mixed surface.
- `pointer_down_double_click.rs` now routes background zoom and edge double-click
  actions through the private `canvas/widget/pointer_down_double_click_background.rs`
  and `canvas/widget/pointer_down_double_click_edge.rs` seams, so retained double-click reducers no
  longer stay embedded in one mixed surface.
- `command_open.rs` now routes insert-picker positioning, group command reducers,
  split-edge open/reroute actions, and conversion-picker presentation through the private
  `canvas/widget/command_open_insert.rs`, `canvas/widget/command_open_group.rs`,
  `canvas/widget/command_open_edge.rs`, and `canvas/widget/command_open_conversion.rs` seams,
  so retained command-open reducers no longer stay embedded in one mixed surface.
- `focus_nav_ports.rs` now routes focused-port validation hints, port-center
  lookup, and click-connect activation handoff through the private
  `canvas/widget/focus_nav_ports_hints.rs`, `canvas/widget/focus_nav_ports_center.rs`, and
  `canvas/widget/focus_nav_ports_activation.rs` seams, so retained focused-port reducers no
  longer stay embedded in one mixed surface.
- `paint_grid.rs` now routes grid paint planning, tile-cache warmup, and cache
  stats publication through the private `canvas/widget/paint_grid_plan.rs`,
  `canvas/widget/paint_grid_cache.rs`, and `canvas/widget/paint_grid_stats.rs` seams,
  so retained grid-paint orchestration no longer stays embedded in one mixed surface.
- `focus_port_direction.rs` now routes wire-drag direction filtering and
  directional candidate ranking through the private
  `canvas/widget/focus_port_direction_candidate.rs` seam, and focus/view-state application
  through `canvas/widget/focus_port_direction_apply.rs`, so retained directional port-focus
  reducers no longer stay embedded in one mixed surface.
- `sticky_wire.rs` now routes connect-target planning/reject handling and
  non-port picker routing through the private `canvas/widget/sticky_wire_connect.rs` and
  `canvas/widget/sticky_wire_targets.rs` seams, so retained sticky-wire pointer reducers no
  longer stay embedded in one mixed surface.
- `pointer_move_release.rs` now routes pan-release cleanup, right-click pan
  arming, missing-left-release finalization, and last-pointer-state sync through the
  private `canvas/widget/pointer_move_release_pan.rs`,
  `canvas/widget/pointer_move_release_left.rs`, and
  `canvas/widget/pointer_move_pointer_state.rs` seams, so retained move-release reducers no
  longer stay embedded in one mixed surface.
- `pointer_wheel_viewport.rs` now routes wheel/pinch motion stop,
  zoom application, and scroll-pan updates through the private
  `canvas/widget/pointer_wheel_motion.rs`, `canvas/widget/pointer_wheel_zoom.rs`, and
  `canvas/widget/pointer_wheel_pan.rs` seams, so retained wheel-viewport reducers no longer
  stay embedded in one mixed surface.
- `searcher_logic.rs` now routes recent-kind/row rebuild helpers, row
  activation handoff, and picker-opening orchestration through the private
  `canvas/widget/searcher_rows.rs`, `canvas/widget/searcher_row_activation.rs`, and
  `canvas/widget/searcher_picker.rs` seams, so retained searcher logic no longer stays
  embedded in one mixed surface.
- `command_focus.rs` now routes cycle commands and directional/activate
  commands through the private `canvas/widget/command_focus_cycle.rs` and
  `canvas/widget/command_focus_port.rs` seams, so retained focus command wrappers no longer
  stay embedded in one mixed surface.
- `retained_widget_frame.rs` now routes semantics sync, layout/update
  orchestration, and prepaint cull-window tracking through the private
  `canvas/widget/retained_widget_semantics.rs`,
  `canvas/widget/retained_widget_layout.rs`, and
  `canvas/widget/retained_widget_cull_window.rs` seams, so retained widget frame
  orchestration no longer stays embedded in one mixed surface.
- `retained_widget_semantics.rs` now routes active-descendant lookup and
  semantics value assembly through the private
  `canvas/widget/retained_widget_semantics_focus.rs` and
  `canvas/widget/retained_widget_semantics_value.rs` seams, so retained semantics sync
  no longer keeps descendant arbitration and accessibility value string assembly in one
  surface.
- `retained_widget_layout.rs` now routes model observation, diagnostics publish,
  child layout, and post-layout queue drain through the private
  `canvas/widget/retained_widget_layout_observe.rs`,
  `canvas/widget/retained_widget_layout_publish.rs`,
  `canvas/widget/retained_widget_layout_children.rs`, and
  `canvas/widget/retained_widget_layout_drain.rs` seams, so retained layout sync no
  longer keeps mixed observation, diagnostics, child placement, and queue drain logic in
  one surface.
- `retained_widget_cull_window.rs` now routes cull-window gating/key derivation and
  key-shift application through the private
  `canvas/widget/retained_widget_cull_window_key.rs` and
  `canvas/widget/retained_widget_cull_window_shift.rs` seams, so retained prepaint cull
  tracking no longer keeps visibility gating, tile-key math, and shift reporting in one
  surface.
- `delete.rs` now routes delete-op construction, removable-id collection,
  and deletable predicates through the private
  `canvas/widget/delete_ops_builder.rs`,
  `canvas/widget/delete_removed_ids.rs`, and
  `canvas/widget/delete_predicates.rs` seams, so retained deletion helpers
  no longer stay embedded in one mixed surface.
- `clipboard_paste.rs` now routes clipboard parsing/offset derivation,
  paste-transaction construction, and inserted-selection replay through the
  private `canvas/widget/clipboard_paste_parse.rs`,
  `canvas/widget/clipboard_paste_transaction.rs`, and
  `canvas/widget/clipboard_paste_selection.rs` seams, so retained clipboard
  paste helpers no longer stay embedded in one mixed surface.
- `keyboard_shortcuts.rs` now routes overlay escape/key-down handling and
  modifier/navigation shortcut dispatch through the private
  `canvas/widget/keyboard_shortcuts_overlay.rs` and
  `canvas/widget/keyboard_shortcuts_commands.rs` seams, so retained keyboard
  shortcut reducers no longer stay embedded in one mixed surface.
- `pointer_up_node_drag.rs` now routes parent-group resolution and
  release-op/commit orchestration through the private
  `canvas/widget/pointer_up_node_drag_parent.rs` and
  `canvas/widget/pointer_up_node_drag_ops.rs` seams, so retained node-drag
  release reducers no longer stay embedded in one mixed surface.
- `node_drag_constraints.rs` now routes anchor/rect clamping and
  multi-drag extent constraint helpers through the private
  `canvas/widget/node_drag_constraints_anchor.rs` and
  `canvas/widget/node_drag_constraints_extent.rs` seams, so retained node-drag
  geometry helpers no longer stay embedded in one mixed surface.
- `command_edit.rs` now routes cut/delete removal orchestration and
  removed-selection cleanup through the private
  `canvas/widget/command_edit_remove.rs` seam, so retained edit command
  reducers no longer keep destructive edit flows embedded in one surface.
- `paint_overlay_feedback.rs` now routes toast overlay painting and
  wire-drag hint painting through the private
  `canvas/widget/paint_overlay_toast.rs` and
  `canvas/widget/paint_overlay_wire_hint.rs` seams, so retained overlay
  feedback paint helpers no longer stay embedded in one mixed surface.
- `auto_measure.rs` now routes node measurement input collection and
  measured-size computation/store updates through the private
  `canvas/widget/auto_measure_collect.rs` and
  `canvas/widget/auto_measure_apply.rs` seams, so retained auto-measure
  reducers no longer keep collection and apply phases embedded in one surface.
- `paint_grid_tiles.rs` now routes line, dot, and cross tile-op painting
  through the private `canvas/widget/paint_grid_tiles_lines.rs`,
  `canvas/widget/paint_grid_tiles_dots.rs`, and
  `canvas/widget/paint_grid_tiles_cross.rs` seams, so retained grid tile
  painters no longer keep all background patterns embedded in one surface.
- `group_resize.rs` now routes preview-rect computation, child-bounds
  clamping, and resize-handle hit helpers through the private
  `canvas/widget/group_resize_apply.rs` and
  `canvas/widget/group_resize_hit.rs` seams, so retained group-resize reducers
  no longer keep geometry math and hit testing embedded in one surface.
- `marquee_selection.rs` now routes marquee query/edge-derivation
  and selection-state writes through the private
  `canvas/widget/marquee_selection_query.rs` and
  `canvas/widget/marquee_selection_apply.rs` seams, so retained marquee reducers
  no longer keep box-selection geometry and state writes embedded in one surface.
- `paint_grid_plan.rs` now routes canvas chrome hint lookup,
  grid metrics/tile scratch preparation, and pattern-size validation through the private
  `canvas/widget/paint_grid_plan_support.rs` seam, so retained grid-plan reducers
  no longer keep paint planning helpers embedded in one surface.
- `cursor.rs` now routes interaction gating and concrete resize/edge-anchor
  cursor resolution through the private `canvas/widget/cursor_gate.rs` and
  `canvas/widget/cursor_resolve.rs` seams, so retained cursor reducers
  no longer keep cursor eligibility checks and hit resolution embedded in one surface.
- `pointer_up_commit.rs` now routes node/group resize commit op assembly
  and group-drag release ops through the private
  `canvas/widget/pointer_up_commit_resize.rs` and
  `canvas/widget/pointer_up_commit_group_drag.rs` seams, so retained pointer-up reducers
  no longer keep multiple commit builders embedded in one surface.
- `paint_searcher.rs` now routes searcher query chrome and row list painting
  through the private `canvas/widget/paint_searcher_query.rs` and
  `canvas/widget/paint_searcher_rows.rs` seams, so retained searcher paint reducers
  no longer keep all query/list paint phases embedded in one surface.
- `view_math.rs` now routes viewport/pan-zoom conversion helpers and
  rect/hit/resize-handle geometry through the private
  `canvas/widget/view_math_viewport.rs` and `canvas/widget/view_math_rect.rs` seams,
  so retained view-math utilities no longer keep mixed viewport and hit geometry in one surface.
- `rect_math.rs` now routes base rect set-ops and path/edge bounds helpers
  through the private `canvas/widget/rect_math_core.rs` and
  `canvas/widget/rect_math_path.rs` seams, so retained rect math utilities
  no longer keep mixed hit-rect and edge/path bounds helpers in one surface.
- `focus_port_direction_candidate.rs` now routes wire-drag required-direction
  lookup and directional port ranking through the private
  `canvas/widget/focus_port_direction_wire.rs` and
  `canvas/widget/focus_port_direction_rank.rs` seams, so retained focus-navigation reducers
  no longer keep input-context lookup and ranking math embedded in one surface.
- `keyboard_shortcuts_commands.rs` now routes shortcut eligibility gates and
  command lookup tables through the private `canvas/widget/keyboard_shortcuts_gate.rs`
  and `canvas/widget/keyboard_shortcuts_map.rs` seams, so retained keyboard shortcut
  reducers no longer keep mixed gating and command mapping embedded in one surface.
- `command_router.rs` now routes nudge vector lookup and
  align/distribute mode mapping through the private
  `canvas/widget/command_router_nudge.rs` and
  `canvas/widget/command_router_align.rs` seams, so retained command routing no
  longer keeps repeated movement/alignment command tables embedded in one surface.
- `paint_overlay_elements.rs` now routes context-menu chrome,
  marquee/snap-guide primitives, and toast/wire-drag hint feedback through the private
  `canvas/widget/paint_overlay_menu.rs`, `canvas/widget/paint_overlay_guides.rs`, and
  `canvas/widget/paint_overlay_feedback.rs` seams, so retained overlay paint helpers no
  longer stay embedded in one surface file.
- `viewport_timers.rs` now routes animation/debounce, inertia,
  and auto-pan timer orchestration through the private
  `canvas/widget/viewport_timer_animation.rs`,
  `canvas/widget/viewport_timer_inertia.rs`, and
  `canvas/widget/viewport_timer_auto_pan.rs` seams, so retained viewport timer helpers
  no longer stay embedded in one surface file.
- `timer_motion.rs` now routes pan-inertia ticks, viewport
  animation/debounce ticks, and auto-pan motion replay through the private
  `canvas/widget/timer_motion_pan_inertia.rs`,
  `canvas/widget/timer_motion_viewport.rs`, and
  `canvas/widget/timer_motion_auto_pan.rs` seams with shared invalidation in
  `canvas/widget/timer_motion_shared.rs`, so retained timer-driven motion reducers no
  longer stay embedded in one surface file.
- The searcher input/pointer activation trio now routes hit
  testing, drag arming, key-step/query reducers, hover sync, and wheel scroll through
  the private `canvas/widget/searcher_activation_hit.rs`,
  `canvas/widget/searcher_activation_state.rs`,
  `canvas/widget/searcher_input_nav.rs`, `canvas/widget/searcher_input_query.rs`,
  `canvas/widget/searcher_pointer_hover.rs`, and
  `canvas/widget/searcher_pointer_wheel.rs` seams, so retained searcher reducers no
  longer stay embedded in three medium-sized surface files.

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
- `ecosystem/fret-node/src/ui/canvas/widget/insert_execution_feedback.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/insert_execution_plan.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/insert_execution_point.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/split_edge_execution.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/connection_execution.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/background_execution.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/edge_execution.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/key_navigation.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/item_builders.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/opening.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/target_hit.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/clipboard.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/clipboard_anchor.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/clipboard_paste.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/clipboard_transfer.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/cancel.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/cancel_cleanup.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/cancel_gesture_state.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/cancel_viewport_state.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pan_zoom.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pan_zoom_begin.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pan_zoom_move.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pan_zoom_zoom.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/marquee.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/marquee_begin.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/marquee_finish.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/marquee_pending.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/marquee_selection.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/callbacks.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/callbacks_connect.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/callbacks_graph.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/callbacks_view.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/command_router.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/command_router_align.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/command_router_nudge.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/command_focus.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/command_focus_cycle.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/command_focus_port.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/retained_widget_frame.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/retained_widget_semantics.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/retained_widget_semantics_focus.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/retained_widget_semantics_value.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/retained_widget_layout.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/retained_widget_layout_children.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/retained_widget_layout_drain.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/retained_widget_layout_observe.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/retained_widget_layout_publish.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/retained_widget_cull_window.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/retained_widget_cull_window_key.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/retained_widget_cull_window_shift.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/delete.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/delete_ops_builder.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/delete_removed_ids.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/delete_predicates.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/clipboard_paste.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/clipboard_paste_parse.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/clipboard_paste_transaction.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/clipboard_paste_selection.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/keyboard_shortcuts.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/keyboard_shortcuts_overlay.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/keyboard_shortcuts_commands.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/keyboard_shortcuts_gate.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/keyboard_shortcuts_map.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_node_drag.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_node_drag_parent.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_node_drag_ops.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/node_drag_constraints.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/node_drag_constraints_anchor.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/node_drag_constraints_extent.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/command_edit.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/command_edit_remove.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_overlay_feedback.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_overlay_toast.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_overlay_wire_hint.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/auto_measure.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/auto_measure_collect.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/auto_measure_apply.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_grid_tiles.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_grid_tiles_lines.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_grid_tiles_dots.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_grid_tiles_cross.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/command_open.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/command_open_conversion.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/command_open_edge.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/command_open_group.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/command_open_insert.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/command_ui.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/searcher_ui.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/searcher_activation.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/searcher_activation_hit.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/searcher_activation_state.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/searcher_input.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/searcher_input_nav.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/searcher_input_query.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/searcher_logic.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/searcher_picker.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/searcher_pointer.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/searcher_row_activation.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/searcher_rows.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/searcher_pointer_hover.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/searcher_pointer_wheel.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/searcher.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/ui.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/right_click.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/left_click/group_background.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/left_click/connection_hits.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/left_click/element_hits.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_commit.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_pending.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_left_route.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_state.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_down_double_click.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_down_double_click_background.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_down_double_click_edge.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/focus.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/focus_draw_order.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/focus_edge_repair.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/focus_nav_traversal.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/focus_nav_traversal_edge.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/focus_nav_traversal_node.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/focus_nav_traversal_port.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_down_double_click.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_down_gesture_start.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/event_pointer_down_route.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/event_pointer_down_state.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/focus_nav_ports.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/focus_nav_ports_activation.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/focus_nav_ports_center.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/focus_nav_ports_hints.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/focus_port_direction.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/focus_port_direction_apply.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/focus_port_direction_candidate.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/sticky_wire.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/sticky_wire_connect.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/sticky_wire_targets.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/event_pointer_move.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/event_pointer_move_state.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/event_pointer_move_tail.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_move_pointer_state.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_move_release.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_move_release_left.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_move_release_pan.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_move_dispatch.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/event_pointer_wheel.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/event_pointer_wheel_state.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/event_pointer_wheel_route.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_wheel_motion.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_wheel_pan.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_wheel_viewport.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_wheel_zoom.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/event_clipboard.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/event_clipboard_pending.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/event_clipboard_feedback.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/event_router.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/event_router_pointer.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/event_router_pointer_button.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/event_router_pointer_wheel.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/event_router_system.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/event_router_system_input.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/event_router_system_lifecycle.rs`
- `ecosystem/fret-node/src/ui/declarative/paint_only/cache.rs`
- `ecosystem/fret-node/src/ui/declarative/paint_only/diag.rs`
- `ecosystem/fret-node/src/ui/declarative/paint_only/selection.rs`
- `ecosystem/fret-node/src/ui/declarative/paint_only/surface_models.rs`
- `ecosystem/fret-node/src/ui/declarative/paint_only/pointer_down.rs`
- `ecosystem/fret-node/src/ui/declarative/paint_only/transactions.rs`
- `ecosystem/fret-node/src/ui/declarative/paint_only/portals.rs`
- `ecosystem/fret-node/src/ui/declarative/paint_only/overlays.rs`
- `ecosystem/fret-node/src/ui/declarative/paint_only/semantics.rs`
- `ecosystem/fret-node/src/ui/declarative/paint_only/input_handlers.rs`
- `ecosystem/fret-node/src/ui/declarative/paint_only/cache.rs`
- `ecosystem/fret-node/src/ui/declarative/paint_only/surface_frame.rs`
- `ecosystem/fret-node/src/ui/declarative/paint_only/surface_content.rs`
- `ecosystem/fret-node/src/ui/declarative/paint_only/surface_shell.rs`
- `ecosystem/fret-node/src/ui/declarative/paint_only/surface_math.rs`
- `ecosystem/fret-node/src/ui/declarative/paint_only/surface_support.rs`
- `ecosystem/fret-node/src/ui/canvas/widget.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/widget_surface.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/interaction_gate.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/focus_session.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/cancel_session.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/press_session.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_session.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pending_resize_session.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pending_drag_session.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pending_connection_session.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/insert_node_drag/session.rs`
- `ecosystem/fret-node/src/ui/canvas/paint.rs`
- `ecosystem/fret-node/src/ui/canvas/paint/paint_wire.rs`
- `ecosystem/fret-node/src/ui/canvas/paint/paint_ports.rs`
- `ecosystem/fret-node/src/ui/canvas/paint/paint_markers.rs`
- `ecosystem/fret-node/src/ui/canvas/paint/paint_text.rs`
- `ecosystem/fret-node/src/ui/canvas/spatial.rs`
- `ecosystem/fret-node/src/ui/canvas/spatial/spatial_index.rs`
- `ecosystem/fret-node/src/ui/canvas/spatial/spatial_adjacency.rs`
- `ecosystem/fret-node/src/ui/canvas/spatial/spatial_derived.rs`
- `ecosystem/fret-node/src/ui/canvas/state.rs`
- `ecosystem/fret-node/src/ui/canvas/state/state_paste_series.rs`
- `ecosystem/fret-node/src/ui/canvas/state/state_viewport_animation.rs`
- `ecosystem/fret-node/src/ui/canvas/state/state_geometry_cache.rs`
- `ecosystem/fret-node/src/ui/canvas/state/state_overlay_sessions.rs`
- `ecosystem/fret-node/src/ui/canvas/state/state_preview_cache.rs`
- `ecosystem/fret-node/src/ui/canvas/state/state_drag_sessions.rs`
- `ecosystem/fret-node/src/ui/canvas/workflow.rs`
- `ecosystem/fret-node/src/ui/canvas/workflow/workflow_insert.rs`
- `ecosystem/fret-node/src/ui/canvas/searcher.rs`
- `ecosystem/fret-node/src/ui/canvas/searcher/searcher_score.rs`
- `ecosystem/fret-node/src/ui/canvas/searcher/searcher_build.rs`
- `ecosystem/fret-node/src/ui/canvas/middleware.rs`
- `ecosystem/fret-node/src/ui/canvas/middleware/middleware_chain.rs`
- `ecosystem/fret-node/src/ui/canvas/middleware/middleware_validation.rs`
- `ecosystem/fret-node/src/ui/canvas/route_math.rs`
- `ecosystem/fret-node/src/ui/canvas/route_math/route_math_curve.rs`
- `ecosystem/fret-node/src/ui/canvas/route_math/route_math_tangent.rs`
- `ecosystem/fret-node/src/ui/canvas/conversion.rs`
- `ecosystem/fret-node/src/ui/canvas/conversion/conversion_candidates.rs`
- `ecosystem/fret-node/src/ui/canvas/conversion/conversion_plan.rs`
- `ecosystem/fret-node/src/ui/canvas/snaplines.rs`
- `ecosystem/fret-node/src/ui/canvas/snaplines/snaplines_align.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/event_timer.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/event_timer_route.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/event_timer_toast.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/timer_motion.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/viewport_timers.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/timer_motion_pan_inertia.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/timer_motion_viewport.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/timer_motion_auto_pan.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/timer_motion_shared.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/viewport_timer_animation.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/viewport_timer_inertia.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/viewport_timer_auto_pan.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/event_keyboard.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/event_keyboard_state.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/event_keyboard_route.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/keyboard_shortcuts.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/keyboard_pan_activation.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/retained_widget.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/retained_widget_frame.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/retained_widget_runtime.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/retained_widget_runtime_command.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/retained_widget_runtime_event.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/retained_widget_runtime_paint.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/retained_widget_runtime_shared.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/retained_widget_command_availability.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/retained_widget_command_availability_gate.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/retained_widget_command_availability_query.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/node_drag.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/node_drag_snap.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/node_drag_preview.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/node_drag_constraints.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_grid.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_grid_cache.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_grid_plan.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_grid_stats.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_grid_tiles.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_overlay_elements.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_overlay_menu.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_overlay_guides.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_overlay_feedback.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_commit.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_node_drag.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_finish.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_pending.rs`
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
