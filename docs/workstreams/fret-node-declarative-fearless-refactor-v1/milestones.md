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





