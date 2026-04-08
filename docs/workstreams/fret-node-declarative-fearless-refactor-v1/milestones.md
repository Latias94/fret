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
- `ecosystem/fret-node/src/ui/binding_queries.rs`
- `ecosystem/fret-node/src/ui/binding_store_sync.rs`
- `ecosystem/fret-node/src/ui/binding_viewport.rs`
- `apps/fret-examples/src/node_graph_demo.rs`
- `apps/fret-examples/src/node_graph_legacy_demo.rs`
- `apps/fret-examples/src/imui_node_graph_demo.rs`
- `docs/workstreams/xyflow-gap-analysis.md`
- `docs/workstreams/crate-audits/fret-node.l0.md`

## M2 - State boundary split

Status target: architectural refactor landed; follow-on cleanup only

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
- Landed slice: `NodeGraphViewState` is pure view state, while the file wrapper persists
  `NodeGraphInteractionConfig` + `NodeGraphRuntimeTuning`, and widget/runtime snapshots still
  resolve a combined `NodeGraphInteractionState` from explicit editor-config seams.
- The retained canvas sync/runtime path no longer reconstructs editor config from
  `NodeGraphViewState` under `cfg(test)`; retained runtime, retained tests, and `--all-features`
  builds all use the same explicit editor-config seam.
- Persistence ownership is now explicit: the file wrapper writes pure view-state under `state`, with
  `interaction` / `runtime_tuning` stored as wrapper-owned fields in `state_version = 2`.
- App/example authoring also follows the split:
  - retained canvas can mirror `NodeGraphEditorConfig`,
  - tuning/controls overlays read explicit editor-config seams, with
    `NodeGraphControlsOverlay::new(...)` now taking the editor-config model directly,
  - example persistence restores and saves `NodeGraphViewStateFileV1` through
    `new(...)`.
  - advanced binding seams now also require explicit editor-config mirrors, so controlled sync no
    longer falls back to default config when graph/view/controller mirrors are caller-owned.
  - retained compatibility constructors now also require explicit editor-config models:
    `NodeGraphCanvas::new(...)`, `NodeGraphCanvas::new_with_middleware(...)`, and
    `NodeGraphSurfaceCompatRetainedProps::new(...)` no longer manufacture default config
    ownership internally.
  - retained widget test harnesses now also provide combined `graph + view + editor_config`
    setup helpers, keeping explicit editor-config ownership visible in conformance tests without
    leaving repeated host/model bootstrap blocks as accidental public teaching surface.
  - overlay-oriented retained harnesses now also provide combined `view + editor_config` setup
    helpers, so focused controls/minimap conformance gates can stay on the same explicit
    editor-config seam without duplicating local bootstrap code.
  - edge label/marker/cache, edge-insert, paint-overrides, skin, invalidation,
    selection/preview/semantic-zoom, measured/spatial, and a11y/fit-view/connection-validity
    retained conformance gates now also consume the shared `graph + view + editor_config` harness
    setup, keeping renderer-focused, edge-insert interaction, paint-only, skin-chrome,
    invalidation, selection/preview, measured/spatial, and a11y/fit-view tests aligned with the
    same explicit editor-config ownership contract.
  - background-style/color-mode, custom-edge-path, derived-geometry invalidation,
    edit-command-availability, escape-cancel, and insert-node-drag retained conformance gates now
    also consume the shared `graph + view + editor_config` harness setup, keeping canvas
    chrome/theme sync, custom-path hit-testing, derived-geometry cache invalidation,
    command-availability gating, pointer-capture cancel, and insert-node threshold coverage aligned
    with the same explicit editor-config ownership contract.
  - drag-preview, node-resize-preview, overlay invalidation, and overlay menu/searcher retained
    conformance gates now also consume the shared `graph + view + editor_config` harness setup,
    keeping preview cache reuse, preview geometry/index drift, overlay-only invalidation, and
    overlay clamp coverage aligned with the same explicit editor-config ownership contract.
  - callback-oriented retained conformance gates now also consume the shared
    `graph + view + editor_config` harness setup, keeping connect/reconnect, pan, and node-drag
    callback coverage aligned with the same explicit editor-config ownership contract.
  - hit-testing and internals retained conformance gates now also consume the shared
    `graph + view + editor_config` harness setup, keeping target-port picking, edge/anchor hit
    resolution, internals snapshot publication, and internals/measured-output stability coverage
    aligned with the same explicit editor-config ownership contract.
  - insert-node-drag-drop, middleware, op-batching determinism, portal measured-internals,
    set-viewport queue, and perf-cache-prune retained conformance gates now also consume the
    shared `graph + view + editor_config` harness setup, keeping drag-drop, middleware rejection,
    group-op batching, portal-measurement-to-internals, view-queue viewport, and cache-prune
    coverage aligned with the same explicit editor-config ownership contract.
  - perf-cache retained coverage now also consumes the shared `graph + view + editor_config`
    harness setup, keeping static node/edge cache reuse, tile-boundary reuse, incremental
    edge-label/marker warmup, and repeated-label auto-measure coverage aligned with the same
    explicit editor-config ownership contract.
  - interaction-conformance and the remaining root retained widget tests now also consume the
    shared `graph + view + editor_config` harness setup, and the now-unused implicit
    `make_host_graph_view(...)` helper plus the test-only 3-arg `new_canvas!(...)` arm are
    deleted from the retained test harness, so no retained conformance gate still teaches implicit
    default editor-config ownership.

### Done criteria

- Reviewers can point to one place for persisted view state, one place for interaction policy, and
  one place for runtime tuning.
- The resulting shapes make it harder to persist accidental performance knobs as if they were view
  semantics.
- Controlled sync and diagnostics still have a stable data contract.
- The previous test-only compatibility bridge is removed; examples, runtime paths, and tests now all
  use explicit editor-config seams.
- Retained public compatibility widgets no longer hide default editor-config ownership behind their
  constructor boundary.
- Retained runtime/test sync no longer carries a stale `cfg(test)` editor-config reconstruction
  fallback; all feature sets now use the same explicit editor-config ownership contract.

### Required regression protection

- focused `cargo nextest run -p fret-node` coverage for view-state migration and store behavior
- at least one diag or integration gate proving the split does not regress viewport/selection flows

### Evidence anchors

- `ecosystem/fret-node/src/io/mod.rs`
- `ecosystem/fret-node/src/runtime/store.rs`
- `ecosystem/fret-node/src/runtime/tests.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/view_state/sync.rs`
- `ecosystem/fret-node/src/ui/controller_store_sync.rs`
- `apps/fret-examples/src/node_graph_legacy_demo.rs`
- `apps/fret-examples/src/node_graph_domain_demo.rs`
- `apps/fret-examples/src/node_graph_tuning_overlay.rs`
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
    facade over store.
  - The controller now also exposes XyFlow-style connection queries via
    `node_connections` / `port_connections`, so app code can query node/handle adjacency
    without reaching into store lookups directly.
  - The controller now also covers the first bounds-aware viewport helpers:
    `set_center_in_bounds*` and `fit_view_nodes_in_bounds*`.
  - The controller and binding now also expose `update_node*` / `update_edge*` ergonomic helpers,
    but those helpers intentionally accept `NodeGraphNodeUpdate` / `NodeGraphEdgeUpdate` drafts
    instead of raw `Node` / `Edge`, keeping structural port edits and endpoint rewires on explicit
    transactions.
  - Retained glue now starts consuming controller-owned viewport transport instead of teaching raw
    queue mutation first: `NodeGraphCanvas::with_controller`, `NodeGraphMiniMapOverlay::with_controller`,
    and the gallery workflow snippet controls now route common viewport actions through the binding-first facade.
  - Those helpers now write through the store-backed controller surface directly; queued viewport
    transport remains a crate-internal retained compatibility detail instead of controller state.
  - Transaction submission helpers (`submit_transaction*`, `submit_transaction_and_sync_*`) are now
    pure store-backed controller operations; raw edit transport remains crate-internal retained
    compatibility plumbing instead of a controller concern.
  - Retained edit glue now also converges on the controller-first path:
    `NodeGraphCanvas::with_controller` now binds store-backed controller state only,
    `NodeGraphPortalHost::with_controller` and `NodeGraphOverlayHost::new(...).with_controller(...)` prefer
    controller-owned transaction submission, `NodeGraphBlackboardOverlay::new(...).with_controller(...)`
    now gives retained symbol actions the same controller-first path, and `compat_retained` now
    takes a controller binding directly instead of exposing public queue transport props.
  - The temporary `NodeGraphViewportHelper` façade is now deleted; controller-first app-facing
    composition calls `NodeGraphController::{set_viewport*, set_center_in_bounds*,`
    `fit_view_nodes_in_bounds*}` directly, and raw queue ownership stays a crate-internal
    compatibility detail rather than a public app-facing choice.
  - Public `NodeGraphFitViewOptions` / `NodeGraphSetViewportOptions` now live in
    `ui/viewport_options.rs` and only expose store-first clamp/padding fields, while
    `ui/canvas/widget/view_queue.rs` keeps queue-era animation overrides as retained-canvas-local
    crate-internal compatibility transport-only types.
  - Raw edit/view transport is now crate-internal; root `fret_node::ui::*` re-exports viewport
    option types but not the underlying queue/request machinery, and the temporary public
    `fret_node::ui::advanced::*` edit seam is deleted.
  - The retained-backed domain demo and legacy demo no longer own raw edit queues, while the
    workflow gallery snippet no longer owns a raw `NodeGraphViewQueue` at all and instead uses
    `NodeGraphSurfaceBinding::{set_viewport_action_host, fit_view_nodes_in_bounds_action_host}`.
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
    mutation, reset, or queue ownership for those core surfaces.
  - Feature-gated retained coverage now also includes a blackboard controller-first gate proving
    symbol creation prefers controller/store commit over raw queue transport.
  - `NodeGraphController` now also owns undo/redo sync helpers for the default store-backed
    app-facing surface, and focused declarative coverage now proves node-drag commit history can be
    undone/redone while graph/view mirrors stay in sync.
  - `NodeGraphSurfaceBinding` now acts as the instance-style app-facing facade for common queries,
    viewport actions, graph adjacency queries, and controlled-sync helpers (`replace_graph`,
    `replace_view_state`, `set_selection`, `outgoers`, `incomers`, `connected_edges`,
    `port_connections`, `node_connections`, `undo`, `redo`), while lower-level controller ownership
    now stays explicit via `NodeGraphController::new(binding.store_model())`.
  - `NodeGraphSurfaceBinding` is now split by responsibility across `binding.rs`,
    `binding_queries.rs`, `binding_store_sync.rs`, and `binding_viewport.rs`, while source-policy
    tests aggregate that companion surface so the public contract is no longer coupled to one
    monolithic file.
  - The advanced mirror-owned binding constructor is now spelled
    `NodeGraphSurfaceBinding::from_models_and_controller(...)`, and it now requires explicit
    `graph + view_state + editor_config + controller` ownership so advanced mirror wiring does not
    masquerade as the default constructor family or silently synthesize config defaults.
  - Internally, `NodeGraphSurfaceBinding` now stores the authoritative `NodeGraphStore` handle rather
    than privately holding a controller instance, so the implementation matches the public
    ownership story: controller construction is explicit, while the binding stays store-backed.
  - `NodeGraphSurfaceBinding` now also mirrors the full common store-first viewport helper family
    (`set_viewport*`, `set_center_in_bounds*`, `fit_view_nodes_in_bounds*`, including option-bearing
    and action-host variants), so routine app-facing viewport hooks can stay on the instance-style
    binding surface instead of dropping to explicit controller wiring.
  - Focused controller/binding gates now also lock viewport read/projection plus
    `set_viewport*`, `set_center_in_bounds*`, `fit_view_nodes_in_bounds*`, and
    `fit_canvas_rect_in_bounds*`, so the current controller-facing XyFlow viewport mapping is
    reviewable as a closed slice rather than an open-ended helper-breadth backlog item.
  - `NodeGraphSurfaceBinding` now also mirrors routine bound-store edit/sync/history helpers
    (`dispatch_transaction*`, `submit_transaction*`, `replace_*_action_host`,
    `set_selection_action_host`, `undo_action_host`, `redo_action_host`), so object-safe app hooks
    can keep model synchronization on the binding surface instead of reaching for controller-only
    plumbing.
  - Declarative `paint_only` routine action/UiHost hooks now also take the binding-first path:
    transaction commit, selection commit, diagnostics presets, keyboard zoom, pointer release/move
    helpers, and fit-to-portals viewport updates no longer thread `graph + view_state +
    controller` triplets for ordinary bound-surface work.
  - Declarative `paint_only` runtime source ownership is now locked by focused source-policy
    coverage: the main surface file plus private runtime submodules must use `binding.store_model()`
    as the authoritative graph/view/editor-config source instead of reading bound mirrors directly.
  - Declarative graph-edit commit authority is now also locked by focused source-policy coverage:
    runtime files must not replace graph/document directly or dispatch/submit transactions outside
    the private `paint_only/transactions.rs` seam.
  - The workflow gallery retained subtree now also keeps its retained controller as explicit local
    state and constructs it from `binding.store_model()` rather than teaching hidden controller
    extraction from the binding, so first-party code makes that advanced seam visible instead of
    hiding it inside the binding helper.
  - Source-policy tests now also lock the retained advanced seam posture itself: retained
    canvas/portal/rename/blackboard/minimap widgets keep `with_controller(...)` as the public
    binding story, while raw queue transport stays crate-internal compatibility plumbing.
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
- source-policy coverage proving declarative `paint_only` runtime files do not read/write bound
  graph/view/editor-config mirrors when a store-backed binding exists
- source-policy coverage proving declarative graph-edit transaction dispatch stays inside
  `paint_only/transactions.rs` and runtime files do not replace graph/document directly

### Evidence anchors

- `ecosystem/fret-node/src/runtime/store.rs`
- `ecosystem/fret-node/src/runtime/changes.rs`
- `ecosystem/fret-node/src/runtime/lookups.rs`
- `ecosystem/fret-node/src/ui/controller.rs`
- `ecosystem/fret-node/src/ui/controller_queries.rs`
- `ecosystem/fret-node/src/ui/controller_viewport.rs`
- `ecosystem/fret-node/src/ui/controller_store_sync.rs`
- `ecosystem/fret-node/src/ui/binding.rs`
- `ecosystem/fret-node/src/ui/binding_queries.rs`
- `ecosystem/fret-node/src/ui/binding_store_sync.rs`
- `ecosystem/fret-node/src/ui/binding_viewport.rs`
- `ecosystem/fret-node/src/ui/declarative/paint_only.rs`
- `ecosystem/fret-node/src/ui/declarative/paint_only/tests.rs`
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
  and `compat_retained` now relies on a controller binding plus explicit editor-config model
  instead of public queue transport props or retained-only default policy payloads.
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
- Declarative visible-subset portal hosting is now also declared on the public surface:
  `NodeGraphSurfaceProps` carries `NodeGraphVisibleSubsetPortalConfig` instead of loose portal
  booleans, so editor-facing code names visible-subset hosting enablement/cap policy explicitly.
- Declarative diagnostics hover-tooltip overlay orchestration now also lives under the private
  `paint_only/overlays.rs` seam, so the main paint-only surface stops re-embedding hover-anchor
  reads, portal-bounds fallback, and tooltip element wiring inline while keeping the same
  portal-vs-hover anchor precedence contract.
- Declarative hover-tooltip overlay lookup no longer reaches into the portal-hosting module for
  node label/port summaries; that shared graph-summary helper now lives under the neutral
  `paint_only/surface_support.rs` seam instead.
- Declarative diagnostics policy is now also explicit on the public surface:
  `NodeGraphSurfaceProps` carries `NodeGraphDiagnosticsConfig`, while the example app decides
  whether `FRET_DIAG` enables diagnostics instead of the mechanism layer reading process env
  directly.
- Root `fret_node::ui::*` now also re-exports `NodeGraphDiagnosticsConfig` and
  `NodeGraphVisibleSubsetPortalConfig`, so app-facing declarative surfaces can configure
  `NodeGraphSurfaceProps` through one import surface instead of mixing root and nested
  declarative modules.
- Declarative marquee overlay append and final overlay-layer wrapping now also live under the
  private `paint_only/overlays.rs` seam, so the main paint-only surface stops re-embedding
  overlay child flush/wrap plumbing inline and keeps the overlay stack reviewable from one seam.
- Retained toolbar target-selection and visibility rules now also live under the private
  `ui/overlays/toolbar_policy.rs` seam, so node and edge toolbar widgets stop duplicating the
  same explicit-target vs selected-target fallback policy while keeping the same pointer/focus
  behavior.
- The public toolbar policy types now also live with that seam:
  `NodeGraphToolbarVisibility` / `NodeGraphToolbarPosition` / `NodeGraphToolbarAlign` /
  `NodeGraphToolbarSize` are no longer declared inside `toolbars.rs`, keeping toolbar widget
  implementation ownership separate from public policy type ownership.
- Retained rename overlays now also live on one active-session policy seam,
  `ui/overlays/rename_policy.rs`, so group/symbol rename state selection, seed-text loading,
  focus-loss cancel policy, and commit-transaction planning no longer stay duplicated inside
  `NodeGraphOverlayHost`, and hidden stale rename sessions can no longer be committed implicitly.
- Retained controls overlays now also live on one button-policy seam,
  `ui/overlays/controls_policy.rs`, so roster order, default command mapping,
  override-resolution, a11y labels, and display labels no longer stay duplicated inside
  `controls.rs` while keyboard navigation and activation keep the same conformance behavior.
- The public controls binding types now also live with that policy seam:
  `NodeGraphControlsCommandBinding` / `NodeGraphControlsBindings` are no longer declared inside
  `controls.rs`, keeping widget implementation ownership separate from public policy type
  ownership.
- Retained blackboard overlays now also live on one action-policy seam,
  `ui/overlays/blackboard_policy.rs`, so roster order, keyboard navigation policy, action labels,
  default symbol naming, transaction planning, and symbol-rename opening no longer stay mixed into
  `blackboard.rs` while blackboard overlay conformance keeps the same rename/transaction behavior.
- Retained minimap overlays now also live on one keyboard-policy seam,
  `ui/overlays/minimap_policy.rs`, so keyboard action mapping, pan/zoom step policy, zoom clamp,
  and center-based zoom planning no longer stay mixed into `minimap.rs` while minimap keyboard
  conformance keeps the same viewport-update and focus-return behavior.
- Retained minimap overlays now also live on one navigation-ownership seam,
  `ui/overlays/minimap_navigation_policy.rs`, so controller/store/default viewport-update
  resolution plus zoom normalization no longer stay mixed into `minimap.rs` while minimap drag and
  controller-binding conformance keep the same behavior.
- `NodeGraphMiniMapNavigationBinding` / `NodeGraphMiniMapBindings` now also live with that seam
  instead of being declared inside `minimap.rs`, keeping public minimap policy type ownership
  separate from widget implementation.
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
- cached-edge single-rect edge replay/build plus tiled edge-cache and tiled label-cache
  orchestration now also route through the private
  `canvas/widget/paint_root/cached_edges/edges.rs` and
  `canvas/widget/paint_root/cached_edges/labels.rs` seams, so
  `paint_root/cached_edges/single_rect.rs` and `paint_root/cached_edges/tile_path.rs` mainly keep
  cache-mode selection, uncached fallbacks, and overlay-order orchestration at the root.
- cached-edge tile geometry plus cached render-data/build-state initialization now also route
  through the private `canvas/widget/paint_root/cached_edges/geometry.rs` and
  `canvas/widget/paint_root/cached_edges/build_state.rs` helpers, so the edge/label cache seams
  stop re-embedding tile-rect math, cull inflation, and render-data collection boilerplate.
- root edge-anchor target selection now also routes through the private
  `canvas/widget/paint_root/edge_anchor.rs` seam, so `paint_root/immediate.rs` and
  `paint_root/cached_edges/mod.rs` stop re-embedding the same reconnectability gate and anchor
  target resolution logic while keeping cached-vs-immediate data sourcing explicit.
- static scene cache tile/window planning now also routes through the private
  `canvas/widget/static_scene_cache_plan.rs` seam, so `paint_root/cached.rs` and
  `retained_widget_cull_window_key.rs` stop re-embedding the same power-of-two tile sizing and
  centered single-tile window math inline.
- root frame/bootstrap orchestration now also routes through the private
  `canvas/widget/paint_root/frame.rs` seam, so `paint_root/cached.rs` stops re-embedding cache
  begin-frame bookkeeping, path-cache diagnostics publication, viewport/cull setup, canvas
  background fill, and grid paint bootstrap inline.
- root cache-plan orchestration now also routes through the private
  `canvas/widget/paint_root/cache_plan.rs` seam, so `paint_root/cached.rs` stops re-embedding
  hovered-edge resolution, derived geometry publication, static cache eligibility, tile sizing,
  cache-rect selection, and style/base-key planning inline.
- cached-path render tail orchestration now also routes through the private
  `canvas/widget/paint_root/cached_pass.rs` seam, so `paint_root/cached.rs` stops re-embedding the
  groups/edges/nodes cached pass ordering, anchor tail, overlay tail, prune tail, and clip pop
  inline.
- immediate-path render pass plus shared paint-root finish tail now also route through the private
  `canvas/widget/paint_root/immediate_pass.rs` and `canvas/widget/paint_root/tail.rs` seams, so
  `paint_root/immediate.rs` and `paint_root/cached_pass.rs` stop re-embedding the immediate draw
  ordering plus the shared anchors/overlays/prune/pop-clip tail inline.
- cached-edge build-state initialization and budget-step tails now also route through smaller
  private helpers in `canvas/widget/paint_root/cached_edges/build_state.rs`, so the edge-vs-label
  cached build path keeps only the budget function choice and state-specific fields at the root
  instead of re-embedding the same clip-op setup and next-edge replay tail inline.
- cached-edge root-shell uncached fallback and tile preparation now also route through smaller
  helpers in `canvas/widget/paint_root/cached_edges/edges.rs` and
  `canvas/widget/paint_root/cached_edges/geometry.rs`, so `single_rect.rs` and `tile_path.rs`
  mainly keep cache-mode choice, overlay ordering, and label-pass orchestration at the root.
- cached-edge label replay and finished-store tails now also route through smaller helpers in
  `canvas/widget/paint_root/cached_edges/labels.rs`, so the label cache paths stop re-embedding
  the same translated replay and empty-vs-populated finished-store bookkeeping.
- cached-edge replay and finished-store tails now also route through smaller helpers in
  `canvas/widget/paint_root/cached_edges/edges.rs`, so the edge cache paths stop re-embedding the
  same translated replay and finished-store bookkeeping when single-rect and tiled passes share the
  same partially built state.
- cached static group/node replay and store tails now also route through the private
  `canvas/widget/paint_root/static_cache.rs` seam, so `cached_groups.rs` and `cached_nodes.rs`
  stop re-embedding the same cache replay/store-and-replay bookkeeping while keeping the
  render-data collection and static paint bodies explicit at the root.
- cached static group/node layer-key planning now also routes through the same private
  `canvas/widget/paint_root/static_cache.rs` seam, so `cached_groups.rs` and `cached_nodes.rs`
  stop re-embedding the same base-key/style-key/tile-origin cache key assembly inline.
- paint-root cache prune tails now also route through smaller private helpers in
  `canvas/widget/paint_root/prune.rs`, so the root prune entry keeps static tile-cache cleanup and
  dynamic paint-cache cleanup as explicit, separately reviewable responsibilities.
- selected-node overlay and dynamic-node paint tails now also route through the private
  `canvas/widget/paint_root/node_layers.rs` seam, so `cached_nodes.rs` and
  `immediate_pass.rs` stop re-embedding the same selected-node replay and dynamic-node overlay tail
  while keeping static node paint ordering explicit at the root.
- selected-group overlay rect replay now also routes through shared helpers in
  `ui/canvas/widget/paint_groups.rs`, so `paint_root/cached_groups.rs` stops re-embedding the same
  selected-group rect collection and quad replay tail inline while keeping static group cache
  orchestration explicit at the root.
- widget-surface color-mode / skin / paint-override sync now also routes through the private
  `ui/canvas/widget/widget_surface/sync.rs` seam, so `widget_surface.rs` stops re-embedding the
  same geometry-reset and scene-cache/build-state invalidation tails inline while keeping
  construction and builder-style surface composition explicit at the root.
- widget-surface fit-view-on-mount builder/runtime now also routes through the private
  `ui/canvas/widget/widget_surface/fit_view.rs` seam, so `widget_surface.rs` stops re-embedding
  the same fit-on-mount option setup, node-id collection, and one-shot framing tail inline while
  keeping the public builder surface unchanged.
- widget-surface style/transport builders now also route through the private
  `ui/canvas/widget/widget_surface/builders.rs` seam, so `widget_surface.rs` stops re-embedding
  the same style-reset, geometry-reset, and transport-key reset tails inline while keeping the
  app-facing builder API unchanged.
- widget-surface construction and middleware transplant now also route through the private
  `ui/canvas/widget/widget_surface/construct.rs` seam, so `widget_surface.rs` stops re-embedding
  the same default state allocation and cross-middleware field transplant block inline while
  keeping the public constructor/composition API unchanged.
- widget-surface runtime helpers now also route through the private
  `ui/canvas/widget/widget_surface/runtime.rs` seam, so `widget_surface.rs` stops re-embedding the
  same render-cull, debug-metrics, interaction-state, and edge-path helper bodies inline.
- widget-surface output/diagnostics builders now also route through the same private
  `ui/canvas/widget/widget_surface/builders.rs` seam, so `widget_surface.rs` no longer keeps the
  measured-output, internals, and diagnostics-anchor builder tails inline.
- paint-render-data node visibility and payload assembly now also route through the private
  `ui/canvas/widget/paint_render_data/nodes.rs` seam, so `collect.rs` and `selected_nodes.rs`
  stop re-embedding the same node chrome/ports payload build tail and visible-node ordering logic
  inline.
- paint-render-data group collection now also routes through the private
  `ui/canvas/widget/paint_render_data/groups.rs` seam, so `collect.rs` stops re-embedding the same
  group ordering, preview-rect projection, cull filtering, and metrics bookkeeping inline.
- paint-render-data edge candidate selection, hint resolution, cull filtering, and render payload
  assembly now also route through the private `ui/canvas/widget/paint_render_data/edges.rs` seam,
  so `collect.rs` stops re-embedding the same edge iteration, override application, bounds
  rejection, rank calculation, and stable sort tail inline.
- full node-paint insert-preview, node chrome/body, and port/pin tails now also route through the
  private `ui/canvas/widget/paint_nodes/full_preview.rs`,
  `ui/canvas/widget/paint_nodes/full_nodes.rs`, and
  `ui/canvas/widget/paint_nodes/full_ports.rs` seams, so `paint_nodes/full.rs` now mainly keeps
  shared paint setup, skin hint collection, and top-level draw ordering explicit.
- dynamic selected-node chrome/ring logic and port-adorners now also route through the private
  `ui/canvas/widget/paint_nodes/dynamic_nodes.rs` and
  `ui/canvas/widget/paint_nodes/dynamic_ports.rs` seams, while
  `paint_nodes/dynamic_from_geometry.rs` reuses the shared insert-preview helper and now mainly
  keeps transient paint setup plus top-level orchestration explicit.
- static node chrome/text and static port-label/shape paint now also route through the private
  `ui/canvas/widget/paint_nodes/static_node_chrome.rs` and
  `ui/canvas/widget/paint_nodes/static_ports.rs` seams, so `paint_nodes/static_nodes.rs` now
  mainly keeps shared paint setup plus top-level node/port pass ordering explicit.
- context-menu connection insert/conversion execution now also routes through the private
  `ui/canvas/widget/context_menu/connection_execution_insert.rs` and
  `ui/canvas/widget/context_menu/connection_execution_conversion.rs` seams, so
  `context_menu/connection_execution.rs` now mainly keeps the plan enums and focused tests.
- edge marker-path planning and wire/highlight replay helpers now also route through the private
  `ui/canvas/widget/paint_edges/markers_support.rs` seam, so `paint_edges/markers.rs` now mainly
  keeps the regular-vs-custom marker orchestration explicit.
- align/distribute planning now also routes element collection, per-mode delta planning,
  extent-shift computation, and group/node op application through the private
  `ui/canvas/widget/move_ops/align_distribute/support.rs` seam, so
  `move_ops/align_distribute/plan.rs` now mainly keeps the top-level planning orchestration
  explicit.
- nudge move planning now also routes moved-set collection, shared extent clamps, and group/node
  op application through the private `ui/canvas/widget/move_ops/nudge_support.rs` seam, so
  `move_ops/nudge.rs` now mainly keeps delta normalization, snap-to-grid primary selection
  handling, and top-level orchestration explicit.
- node-resize math now also routes rect utilities and resize-handle geometry/clamp flow through
  the private `ui/canvas/widget/node_resize/math/rects.rs` and
  `ui/canvas/widget/node_resize/math/resize.rs` seams, so `node_resize/math.rs` now mainly keeps
  the root re-exports and focused resize conformance tests explicit.
- press-session preparation now also routes session clearing helpers and hit-specific preparation
  profiles through the private `ui/canvas/widget/press_session/clear.rs` and
  `ui/canvas/widget/press_session/prepare.rs` seams, so `press_session.rs` now mainly keeps the
  root re-exports and focused interaction-state fixture tests explicit.
- pending pointer-up release routing now also routes click-selection, generic pending release, and
  click-connect promotion through the private
  `ui/canvas/widget/pointer_up_pending/click_select.rs`,
  `ui/canvas/widget/pointer_up_pending/release.rs`, and
  `ui/canvas/widget/pointer_up_pending/wire_drag.rs` seams, so `pointer_up_pending.rs` now mainly
  keeps the root re-exports explicit while the click-threshold and click-connect policy helpers
  gain focused unit coverage.
- pointer-up resize commit op building now also routes node resize and group resize planners
  through the private `ui/canvas/widget/pointer_up_commit_resize/node.rs` and
  `ui/canvas/widget/pointer_up_commit_resize/group.rs` seams, so
  `pointer_up_commit_resize.rs` now mainly keeps the root re-exports explicit while each resize
  planner keeps its own focused unit coverage.
- left-button pointer-up routing now also routes edge-insert double-click activation and the
  release arbitration chain through the private
  `ui/canvas/widget/pointer_up_left_route/double_click.rs` and
  `ui/canvas/widget/pointer_up_left_route/dispatch.rs` seams, so
  `pointer_up_left_route.rs` now mainly keeps stop-auto-pan plus top-level orchestration explicit
  while the plain-double-click gate keeps focused unit coverage.
- committed pointer-up release handling now also routes resize and group-drag commit branches
  through the private `ui/canvas/widget/pointer_up_commit/resize.rs` and
  `ui/canvas/widget/pointer_up_commit/group_drag.rs` seams, so `pointer_up_commit.rs` now mainly
  keeps root re-exports plus node-drag delegation explicit while the commit wrappers stop
  accumulating inline orchestration.
- pointer-up state synchronization and release guards now also route through the private
  `ui/canvas/widget/pointer_up_state/sync.rs` and
  `ui/canvas/widget/pointer_up_state/release.rs` seams, so `pointer_up_state.rs` now mainly keeps
  root re-exports explicit while pointer-state projection and sticky-wire/pan release branches stop
  living inline together.
- pointer-up session helpers now also route generic release-slot handling and interaction cleanup
  through the private `ui/canvas/widget/pointer_up_session/release.rs` and
  `ui/canvas/widget/pointer_up_session/cleanup.rs` seams, so `pointer_up_session.rs` now mainly
  keeps root re-exports explicit while pending-release and snap-guide cleanup helpers stop sharing
  one inline module body.
- focus-session helpers now also route hint clearing, focus transitions, and selection-only
  view-state updates through the private `ui/canvas/widget/focus_session/hints.rs`,
  `ui/canvas/widget/focus_session/focus.rs`, and
  `ui/canvas/widget/focus_session/selection.rs` seams, so `focus_session.rs` now mainly keeps root
  re-exports explicit while edge/port/node focus bookkeeping stops sharing one inline helper file.
- cancel-session helpers now also route residual interaction cleanup and pan-release state helpers
  through the private `ui/canvas/widget/cancel_session/residuals.rs` and
  `ui/canvas/widget/cancel_session/pan.rs` seams, so `cancel_session.rs` now mainly keeps root
  re-exports explicit while sticky-wire/right-click cleanup and pan-release matching stop sharing
  one inline helper file.
- gesture-cancel handling now also routes wire-drag cancel callbacks and the remaining session
  clears through the private `ui/canvas/widget/cancel_gesture_state/wire.rs` and
  `ui/canvas/widget/cancel_gesture_state/sessions.rs` seams, so `cancel_gesture_state.rs` now
  mainly keeps top-level orchestration explicit while the bulk session reset logic gains focused
  state-only coverage.
- interaction gating now also routes cursor-detail, edge-hover, cache, and pan-inertia predicates
  through the private `ui/canvas/widget/interaction_gate/detail.rs`,
  `ui/canvas/widget/interaction_gate/hover.rs`, `ui/canvas/widget/interaction_gate/cache.rs`, and
  `ui/canvas/widget/interaction_gate/motion.rs` seams, so `interaction_gate.rs` now mainly keeps
  the gate surface explicit while each predicate family gains focused unit coverage.
- reconnect helpers now also route port-edge yank logic and reconnectable flag predicates through
  the private `ui/canvas/widget/reconnect/edges.rs` and
  `ui/canvas/widget/reconnect/flags.rs` seams, so `reconnect.rs` now mainly keeps the module split
  explicit while reconnect eligibility and endpoint derivation gain focused unit coverage.
- selection helpers now also route marquee edge-derivation and selectable predicates through the
  private `ui/canvas/widget/selection/box_edges.rs` and
  `ui/canvas/widget/selection/selectable.rs` seams, so `selection.rs` now mainly keeps the module
  split explicit while box-select edge modes and selectable overrides gain focused unit coverage.
- interaction policy helpers now also route node drag/connectable predicates plus port
  connectable/bundle checks through the private `ui/canvas/widget/interaction_policy/node.rs` and
  `ui/canvas/widget/interaction_policy/port.rs` seams, so `interaction_policy.rs` now mainly
  keeps the module split explicit while per-node and per-port policy overrides gain focused unit
  coverage.
- view commands now also route frame-all selection collection plus reset/zoom viewport helpers
  through the private `ui/canvas/widget/command_view/frame.rs` and
  `ui/canvas/widget/command_view/zoom.rs` seams, so `command_view.rs` now mainly keeps the module
  split explicit while frame-node collection and reset/zoom helper behavior gain focused unit
  coverage.
- hover-edge updates now also route target-edge resolution, hover hit queries, and hover-state
  sync through the private `ui/canvas/widget/hover/target.rs`,
  `ui/canvas/widget/hover/hit.rs`, and `ui/canvas/widget/hover/state.rs` seams, so `hover.rs`
  now mainly keeps the orchestration explicit while edge-target precedence and hover-state diff
  behavior gain focused unit coverage.
- command routing now also routes string-to-command dispatch through the private
  `ui/canvas/widget/command_router/dispatch.rs` seam, so `command_router.rs` now mainly keeps
  execution dispatch explicit while direct command aliases and canonical route mapping gain focused
  unit coverage.
- graph construction helpers now also route reroute-node op assembly and group-create
  selection/update helpers through the private `ui/canvas/widget/graph_construction/node.rs` and
  `ui/canvas/widget/graph_construction/group.rs` seams, so `graph_construction.rs` now mainly
  keeps the module split explicit while reroute/group construction helpers gain focused unit
  coverage.
- pending drag session helpers now also route group/node activation and node-abort behavior
  through the private `ui/canvas/widget/pending_drag_session/group.rs` and
  `ui/canvas/widget/pending_drag_session/node.rs` seams, so `pending_drag_session.rs` now mainly
  keeps the re-export surface explicit while pending drag activation helpers gain focused unit
  coverage.
- group paint helpers now also route static chrome/text layout and selected overlay filtering
  through the private `ui/canvas/widget/paint_groups/chrome.rs` and
  `ui/canvas/widget/paint_groups/overlay.rs` seams, so `paint_groups.rs` now mainly keeps the
  module split explicit while zoom-scaled group chrome and selected-overlay filtering gain focused
  unit coverage.
- press-session prepare helpers now also route target-hit and surface/pan preparation through the
  private `ui/canvas/widget/press_session/prepare/target.rs` and
  `ui/canvas/widget/press_session/prepare/surface.rs` seams, so `press_session/prepare.rs` now
  mainly keeps the re-export surface explicit while pointer-session clearing variants gain focused
  unit coverage.
- wire-drag hint paint helpers now also route hint message and border-color semantics through the
  private `ui/canvas/widget/paint_overlay_wire_hint/message.rs` and
  `ui/canvas/widget/paint_overlay_wire_hint/style.rs` seams, so `paint_overlay_wire_hint.rs` now
  mainly keeps the paint orchestration explicit while invalid-hover diagnostics and bundle/yank
  hint semantics gain focused unit coverage.
- toast overlay paint helpers now also route zoom-scaled layout and severity/style semantics
  through the private `ui/canvas/widget/paint_overlay_toast/layout.rs` and
  `ui/canvas/widget/paint_overlay_toast/style.rs` seams, so `paint_overlay_toast.rs` now mainly
  keeps the paint orchestration explicit while toast placement and severity color mapping gain
  focused unit coverage.
- pointer-down routing now also routes double-click arbitration and tail-lane dispatch through the
  private `ui/canvas/widget/event_pointer_down_route/double_click.rs` and
  `ui/canvas/widget/event_pointer_down_route/dispatch.rs` seams, so
  `event_pointer_down_route.rs` now mainly keeps early-return orchestration explicit while button
  lane selection retains focused unit coverage.
- grid-tile paint helpers now also route tile-index projection and pattern-density capacity
  estimation through the private `ui/canvas/widget/paint_grid_tiles/support.rs` seam, so
  `paint_grid_tiles.rs` now mainly keeps pattern-to-painter orchestration explicit while tile
  bounds projection and capacity heuristics gain focused unit coverage.
- keyboard-shortcut mapping now also routes modifier/history bindings and tab/arrow navigation
  bindings through the private `ui/canvas/widget/keyboard_shortcuts_map/modifier.rs` and
  `ui/canvas/widget/keyboard_shortcuts_map/navigation.rs` seams, so
  `keyboard_shortcuts_map.rs` now mainly keeps the re-export surface explicit while shortcut
  family mapping retains focused unit coverage.
- keyboard-shortcut gating now also routes modifier, navigation, and delete-binding predicates
  through the private `ui/canvas/widget/keyboard_shortcuts_gate/modifier.rs`,
  `ui/canvas/widget/keyboard_shortcuts_gate/navigation.rs`, and
  `ui/canvas/widget/keyboard_shortcuts_gate/editing.rs` seams, so
  `keyboard_shortcuts_gate.rs` now mainly keeps the re-export surface explicit while per-family
  gate predicates retain focused unit coverage.
- overlay hit helpers now also route context-menu geometry/item hit-testing and searcher
  geometry/row hit-testing through the private
  `ui/canvas/widget/overlay_hit/context_menu.rs` and
  `ui/canvas/widget/overlay_hit/searcher.rs` seams, so `overlay_hit.rs` now mainly keeps the
  re-export surface explicit while overlay hit geometry retains focused unit coverage.
- viewport math helpers now also route viewport construction/clamp helpers and canvas snap helpers
  through the private `ui/canvas/widget/view_math_viewport/viewport.rs` and
  `ui/canvas/widget/view_math_viewport/snap.rs` seams, so `view_math_viewport.rs` now mainly keeps
  the re-export surface explicit while viewport construction equivalence and snap behavior retain
  focused unit coverage.
- delete-op building now also routes group, node, and edge removal planners through the private
  `ui/canvas/widget/delete_ops_builder/group.rs`,
  `ui/canvas/widget/delete_ops_builder/node.rs`, and
  `ui/canvas/widget/delete_ops_builder/edge.rs` seams, so `delete_ops_builder.rs` now mainly keeps
  top-level delete orchestration explicit while edge de-duplication across node removal retains
  focused unit coverage.
- delete command helpers now also route remove-op collection and selection/view cleanup through the
  private `ui/canvas/widget/command_edit_remove/collect.rs` and
  `ui/canvas/widget/command_edit_remove/apply.rs` seams, so `command_edit_remove.rs` now mainly
  keeps cut/delete command orchestration explicit while remove-op collection and commit/view-state
  cleanup stop sharing one inline tail.
- right-click helpers now also route pending-release handling and click-threshold predicates
  through the private `ui/canvas/widget/right_click/pending.rs` and
  `ui/canvas/widget/right_click/threshold.rs` seams, so `right_click.rs` now mainly keeps the
  public helper surface explicit while pending click-threshold behavior retains focused unit
  coverage.
- searcher activation hit helpers now also route pointer-hit geometry and candidate-row lookup
  through the private `ui/canvas/widget/searcher_activation_hit/hit.rs` and
  `ui/canvas/widget/searcher_activation_hit/candidate.rs` seams, so
  `searcher_activation_hit.rs` now mainly keeps the re-export surface explicit while candidate-row
  mapping retains focused unit coverage.
- searcher activation state helpers now also route clear/dismiss, row-arm, and release/activation
  tails through the private `ui/canvas/widget/searcher_activation_state/clear.rs`,
  `ui/canvas/widget/searcher_activation_state/arm.rs`, and
  `ui/canvas/widget/searcher_activation_state/release.rs` seams, so
  `searcher_activation_state.rs` now mainly keeps the re-export surface explicit while searcher
  overlay clearing retains focused unit coverage.
- searcher wheel helpers now also route scroll-delta application through the private
  `ui/canvas/widget/searcher_pointer_wheel/delta.rs` seam, so
  `searcher_pointer_wheel.rs` now mainly keeps canvas-level wheel routing explicit while scroll
  clamping behavior retains focused unit coverage.
- searcher hover helpers now also route hovered-row state sync through the private
  `ui/canvas/widget/searcher_pointer_hover/state.rs` seam, so
  `searcher_pointer_hover.rs` now mainly keeps pointer-position to hovered-row orchestration
  explicit while hovered-row promotion behavior retains focused unit coverage.
- searcher navigation helpers now also route active-row step planning through the private
  `ui/canvas/widget/searcher_input_nav/step.rs` seam, so
  `searcher_input_nav.rs` now mainly keeps canvas-level active-row update orchestration explicit
  while selectable-row step planning retains focused unit coverage.
- view/gesture callback helpers now also route viewport lifecycle, node-drag gesture, and
  view-change fanout through the private `ui/canvas/widget/callbacks_view/viewport.rs`,
  `ui/canvas/widget/callbacks_view/node_drag.rs`, and
  `ui/canvas/widget/callbacks_view/view_change.rs` seams, so `callbacks_view.rs` now mainly keeps
  the re-export surface explicit while retained callback emission stops accumulating unrelated
  gesture/view tails inline.
- auto-measure sizing helpers now also route text-metric and width-planning logic through the
  private `ui/canvas/widget/auto_measure_apply/measure.rs` seam, so `auto_measure_apply.rs` now
  mainly keeps size-apply synchronization explicit while measured width planning stops sharing the
  same inline helper body.
- retained callback connect/graph helpers now also route wire-drag kind mapping and committed
  connection/delete fanout through the private `ui/canvas/widget/callbacks_connect/kind.rs`,
  `ui/canvas/widget/callbacks_graph/connection.rs`, and
  `ui/canvas/widget/callbacks_graph/delete.rs` seams, so `callbacks_connect.rs` and
  `callbacks_graph.rs` now mainly keep lifecycle orchestration explicit while callback payload
  mapping stops accumulating inline in the root helpers.
- auto-measure cache-key, per-node collect, and measured-size apply tails now also route through
  the private `ui/canvas/widget/auto_measure/key.rs`,
  `ui/canvas/widget/auto_measure_collect/input.rs`, and
  `ui/canvas/widget/auto_measure_apply/apply.rs` seams, so the `auto_measure*` roots now mainly
  keep cache invalidation and pipeline orchestration explicit while the collect/apply tails stop
  sharing root helper bodies.
- searcher query-edit and row-state helpers now also route key-to-query mutation plus recent-kind
  and active-row/scroll maintenance through the private
  `ui/canvas/widget/searcher_input_query/query.rs`,
  `ui/canvas/widget/searcher_rows/recent.rs`, and
  `ui/canvas/widget/searcher_rows/active.rs` seams, so `searcher_input_query.rs` and
  `searcher_rows.rs` now mainly keep canvas-level orchestration explicit while query mutation and
  row-state tails stop accumulating in the root helpers.
- searcher pointer activation now also routes pointer-down and pointer-up event tails through the
  private `ui/canvas/widget/searcher_activation/pointer_down.rs` and
  `ui/canvas/widget/searcher_activation/pointer_up.rs` seams, so `searcher_activation.rs` now
  mainly keeps shared hit shape plus activation-state façade methods explicit while event tails
  stop sharing the same root helper body.
- searcher picker and row-activation helpers now also route picker-request assembly, overlay-open
  tails, and activation-item mapping through the private
  `ui/canvas/widget/searcher_picker/catalog.rs`,
  `ui/canvas/widget/searcher_picker/overlay.rs`, and
  `ui/canvas/widget/searcher_row_activation/item.rs` seams, so `searcher_picker.rs` and
  `searcher_row_activation.rs` now mainly keep canvas-level orchestration explicit while picker
  request shaping and activation-item validation gain their own helper boundaries.
- searcher keyboard/input and overlay UI helpers now also route key dispatch plus overlay
  install/open and dismiss/finish tails through the private
  `ui/canvas/widget/searcher_input/dispatch.rs`,
  `ui/canvas/widget/searcher_ui/overlay.rs`, and
  `ui/canvas/widget/searcher_ui/event.rs` seams, so `searcher_input.rs` and `searcher_ui.rs` now
  mainly keep façade methods explicit while key routing and overlay event tails stop accumulating
  in the root files.
- searcher pointer helpers now also route pointer-move and wheel event tails through the private
  `ui/canvas/widget/searcher_pointer/move_event.rs` and
  `ui/canvas/widget/searcher_pointer/wheel_event.rs` seams, so `searcher_pointer.rs` now mainly
  keeps façade forwarding explicit while pointer invalidation tails stop sharing the same root
  helper body.
- menu/searcher session builders now also route context-menu state assembly and searcher state/row
  builders through the private `ui/canvas/widget/menu_session/context_menu.rs` and
  `ui/canvas/widget/menu_session/searcher.rs` seams, so `menu_session.rs` now mainly keeps the
  shared session-builder surface explicit while context-menu and searcher state assembly stop
  sharing one root helper body.
- insert-candidate helpers now also route reroute candidate synthesis, menu-item mapping, and
  presenter-backed candidate list loading through the private
  `ui/canvas/widget/insert_candidates/reroute.rs`,
  `ui/canvas/widget/insert_candidates/menu.rs`, and
  `ui/canvas/widget/insert_candidates/list.rs` seams, so `insert_candidates.rs` now mainly keeps
  the façade surface explicit while candidate synthesis and list loading stop sharing the same root
  helper body.
- group open-command helpers now also route create, draw-order, and rename overlay tails through
  the private `ui/canvas/widget/command_open_group/create.rs`,
  `ui/canvas/widget/command_open_group/order.rs`, and
  `ui/canvas/widget/command_open_group/rename.rs` seams, so `command_open_group.rs` now mainly
  keeps the command façade surface explicit while group command tails stop sharing one root helper
  body.
- insert/edge/conversion open-command helpers now also route insert fallback math, edge
  picker/reroute command tails, and conversion overlay open tails through the private
  `ui/canvas/widget/command_open_insert/fallback.rs`,
  `ui/canvas/widget/command_open_edge/picker.rs`,
  `ui/canvas/widget/command_open_edge/reroute.rs`, and
  `ui/canvas/widget/command_open_conversion/overlay.rs` seams, so the remaining `command_open_*`
  roots now mainly keep façade forwarding explicit.
- context-menu activation dispatch now also routes command actions and target-specific activation
  branches through the private `ui/canvas/widget/context_menu/activate/command.rs` and
  `ui/canvas/widget/context_menu/activate/target.rs` seams, so `context_menu/activate.rs` now
  mainly keeps the top-level dispatch surface explicit.
- context-menu item builders now also route shared command-item construction plus
  background/group/edge item families through the private
  `ui/canvas/widget/context_menu/item_builders/command_item.rs`,
  `ui/canvas/widget/context_menu/item_builders/background.rs`,
  `ui/canvas/widget/context_menu/item_builders/group.rs`, and
  `ui/canvas/widget/context_menu/item_builders/edge.rs` seams, so
  `context_menu/item_builders.rs` now mainly keeps the public builder surface explicit.
- context-menu selection activation now also routes activation-payload assembly and pointer-down
  activation tails through the private
  `ui/canvas/widget/context_menu/selection_activation/payload.rs` and
  `ui/canvas/widget/context_menu/selection_activation/pointer_down.rs` seams, so
  `context_menu/selection_activation.rs` now mainly keeps the selection façade explicit.
- context-menu opening now also routes group-hit, edge-hit, and background fallback branches
  through the private `ui/canvas/widget/context_menu/opening/group.rs`,
  `ui/canvas/widget/context_menu/opening/edge.rs`, and
  `ui/canvas/widget/context_menu/opening/background.rs` seams, so
  `context_menu/opening.rs` now mainly keeps the right-click orchestration explicit.
- keyboard context-menu navigation now also routes active-item stepping, typeahead, hover sync,
  key handling, and pointer-move tails through the private
  `ui/canvas/widget/context_menu/key_navigation/active_item.rs`,
  `ui/canvas/widget/context_menu/key_navigation/typeahead.rs`,
  `ui/canvas/widget/context_menu/key_navigation/hover.rs`,
  `ui/canvas/widget/context_menu/key_navigation/key_down.rs`, and
  `ui/canvas/widget/context_menu/key_navigation/pointer_move.rs` seams, so
  `context_menu/key_navigation.rs` now mainly keeps the navigation façade explicit.
- background context-menu execution now also routes insert planning, plan application, and action
  activation through the private `ui/canvas/widget/context_menu/background_execution/plan.rs`,
  `ui/canvas/widget/context_menu/background_execution/apply.rs`, and
  `ui/canvas/widget/context_menu/background_execution/activate.rs` seams, so
  `context_menu/background_execution.rs` now mainly keeps the plan enum plus execution façade
  explicit.
- connection insert/conversion menu execution now also routes activation, planning, plan
  application, and wire-drag recovery through the private
  `ui/canvas/widget/context_menu/connection_execution_insert/activate.rs`,
  `ui/canvas/widget/context_menu/connection_execution_insert/plan.rs`,
  `ui/canvas/widget/context_menu/connection_execution_insert/apply.rs`,
  `ui/canvas/widget/context_menu/connection_execution_insert/recovery.rs`,
  `ui/canvas/widget/context_menu/connection_execution_conversion/activate.rs`,
  `ui/canvas/widget/context_menu/connection_execution_conversion/plan.rs`, and
  `ui/canvas/widget/context_menu/connection_execution_conversion/apply.rs` seams, so
  `context_menu/connection_execution_insert.rs` and
  `context_menu/connection_execution_conversion.rs` now mainly keep the execution façades
  explicit.
- context target hit/selection now also routes group-vs-edge hit tests plus group-vs-edge
  selection reducers through the private `ui/canvas/widget/context_menu/target_hit/group.rs`,
  `ui/canvas/widget/context_menu/target_hit/edge.rs`,
  `ui/canvas/widget/context_menu/target_selection/group.rs`, and
  `ui/canvas/widget/context_menu/target_selection/edge.rs` seams, so
  `context_menu/target_hit.rs` and `context_menu/target_selection.rs` now mainly keep the façade
  surfaces explicit.
- edge context-menu execution now also routes open-picker, reroute, delete, and custom action
  branches through the private `ui/canvas/widget/context_menu/edge_execution/open_insert.rs`,
  `ui/canvas/widget/context_menu/edge_execution/reroute.rs`,
  `ui/canvas/widget/context_menu/edge_execution/delete.rs`, and
  `ui/canvas/widget/context_menu/edge_execution/custom_action.rs` seams, so
  `context_menu/edge_execution.rs` now mainly keeps the edge action dispatch explicit.
- split-edge reroute execution now also routes reroute planning, rejection-toast mapping,
  commit/apply tails, and outcome execution through the private
  `ui/canvas/widget/split_edge_execution/plan.rs`,
  `ui/canvas/widget/split_edge_execution/toast.rs`,
  `ui/canvas/widget/split_edge_execution/apply.rs`, and
  `ui/canvas/widget/split_edge_execution/execute.rs` seams, so
  `split_edge_execution.rs` now mainly keeps the public execution façade explicit.
- edge double-click handling now also routes insert-picker opening, reroute execution, double-click
  target hit-testing, and finish tails through the private
  `ui/canvas/widget/pointer_down_double_click_edge/insert_picker.rs`,
  `ui/canvas/widget/pointer_down_double_click_edge/reroute.rs`,
  `ui/canvas/widget/pointer_down_double_click_edge/target.rs`, and
  `ui/canvas/widget/pointer_down_double_click_edge/finish.rs` seams, and the insert-picker path
  now reuses the shared `select_edge_context_target` reducer instead of duplicating edge selection
  updates inline.
- pointer-down gesture start handling now also routes close-button dispatch,
  pending-right-click startup, and pan-start gating through the private
  `ui/canvas/widget/pointer_down_gesture_start/close_button.rs`,
  `ui/canvas/widget/pointer_down_gesture_start/pending_right_click.rs`, and
  `ui/canvas/widget/pointer_down_gesture_start/pan_start.rs` seams, so
  `pointer_down_gesture_start.rs` now mainly keeps the gesture-start façade explicit while
  preserving the existing context-menu and sticky-wire delegation.
- node-drag release op building now also routes release-op assembly, group-rect mapping, and
  commit-label selection through the private
  `ui/canvas/widget/pointer_up_node_drag_ops/build.rs`,
  `ui/canvas/widget/pointer_up_node_drag_ops/group_rect.rs`, and
  `ui/canvas/widget/pointer_up_node_drag_ops/commit_label.rs` seams, so
  `pointer_up_node_drag_ops.rs` now mainly keeps the public release-op façade explicit.
- node-drag preview now also routes preview-position computation and preview-state revision updates
  through the private `ui/canvas/widget/node_drag_preview/compute.rs` and
  `ui/canvas/widget/node_drag_preview/state.rs` seams, so `node_drag_preview.rs` now mainly keeps
  the node-drag preview façade explicit while the heavy preview calculation stops living in one
  monolithic root function.
- overlay painting now also routes close-button chrome/text paint and overlay-layer dispatch
  through the private `ui/canvas/widget/paint_overlays/close_button.rs` and
  `ui/canvas/widget/paint_overlays/layers.rs` seams, so `paint_overlays.rs` now mainly keeps the
  overlay paint orchestration explicit.
- multi-node extent clamping now also routes dragged-bounds collection and extent-delta clamping
  through the private `ui/canvas/widget/node_drag_constraints_extent/bounds.rs` and
  `ui/canvas/widget/node_drag_constraints_extent/clamp_delta.rs` seams, so
  `node_drag_constraints_extent.rs` now mainly keeps the extent-clamp entrypoint explicit.
- group resize planning now also routes pointer-to-rect mapping, child-size minimums, and
  snap-to-grid sizing through the private
  `ui/canvas/widget/group_resize_apply/pointer_rect.rs`,
  `ui/canvas/widget/group_resize_apply/children_min.rs`, and
  `ui/canvas/widget/group_resize_apply/snap.rs` seams, so
  `group_resize_apply.rs` now mainly keeps the resize-planning entrypoint explicit.
- viewport timer motion now also routes animation tick advancement and move-end debounce handling
  through the private `ui/canvas/widget/timer_motion_viewport/animation.rs` and
  `ui/canvas/widget/timer_motion_viewport/debounce.rs` seams, so `timer_motion_viewport.rs` now
  mainly keeps the viewport timer-motion façades explicit.
- pan-inertia timer motion now also routes stop guards and per-frame advancement through the
  private `ui/canvas/widget/timer_motion_pan_inertia/guards.rs` and
  `ui/canvas/widget/timer_motion_pan_inertia/advance.rs` seams, so
  `timer_motion_pan_inertia.rs` now mainly keeps the inertia tick orchestration explicit.
- viewport auto-pan timers now also route delta calculation, tick policy, and timer start/stop
  through the private `ui/canvas/widget/viewport_timer_auto_pan/delta.rs`,
  `ui/canvas/widget/viewport_timer_auto_pan/policy.rs`, and
  `ui/canvas/widget/viewport_timer_auto_pan/timer.rs` seams, so
  `viewport_timer_auto_pan.rs` now mainly keeps the auto-pan timer orchestration explicit.
- auto-pan motion ticks now also route drag-move dispatch through the private
  `ui/canvas/widget/timer_motion_auto_pan/dispatch.rs` seam, so
  `timer_motion_auto_pan.rs` now mainly keeps the auto-pan tick orchestration explicit.
- viewport animation timers now also route animation start/stop and move-end debounce handling
  through the private `ui/canvas/widget/viewport_timer_animation/animation.rs` and
  `ui/canvas/widget/viewport_timer_animation/debounce.rs` seams, so
  `viewport_timer_animation.rs` now mainly keeps the viewport timer façades explicit.
- sticky-wire connect handling now also routes target filtering / outcome planning and
  pointer-down finish cleanup through the private
  `ui/canvas/widget/sticky_wire_connect/plan.rs` and
  `ui/canvas/widget/sticky_wire_connect/finish.rs` seams, so
  `sticky_wire_connect.rs` now mainly keeps the sticky-wire connect orchestration explicit.
- node-drag move handling now also routes drag-delta planning and move-tail pan / callback /
  invalidation through the private `ui/canvas/widget/node_drag/delta.rs` and
  `ui/canvas/widget/node_drag/tail.rs` seams, so `node_drag.rs` now mainly keeps the node-drag
  move orchestration explicit.
- marquee selection queries now also route node hit collection and connected-edge selection
  through the private `ui/canvas/widget/marquee_selection_query/nodes.rs` and
  `ui/canvas/widget/marquee_selection_query/edges.rs` seams, so
  `marquee_selection_query.rs` now mainly keeps the marquee query orchestration explicit.
- group draw-order reducers now also route selected-group ordering and front/back application
  through the private `ui/canvas/widget/group_draw_order/selection.rs` and
  `ui/canvas/widget/group_draw_order/apply.rs` seams, so `group_draw_order.rs` now mainly keeps
  the group draw-order orchestration explicit.
- cursor resolution now also routes resize-handle hit resolution and edge-anchor target selection
  through the private `ui/canvas/widget/cursor_resolve/resize.rs` and
  `ui/canvas/widget/cursor_resolve/edge.rs` seams, so `cursor_resolve.rs` now mainly keeps the
  cursor-resolution façades explicit.
- pointer-move pan-release handling now also routes missing pan-release recovery and pending
  right-click pan-start arming through the private
  `ui/canvas/widget/pointer_move_release_pan/missing_release.rs` and
  `ui/canvas/widget/pointer_move_release_pan/pending_right_click.rs` seams, so
  `pointer_move_release_pan.rs` now mainly keeps the pan-release orchestration explicit.
- pending resize-session activation now also routes group/node activation state assembly through
  the private `ui/canvas/widget/pending_resize_session/group.rs` and
  `ui/canvas/widget/pending_resize_session/node.rs` seams, so
  `pending_resize_session.rs` now mainly keeps the resize-session activation façades explicit.
- drag-threshold checks now also route threshold normalization and squared-distance comparison
  through the private `ui/canvas/widget/threshold/normalize.rs` and
  `ui/canvas/widget/threshold/distance.rs` seams, so `threshold.rs` now mainly keeps the drag
  threshold façade explicit.
- pending node-drag startup now also routes threshold/draggable gating and selection/start-node
  activation through the private `ui/canvas/widget/pending_drag/checks.rs` and
  `ui/canvas/widget/pending_drag/activate.rs` seams, so `pending_drag.rs` now mainly keeps the
  pending node-drag orchestration explicit.
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
- Menu/searcher overlay-session policy types now also live on a dedicated private seam,
  `ui/canvas/state/state_overlay_policy.rs`, so `ContextMenuTarget` and `SearcherRowsMode` no
  longer stay declared inside `state_overlay_sessions.rs` while session container structs keep
  their original ownership.
- Searcher picker opener policy now also lives on one request seam,
  `ui/canvas/widget/searcher_picker/request.rs`, so `SearcherPickerRequest` owns `rows_mode` and
  background/connection insert pickers, edge-insert pickers, and conversion picker openers no
  longer duplicate `Catalog` / `Flat` request policy outside the same picker request authority.
- Searcher row activation now also reuses the insert-candidate menu authority,
  `ui/canvas/widget/insert_candidates/menu.rs`, so candidate-row activation no longer duplicates
  single candidate-to-menu-item synthesis outside the same seam used by context-menu candidate
  lists.
- Searcher row activation now also reuses selectable-row policy from `searcher_rows`, so
  activation gating no longer duplicates the same candidate-enabled rule already used by active-row
  selection and keyboard navigation.
- Context-menu target dispatch now also routes non-command activation through the private
  `ui/canvas/widget/context_menu/activate/target.rs` seam, so `activate.rs` keeps the
  command-vs-target action-kind split while background/connection/edge/conversion target routing
  stops living as an unowned inline match and gains focused route-mapping coverage.
- Command context-menu activation now also routes target-scoped selection side effects through the
  private `ui/canvas/widget/context_menu/activate/command.rs` seam, so the
  group-selection-vs-ignore policy becomes explicit and gains focused route-mapping coverage
  instead of staying hidden inside command dispatch glue.
- Edge context-menu activation now also routes edge-action planning through the private
  `ui/canvas/widget/context_menu/edge_execution.rs` seam, so insert-picker / reroute / delete /
  custom edge actions no longer stay as an unowned inline match before delegating to their
  executor modules and focused route-mapping coverage.
- Right-click context-menu opening now also routes target-hit priority through the private
  `ui/canvas/widget/context_menu/opening.rs` seam, so group-vs-edge-vs-background precedence
  becomes explicit, gains focused route-mapping coverage, and no longer lives as an inline `if`
  chain in the opening event glue.
- Context-menu presentation now also routes open-event state effects through the private
  `ui/canvas/widget/context_menu/ui.rs` seam, so menu install, hover-edge cleanup, focus request,
  and event-finish invalidation stop living in `show_context_menu(...)`; the open-path hover-edge
  behavior now also uses an explicit policy type instead of a boolean flag.
- Context-menu presentation lifecycle now also mirrors the searcher split:
  `ui/canvas/widget/context_menu/ui/overlay.rs` owns state install/restore/take/clear plus
  hover-edge cleanup policy, `ui/canvas/widget/context_menu/ui/event.rs` owns
  open/restore/dismiss event tails plus finish/invalidation, and the root `ui.rs` now stays a thin
  wrapper surface instead of a mixed state-and-event file.
- Searcher overlay install now also has an explicit replacement seam in
  `ui/canvas/widget/searcher_ui/overlay.rs`, so the "clear context menu, then install/replace
  searcher state" rule becomes a named helper with focused state tests instead of staying hidden in
  the root install path.
- Context-menu/searcher event tails now also share the retained widget runtime finish helper, so
  `ui/canvas/widget/context_menu/ui/event.rs` and `ui/canvas/widget/searcher_ui/event.rs` stop
  duplicating the same stop-propagation plus paint invalidation tail logic inline.
- Active menu-session occupancy now also routes through the private
  `canvas/widget/menu_session.rs` seam, so window-focus deferral, space-to-pan gating,
  Tab-navigation suppression, edge double-click preflight, motion/auto-pan tick guards, and
  retained `view_interacting(...)` all reuse one `context_menu || searcher` authority instead of
  re-embedding that overlay-session policy across multiple runtime files.
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
  `ui/retained_event_tail.rs` seam, so portal commands plus
  controls/blackboard/minimap/group-rename overlays share one authority for focus-to-canvas,
  stop-propagation, redraw, and paint/layout invalidation tails instead of duplicating those
  handled-event endings inline.
- Retained action-panel pointer state now also routes through the private
  `ui/overlays/panel_pointer_policy.rs` seam, so controls and blackboard overlays share one
  hover sync plus press-on-down / activate-on-matching-up authority instead of each
  re-embedding that pointer-state policy inline.
- Retained minimap projection math now also routes through the private
  `ui/overlays/minimap_projection.rs` seam, so world-bounds union, project/unproject
  transforms, and center-pan math live behind one focused authority instead of staying
  embedded in the overlay widget file.
- Retained blackboard layout and hit-testing now also route through the private
  `ui/overlays/blackboard_layout.rs` seam, so panel/header/row geometry plus action hit
  detection live behind one focused authority instead of staying embedded in the overlay
  widget file.
- Retained controls layout and hit-testing now also route through the private
  `ui/overlays/controls_layout.rs` seam, so panel geometry plus button hit detection live
  behind one focused authority instead of staying embedded in the overlay widget file.
- Retained action-panel item state now also routes through the private
  `ui/overlays/panel_item_state.rs` seam, so controls and blackboard overlays share one
  authority for keyboard selection resets, pointer-to-keyboard promotion, and visible
  item-state evaluation instead of each re-embedding that panel-state policy inline.
- Retained rename host layout lifecycle now also routes through the private
  `ui/overlays/rename_host_layout.rs` seam, so hidden/cancelled/active overlay layout
  planning lives behind one focused authority instead of staying embedded in the rename host
  widget file.
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
  authority for centered button-label placement and left-aligned panel text placement instead
  of each re-embedding that paint-side text geometry inline.
- Retained blackboard paint orchestration now also routes through the private
  `ui/overlays/blackboard_paint.rs` seam, so header/title paint ordering, action-button
  highlight resolution, and missing-symbol label fallback live behind one focused authority
  instead of staying embedded in the overlay widget file.
- Context-menu open-state replacement now also routes edge-insert submenu reopening through the
  private `ui/canvas/widget/context_menu/ui/overlay.rs` seam, so hover-edge preserve-vs-clear
  policy stops being bypassed by direct `interaction.context_menu = Some(...)` writes in
  `edge_insert/context_menu.rs`.
- Searcher teardown for insert-node drag handoff now also routes through the private
  `ui/canvas/widget/searcher_activation_state/clear.rs` seam, so `pending_insert_node_drag`
  cleanup and searcher dismissal stop being bypassed by direct `interaction.searcher = None`
  writes in `insert_node_drag/session.rs`.
- Searcher session take/restore now also routes through the private
  `ui/canvas/widget/searcher_ui/overlay.rs` seam, so failed row activation no longer keeps a
  second direct `interaction.searcher.take()/= Some(...)` lifecycle path in
  `searcher_row_activation.rs`.
- Command-driven conversion-picker menu replacement now also routes through the private
  `ui/canvas/widget/searcher_ui/overlay.rs` install seam via `searcher_picker`, so
  `command_open_conversion/overlay.rs` no longer clears `interaction.context_menu` before opening
  the searcher and menu-to-searcher replacement keeps one authority path.
- Context-menu command activation no longer clears the menu slot inside
  `ui/canvas/widget/context_menu/activate/command.rs`, because pointer/key selection activation
  already owns the menu `take(...)` lifecycle before dispatch and searcher-row command activation
  now reuses the same route without a redundant second dismiss path.
- Active menu/searcher occupancy now also routes through the private
  `ui/canvas/widget/menu_session.rs` seam for edge-insert picker fallback, background
  double-click zoom preflight, and detail/hover cursor gates, so those paths stop
  re-embedding direct `context_menu/searcher` slot checks inline.
- The `menu_session.rs` wrapper now also delegates `build_searcher_rows(...)` directly to
  `canvas/widget/menu_session/searcher.rs`, so flat-vs-catalog row policy keeps one authority
  seam instead of remaining duplicated across both wrapper and submodule entrypoints.
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
- `sticky_wire.rs` now also routes pointer-down eligibility/new-wire extraction
  and target resolution through the private `canvas/widget/sticky_wire/checks.rs` and
  `canvas/widget/sticky_wire/target.rs` seams, while `sticky_wire_targets.rs` now routes
  non-port hit inspection and picker-open finalization through the private
  `canvas/widget/sticky_wire_targets/inspect.rs` and
  `canvas/widget/sticky_wire_targets/picker.rs` seams, so the retained sticky-wire reducers keep
  shrinking toward façade-only orchestration.
- `pending_connection_session.rs` now routes pending edge-insert and pending
  wire promotion through the private `canvas/widget/pending_connection_session/edge.rs` and
  `canvas/widget/pending_connection_session/wire.rs` seams, so retained connection-session helpers
  keep shrinking toward façade-only orchestration.
- `pending_wire_drag.rs` now routes drag-threshold preparation and
  activation side effects through the private `canvas/widget/pending_wire_drag/checks.rs` and
  `canvas/widget/pending_wire_drag/activate.rs` seams, while
  `pointer_up_pending/wire_drag.rs` now routes click-connect promotion predicates and activation
  through the private `canvas/widget/pointer_up_pending/wire_drag/checks.rs` and
  `canvas/widget/pointer_up_pending/wire_drag/activate.rs` seams, so retained wire-entry reducers
  keep shrinking toward façade-only orchestration.
- `edge_insert_drag/pending.rs` now routes drag-threshold preparation and
  activation through the private `canvas/widget/edge_insert_drag/pending/checks.rs` and
  `canvas/widget/edge_insert_drag/pending/activate.rs` seams, so retained edge-insert pending move
  reducers keep shrinking toward façade-only orchestration.
- `left_click/connection_hits/port.rs` now routes connectability resolution,
  click-connect target handling, and wire-kind arming through the private
  `canvas/widget/left_click/connection_hits/port/connectable.rs`,
  `canvas/widget/left_click/connection_hits/port/click_connect.rs`, and
  `canvas/widget/left_click/connection_hits/port/kind.rs` seams, so retained connection-port
  reducers keep shrinking toward façade-only orchestration.
- `edge_insert_drag/pointer_up.rs` now routes pending-release cleanup and
  active picker-opening tail handling through the private
  `canvas/widget/edge_insert_drag/pointer_up/pending.rs` and
  `canvas/widget/edge_insert_drag/pointer_up/active.rs` seams, so retained edge-insert left-up
  reducers keep shrinking toward façade-only orchestration.
- `left_click/connection_hits/edge_anchor.rs` now routes shared edge
  selectable/focus/selection reduction through the private
  `canvas/widget/left_click/edge_selection.rs` seam and reconnect arming through
  `canvas/widget/left_click/connection_hits/edge_anchor/arm.rs`, while
  `left_click/element_hits/edge.rs` now routes the same shared edge-selection seam plus
  drag-vs-insert arming through `canvas/widget/left_click/element_hits/edge/drag.rs`, so retained
  edge-hit reducers keep shrinking toward façade-only orchestration.
- `left_click/element_hits/node.rs` now routes node
  selectable/draggable capability lookup, shared node-selection reducers, and pending-drag
  planning through the private `canvas/widget/left_click/node_selection.rs`,
  `canvas/widget/left_click/element_hits/node/capabilities.rs`, and
  `canvas/widget/left_click/element_hits/node/drag.rs` seams, while
  `left_click/element_hits/resize.rs` now routes the same shared node-selection seam plus
  pending-resize seed construction through `canvas/widget/left_click/element_hits/resize/state.rs`,
  so retained node/resize hit reducers keep shrinking toward façade-only orchestration.
- `pending_resize.rs` now routes node-resize drag-threshold checks and
  activation through the private `canvas/widget/pending_resize/checks.rs` and
  `canvas/widget/pending_resize/activate.rs` seams, while `edge_insert_drag/drag.rs` now routes
  active edge-insert drag state writeback and invalidate tail handling through the private
  `canvas/widget/edge_insert_drag/drag/state.rs` and
  `canvas/widget/edge_insert_drag/drag/tail.rs` seams, so both retained move reducers keep
  shrinking toward façade-only orchestration.
- `pointer_move_dispatch/primary.rs` now routes surface pan/marquee,
  group drag/resize, node pending drag/resize, and pending connection move arbitration through the
  private `canvas/widget/pointer_move_dispatch/primary/surface.rs`,
  `canvas/widget/pointer_move_dispatch/primary/group.rs`,
  `canvas/widget/pointer_move_dispatch/primary/node.rs`, and
  `canvas/widget/pointer_move_dispatch/primary/connection.rs` seams, so retained primary
  pointer-move ordering keeps shrinking toward façade-only orchestration.
- `pointer_move_dispatch/secondary.rs` now routes active node
  resize/drag, active connection drag arbitration, and pending insert-node drag promotion through
  the private `canvas/widget/pointer_move_dispatch/secondary/node.rs`,
  `canvas/widget/pointer_move_dispatch/secondary/connection.rs`, and
  `canvas/widget/pointer_move_dispatch/secondary/insert.rs` seams, so retained secondary
  pointer-move ordering keeps shrinking toward façade-only orchestration.
- `insert_execution.rs` now routes reroute-candidate detection, plan
  delegates, feedback delegates, and local regression tests through the private
  `canvas/widget/insert_execution/candidate.rs`,
  `canvas/widget/insert_execution/plan.rs`,
  `canvas/widget/insert_execution/feedback.rs`, and
  `canvas/widget/insert_execution/tests.rs` seams, so the root retained insert-execution file now
  mainly keeps module wiring and façade exports explicit.
- `command_router.rs` now routes direct insert/group/view/focus/edit
  execution through the private `canvas/widget/command_router/insert.rs`,
  `canvas/widget/command_router/group.rs`,
  `canvas/widget/command_router/view.rs`,
  `canvas/widget/command_router/focus.rs`, and
  `canvas/widget/command_router/edit.rs` seams, so the root command router now mainly keeps
  nudge/align precedence and direct-route orchestration explicit.
- `paint_edge_anchors.rs` now routes target-edge reconnectability
  lookup, hover/active interaction state resolution, anchor paint-style calculation, and scene-op
  push through the private `canvas/widget/paint_edge_anchors/resolve.rs`,
  `canvas/widget/paint_edge_anchors/state.rs`,
  `canvas/widget/paint_edge_anchors/style.rs`, and
  `canvas/widget/paint_edge_anchors/render.rs` seams, so the root edge-anchor painter now mainly
  keeps draw-order and endpoint iteration orchestration explicit.
- `paint_root_helpers.rs` now routes static-scene paint-token key
  writes and geometry-token key writes through the private
  `canvas/widget/paint_root_helpers/paint.rs` and
  `canvas/widget/paint_root_helpers/geometry.rs` seams, so the root paint-root helper file now
  mainly keeps static-scene style-key wiring explicit.
- `paint_root/cache_plan.rs` now routes hovered-edge resolution and
  static-scene cache eligibility / tile / rect planning through the private
  `canvas/widget/paint_root/cache_plan/hover.rs` and
  `canvas/widget/paint_root/cache_plan/tiles.rs` seams, so the root paint-root cache-plan file now
  mainly keeps derived-output publish and cache-plan orchestration explicit.
- `widget_surface.rs` now routes retained-only runtime constants through
  the private `canvas/widget/widget_surface/constants.rs` seam, so the root widget-surface file
  now mainly keeps module wiring and the noop `new` constructor explicit while keeping the public
  widget contract unchanged.
- `widget_surface/runtime.rs` now routes render-cull/debug helpers,
  interaction activity grouping, and edge-path resolution through the private
  `canvas/widget/widget_surface/runtime/render.rs`,
  `canvas/widget/widget_surface/runtime/interaction.rs`, and
  `canvas/widget/widget_surface/runtime/edge.rs` seams, so the runtime façade now mainly keeps
  retained helper routing explicit without changing the app-facing surface.
- `paint_root/frame.rs` now routes begin-frame cache bookkeeping,
  path-cache diagnostics publication, and canvas background fill through the private
  `canvas/widget/paint_root/frame/cache.rs` and
  `canvas/widget/paint_root/frame/background.rs` seams while reusing the shared
  `paint_grid_plan_support::resolve_canvas_chrome_hint` helper, so the root frame file now mainly
  keeps viewport/cull setup and grid bootstrap explicit.
- `paint_root/cached_edges/edges.rs` now routes uncached fallback
  rendering, cache replay/store helpers, single-rect cache build, and tiled cache build through
  the private `canvas/widget/paint_root/cached_edges/edges/fallback.rs`,
  `canvas/widget/paint_root/cached_edges/edges/replay.rs`,
  `canvas/widget/paint_root/cached_edges/edges/single.rs`, and
  `canvas/widget/paint_root/cached_edges/edges/tiled.rs` seams, so the root cached-edge file now
  mainly keeps the retained façade surface explicit.
- `paint_root/cached_edges/labels.rs` now routes label-cache
  replay/store helpers, single-rect label build, and tiled label build through the private
  `canvas/widget/paint_root/cached_edges/labels/replay.rs`,
  `canvas/widget/paint_root/cached_edges/labels/single.rs`, and
  `canvas/widget/paint_root/cached_edges/labels/tiled.rs` seams, so the root cached-edge label
  file now mainly keeps the retained façade surface explicit.
- `paint_root/cached_edges/build_state.rs` now routes clip-op stack
  assembly, initial cached-edge/label state allocation, and budget-step advancement through the
  private `canvas/widget/paint_root/cached_edges/build_state/ops.rs`,
  `canvas/widget/paint_root/cached_edges/build_state/init.rs`, and
  `canvas/widget/paint_root/cached_edges/build_state/step.rs` seams, so the root build-state file
  now mainly keeps the retained façade surface explicit.
- static group/node scene-cache orchestration now also routes through
  the private `canvas/widget/paint_root/static_layer.rs` seam, so `cached_groups.rs` and
  `cached_nodes.rs` stop re-embedding the same layer-target replay/store routing while keeping
  each layer's render-data collection, static paint body, and overlay tail explicit.
- `paint_root/edge_anchor.rs` now routes target edge-id selection,
  render-data target lookup, and geometry-based target reconstruction through the private
  `canvas/widget/paint_root/edge_anchor/target_id.rs`,
  `canvas/widget/paint_root/edge_anchor/render.rs`, and
  `canvas/widget/paint_root/edge_anchor/geometry.rs` seams, so the root edge-anchor file now
  mainly keeps the retained façade surface explicit.
- `paint_root/cached_edges/mod.rs` now routes anchor-target
  resolution, tiled vs single-rect dispatch, and cache-disabled uncached fallback through the
  private `canvas/widget/paint_root/cached_edges/anchor_target.rs`,
  `canvas/widget/paint_root/cached_edges/dispatch.rs`, and
  `canvas/widget/paint_root/cached_edges/fallback.rs` seams, so the root cached-edge orchestration
  file now mainly keeps retained cache routing explicit.
- `paint_nodes/static_node_chrome.rs` now routes node
  paint-style resolution, shadow effect setup, static chrome quads, and title/body text paint
  through the private `canvas/widget/paint_nodes/static_node_chrome/style.rs`,
  `canvas/widget/paint_nodes/static_node_chrome/shadow.rs`,
  `canvas/widget/paint_nodes/static_node_chrome/quads.rs`, and
  `canvas/widget/paint_nodes/static_node_chrome/text.rs` seams, so the root static-node chrome
  file now mainly keeps retained node-chrome orchestration explicit.
- `paint_nodes/static_ports.rs` now routes label text paint,
  inner-scale port geometry, fill-path fallback paint, stroke-path fallback paint, and top-level
  shape iteration through the private `canvas/widget/paint_nodes/static_ports/labels.rs`,
  `canvas/widget/paint_nodes/static_ports/geometry.rs`,
  `canvas/widget/paint_nodes/static_ports/fill.rs`,
  `canvas/widget/paint_nodes/static_ports/stroke.rs`, and
  `canvas/widget/paint_nodes/static_ports/shapes.rs` seams, so the root static-port file now
  mainly keeps retained port-paint orchestration explicit without changing port chrome/path
  behavior.
- focused `cargo check -p fret-node --features compat-retained-canvas --tests`
  and `cargo nextest run -p fret-node --features compat-retained-canvas` coverage now also locks
  `skin_port_chrome_hints_apply_fill_stroke_and_inner_scale_paint_only`,
  `skin_port_shape_hint_renders_path_ops_for_non_circle_shapes`, and
  `preset_exec_ports_use_triangle_shape_and_emit_path_ops` after the static-port split.
- `paint_edges/markers_support.rs` now routes marker paint-binding
  resolution, scene path replay, route-marker path preparation, and custom-marker/custom-wire path
  preparation through the private `canvas/widget/paint_edges/markers_support/paint.rs`,
  `canvas/widget/paint_edges/markers_support/route.rs`, and
  `canvas/widget/paint_edges/markers_support/custom.rs` seams, so the root marker-support file now
  mainly keeps the retained helper façade explicit without changing marker budget or wire replay
  behavior.
- focused `cargo nextest run -p fret-node --features compat-retained-canvas`
  coverage now also locks `paint_overrides_can_drive_edge_marker_paint_binding`,
  `bezier_markers_align_with_bezier_start_end_tangents`, and
  `custom_edge_marker_falls_back_to_from_to_tangent_when_path_has_no_tangents` after the marker
  helper split.
- `paint_edges/cached_budgeted.rs` now routes retained static-edge
  wire/marker warmup and label warmup through the private
  `canvas/widget/paint_edges/cached_budgeted/wires.rs` and
  `canvas/widget/paint_edges/cached_budgeted/labels.rs` seams, so the root cached-budgeted file
  now mainly keeps the cache-facing façade explicit without changing tile-budget or label
  placement behavior.
- focused `cargo nextest run -p fret-node --features compat-retained-canvas`
  coverage now also locks `cached_edge_paths_match_between_tiled_and_single_tile_cache_modes`,
  `paint_warms_edge_scene_cache_incrementally`, and
  `paint_warms_edge_label_scene_cache_incrementally_for_large_viewport_tiles` after the
  cached-budgeted split.
- `paint_render_data/edges.rs` now routes edge candidate enumeration,
  normalized hint/paint-override resolution, cull-bounds gating, and per-edge render payload
  append through the private `canvas/widget/paint_render_data/edges/candidate.rs`,
  `canvas/widget/paint_render_data/edges/hint.rs`,
  `canvas/widget/paint_render_data/edges/cull.rs`, and
  `canvas/widget/paint_render_data/edges/append.rs` seams, so the root edge render-data file now
  mainly keeps the top-level collection loop and stable sort explicit without changing edge
  payload semantics.
- focused `cargo nextest run -p fret-node --features compat-retained-canvas`
  coverage now also locks `edge_render_hint_is_resolved_in_stage_order_presenter_edge_types_skin`,
  `edge_label_border_uses_edge_render_hint_color_override`,
  `paint_reuses_static_edge_scene_cache_without_revisiting_presenter`, and
  `cached_edge_paths_match_between_tiled_and_single_tile_cache_modes` after the edge render-data
  split.
- `paint_render_data/nodes.rs` now routes node visibility
  filtering/order, label-overhead sizing, node tuple append, and port label/pin payload append
  through the private `canvas/widget/paint_render_data/nodes/visible.rs`,
  `canvas/widget/paint_render_data/nodes/overhead.rs`,
  `canvas/widget/paint_render_data/nodes/append.rs`, and
  `canvas/widget/paint_render_data/nodes/ports.rs` seams, so the root node render-data file now
  mainly keeps the retained façade entry points explicit without changing node/port payload
  semantics.
- focused `cargo nextest run -p fret-node --features compat-retained-canvas`
  coverage now also locks `elevate_nodes_on_select_draws_selected_node_body_last`,
  `skin_port_chrome_hints_apply_fill_stroke_and_inner_scale_paint_only`,
  `per_node_header_palette_draws_distinct_header_quads`, and
  `paint_reuses_static_node_scene_cache_without_revisiting_presenter` after the node render-data
  split.
- `paint_render_data/collect.rs` now also routes render-selection
  snapshot extraction and the graph-read collection body through the private
  `canvas/widget/paint_render_data/collect/selection.rs` and
  `canvas/widget/paint_render_data/collect/body.rs` seams, so the root collect file now mainly
  keeps the retained render-data façade explicit without changing collect-time grouping,
  visibility, or edge ordering semantics.
- focused `cargo test -p fret-node --features compat-retained-canvas`
  coverage now also locks `edge_render_hint_is_resolved_in_stage_order_presenter_edge_types_skin`,
  `elevate_nodes_on_select_draws_selected_node_body_last`,
  `skin_wire_outline_selected_draws_outline_path_before_core`, and
  `edges_are_sorted_by_endpoint_z_order` after the render-data collect split.
- `paint_render_data/selected_nodes.rs` now also routes selected-node
  snapshot extraction and the graph-read selected-node append/sort body through the private
  `canvas/widget/paint_render_data/selected_nodes/selection.rs` and
  `canvas/widget/paint_render_data/selected_nodes/body.rs` seams, so the root selected-node
  render-data file now mainly keeps the retained façade explicit without changing elevate-on-select
  filtering, append, or rank-sort semantics.
- focused `cargo test -p fret-node --features compat-retained-canvas`
  coverage now also locks `elevate_nodes_on_select_draws_selected_node_body_last`,
  `selection_updates_do_not_rebuild_geometry_when_elevate_nodes_on_select_is_enabled`, and
  `paint_reuses_static_node_scene_cache_without_revisiting_presenter` after the selected-node
  render-data split.
- `paint_overlay_menu.rs` now also routes menu frame/background
  layout and per-item highlight/text painting through the private
  `canvas/widget/paint_overlay_menu/frame.rs` and
  `canvas/widget/paint_overlay_menu/items.rs` seams, so the root context-menu paint file now
  mainly keeps the retained overlay façade explicit without changing overlay sizing, hover, or text
  placement semantics.
- focused `cargo test -p fret-node --features compat-retained-canvas`
  coverage now also locks `overlay_menu_searcher_conformance`,
  `right_click_cancels_wire_drag_and_opens_context_menu`,
  `right_pan_drag_does_not_open_context_menu`,
  `overlay_state_changes_do_not_rebuild_derived_geometry_or_spatial_index`, and
  `overlay_hover_and_scroll_updates_do_not_rebuild_derived_geometry_or_spatial_index` after the
  context-menu paint split.
- `paint_searcher.rs` now also routes searcher frame/background
  layout, text-style preparation, and shared text constraints through the private
  `canvas/widget/paint_searcher/frame.rs` seam, so the root searcher paint file now mainly keeps
  query/row orchestration explicit without changing searcher sizing, query chrome, or row paint
  semantics.
- focused `cargo test -p fret-node --features compat-retained-canvas`
  coverage now also locks `overlay_menu_searcher_conformance`,
  `overlay_state_changes_do_not_rebuild_derived_geometry_or_spatial_index`, and
  `overlay_hover_and_scroll_updates_do_not_rebuild_derived_geometry_or_spatial_index` after the
  searcher frame split.
- `paint_overlay_wire_hint.rs` now also routes wire-hint text/layout
  preparation and final quad/text scene emission through the private
  `canvas/widget/paint_overlay_wire_hint/layout.rs` and
  `canvas/widget/paint_overlay_wire_hint/draw.rs` seams, so the root wire-hint paint file now
  mainly keeps message/border resolution orchestration explicit without changing invalid-hover,
  bundle/yank hint, or wire-hint placement semantics.
- focused `cargo test -p fret-node --features compat-retained-canvas`
  coverage now also locks `invalid_hover_message_prefers_hover_diagnostic`,
  `hint_text_reports_bundle_and_yank_counts`,
  `resolved_hint_border_color_uses_context_border_for_valid_hover`,
  `diagnostic_hint_border_color_prefers_convertible_warning_color`, and
  `hover_state_updates_do_not_rebuild_canvas_derived_geometry_or_spatial_index` after the
  wire-hint split.
- `paint_overlay_toast.rs` now also routes final toast quad/text
  scene emission through the private `canvas/widget/paint_overlay_toast/draw.rs` seam, so the root
  toast paint file now mainly keeps style/layout orchestration explicit without changing toast
  border-color, box sizing, or viewport anchoring semantics.
- focused `cargo test -p fret-node --features compat-retained-canvas`
  coverage now also locks `toast_border_color_matches_diagnostic_severity`,
  `toast_rect_clamps_box_width_to_minimum`,
  `toast_rect_clamps_box_width_to_maximum`, and
  `toast_rect_places_box_at_viewport_bottom_left` after the toast draw split.
- `paint_grid_plan_support.rs` now also routes canvas chrome-hint
  lookup, grid width / thickness / tile-size metric resolution, tile scratch population, and
  pattern-size validation through the private `canvas/widget/paint_grid_plan_support/hint.rs`,
  `canvas/widget/paint_grid_plan_support/metrics.rs`,
  `canvas/widget/paint_grid_plan_support/tiles.rs`, and
  `canvas/widget/paint_grid_plan_support/validate.rs` seams, so the root grid-plan support file now
  mainly keeps the retained façade explicit without changing grid-plan preparation semantics.
- focused `cargo test -p fret-node --features compat-retained-canvas`
  coverage now also locks `background_style_updates_do_not_rebuild_canvas_derived_geometry`,
  `background_style_override_survives_color_mode_theme_sync`,
  `dots_pattern_emits_rounded_quads`, and
  `cross_pattern_emits_axis_aligned_segments` after the grid-plan-support split.
- `pointer_down_double_click_background.rs` now also routes
  background zoom preflight gating, background hit testing, and double-click zoom application
  through the private `canvas/widget/pointer_down_double_click_background/preflight.rs`,
  `canvas/widget/pointer_down_double_click_background/hit.rs`, and
  `canvas/widget/pointer_down_double_click_background/apply.rs` seams, so the root
  background-double-click reducer now mainly keeps the retained façade explicit without changing
  zoom-on-double-click gating, hit exclusion, or viewport move callback semantics.
- focused `cargo test -p fret-node --features compat-retained-canvas`
  coverage now also locks `double_click_background_zooms_in_about_pointer`,
  `shift_double_click_background_zooms_out_about_pointer`, and
  `double_click_background_zoom_emits_move_start_and_move_end` after the background-double-click
  split.
- `paint_grid_cache.rs` now also routes warmup orchestration,
  per-tile op generation, and cache-key construction through the private
  `canvas/widget/paint_grid_cache/warm.rs`,
  `canvas/widget/paint_grid_cache/ops.rs`, and
  `canvas/widget/paint_grid_cache/key.rs` seams, so the root grid-cache file now mainly keeps tile
  warmup stats plus the retained façade explicit without changing grid cache warmup or key
  semantics.
- focused `cargo test -p fret-node --features compat-retained-canvas`
  coverage now also locks `background_style_updates_do_not_rebuild_canvas_derived_geometry`,
  `background_style_override_survives_color_mode_theme_sync`,
  `dots_pattern_emits_rounded_quads`, and
  `cross_pattern_emits_axis_aligned_segments` after the grid-cache split.
- `pointer_wheel_zoom.rs` now also routes wheel-zoom factor
  resolution, pinch-zoom factor resolution, and shared viewport zoom application through the
  private `canvas/widget/pointer_wheel_zoom/wheel.rs`,
  `canvas/widget/pointer_wheel_zoom/pinch.rs`, and
  `canvas/widget/pointer_wheel_zoom/apply.rs` seams, so the root wheel-zoom reducer now mainly
  keeps the retained facade entry points explicit without changing wheel/pinch gating, viewport
  move debounce, or zoom-about-pointer semantics.
- focused `cargo test -p fret-node --features compat-retained-canvas`
  coverage now also locks `wheel_zoom_zooms_about_pointer`,
  `wheel_zoom_emits_move_start_and_debounced_move_end`,
  `pinch_gesture_zooms_in_about_pointer`,
  `pinch_gesture_respects_toggle`,
  `pinch_zoom_emits_move_start_and_debounced_move_end`, and
  `wheel_pan_then_wheel_zoom_ends_pan_and_starts_zoom` after the wheel-zoom split.
- `pointer_wheel_pan.rs` now also routes scroll-pan gating,
  mode/platform delta resolution, and shared viewport pan application through the private
  `canvas/widget/pointer_wheel_pan/gate.rs`,
  `canvas/widget/pointer_wheel_pan/resolve.rs`, and
  `canvas/widget/pointer_wheel_pan/apply.rs` seams, so the root wheel-pan reducer now mainly keeps
  the retained facade entry point explicit without changing pan-on-scroll enablement, shift
  remapping, viewport move debounce, or state update semantics.
- focused `cargo test -p fret-node --features compat-retained-canvas`
  coverage now also locks `pan_on_scroll_mode_horizontal_ignores_vertical_wheel_delta`,
  `pan_on_scroll_shift_maps_vertical_wheel_to_horizontal_on_windows`,
  `space_enables_pan_on_scroll_even_when_pan_on_scroll_is_disabled`,
  `wheel_pan_emits_move_start_and_debounced_move_end`, and
  `wheel_pan_then_wheel_zoom_ends_pan_and_starts_zoom` after the wheel-pan split.
- `pointer_up_node_drag_parent.rs` now also routes drag-release
  parent-change collection, node-rect construction, and best-parent-group selection through the
  private `canvas/widget/pointer_up_node_drag_parent/collect.rs`,
  `canvas/widget/pointer_up_node_drag_parent/rect.rs`, and
  `canvas/widget/pointer_up_node_drag_parent/target.rs` seams, so the root node-drag-parent helper
  now mainly keeps the retained facade entry point explicit without changing release-time
  reparenting, group-override precedence, or smallest-containing-group selection semantics.
- focused `cargo test -p fret-node --features compat-retained-canvas`
  coverage now also locks `best_parent_group_prefers_smallest_containing_group`,
  `best_parent_group_uses_group_override_bounds`,
  `best_parent_group_returns_none_when_rect_is_not_fully_contained`,
  `group_drag_drives_canvas_derived_preview_and_edge_index`, and
  `group_drag_preview_cache_reuses_geometry_across_preview_rev_updates` after the node-drag-parent
  split.
- `focus_nav_ports_hints.rs` now also routes focused-port hint
  refresh preflight, connection-hint evaluation, and interaction-state writeback through the
  private `canvas/widget/focus_nav_ports_hints/preflight.rs`,
  `canvas/widget/focus_nav_ports_hints/evaluate.rs`, and
  `canvas/widget/focus_nav_ports_hints/apply.rs` seams, so the root focus-nav hint helper now
  mainly keeps the retained facade entry point explicit without changing focused-port clearing,
  connection-mode-aware validity checks, conversion fallback checks, or late state-apply guards.
- focused `cargo test -p fret-node --features compat-retained-canvas`
  coverage now also locks `focus_next_port_filters_by_wire_direction` and
  `activate_starts_and_commits_wire_drag` after the focus-nav-hints split.
- `focus_nav_ports_activation.rs` now also routes focused-port
  activation preflight, click-connect start arming, and click-connect commit/position sync through
  the private `canvas/widget/focus_nav_ports_activation/preflight.rs`,
  `canvas/widget/focus_nav_ports_activation/start.rs`, and
  `canvas/widget/focus_nav_ports_activation/commit.rs` seams, so the root focus-nav activation
  helper now mainly keeps the retained facade entry point explicit without changing activation
  gating, activation-point lookup, click-connect state reset, or forced-target commit semantics.
- focused `cargo test -p fret-node --features compat-retained-canvas`
  coverage now also locks `activate_starts_and_commits_wire_drag`,
  `click_connect_emits_connect_start_and_committed_end`, and
  `click_connect_target_port_click_commits_wire_and_clears_click_connect_state` after the
  focus-nav-activation split.
- `focus_nav_traversal_port.rs` now also routes traversal
  preflight, candidate-port collection, next-port selection, and final focus/apply through the
  private `canvas/widget/focus_nav_traversal_port/preflight.rs`,
  `canvas/widget/focus_nav_traversal_port/collect.rs`,
  `canvas/widget/focus_nav_traversal_port/select.rs`, and
  `canvas/widget/focus_nav_traversal_port/apply.rs` seams, so the root focus-nav traversal helper
  now mainly keeps the retained facade entry point explicit without changing focused-node fallback,
  wire-direction filtering, cycling order, or focused-port apply semantics.
- focused `cargo test -p fret-node --features compat-retained-canvas`
  coverage now also locks `focus_next_port_cycles_ports_within_focused_node` and
  `focus_next_port_filters_by_wire_direction` after the focus-nav-traversal split.
- `focus_nav_traversal_node.rs` now also routes traversal
  snapshot preflight, selectable-node ordering, next-node selection, and final focus/apply +
  optional auto-pan through the private
  `canvas/widget/focus_nav_traversal_node/preflight.rs`,
  `canvas/widget/focus_nav_traversal_node/collect.rs`,
  `canvas/widget/focus_nav_traversal_node/select.rs`, and
  `canvas/widget/focus_nav_traversal_node/apply.rs` seams, so the root node-traversal helper now
  mainly keeps the retained facade entry point explicit without changing draw-order-first
  traversal, unselectable-node skipping, current-node fallback, or focus-driven auto-pan
  semantics.
- focused `cargo test -p fret-node --features compat-retained-canvas`
  coverage now also locks `focus_next_cycles_nodes_and_updates_selection`,
  `focus_next_skips_unselectable_nodes`, and
  `focus_next_can_pan_viewport_when_auto_pan_on_node_focus_is_enabled` after the
  focus-nav-node-traversal split.
- `focus_port_direction_candidate.rs` now also routes
  directional candidate center projection and best-candidate search through the private
  `canvas/widget/focus_port_direction_candidate/center.rs` and
  `canvas/widget/focus_port_direction_candidate/search.rs` seams, so the root directional-port
  candidate helper now mainly keeps the retained facade entry points explicit without changing
  required-direction filtering, rank-based comparison, or neighbor-port selection semantics.
- focused `cargo test -p fret-node --features compat-retained-canvas`
  coverage now also locks `focus_port_right_moves_to_neighbor_node`,
  `focus_port_left_moves_back`, and
  `better_directional_rank_prefers_smaller_angle_then_parallel_then_distance` after the
  direction-candidate split.
- `focus_nav_ports_center.rs` now also routes focused-port center
  lookup and activation-point fallback resolution through the private
  `canvas/widget/focus_nav_ports_center/port.rs` and
  `canvas/widget/focus_nav_ports_center/activation.rs` seams, so the root focused-port center
  helper now mainly keeps the retained facade entry points explicit without changing port-center
  preference, last-pointer fallback, or bounds-center fallback semantics.
- focused `cargo test -p fret-node --features compat-retained-canvas`
  coverage now also locks `resolve_activation_point_prefers_port_center`,
  `resolve_activation_point_falls_back_to_last_pos`,
  `resolve_activation_point_falls_back_to_bounds_center`,
  `activate_starts_and_commits_wire_drag`, and
  `focus_port_right_moves_to_neighbor_node` after the focus-nav-center split.
- `move_ops/align_distribute/support.rs` now also routes support
  types, element collection, per-mode delta planning, extent-shift computation, and group/node op
  application through the private `canvas/widget/move_ops/align_distribute/support/types.rs`,
  `canvas/widget/move_ops/align_distribute/support/collect.rs`,
  `canvas/widget/move_ops/align_distribute/support/delta.rs`,
  `canvas/widget/move_ops/align_distribute/support/shift.rs`, and
  `canvas/widget/move_ops/align_distribute/support/append.rs` seams, so the root support file now
  mainly keeps the retained re-export surface explicit without changing align/distribute planning,
  extent clamping, or graph-op emission semantics.
- focused `cargo test -p fret-node --features compat-retained-canvas`
  coverage now also locks `align_left_moves_selected_nodes_and_records_history_entry`,
  `align_center_x_preserves_alignment_under_node_extent_bounds`,
  `distribute_x_clamps_selected_group_children_to_node_extent_rect_like_xyflow`, and
  `distribute_x_clamps_selected_group_children_to_node_extent_rect_from_node_extents` after the
  align-distribute-support split.
- `view_state/viewport.rs` now also routes viewport request
  application, view-state normalization + callback emission, and focused-point visibility
  adjustment through the private `canvas/widget/view_state/viewport/set.rs`,
  `canvas/widget/view_state/viewport/update.rs`, and
  `canvas/widget/view_state/viewport/visible.rs` seams, so the root viewport file now mainly keeps
  the retained private module split explicit without changing viewport clamping, view-callback
  emission, or focus-driven auto-pan semantics.
- focused `cargo test -p fret-node --features compat-retained-canvas`
  coverage now also locks `set_viewport_via_view_queue_updates_pan_and_zoom`,
  `set_viewport_via_view_queue_clamps_zoom_to_style_limits`,
  `set_viewport_clamps_pan_to_translate_extent`,
  `translate_extent_centers_when_viewport_is_larger_than_extent`, and
  `focus_next_can_pan_viewport_when_auto_pan_on_node_focus_is_enabled` after the viewport split.
- `press_session/prepare/surface.rs` now also routes
  surface-session clear sequences, focus-session clearing, and local regression fixtures through
  the private `canvas/widget/press_session/prepare/surface/clear.rs`,
  `canvas/widget/press_session/prepare/surface/focus.rs`, and
  `canvas/widget/press_session/prepare/surface/tests.rs` seams, while
  `press_session/prepare/target.rs` now also routes target-hit clear sequences, focus-session
  clearing, and local regression fixtures through the private
  `canvas/widget/press_session/prepare/target/clear.rs`,
  `canvas/widget/press_session/prepare/target/focus.rs`, and
  `canvas/widget/press_session/prepare/target/tests.rs` seams, with shared fixture state in
  `canvas/widget/press_session/prepare/test_support.rs`, so both root prepare files now mainly keep
  the retained re-export surfaces explicit without changing press-session clearing or
  hover/focus-reset semantics.
- focused `cargo test -p fret-node --features compat-retained-canvas`
  coverage now also locks `prepare_for_pan_begin_preserves_edge_insert_sessions`,
  `prepare_for_background_interaction_clears_all_surface_pointer_sessions`,
  `prepare_for_port_hit_preserves_node_resize_but_clears_competing_pointer_sessions`, and
  `prepare_for_edge_anchor_hit_clears_hover_edge_and_node_resize` after the press-session-prepare
  split.
- `wire_drag/move_update/hover.rs` now also routes
  source-port extraction, hover hit-picking, hover validity/diagnostic evaluation, and conversion
  probing through the private `canvas/widget/wire_drag/move_update/hover/source.rs`,
  `canvas/widget/wire_drag/move_update/hover/pick.rs`,
  `canvas/widget/wire_drag/move_update/hover/validity.rs`, and
  `canvas/widget/wire_drag/move_update/hover/convertible.rs` seams, so the root hover file now
  mainly keeps the retained façade entry points explicit without changing wire-hover hit
  resolution, invalid-connection diagnostics, or conversion probing semantics.
- focused `cargo test -p fret-node --features compat-retained-canvas`
  coverage now also locks `wire_drag_hover_tracks_invalid_port_in_strict_mode`,
  `wire_drag_hover_tracks_non_connectable_end_port_as_invalid`, and
  `wire_drag_hover_marks_valid_target_port_as_valid` after the wire-drag-hover split.
- `selection/box_edges.rs` now also routes box-select mode
  resolution, graph fallback edge collection, store-backed connection harvesting, and local
  regression fixtures through the private `canvas/widget/selection/box_edges/mode.rs`,
  `canvas/widget/selection/box_edges/graph.rs`,
  `canvas/widget/selection/box_edges/store.rs`,
  `canvas/widget/selection/box_edges/test_support.rs`, and
  `canvas/widget/selection/box_edges/tests.rs` seams, so the root box-edges file now mainly keeps
  the retained facade module surface explicit without changing box-select edge eligibility or
  store-fallback semantics.
- `cargo check -p fret-node --features compat-retained-canvas --tests`,
  `cargo fmt -p fret-node --check`, and focused
  `cargo test -p fret-node --features compat-retained-canvas collect_box_select_edges_`
  coverage now also locks
  `collect_box_select_edges_connected_selects_any_connected_endpoint`,
  `collect_box_select_edges_both_endpoints_requires_both_nodes_selected`, and
  `collect_box_select_edges_respects_global_selection_gates` after the selection-box-edges split.
- `interaction_policy/port.rs` now also routes
  port connectability predicates, bundle-candidate filtering, and local regression fixtures through
  the private `canvas/widget/interaction_policy/port/connectable.rs`,
  `canvas/widget/interaction_policy/port/bundle.rs`,
  `canvas/widget/interaction_policy/port/test_support.rs`, and
  `canvas/widget/interaction_policy/port/tests.rs` seams, so the root port-policy file now mainly
  keeps the retained facade module surface explicit without changing port-connectable or
  bundle-dedup semantics.
- `cargo check -p fret-node --features compat-retained-canvas --tests`,
  `cargo fmt -p fret-node --check`, focused
  `cargo test -p fret-node --features compat-retained-canvas
  port_connectable_helpers_respect_node_and_port_overrides`, and focused
  `cargo test -p fret-node --features compat-retained-canvas
  should_add_bundle_port_requires_unique_same_direction_candidate` now also lock the
  interaction-policy-port split.
- `wire_math.rs` now also routes route-kind distance/closest-point
  helpers and local regression tests through the private `canvas/widget/wire_math/route.rs` and
  `canvas/widget/wire_math/tests.rs` seams, while keeping the root wire-math file as the retained
  facade over existing `path/segment/step` helpers without changing bezier/step/path geometry
  semantics.
- `cargo check -p fret-node --features compat-retained-canvas --tests`,
  `cargo fmt -p fret-node --check`, focused
  `cargo test -p fret-node --features compat-retained-canvas path_distance2_on_line_is_zeroish`,
  focused `cargo test -p fret-node --features compat-retained-canvas
  path_midpoint_and_normal_is_finite`, and focused
  `cargo test -p fret-node --features compat-retained-canvas
  path_start_end_tangents_follow_control_points` now also lock the wire-math split.
- `delete_ops_builder.rs` now also routes delete-op
  orchestration and shared regression fixtures through the private
  `canvas/widget/delete_ops_builder/assemble.rs`,
  `canvas/widget/delete_ops_builder/test_support.rs`, and
  `canvas/widget/delete_ops_builder/tests.rs` seams, while existing edge/group/node helpers remain
  in place, so the root delete-ops builder file now mainly keeps the retained facade surface
  explicit without changing delete ordering or edge-dedup semantics.
- `cargo check -p fret-node --features compat-retained-canvas --tests`,
  `cargo fmt -p fret-node --check`, focused
  `cargo test -p fret-node --features compat-retained-canvas
  delete_selection_ops_does_not_double_remove_edges_already_owned_by_removed_nodes`, and focused
  `cargo test -p fret-node --features compat-retained-canvas
  collect_node_edges_deduplicates_edges_already_marked_removed` now also lock the
  delete-ops-builder split.
- `paint_grid_tiles.rs` now also routes tile-op assembly
  and local regression coverage through the private `canvas/widget/paint_grid_tiles/ops.rs` and
  `canvas/widget/paint_grid_tiles/tests.rs` seams, while keeping `GridTileSpec` rooted in the
  existing support module, so the root paint-grid-tiles file now mainly keeps the retained facade
  surface explicit without changing line/dot/cross tile-op semantics.
- `cargo check -p fret-node --features compat-retained-canvas --tests`,
  `cargo fmt -p fret-node --check`, focused
  `cargo test -p fret-node --features compat-retained-canvas dots_pattern_emits_rounded_quads`,
  and focused `cargo test -p fret-node --features compat-retained-canvas
  cross_pattern_emits_axis_aligned_segments` now also lock the paint-grid-tiles split.
- `context_menu/key_navigation.rs` now also routes local
  regression fixtures and root test coverage through the private
  `canvas/widget/context_menu/key_navigation/test_support.rs` and
  `canvas/widget/context_menu/key_navigation/tests.rs` seams, so the root key-navigation file now
  mainly keeps the retained facade entry points explicit without changing context-menu active item,
  hover, or typeahead semantics.
- `cargo check -p fret-node --features compat-retained-canvas --tests`,
  `cargo fmt -p fret-node --check`, focused
  `cargo test -p fret-node --features compat-retained-canvas advance_active_item_skips_disabled_entries`,
  and focused `cargo test -p fret-node --features compat-retained-canvas
  sync_hovered_item_promotes_enabled_item_and_clears_typeahead` now also lock the
  context-menu-key-navigation split.
- `group_draw_order.rs` now also routes shared regression
  fixtures and root test coverage through the private
  `canvas/widget/group_draw_order/test_support.rs` and
  `canvas/widget/group_draw_order/tests.rs` seams, so the root group-draw-order file now mainly
  keeps the retained facade entry points explicit without changing selected-group ordering
  semantics.
- `cargo check -p fret-node --features compat-retained-canvas --tests`,
  `cargo fmt -p fret-node --check`, focused
  `cargo test -p fret-node --features compat-retained-canvas
  bring_to_front_preserves_existing_draw_order_for_selected_groups`, and focused
  `cargo test -p fret-node --features compat-retained-canvas
  send_to_back_preserves_existing_draw_order_for_selected_groups` now also lock the
  group-draw-order split.
- `pending_resize_session.rs` now also routes shared
  pending-resize regression fixtures and root test coverage through the private
  `canvas/widget/pending_resize_session/test_support.rs` and
  `canvas/widget/pending_resize_session/tests.rs` seams, so the root pending-resize-session file
  now mainly keeps the retained facade entry points explicit without changing pending→active resize
  activation semantics.
- `cargo check -p fret-node --features compat-retained-canvas --tests`,
  `cargo fmt -p fret-node --check`, focused
  `cargo test -p fret-node --features compat-retained-canvas
  activate_pending_group_resize_moves_pending_into_active`, and focused
  `cargo test -p fret-node --features compat-retained-canvas
  activate_pending_node_resize_moves_pending_into_active` now also lock the
  pending-resize-session split.
- `context_menu/target_selection.rs` now also routes shared
  regression fixtures and root test coverage through the private
  `canvas/widget/context_menu/target_selection/test_support.rs` and
  `canvas/widget/context_menu/target_selection/tests.rs` seams, so the root target-selection file
  now mainly keeps the retained facade entry points explicit without changing edge/group
  context-target selection semantics.
- `cargo check -p fret-node --features compat-retained-canvas --tests`,
  `cargo fmt -p fret-node --check`, focused
  `cargo test -p fret-node --features compat-retained-canvas
  select_group_context_target_clears_node_and_edge_selection`, and focused
  `cargo test -p fret-node --features compat-retained-canvas
  select_edge_context_target_clears_node_and_group_selection` now also lock the
  context-menu-target-selection split.
- `context_menu/background_execution.rs` now also routes shared
  background-insert regression fixtures and root test coverage through the private
  `canvas/widget/context_menu/background_execution/test_support.rs` and
  `canvas/widget/context_menu/background_execution/tests.rs` seams, so the root
  background-execution file now mainly keeps the insert plan enum plus retained execution facade
  entry points explicit without changing background insert planning or rejection semantics.
- `cargo check -p fret-node --features compat-retained-canvas --tests`,
  `cargo fmt -p fret-node --check`, and focused
  `cargo test -p fret-node --features compat-retained-canvas
  background_insert_menu_plan_surfaces_create_node_errors` now also lock the
  context-menu-background-execution test seam split.
- `context_menu/connection_execution.rs` now also routes shared
  connection-insert / conversion regression fixtures and root test coverage through the private
  `canvas/widget/context_menu/connection_execution/test_support.rs` and
  `canvas/widget/context_menu/connection_execution/tests.rs` seams, so the root
  connection-execution file now mainly keeps the insert/conversion plan enums explicit without
  changing connection insert or conversion rejection semantics.
- `cargo check -p fret-node --features compat-retained-canvas --tests`,
  `cargo fmt -p fret-node --check`, focused
  `cargo test -p fret-node --features compat-retained-canvas
  connection_insert_menu_plan_surfaces_create_node_errors`, and focused
  `cargo test -p fret-node --features compat-retained-canvas
  connection_conversion_menu_plan_rejects_missing_template` now also lock the
  context-menu-connection-execution test seam split.
- `context_menu/item_builders.rs` now also routes root test coverage
  through the private `canvas/widget/context_menu/item_builders/tests.rs` seam, so the root
  item-builders file now mainly keeps the retained context-menu builder facade explicit without
  changing background/group/edge menu item ordering or command wiring semantics.
- `cargo check -p fret-node --features compat-retained-canvas --tests`,
  `cargo fmt -p fret-node --check`, and focused
  `cargo test -p fret-node --features compat-retained-canvas
  context_menu::item_builders::tests` now also locks the context-menu-item-builders test seam
  split.
- `context_menu/selection_activation/payload.rs` now also routes shared
  activation payload regression fixtures and local test coverage through the private
  `canvas/widget/context_menu/selection_activation/payload/test_support.rs` and
  `canvas/widget/context_menu/selection_activation/payload/tests.rs` seams, so the payload module
  now mainly keeps activation payload extraction explicit without changing enabled-item or
  out-of-range index handling semantics.
- `cargo check -p fret-node --features compat-retained-canvas --tests`,
  `cargo fmt -p fret-node --check`, and focused
  `cargo test -p fret-node --features compat-retained-canvas
  selection_activation::payload::tests` now also locks the selection-activation payload test seam
  split.
- `command_router_align.rs` and `command_router_nudge.rs` now also route
  local command mapping test coverage through the private
  `canvas/widget/command_router_align/tests.rs` and
  `canvas/widget/command_router_nudge/tests.rs` seams, so the root command-router helper files now
  mainly keep align/distribute and nudge request mapping explicit without changing command dispatch
  semantics.
- `cargo check -p fret-node --features compat-retained-canvas --tests`,
  `cargo fmt -p fret-node --check`, focused
  `cargo test -p fret-node --features compat-retained-canvas command_router_align::tests`, and
  focused `cargo test -p fret-node --features compat-retained-canvas command_router_nudge::tests`
  now also lock the command-router mapping test seam split.
- `command_view/frame.rs` and `command_view/zoom.rs` now also route local
  helper test coverage through the private `canvas/widget/command_view/frame/tests.rs` and
  `canvas/widget/command_view/zoom/tests.rs` seams, so the command-view helper files now mainly
  keep framing node-id collection plus reset/zoom viewport helper logic explicit without changing
  view command semantics.
- `cargo check -p fret-node --features compat-retained-canvas --tests`,
  `cargo fmt -p fret-node --check`, focused
  `cargo test -p fret-node --features compat-retained-canvas command_view::frame::tests`, and
  focused `cargo test -p fret-node --features compat-retained-canvas command_view::zoom::tests`
  now also lock the command-view helper test seam split.
- `command_open_insert/fallback.rs` and `command_router/dispatch.rs`
  now also route local fallback / direct-command route test coverage through the private
  `canvas/widget/command_open_insert/fallback/tests.rs` and
  `canvas/widget/command_router/dispatch/tests.rs` seams, so the helper files now mainly keep
  insert-picker fallback projection plus direct command route lookup explicit without changing
  command-open-insert or command-router behavior.
- `cargo check -p fret-node --features compat-retained-canvas --tests`,
  `cargo fmt -p fret-node --check`, focused
  `cargo test -p fret-node --features compat-retained-canvas command_open_insert::fallback::tests`,
  and focused
  `cargo test -p fret-node --features compat-retained-canvas command_router::dispatch::tests` now
  also lock the command-open-insert / command-router test seam split.
- `cursor_gate.rs`, `cursor_resolve/edge.rs`, and `cursor_resolve/resize.rs`
  now also route local cursor helper test coverage through the private
  `canvas/widget/cursor_gate/tests.rs`,
  `canvas/widget/cursor_resolve/edge/tests.rs`, and
  `canvas/widget/cursor_resolve/resize/tests.rs` seams, so the helper files now mainly keep cursor
  gating plus anchor/resize cursor resolution rules explicit without changing runtime cursor
  semantics.
- `cargo check -p fret-node --features compat-retained-canvas --tests`,
  `cargo fmt -p fret-node --check`, focused
  `cargo test -p fret-node --features compat-retained-canvas cursor_gate::tests`, focused
  `cargo test -p fret-node --features compat-retained-canvas cursor_resolve::edge::tests`, and
  focused `cargo test -p fret-node --features compat-retained-canvas
  cursor_resolve::resize::tests` now also lock the cursor helper test seam split.
- `hover/state.rs` and `hover/target.rs` now also route local hover helper
  test coverage through the private `canvas/widget/hover/state/tests.rs` and
  `canvas/widget/hover/target/tests.rs` seams, so the hover helper files now mainly keep hover
  state synchronization plus anchor target resolution explicit without changing hover behavior.
- `cargo check -p fret-node --features compat-retained-canvas --tests`,
  `cargo fmt -p fret-node --check`, focused
  `cargo test -p fret-node --features compat-retained-canvas hover::state::tests`, and focused
  `cargo test -p fret-node --features compat-retained-canvas hover::target::tests` now also lock
  the hover helper test seam split.
- `insert_candidates/menu.rs` and `insert_candidates/reroute.rs` now also
  route local insert-candidate helper test coverage through the private
  `canvas/widget/insert_candidates/menu/tests.rs` and
  `canvas/widget/insert_candidates/reroute/tests.rs` seams, so the helper files now mainly keep
  insert-candidate menu projection plus reroute prepending explicit without changing
  candidate-list semantics.
- `cargo check -p fret-node --features compat-retained-canvas --tests`,
  `cargo fmt -p fret-node --check`, focused
  `cargo test -p fret-node --features compat-retained-canvas insert_candidates::menu::tests`, and
  focused `cargo test -p fret-node --features compat-retained-canvas
  insert_candidates::reroute::tests` now also lock the insert-candidates helper test seam split.
- `overlay_hit/context_menu.rs` and `overlay_hit/searcher.rs` now also
  route local overlay-hit helper test coverage through the private
  `canvas/widget/overlay_hit/context_menu/tests.rs` and
  `canvas/widget/overlay_hit/searcher/tests.rs` seams, so the helper files now mainly keep overlay
  rect sizing plus pointer-hit row/item mapping explicit without changing overlay-hit semantics.
- `cargo check -p fret-node --features compat-retained-canvas --tests`,
  `cargo fmt -p fret-node --check`, focused
  `cargo test -p fret-node --features compat-retained-canvas overlay_hit::context_menu::tests`,
  and focused
  `cargo test -p fret-node --features compat-retained-canvas overlay_hit::searcher::tests` now
  also lock the overlay-hit helper test seam split.
- `interaction_gate/cache.rs`, `interaction_gate/detail.rs`,
  `interaction_gate/hover.rs`, and `interaction_gate/motion.rs` now also route local interaction
  gate test coverage through the private
  `canvas/widget/interaction_gate/cache/tests.rs`,
  `canvas/widget/interaction_gate/detail/tests.rs`,
  `canvas/widget/interaction_gate/hover/tests.rs`, and
  `canvas/widget/interaction_gate/motion/tests.rs` seams, so the helper files now mainly keep
  cache/detail/hover/motion gating rules explicit without changing runtime interaction behavior.
- `cargo check -p fret-node --features compat-retained-canvas --tests`,
  `cargo fmt -p fret-node --check`, focused
  `cargo test -p fret-node --features compat-retained-canvas interaction_gate::cache::tests`,
  focused `cargo test -p fret-node --features compat-retained-canvas
  interaction_gate::detail::tests`, focused `cargo test -p fret-node --features
  compat-retained-canvas interaction_gate::hover::tests`, and focused `cargo test -p fret-node
  --features compat-retained-canvas interaction_gate::motion::tests` now also lock the
  interaction-gate helper test seam split.
- `keyboard_shortcuts_gate/editing.rs`,
  `keyboard_shortcuts_gate/modifier.rs`, `keyboard_shortcuts_gate/navigation.rs`,
  `keyboard_shortcuts_map/modifier.rs`, and `keyboard_shortcuts_map/navigation.rs` now also route
  local shortcut gate/map test coverage through the private
  `canvas/widget/keyboard_shortcuts_gate/editing/tests.rs`,
  `canvas/widget/keyboard_shortcuts_gate/modifier/tests.rs`,
  `canvas/widget/keyboard_shortcuts_gate/navigation/tests.rs`,
  `canvas/widget/keyboard_shortcuts_map/modifier/tests.rs`, and
  `canvas/widget/keyboard_shortcuts_map/navigation/tests.rs` seams, so the helper files now mainly
  keep shortcut gating and command mapping explicit without changing keyboard shortcut semantics.
- `cargo check -p fret-node --features compat-retained-canvas --tests`,
  `cargo fmt -p fret-node --check`, focused
  `cargo test -p fret-node --features compat-retained-canvas
  keyboard_shortcuts_gate::editing::tests`, focused `cargo test -p fret-node --features
  compat-retained-canvas keyboard_shortcuts_gate::modifier::tests`, focused
  `cargo test -p fret-node --features compat-retained-canvas
  keyboard_shortcuts_gate::navigation::tests`, focused `cargo test -p fret-node --features
  compat-retained-canvas keyboard_shortcuts_map::modifier::tests`, and focused `cargo test -p
  fret-node --features compat-retained-canvas keyboard_shortcuts_map::navigation::tests` now also
  lock the keyboard-shortcuts helper test seam split.
- `focus_session/focus.rs`, `focus_session/selection.rs`,
  `focus_session/hints.rs`, `view_math_viewport/snap.rs`, and
  `view_math_viewport/viewport.rs` now also route local focus/view helper test coverage through the
  private `canvas/widget/focus_session/focus/tests.rs`,
  `canvas/widget/focus_session/selection/tests.rs`,
  `canvas/widget/focus_session/hints/tests.rs`,
  `canvas/widget/view_math_viewport/snap/tests.rs`, and
  `canvas/widget/view_math_viewport/viewport/tests.rs` seams, so the helper files now mainly keep
  focus-state mutation and viewport math explicit without changing runtime semantics.
- `cargo check -p fret-node --features compat-retained-canvas --tests`,
  `cargo fmt -p fret-node --check`, focused
  `cargo test -p fret-node --features compat-retained-canvas focus_session::focus::tests`,
  focused `cargo test -p fret-node --features compat-retained-canvas
  focus_session::selection::tests`, focused `cargo test -p fret-node --features
  compat-retained-canvas focus_session::hints::tests`, focused `cargo test -p fret-node
  --features compat-retained-canvas view_math_viewport::snap::tests`, and focused `cargo test -p
  fret-node --features compat-retained-canvas view_math_viewport::viewport::tests` now also lock
  the focus/view helper test seam split.
- `left_click/edge_selection.rs`, `left_click/node_selection.rs`,
  `left_click/connection_hits/port/kind.rs`, and
  `left_click/connection_hits/port/click_connect.rs` now also route local left-click helper test
  coverage through the private `canvas/widget/left_click/edge_selection/tests.rs`,
  `canvas/widget/left_click/node_selection/tests.rs`,
  `canvas/widget/left_click/connection_hits/port/kind/tests.rs`, and
  `canvas/widget/left_click/connection_hits/port/click_connect/tests.rs` seams, so these helper
  files now mainly keep edge/node selection plus port-hit wire-drag/click-connect logic explicit
  without changing left-click runtime semantics.
- `cargo check -p fret-node --features compat-retained-canvas --tests`,
  `cargo fmt -p fret-node --check`, focused
  `cargo test -p fret-node --features compat-retained-canvas left_click::edge_selection::tests`,
  focused `cargo test -p fret-node --features compat-retained-canvas
  left_click::node_selection::tests`, focused `cargo test -p fret-node --features
  compat-retained-canvas left_click::connection_hits::port::kind::tests`, and focused `cargo test
  -p fret-node --features compat-retained-canvas
  left_click::connection_hits::port::click_connect::tests` now also lock the first left-click
  helper test seam split.
- `left_click/element_hits/edge/drag.rs`,
  `left_click/element_hits/node/drag.rs`, and `left_click/element_hits/resize/state.rs` now also
  route local left-click element-hit helper test coverage through the private
  `canvas/widget/left_click/element_hits/edge/drag/tests.rs`,
  `canvas/widget/left_click/element_hits/node/drag/tests.rs`, and
  `canvas/widget/left_click/element_hits/resize/state/tests.rs` seams, so these helper files now
  mainly keep edge/node-drag arming and resize-start sizing explicit without changing element-hit
  runtime semantics.
- `cargo check -p fret-node --features compat-retained-canvas --tests`,
  `cargo fmt -p fret-node --check`, focused
  `cargo test -p fret-node --features compat-retained-canvas
  left_click::element_hits::edge::drag::tests`, focused `cargo test -p fret-node --features
  compat-retained-canvas left_click::element_hits::node::drag::tests`, and focused `cargo test -p
  fret-node --features compat-retained-canvas left_click::element_hits::resize::state::tests` now
  also lock the second left-click helper test seam split.
- `graph_construction/node.rs` and `graph_construction/group.rs` now also route local
  graph-construction helper test coverage through the private
  `canvas/widget/graph_construction/node/tests.rs` and
  `canvas/widget/graph_construction/group/tests.rs` seams, so these helper files now mainly keep
  reroute-create op assembly plus centered group creation/selection reducers explicit without
  changing retained graph-construction runtime semantics.
- `cargo check -p fret-node --features compat-retained-canvas --tests`,
  `cargo fmt -p fret-node --check`, focused
  `cargo test -p fret-node --features compat-retained-canvas graph_construction::node::tests`, and
  focused `cargo test -p fret-node --features compat-retained-canvas
  graph_construction::group::tests` now also lock the graph-construction helper test seam split.
- `pending_connection_session/edge.rs` and `pending_connection_session/wire.rs` now also route
  local pending-connection helper test coverage through the private
  `canvas/widget/pending_connection_session/edge/tests.rs` and
  `canvas/widget/pending_connection_session/wire/tests.rs` seams, so these helper files now mainly
  keep pending edge-insert activation plus pending wire-drag promotion explicit without changing
  retained connection-session runtime semantics.
- `cargo check -p fret-node --features compat-retained-canvas --tests`,
  `cargo fmt -p fret-node --check`, focused
  `cargo test -p fret-node --features compat-retained-canvas pending_connection_session::edge::tests`,
  and focused `cargo test -p fret-node --features compat-retained-canvas
  pending_connection_session::wire::tests` now also lock the pending-connection helper test seam
  split.
- `paint_overlay_toast/layout.rs` and `paint_overlay_toast/style.rs` now also route local
  toast-overlay helper test coverage through the private
  `canvas/widget/paint_overlay_toast/layout/tests.rs` and
  `canvas/widget/paint_overlay_toast/style/tests.rs` seams, so these helper files now mainly keep
  toast rect layout math plus severity border-color mapping explicit without changing retained
  toast-overlay runtime semantics.
- `cargo check -p fret-node --features compat-retained-canvas --tests`,
  `cargo fmt -p fret-node --check`, focused
  `cargo test -p fret-node --features compat-retained-canvas paint_overlay_toast::layout::tests`,
  and focused `cargo test -p fret-node --features compat-retained-canvas
  paint_overlay_toast::style::tests` now also lock the toast-overlay helper test seam split.
- `paint_overlay_wire_hint/message.rs` and `paint_overlay_wire_hint/style.rs` now also route local
  wire-hint helper test coverage through the private
  `canvas/widget/paint_overlay_wire_hint/message/tests.rs` and
  `canvas/widget/paint_overlay_wire_hint/style/tests.rs` seams, so these helper files now mainly
  keep invalid-hover/bundle/yank hint text plus diagnostic border-color resolution explicit
  without changing retained wire-hint runtime semantics.
- `cargo check -p fret-node --features compat-retained-canvas --tests`,
  `cargo fmt -p fret-node --check`, focused
  `cargo test -p fret-node --features compat-retained-canvas paint_overlay_wire_hint::message::tests`,
  and focused `cargo test -p fret-node --features compat-retained-canvas
  paint_overlay_wire_hint::style::tests` now also lock the wire-hint helper test seam split.
- `pointer_up_pending/click_select.rs`, `pointer_up_pending/wire_drag.rs`,
  `pointer_up_session/cleanup.rs`, and `pointer_up_session/release.rs` now also route local
  pointer-up helper test coverage through the private
  `canvas/widget/pointer_up_pending/click_select/tests.rs`,
  `canvas/widget/pointer_up_pending/wire_drag/tests.rs`,
  `canvas/widget/pointer_up_session/cleanup/tests.rs`, and
  `canvas/widget/pointer_up_session/release/tests.rs` seams, so these helper files now mainly keep
  click-release thresholding, pending wire promotion gating, snap-guide cleanup, and active
  release clearing explicit without changing retained pointer-up runtime semantics.
- `cargo check -p fret-node --features compat-retained-canvas --tests`,
  `cargo fmt -p fret-node --check`, focused
  `cargo test -p fret-node --features compat-retained-canvas pointer_up_pending::click_select::tests`,
  focused `cargo test -p fret-node --features compat-retained-canvas
  pointer_up_pending::wire_drag::tests`, focused `cargo test -p fret-node --features
  compat-retained-canvas pointer_up_session::cleanup::tests`, and focused `cargo test -p
  fret-node --features compat-retained-canvas pointer_up_session::release::tests` now also lock
  the pointer-up helper test seam split.
- `pointer_up_commit_resize/group.rs`, `pointer_up_commit_resize/node.rs`,
  `reconnect/edges.rs`, and `reconnect/flags.rs` now also route local resize/reconnect helper test
  coverage through the private `canvas/widget/pointer_up_commit_resize/group/tests.rs`,
  `canvas/widget/pointer_up_commit_resize/node/tests.rs`,
  `canvas/widget/reconnect/edges/tests.rs`, and `canvas/widget/reconnect/flags/tests.rs` seams, so
  these helper files now mainly keep group and node resize op assembly plus reconnect
  yank/reconnectable-flag resolution explicit without changing retained pointer-up or reconnect
  runtime semantics.
- `cargo check -p fret-node --features compat-retained-canvas --tests`,
  `cargo fmt -p fret-node --check`, focused
  `cargo test -p fret-node --features compat-retained-canvas pointer_up_commit_resize::group::tests`,
  focused `cargo test -p fret-node --features compat-retained-canvas
  pointer_up_commit_resize::node::tests`, focused `cargo test -p fret-node --features
  compat-retained-canvas reconnect::edges::tests`, and focused `cargo test -p fret-node
  --features compat-retained-canvas reconnect::flags::tests` now also lock the
  resize/reconnect helper test seam split.
- `rect_math_core.rs`, `rect_math_path.rs`, `threshold.rs`, and `right_click/threshold.rs` now
  also route local math/threshold helper test coverage through the private
  `canvas/widget/rect_math_core/tests.rs`, `canvas/widget/rect_math_path/tests.rs`,
  `canvas/widget/threshold/tests.rs`, and `canvas/widget/right_click/threshold/tests.rs` seams, so
  these helper files now mainly keep rect extents/intersection math, path bounds derivation,
  generic drag-threshold checks, and pending right-click threshold checks explicit without changing
  retained input/runtime semantics.
- `cargo check -p fret-node --features compat-retained-canvas --tests`,
  `cargo fmt -p fret-node --check`, focused
  `cargo test -p fret-node --features compat-retained-canvas rect_math_core::tests`, focused
  `cargo test -p fret-node --features compat-retained-canvas rect_math_path::tests`, focused
  `cargo test -p fret-node --features compat-retained-canvas threshold::tests`, and focused
  `cargo test -p fret-node --features compat-retained-canvas right_click::threshold::tests` now
  also lock the math/threshold helper test seam split.
- `pointer_up_commit_group_drag.rs`, `pointer_up_left_route/double_click.rs`,
  `pointer_up_node_drag_parent/target.rs`, and `pointer_up_state/sync.rs` now also route local
  pointer-up/drag helper test coverage through the private
  `canvas/widget/pointer_up_commit_group_drag/tests.rs`,
  `canvas/widget/pointer_up_left_route/double_click/tests.rs`,
  `canvas/widget/pointer_up_node_drag_parent/target/tests.rs`, and
  `canvas/widget/pointer_up_state/sync/tests.rs` seams, so these helper files now mainly keep
  group-drag op assembly, edge-insert double-click gating, parent-group targeting, and pointer-up
  state sync math explicit without changing retained pointer-up runtime semantics.
- `cargo check -p fret-node --features compat-retained-canvas --tests`,
  `cargo fmt -p fret-node --check`, focused
  `cargo test -p fret-node --features compat-retained-canvas pointer_up_commit_group_drag::tests`,
  focused `cargo test -p fret-node --features compat-retained-canvas
  pointer_up_left_route::double_click::tests`, focused `cargo test -p fret-node --features
  compat-retained-canvas pointer_up_node_drag_parent::target::tests`, and focused `cargo test -p
  fret-node --features compat-retained-canvas pointer_up_state::sync::tests` now also lock the
  pointer-up/drag helper test seam split.
- `searcher_activation_hit/candidate.rs`, `searcher_activation_state/clear.rs`,
  `searcher_input_nav/step.rs`, and `searcher_input_query/query.rs` now also route local searcher
  helper test coverage through the private
  `canvas/widget/searcher_activation_hit/candidate/tests.rs`,
  `canvas/widget/searcher_activation_state/clear/tests.rs`,
  `canvas/widget/searcher_input_nav/step/tests.rs`, and
  `canvas/widget/searcher_input_query/query/tests.rs` seams, so these helper files now mainly keep
  searcher row-to-candidate resolution, overlay clearing, active-row stepping, and query key
  filtering explicit without changing retained searcher runtime semantics.
- `cargo check -p fret-node --features compat-retained-canvas --tests`,
  `cargo fmt -p fret-node --check`, focused
  `cargo test -p fret-node --features compat-retained-canvas
  searcher_activation_hit::candidate::tests`, focused `cargo test -p fret-node --features
  compat-retained-canvas searcher_activation_state::clear::tests`, focused `cargo test -p
  fret-node --features compat-retained-canvas searcher_input_nav::step::tests`, and focused
  `cargo test -p fret-node --features compat-retained-canvas searcher_input_query::query::tests`
  now also lock the searcher helper test seam split.
- `searcher_pointer_hover/state.rs`, `searcher_pointer_wheel/delta.rs`,
  `searcher_row_activation/item.rs`, and `selection/selectable.rs` now also route local
  searcher/selection helper test coverage through the private
  `canvas/widget/searcher_pointer_hover/state/tests.rs`,
  `canvas/widget/searcher_pointer_wheel/delta/tests.rs`,
  `canvas/widget/searcher_row_activation/item/tests.rs`, and
  `canvas/widget/selection/selectable/tests.rs` seams, so these helper files now mainly keep
  hovered-row promotion, wheel-scroll clamping, row activation item synthesis, and selectable
  guard checks explicit without changing retained searcher or selection runtime semantics.
- `cargo check -p fret-node --features compat-retained-canvas --tests`,
  `cargo fmt -p fret-node --check`, focused
  `cargo test -p fret-node --features compat-retained-canvas searcher_pointer_hover::state::tests`,
  focused `cargo test -p fret-node --features compat-retained-canvas
  searcher_pointer_wheel::delta::tests`, focused `cargo test -p fret-node --features
  compat-retained-canvas searcher_row_activation::item::tests`, and focused `cargo test -p
  fret-node --features compat-retained-canvas selection::selectable::tests` now also lock the
  searcher/selection helper test seam split.
- `cancel_gesture_state/sessions.rs`, `cancel_session/pan.rs`, `cancel_session/residuals.rs`, and
  `event_keyboard_state.rs` now also route local cancel/keyboard helper test coverage through the
  private `canvas/widget/cancel_gesture_state/sessions/tests.rs`,
  `canvas/widget/cancel_session/pan/tests.rs`, `canvas/widget/cancel_session/residuals/tests.rs`,
  and `canvas/widget/event_keyboard_state/tests.rs` seams, so these helper files now mainly keep
  gesture-session clearing, pan reset, cancel residual cleanup, and keyboard modifier sync explicit
  without changing retained cancel/input runtime semantics.
- `cargo check -p fret-node --features compat-retained-canvas --tests`,
  `cargo fmt -p fret-node --check`, focused
  `cargo test -p fret-node --features compat-retained-canvas cancel_gesture_state::sessions::tests`,
  focused `cargo test -p fret-node --features compat-retained-canvas
  cancel_session::pan::tests`, focused `cargo test -p fret-node --features compat-retained-canvas
  cancel_session::residuals::tests`, and focused `cargo test -p fret-node --features
  compat-retained-canvas event_keyboard_state::tests` now also lock the cancel/keyboard helper
  test seam split.
- `pending_resize/checks.rs`, `pending_wire_drag/checks.rs`, `sticky_wire/checks.rs`,
  `edge_insert_drag/pending/checks.rs`, and `edge_insert_drag/drag/state.rs` now also route local
  pending/sticky edge-drag helper test coverage through the private
  `canvas/widget/pending_resize/checks/tests.rs`,
  `canvas/widget/pending_wire_drag/checks/tests.rs`,
  `canvas/widget/sticky_wire/checks/tests.rs`,
  `canvas/widget/edge_insert_drag/pending/checks/tests.rs`, and
  `canvas/widget/edge_insert_drag/drag/state/tests.rs` seams, so these helper files now mainly
  keep pending activation thresholds, sticky-wire gating, and edge-insert drag state updates
  explicit without changing retained drag/runtime semantics.
- `cargo check -p fret-node --features compat-retained-canvas --tests`,
  `cargo fmt -p fret-node --check`, focused
  `cargo test -p fret-node --features compat-retained-canvas pending_resize::checks::tests`,
  focused `cargo test -p fret-node --features compat-retained-canvas
  pending_wire_drag::checks::tests`, focused `cargo test -p fret-node --features
  compat-retained-canvas sticky_wire::checks::tests`, focused `cargo test -p fret-node --features
  compat-retained-canvas edge_insert_drag::pending::checks::tests`, and focused `cargo test -p
  fret-node --features compat-retained-canvas edge_insert_drag::drag::state::tests` now also lock
  the pending/sticky edge-drag helper test seam split.
- `focus_nav_ports_center/activation.rs`, `focus_port_direction_rank.rs`,
  `focus_port_direction_wire.rs`, and `view_math_rect.rs` now also route local focus/view helper
  test coverage through the private
  `canvas/widget/focus_nav_ports_center/activation/tests.rs`,
  `canvas/widget/focus_port_direction_rank/tests.rs`,
  `canvas/widget/focus_port_direction_wire/tests.rs`, and
  `canvas/widget/view_math_rect/tests.rs` seams, so these helper files now mainly keep
  activation-point fallback, directional port ranking, wire direction resolution, and view rect
  containment math explicit without changing retained focus/navigation semantics.
- `cargo check -p fret-node --features compat-retained-canvas --tests`,
  `cargo fmt -p fret-node --check`, focused
  `cargo test -p fret-node --features compat-retained-canvas focus_nav_ports_center::activation::tests`,
  focused `cargo test -p fret-node --features compat-retained-canvas
  focus_port_direction_rank::tests`, focused `cargo test -p fret-node --features
  compat-retained-canvas focus_port_direction_wire::tests`, and focused `cargo test -p fret-node
  --features compat-retained-canvas view_math_rect::tests` now also lock the focus/view helper
  test seam split.
- `event_pointer_down_route/dispatch.rs`, `event_pointer_down_state.rs`,
  `event_pointer_move_state.rs`, `event_pointer_wheel_state.rs`, `delete_ops_builder/node.rs`,
  `node_resize/math.rs`, and `paint_grid_tiles/support.rs` now also route local event/math helper
  test coverage through the private
  `canvas/widget/event_pointer_down_route/dispatch/tests.rs`,
  `canvas/widget/event_pointer_down_state/tests.rs`,
  `canvas/widget/event_pointer_move_state/tests.rs`,
  `canvas/widget/event_pointer_wheel_state/tests.rs`,
  `canvas/widget/delete_ops_builder/node/tests.rs`,
  `canvas/widget/node_resize/math/tests.rs`, and
  `canvas/widget/paint_grid_tiles/support/tests.rs` seams, so these helper files now mainly keep
  pointer-event routing/state sync, delete-op edge deduping, node-resize math, and grid-tile
  support math explicit without changing retained runtime semantics.
- `cargo check -p fret-node --features compat-retained-canvas --tests`,
  `cargo fmt -p fret-node --check`, focused
  `cargo test -p fret-node --features compat-retained-canvas
  event_pointer_down_route::dispatch::tests`, focused `cargo test -p fret-node --features
  compat-retained-canvas event_pointer_down_state::tests`, focused `cargo test -p fret-node
  --features compat-retained-canvas event_pointer_move_state::tests`, focused `cargo test -p
  fret-node --features compat-retained-canvas event_pointer_wheel_state::tests`, focused `cargo
  test -p fret-node --features compat-retained-canvas delete_ops_builder::node::tests`, focused
  `cargo test -p fret-node --features compat-retained-canvas node_resize::math::tests`, and
  focused `cargo test -p fret-node --features compat-retained-canvas
  paint_grid_tiles::support::tests` now also lock the event/math helper test seam split.
- `interaction_policy/node.rs`, `pending_drag_session/group.rs`, `pending_drag_session/node.rs`,
  `split_edge_execution.rs`, and `stores/internals.rs` now also route local
  interaction/pending/store helper test coverage through the private
  `canvas/widget/interaction_policy/node/tests.rs`,
  `canvas/widget/pending_drag_session/group/tests.rs`,
  `canvas/widget/pending_drag_session/node/tests.rs`,
  `canvas/widget/split_edge_execution/tests.rs`, and
  `canvas/widget/stores/internals/tests.rs` seams, so these helper files now mainly keep node
  draggable/connectable policy, pending drag activation, split-edge rejection toast fallback, and
  internals edge-center math explicit without changing retained runtime semantics.
- `cargo check -p fret-node --features compat-retained-canvas --tests`,
  `cargo fmt -p fret-node --check`, focused
  `cargo test -p fret-node --features compat-retained-canvas interaction_policy::node::tests`,
  focused `cargo test -p fret-node --features compat-retained-canvas
  pending_drag_session::group::tests`, focused `cargo test -p fret-node --features
  compat-retained-canvas pending_drag_session::node::tests`, focused `cargo test -p fret-node
  --features compat-retained-canvas split_edge_execution::tests`, and focused `cargo test -p
  fret-node --features compat-retained-canvas stores::internals::tests` now also lock the
  interaction/pending/store helper test seam split.
- `paint_edge_anchors/{resolve,state,style}.rs`, `paint_groups/{chrome,overlay}.rs`,
  `paint_root_helpers/{geometry,paint}.rs`, and `paint_root/cache_plan/{hover,tiles}.rs` now also
  route local paint/cache helper test coverage through the private
  `canvas/widget/paint_edge_anchors/resolve/tests.rs`,
  `canvas/widget/paint_edge_anchors/state/tests.rs`,
  `canvas/widget/paint_edge_anchors/style/tests.rs`,
  `canvas/widget/paint_groups/chrome/tests.rs`,
  `canvas/widget/paint_groups/overlay/tests.rs`,
  `canvas/widget/paint_root_helpers/geometry/tests.rs`,
  `canvas/widget/paint_root_helpers/paint/tests.rs`,
  `canvas/widget/paint_root/cache_plan/hover/tests.rs`, and
  `canvas/widget/paint_root/cache_plan/tiles/tests.rs` seams, so these helper files now mainly
  keep edge-anchor paint gating, group chrome/overlay selection, static-scene cache key
  derivation, and paint-root hover/tile planning math explicit without changing retained paint
  runtime semantics.
- `cargo check -p fret-node --features compat-retained-canvas --tests`,
  `cargo fmt -p fret-node --check`, focused
  `cargo test -p fret-node --features compat-retained-canvas paint_edge_anchors::resolve::tests`,
  focused `cargo test -p fret-node --features compat-retained-canvas
  paint_edge_anchors::state::tests`, focused `cargo test -p fret-node --features
  compat-retained-canvas paint_edge_anchors::style::tests`, focused `cargo test -p fret-node
  --features compat-retained-canvas paint_groups::chrome::tests`, focused `cargo test -p
  fret-node --features compat-retained-canvas paint_groups::overlay::tests`, focused `cargo test
  -p fret-node --features compat-retained-canvas paint_root_helpers::geometry::tests`, focused
  `cargo test -p fret-node --features compat-retained-canvas paint_root_helpers::paint::tests`,
  focused `cargo test -p fret-node --features compat-retained-canvas
  paint_root::cache_plan::hover::tests`, and focused `cargo test -p fret-node --features
  compat-retained-canvas paint_root::cache_plan::tiles::tests` now also lock the paint/cache
  helper test seam split.
- `paint_edges/pass.rs` now routes edge-pass budget initialization,
  batch paint replay, and budget-driven redraw requests through the private
  `canvas/widget/paint_edges/pass/budgets.rs`,
  `canvas/widget/paint_edges/pass/batch.rs`, and
  `canvas/widget/paint_edges/pass/redraw.rs` seams, so the root pass file now mainly keeps the
  retained pass state type plus façade entry points explicit without changing edge-pass ordering or
  budget semantics.
- focused `cargo nextest run -p fret-node --features compat-retained-canvas`
  coverage now also locks `skin_wire_outline_selected_draws_outline_path_before_core`,
  `skin_wire_highlight_selected_draws_highlight_after_core`,
  `paint_overrides_can_drive_edge_marker_paint_binding`, and
  `bezier_markers_align_with_bezier_start_end_tangents` after the edge-pass split.
- `paint_edges/prepare.rs` now routes edge paint-model
  construction, insert-marker projection, and batch partitioning through the private
  `canvas/widget/paint_edges/prepare/build.rs`,
  `canvas/widget/paint_edges/prepare/marker.rs`, and
  `canvas/widget/paint_edges/prepare/batches.rs` seams, so the root prepare file now mainly keeps
  the retained edge-paint state types plus façade entry points explicit without changing
  insert-marker or edge-batch semantics.
- focused `cargo nextest run -p fret-node --features compat-retained-canvas`
  coverage now also locks `skin_wire_outline_selected_draws_outline_path_before_core`,
  `paint_overrides_can_drive_edge_marker_paint_binding`,
  `alt_drag_edge_opens_insert_node_picker_when_enabled`, and
  `custom_edge_marker_falls_back_to_from_to_tangent_when_path_has_no_tangents` after the edge
  prepare split.
- `paint_edges/labels.rs` now routes label-tail budgeted
  paint/repaint handling and cache-stat publication through the private
  `canvas/widget/paint_edges/labels/tail.rs` and
  `canvas/widget/paint_edges/labels/stats.rs` seams, so the root label file now mainly keeps the
  retained façade entry points explicit without changing label-tail or budget-stat semantics.
- focused `cargo nextest run -p fret-node --features compat-retained-canvas`
  coverage now also locks `edge_label_anchor_matches_bezier_route_math`,
  `edge_label_border_uses_edge_render_hint_color_override`,
  `cached_edge_labels_match_between_tiled_and_single_tile_cache_modes`,
  `paint_warms_edge_label_scene_cache_incrementally`, and
  `paint_warms_edge_label_scene_cache_incrementally_for_large_viewport_tiles` after the edge-label
  split.
- `paint_edges/main.rs` now also routes interaction-hint /
  custom-path frame preparation and optional drop-marker emission through the private
  `canvas/widget/paint_edges/main/context.rs` and
  `canvas/widget/paint_edges/main/markers.rs` seams, so the root edge main file now mainly keeps
  the top-level pass/labels/preview orchestration explicit without changing edge-main behavior.
- focused `cargo nextest run -p fret-node --features compat-retained-canvas`
  coverage now also locks `skin_wire_outline_selected_draws_outline_path_before_core`,
  `skin_wire_highlight_hovered_draws_highlight_after_core`,
  `alt_drag_edge_opens_insert_node_picker_when_enabled`,
  `paint_warms_edge_label_scene_cache_incrementally`, and
  `paint_warms_edge_label_scene_cache_incrementally_for_large_viewport_tiles` after the edge-main
  split.
- `paint_edges/preview.rs` now routes preview target/style
  resolution, drop-marker quads, and preview wire draw/effect tails through the private
  `canvas/widget/paint_edges/preview/target.rs`,
  `canvas/widget/paint_edges/preview/marker.rs`, and
  `canvas/widget/paint_edges/preview/draw.rs` seams, so the root edge-preview file now mainly
  keeps retained preview orchestration explicit.
- `paint_edges/chrome.rs` now routes wire outline paint,
  selected-edge glow effect setup, and selected/hovered highlight resolution through the private
  `canvas/widget/paint_edges/chrome/outline.rs`,
  `canvas/widget/paint_edges/chrome/glow.rs`, and
  `canvas/widget/paint_edges/chrome/highlight.rs` seams, so the root edge-chrome file now mainly
  keeps retained chrome orchestration explicit.
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
- keep the visible-subset portal-hosting config defaults/source-policy gate green
- keep the diagnostics-config defaults/source-policy gate green

### Evidence anchors

- `ecosystem/fret-node/src/ui/declarative/paint_only.rs`
- `ecosystem/fret-node/src/ui/declarative/mod.rs`
- `ecosystem/fret-node/src/ui/declarative/paint_only/portals.rs`
- `ecosystem/fret-node/src/ui/declarative/paint_only/overlays.rs`
- `ecosystem/fret-node/src/ui/declarative/paint_only/surface_support.rs`
- `ecosystem/fret-node/src/ui/declarative/paint_only/tests.rs`
- `apps/fret-examples/src/node_graph_demo.rs`
- `ecosystem/fret-node/src/ui/screen_space_placement.rs`
- `ecosystem/fret-node/src/ui/canvas/state.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/menu_session.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/insert_candidates.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/insert_execution.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/insert_execution/candidate.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/insert_execution/plan.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/insert_execution/feedback.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/insert_execution/tests.rs`
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
- `ecosystem/fret-node/src/ui/canvas/widget/command_router/insert.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/command_router/group.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/command_router/view.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/command_router/focus.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/command_router/edit.rs`
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
- `ecosystem/fret-node/src/ui/canvas/widget/paint_edge_anchors.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_edge_anchors/resolve.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_edge_anchors/state.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_edge_anchors/style.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_edge_anchors/render.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root_helpers.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root_helpers/paint.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root_helpers/geometry.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/cache_plan.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/cache_plan/hover.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/cache_plan/tiles.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/frame.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/frame/cache.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/frame/background.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/cached_edges/edges.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/cached_edges/edges/fallback.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/cached_edges/edges/replay.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/cached_edges/edges/single.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/cached_edges/edges/tiled.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/cached_edges/labels.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/cached_edges/labels/replay.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/cached_edges/labels/single.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/cached_edges/labels/tiled.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/cached_edges/build_state.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/cached_edges/build_state/ops.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/cached_edges/build_state/init.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/cached_edges/build_state/step.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/static_layer.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/edge_anchor.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/edge_anchor/target_id.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/edge_anchor/render.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/edge_anchor/geometry.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/cached_edges/mod.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/cached_edges/anchor_target.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/cached_edges/dispatch.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/cached_edges/fallback.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_nodes/static_node_chrome.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_nodes/static_node_chrome/style.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_nodes/static_node_chrome/shadow.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_nodes/static_node_chrome/quads.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_nodes/static_node_chrome/text.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_nodes/static_ports.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_nodes/static_ports/labels.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_nodes/static_ports/geometry.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_nodes/static_ports/fill.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_nodes/static_ports/stroke.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_nodes/static_ports/shapes.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_edges/markers_support.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_edges/markers_support/paint.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_edges/markers_support/route.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_edges/markers_support/custom.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_edges/cached_budgeted.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_edges/cached_budgeted/wires.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_edges/cached_budgeted/labels.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_render_data/edges.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_render_data/edges/candidate.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_render_data/edges/hint.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_render_data/edges/cull.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_render_data/edges/append.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_render_data/nodes.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_render_data/nodes/visible.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_render_data/nodes/overhead.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_render_data/nodes/append.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_render_data/nodes/ports.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_render_data/collect.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_render_data/collect/selection.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_render_data/collect/body.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_render_data/selected_nodes.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_render_data/selected_nodes/selection.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_render_data/selected_nodes/body.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_edges/pass.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_edges/pass/budgets.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_edges/pass/batch.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_edges/pass/redraw.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_edges/prepare.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_edges/prepare/build.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_edges/prepare/marker.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_edges/prepare/batches.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_edges/labels.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_edges/labels/tail.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_edges/labels/stats.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_edges/main.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_edges/main/context.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_edges/main/markers.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_edges/preview.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_edges/preview/target.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_edges/preview/marker.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_edges/preview/draw.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_edges/chrome.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_edges/chrome/outline.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_edges/chrome/glow.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_edges/chrome/highlight.rs`
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
- `ecosystem/fret-node/src/ui/canvas/widget/sticky_wire/checks.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/sticky_wire/target.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/sticky_wire_connect.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/sticky_wire_targets.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/sticky_wire_targets/inspect.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/sticky_wire_targets/picker.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/event_pointer_move.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/event_pointer_move_state.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/event_pointer_move_tail.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_move_pointer_state.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_move_release.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_move_release_left.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_move_release_pan.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_move_dispatch.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_move_dispatch/primary.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_move_dispatch/primary/surface.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_move_dispatch/primary/group.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_move_dispatch/primary/node.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_move_dispatch/primary/connection.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_move_dispatch/secondary.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_move_dispatch/secondary/node.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_move_dispatch/secondary/connection.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_move_dispatch/secondary/insert.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/event_pointer_wheel.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/event_pointer_wheel_state.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/event_pointer_wheel_route.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_wheel_motion.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_wheel_pan.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_wheel_pan/gate.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_wheel_pan/resolve.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_wheel_pan/apply.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_wheel_viewport.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_wheel_zoom.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_wheel_zoom/wheel.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_wheel_zoom/pinch.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_wheel_zoom/apply.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/event_clipboard.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/event_clipboard_pending.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/event_clipboard_feedback.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/event_router.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/event_router_pointer.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/event_router_pointer_button.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/event_router_pointer_wheel.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/event_router_system.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/event_router_system_input.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_node_drag_parent.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_node_drag_parent/collect.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_node_drag_parent/rect.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_node_drag_parent/target.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/focus_nav_ports_hints.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/focus_nav_ports_hints/preflight.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/focus_nav_ports_hints/evaluate.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/focus_nav_ports_hints/apply.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/focus_nav_ports_activation.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/focus_nav_ports_activation/preflight.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/focus_nav_ports_activation/start.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/focus_nav_ports_activation/commit.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/focus_nav_traversal_port.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/focus_nav_traversal_port/preflight.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/focus_nav_traversal_port/collect.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/focus_nav_traversal_port/select.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/focus_nav_traversal_port/apply.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/focus_nav_traversal_node.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/focus_nav_traversal_node/preflight.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/focus_nav_traversal_node/collect.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/focus_nav_traversal_node/select.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/focus_nav_traversal_node/apply.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/focus_port_direction_candidate.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/focus_port_direction_candidate/center.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/focus_port_direction_candidate/search.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/focus_nav_ports_center.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/focus_nav_ports_center/port.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/focus_nav_ports_center/activation.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/move_ops/align_distribute/support.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/move_ops/align_distribute/support/types.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/move_ops/align_distribute/support/collect.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/move_ops/align_distribute/support/delta.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/move_ops/align_distribute/support/shift.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/move_ops/align_distribute/support/append.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/view_state/viewport.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/view_state/viewport/set.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/view_state/viewport/update.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/view_state/viewport/visible.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/press_session/prepare.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/press_session/prepare/test_support.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/press_session/prepare/surface.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/press_session/prepare/surface/clear.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/press_session/prepare/surface/focus.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/press_session/prepare/surface/tests.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/press_session/prepare/target.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/press_session/prepare/target/clear.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/press_session/prepare/target/focus.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/press_session/prepare/target/tests.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/wire_drag/move_update/hover.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/wire_drag/move_update/hover/source.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/wire_drag/move_update/hover/pick.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/wire_drag/move_update/hover/validity.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/wire_drag/move_update/hover/convertible.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/selection/box_edges.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/selection/box_edges/mode.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/selection/box_edges/graph.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/selection/box_edges/store.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/selection/box_edges/test_support.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/selection/box_edges/tests.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/interaction_policy/port.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/interaction_policy/port/connectable.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/interaction_policy/port/bundle.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/interaction_policy/port/test_support.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/interaction_policy/port/tests.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/wire_math.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/wire_math/route.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/wire_math/tests.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/delete_ops_builder.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/delete_ops_builder/assemble.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/delete_ops_builder/test_support.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/delete_ops_builder/tests.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_grid_tiles.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_grid_tiles/ops.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_grid_tiles/tests.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/key_navigation.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/key_navigation/test_support.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/key_navigation/tests.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/group_draw_order.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/group_draw_order/test_support.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/group_draw_order/tests.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pending_resize_session.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pending_resize_session/test_support.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pending_resize_session/tests.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/target_selection.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/target_selection/test_support.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/target_selection/tests.rs`
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
- `ecosystem/fret-node/src/ui/canvas/widget/widget_surface/constants.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/widget_surface/runtime.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/widget_surface/runtime/render.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/widget_surface/runtime/interaction.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/widget_surface/runtime/edge.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/interaction_gate.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/focus_session.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/cancel_session.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/press_session.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_session.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pending_resize_session.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pending_drag_session.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pending_connection_session.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pending_connection_session/edge.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pending_connection_session/wire.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pending_wire_drag.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pending_wire_drag/checks.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pending_wire_drag/activate.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_pending/wire_drag.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_pending/wire_drag/checks.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_pending/wire_drag/activate.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/edge_insert_drag/pending.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/edge_insert_drag/pending/checks.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/edge_insert_drag/pending/activate.rs`
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
- `ecosystem/fret-node/src/ui/canvas/widget/group_drag.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pending_group_drag.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/group_resize.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pending_group_resize.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pending_resize.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pending_resize/checks.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pending_resize/activate.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/left_click/group_background.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_commit/resize.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/left_click/element_hits.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/left_click/connection_hits.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/left_click/connection_hits/port.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/left_click/connection_hits/port/connectable.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/left_click/connection_hits/port/click_connect.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/left_click/connection_hits/port/kind.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/left_click/edge_selection.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/left_click/connection_hits/edge_anchor.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/left_click/connection_hits/edge_anchor/arm.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/left_click/element_hits/edge.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/left_click/element_hits/edge/drag.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/left_click/node_selection.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/left_click/element_hits/node.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/left_click/element_hits/node/capabilities.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/left_click/element_hits/node/drag.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/left_click/element_hits/resize.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/left_click/element_hits/resize/state.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/edge_insert_drag/pointer_up.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/edge_insert_drag/pointer_up/pending.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/edge_insert_drag/pointer_up/active.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/edge_insert_drag/drag.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/edge_insert_drag/drag/state.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/edge_insert_drag/drag/tail.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/left_click/hit.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_left_route/dispatch.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_pending/release.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/tests/interaction_conformance.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_grid.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/event_pointer_move.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/event_pointer_move_tail.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_up.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/event_pointer_up.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/event_router_pointer_button.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/event_pointer_down.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/event_pointer_down_route.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_down_gesture_start.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_down_double_click_background.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_down_double_click_background/preflight.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_down_double_click_background/hit.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_down_double_click_background/apply.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_grid_plan_support.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_grid_plan_support/hint.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_grid_plan_support/metrics.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_grid_plan_support/tiles.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_grid_plan_support/validate.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_grid_cache.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_grid_plan.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_grid_stats.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_grid_tiles.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_grid_cache/warm.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_grid_cache/ops.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_grid_cache/key.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_overlay_elements.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_overlay_menu.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_overlay_menu/frame.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_overlay_menu/items.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_searcher.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_searcher/frame.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_overlay_wire_hint.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_overlay_wire_hint/layout.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_overlay_wire_hint/draw.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_overlay_toast.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_overlay_toast/draw.rs`
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
