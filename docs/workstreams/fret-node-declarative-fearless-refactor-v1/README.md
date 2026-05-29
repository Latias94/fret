# Workstream: `fret-node` Fearless Refactor (v1)

Status: Reframed and active (last updated 2026-05-29)
Quick navigation:

- `design.md` - current surface map + next worktree order
- `todo.md` - actionable backlog
- `milestones.md` - done criteria + regression expectations
- `xyflow-interface-review.md` - interface/architecture deltas vs XYFlow + refactor opportunities
- `../crate-audits/fret-node.l0.md` - L0 public surface + hazard scan
Scope: `ecosystem/fret-node` with focused touch points in `ecosystem/fret-canvas`, `apps/fret-examples`, and node-graph diagnostics

Historical note:

- This folder keeps its original path for continuity, but its scope is now broader than the earlier
  "paint-only declarative" slice.
- The workstream now covers the full landing plan for `fret-node` as a **declarative-first,
  editor-grade reference surface** for the Fret ecosystem.
- Current code has removed the old retained compatibility feature and widget surface. Older
  retained/compatibility references below are retained as historical context until the long-form
  README is normalized; use `design.md`, `todo.md`, `EVIDENCE_AND_GATES.md`, and `HANDOFF.md` as the
  current execution authority.

## Intent

Make `ecosystem/fret-node` the canonical example of how Fret should ship a complex editor surface:

- **headless asset model first** (`Graph`, `GraphTransaction`, rules, profiles, diagnostics),
- **declarative-first public authoring** for ecosystem and app code,
- **editor-grade runtime semantics** without leaking obsolete retained authoring into long-term APIs,
- **transaction-safe editor interactions** instead of ad-hoc graph mutations,
- **clear mechanism vs policy boundaries** so `fret-node` teaches the right layering habits.

This workstream is not a rewrite for its own sake. It exists because `fret-node` is doing two jobs
at once:

1. it is a real product surface for node-graph editors, and
2. it is one of the most important ecosystem teaching surfaces for Fret authoring patterns.

If `fret-node` is architecturally muddy, downstream crates will copy the wrong patterns.

## Why this workstream exists now

`fret-node` already has strong building blocks:

- a long-lived graph document model,
- reversible edits and history,
- typed connection planning and validation,
- a powerful retained interaction engine,
- a promising declarative paint-only surface.

However, the overall authoring story is still not fully converged:

- the public recommendation is split between paint-only and retained-backed paths,
- the declarative surface is not yet the transaction-safe editor-grade path,
- the pure view-state vs editor-config split is now landed in code, but the authoring story still
  needs to teach that boundary consistently,
- runtime capabilities are spread across store, queues, lookups, commands, and helpers without one
  obvious app-facing facade pair (`NodeGraphSurfaceBinding` + `NodeGraphController`),
- some workstream content has become too implementation-local and no longer helps reviewers decide
  what must land next.

This document resets the workstream around the smallest set of decisions and milestones needed to
land the right long-term shape.

## Locked decisions

These are the decision gates for this workstream. Changes that violate them should require an ADR
update rather than an incidental refactor.

1. **Public authoring posture is declarative-first.**
   - Downstream authors should compose node-graph surfaces as elements, not as retained widgets.
   - Retained implementation details may remain internally for a time, but must not be the taught
     default.

2. **Retained compatibility has exited the current public surface.**
   - The current source-policy gates require the retained compatibility feature, raw queues, and
     retained widget authoring surface to stay removed.
   - Any future fallback must be reintroduced through an explicit task/ADR instead of by reviving
     stale compatibility APIs.

3. **Editor-grade graph edits must converge on transactions/store, not direct `Graph` mutation.**
   - The authoritative editor commit path is `GraphTransaction` / `NodeGraphStore`.
   - Declarative surfaces may hold transient drag/hover state locally, but committed edits should go
     through store/controller entry points.

4. **`NodeGraphViewState` must shrink back to true view state.**
   - Pan/zoom/selection/draw order belong there.
   - Interaction policy, key bindings, and performance tuning must not all live in the same bucket.

5. **A unified controller/instance surface is required.**
   - Apps need one ergonomic place to drive viewport actions, graph updates, lookups, and controlled
     synchronization.
   - The current split across store/lookups/view queue/commands is acceptable internally, but not as
     the final teaching surface.

6. **Mechanism vs policy boundaries stay aligned with Fret architecture.**
   - `fret-node` may own editor-specific mechanism and contracts.
   - Default overlay behavior, spacing defaults, recipe chrome, and design-system policy should not
     silently harden inside mechanism code just because the node graph needs them.

## Current state snapshot

### Already strong

- **Headless asset layer**
  - `Graph`, stable IDs, imports, symbols, groups, sticky notes.
  - `GraphOp`, `GraphTransaction`, `GraphHistory`.
  - `rules`, `profile`, diagnostics, typed connection planning.

- **Store-first declarative interaction surface**
  - `NodeGraphSurfaceBinding`, `NodeGraphController`, and `NodeGraphStore` are the current public
    teaching surfaces.
  - Source-policy gates now keep raw retained compatibility terms, queue transport, and direct
    retained widget authoring out of supported UI sources.

- **Declarative-first direction is already visible**
  - `node_graph_surface` is the default lightweight declarative demo path.
  - Declarative overlays, portals, minimap, controls, blackboard, and rename flows now carry focused
    gates without depending on a retained compatibility feature.
  - `NodeGraphSurfaceProps.edge_types` and `NodeGraphSurfaceProps.skin` now wire narrow UI-only
    edge view policy into the default declarative paint path without exposing the broad presenter
    trait.

### Still unresolved

- **Public posture is clearer, but editor-grade closure is still incomplete**
  - The canonical declarative entrypoint is now `node_graph_surface(...)` with a
    store-backed `NodeGraphSurfaceBinding` carried by `NodeGraphSurfaceProps`.
  - The remaining ambiguity is no longer naming; it is how much editor-grade behavior is already
    closed on the declarative path versus still landing.

- **Transaction boundary ambiguity in the declarative path**
  - The first M3 slice is now landed: the paint-only node-drag commit builds a
    `GraphTransaction` and dispatches through `NodeGraphController` (store-backed).
  - `NodeGraphSurfaceProps` now takes a single `NodeGraphSurfaceBinding`, and that binding
    still requires the controller-backed store path; the no-controller fallback remains
    intentionally removed so the demo surface teaches the controller-first contract.
  - Broader declarative commit coverage is still unresolved: this pattern must extend to the rest of
    committed edit flows and converge behind a clearer controller/instance surface.

- **Overgrown view-state boundary**
  - The non-test M2 closure is now landed: runtime `NodeGraphViewState` is pure view state
    (pan/zoom/selection/draw order), while persisted/editor-owned interaction policy lives in
    `NodeGraphEditorConfig` (`NodeGraphInteractionConfig` + `NodeGraphRuntimeTuning`).
  - Runtime/widget code still resolves a combined `NodeGraphInteractionState`, but it is now
    assembled from `NodeGraphEditorConfig` / store-owned seams instead of being stored inside
    release `NodeGraphViewState`.
  - Store selector subscriptions now observe non-viewport view-state changes (draw order,
    interaction config, runtime tuning) without emitting misleading empty `ViewChanged` events.
  - Persistence ownership is now explicit: `NodeGraphEditorStateFile` writes pure view-state in
    `view_state`, with interaction policy and runtime tuning grouped under `editor_config`.
  - Example surfaces now follow that split too: declarative controls consume the editor-config seam,
    and view-state persistence saves the wrapper payload instead of mutating `NodeGraphViewState`.
  - The old test-only compatibility bridge is removed: tests bind explicit editor-config seams
    instead of mirroring editor config back into `NodeGraphViewState`.
  - FNDX-044 removes the public `NodeGraphStore::view_state_mut` bypass, so store view-state writes
    must flow through the notifying/sanitizing helper paths.

- **Default view-policy surface**
  - FNDX-045 wires `NodeGraphEdgeTypes` and `NodeGraphSkin` into `node_graph_surface(...)` for edge
    hint/custom paint-path and paint-only skin refinement.
  - FNDX-046 feeds custom edge path conservative AABBs into the spatial index candidate set.
  - FNDX-047 feeds the same custom path command stream into exact path-distance hit filtering for
    edge interaction candidates.
  - FNDX-048 feeds the same custom path command stream into default edge-center anchors by computing
    custom path midpoint/normal for internals.
  - FNDX-049 feeds those custom-path-derived edge-center anchors into declarative EdgeToolbar host
    child placement.
  - FNDX-050 feeds default `EdgeRenderHint.label` output into a screen-space declarative edge-label
    child layer centered on the same custom-path-derived edge-center anchors.
  - FNDX-051 adds `NodeGraphDeclarativeEdgeLabelRenderer` for non-interactive custom edge-label
    children on that same anchor, plus a combined renderers bag for composing it with node portal
    renderers.
  - FNDX-052 adds `NodeGraphEdgeLabelHitTestMode::ChildBounds` for the first pointer-interactive
    custom edge-label control contract while keeping transparent labels as the default.
  - FNDX-053 feeds the same custom-path-aware edge hit-test into default declarative click-edge
    selection through the store-backed view-state helper.
  - FNDX-054 feeds store-backed selected-edge state into default declarative edge paint and surface
    diagnostics.
  - FNDX-055 plans selected/focused edge update anchors from authoritative port centers with
    global/per-edge reconnectability gates, without rendering controls or starting reconnect drags.
  - FNDX-056 renders those planned update anchors as hit-testable default declarative controls with
    anchor-click priority, while keeping reconnect drag lifecycle as follow-up work.
  - FNDX-057 starts reconnect drags from those rendered controls, reusing the existing connection
    drag threshold and cancel/up cleanup policy.
  - FNDX-058 adds target-port hit-testing and accepted store-backed reconnect commit/callback
    dispatch for active update-anchor drops while keeping endpoint-gated and empty-canvas drops
    cleanup-only.
  - FNDX-059 emits reconnect gesture start/end callback aliases for successful arm plus committed,
    rejected, empty/no-op, Escape, PointerCancel, and missed-left-button cleanup end paths.
  - FNDX-060 paints an active reconnect preview wire from the fixed port to the current pointer and
    removes it on pointer-up/cancel cleanup.
  - FNDX-061 adds the minimal `reconnect_on_drop_empty` event outcome: opt-in empty reconnect drops
    emit `OpenInsertNodePicker` without graph commits or concrete picker UI, while the default
    remains no-op.
  - Custom `NodeGraphPresenter` remains advanced/internal on the default path because it still mixes
    geometry, labels, context menus, and insertion/search policy.
  - Broader XyFlow edge-wrapper lifecycle parity remains follow-up work beyond click selection,
    the first child-bounds control contract, rendered update-anchor controls, reconnect drag
    start/cancel lifecycle, accepted reconnect commits, and opt-in empty-drop outcome semantics.

- **Ergonomic API fragmentation**
  - The surface naming is now closed around `NodeGraphSurfaceBinding` (instance-style app-facing
    bundle) plus `NodeGraphController` (lower-level imperative/runtime facade).
  - Advanced mirror-owned bindings now also carry an explicit `NodeGraphEditorConfig` model, so
    binding-driven controlled sync no longer depends on optional config mirrors or implicit defaults.
  - The remaining gap is helper breadth and internal organization: viewport helpers, lookups,
    commands, store subscriptions, and controlled updates still need to keep converging on that
    pair without regrowing god files.

- **Mixed callback responsibilities**
  - The current callback surface mixes store/headless commit signals with UI gesture lifecycle.

## Current hazards

These are the hazards reviewers should keep in mind even after the recent controller/store-backed
convergence slices.

### H1. Declarative commit-path regressions are still the highest-risk failure mode

- The biggest regression risk is reintroducing direct `Graph` / `NodeGraphViewState` writes in
  `paint_only` once a `NodeGraphController` / `NodeGraphStore` is available.
- Current evidence that the preferred path is converging:
  - `ecosystem/fret-node/src/ui/declarative/paint_only.rs` (`commit_graph_transaction`,
    `update_view_state_action_host`, `update_selection_action_host`)
  - `ecosystem/fret-node/src/ui/controller.rs`
  - `ecosystem/fret-node/src/ui/declarative/paint_only.rs` focused controller/store-backed tests

### H2. Store and editor-config surfaces must keep explicit ownership

- Release/runtime `NodeGraphViewState` is now pure view state, and the old `cfg(test)` mirror
  bridge is gone. The remaining hazard is API-story drift where public surfaces bypass the
  store/editor-config contracts.
- If first-party demos or public helpers reintroduce implicit `NodeGraphEditorConfig` fallbacks or
  raw mutable view-state access, downstream authors will relearn the wrong seam even though the
  runtime split is already correct.
- Evidence:
  - `ecosystem/fret-node/src/io/mod.rs` (`NodeGraphViewState`, `NodeGraphInteractionConfig`,
    `NodeGraphRuntimeTuning`, `NodeGraphInteractionState`)
  - `ecosystem/fret-node/src/runtime/store.rs`
  - `ecosystem/fret-node/src/ui/controller_store_sync.rs`
  - `ecosystem/fret-node/src/ui/binding_store_sync.rs`
  - `ecosystem/fret-node/src/ui/declarative/paint_only/transactions.rs`
  - `ecosystem/fret-node/src/ui/overlays/controls_declarative.rs`
  - `docs/workstreams/fret-node-declarative-fearless-refactor-v1/milestones.md` (`M2`)

### H3. `NodeGraphController` is landed, but not yet fully closed as the teaching surface

- The controller now covers the first query / transaction / viewport / selection helpers, including
  XyFlow-style node/handle connection lookups; retained canvas / minimap glue can also bind through
  the controller now, public viewport option types now stay store-first instead of leaking retained
  queue animation knobs, while richer edit commands, helper breadth, and broader callback cleanup
  are still open.
- Evidence:
  - `ecosystem/fret-node/src/ui/controller.rs`
  - `docs/workstreams/fret-node-declarative-fearless-refactor-v1/milestones.md` (`M3`)

## Problems this refactor must solve

This workstream treats the following as the root problems to address.

### P1. `fret-node` needs one recommended authoring story

We need to be able to say, without caveats:

- what is recommended **today** for production editor-grade usage,
- what is the **target** authoring posture,
- what is merely a **temporary compatibility** path,
- what is explicitly **not** the public best practice.

### P2. The declarative path must stop bypassing the transaction architecture

The point of `fret-node` is not just to draw graphs; it is to model long-lived, undoable,
diagnostic-rich graph editing. Any declarative-first editor path must preserve that value.

### P3. State boundaries must match intent

We need separate concepts for:

- persisted viewport/selection state,
- interaction config and key semantics,
- runtime tuning and cache behavior,
- ephemeral widget-local interaction session state.

Current landed boundary for the declarative paint-only path:

- **store-backed**: committed graph document, undo/redo history, viewport, committed selection,
  and draw order,
- **local surface state**: active pan session, node-drag preview/arming, marquee preview/arming,
  pending click-selection preview, hover target/anchor, and hit-test scratch/cache inputs,
- **paint precedence**: active marquee preview > pending selection preview > committed selection.
- **authoritative reset rule**: external authoritative graph replacement now drops local pan / node-drag / marquee / pending-selection / hover / portal transient state, while committed-selection-only authority updates drop selection-scoped preview state without flushing pan/hover caches.

### P4. The API surface needs a coherent controller facade

The final reference architecture should let app authors ask for one clear surface for:

- viewport manipulation,
- controlled graph replacement/synchronization,
- common graph queries,
- canonical edit/update entry points,
- subscription and callback wiring.

A first minimal slice is now landed in `ecosystem/fret-node/src/ui/controller.rs` as
`NodeGraphController`:

- it wraps `NodeGraphStore`,
- it provides common query helpers and transaction-safe commit helpers,
- it now includes the first bounds-aware viewport helpers (`set_center_in_bounds*`,
  `fit_view_nodes_in_bounds*`) plus a canvas-rect framing helper (`fit_canvas_rect_in_bounds*`),
  plus viewport projection helpers (`screen_to_canvas`, `canvas_to_screen`), so paint-only /
  fallback hosts can drive viewport state without requiring a retained widget queue,
- declarative keyboard/wheel/pinch/pan viewport updates have started converging on the same
  controller/store-backed view-state path when a controller/store is present,
- the deferred `fit-to-portals` viewport application path now also uses that same view-state
  replacement route,
- declarative selection and marquee preview/cancel flows have also started converging on
  controller/store-backed selection helpers when a controller/store is present,
- diagnostics-only paint-only graph hotkeys now also build/commit transactions instead of teaching
  direct `Graph` mutation,
- it can sync external graph/view models from store after commits,
- `NodeGraphSurfaceBinding` now also exposes the full store-first viewport helper family for
  declarative action hooks and common instance-style app code (`set_viewport*`,
  `set_center_in_bounds*`, `fit_view_nodes_in_bounds*`, including option-bearing variants), so
  first-party controls do not need to teach raw view queues or explicit controller wiring for
  routine viewport work,
- `NodeGraphSurfaceBinding` now also mirrors common instance-style edit/sync/history helpers
  (`dispatch_transaction*`, `submit_transaction*`, `replace_*_action_host`,
  `set_selection_action_host`, `undo_action_host`, `redo_action_host`), so object-safe app hooks
  no longer need to bypass the binding for routine bound-store coordination,
- declarative `paint_only` routine action/UiHost hooks now also consume the binding facade for
  transaction commit, selection commit, keyboard zoom, diagnostics presets, pointer release/move
  flows, and fit-to-portals viewport updates, so internal declarative orchestration starts from the
  same binding-first contract taught to apps,
- declarative rename, portal command, blackboard, controls, minimap, and toolbar paths now have
  focused controller/binding-facing gates,
- the default declarative demo now uses it.

This is intentionally not the final shape yet. Richer viewport commands, callback layering, and
broader declarative closure are still open, but the public naming/ownership story is now closed
around `NodeGraphSurfaceBinding` plus `NodeGraphController`. `edit_queue` is no longer a public
teaching surface, and source-policy tests now require retained compatibility terms and raw queue
transport to stay out of supported UI sources.

Source-policy tests now lock that posture across declarative surfaces, first-party demos, workflow
gallery snippets, and public docs.
The explicit advanced binding constructor is now named
`NodeGraphSurfaceBinding::from_models_and_controller(...)`, and it now requires explicit
`graph + view_state + editor_config + controller` ownership, so mirror-owned/controller-owned
wiring does not read like a routine convenience constructor or silently fall back to default config.
`NodeGraphSurfaceBinding` itself is now split across `binding.rs` plus focused companion modules
(`binding_queries.rs`, `binding_store_sync.rs`, `binding_viewport.rs`), and source-policy tests now
aggregate that surface instead of forcing the contract to live in one growing file.
Queue-first APIs such as `NodeGraphEditQueue` are no longer public app-facing seams. Raw edit/view
transport is crate-internal only, and the temporary `NodeGraphViewportHelper` facade is deleted, so
app-facing composition can stay on either the instance-style
`NodeGraphSurfaceBinding::{set_viewport*, set_center_in_bounds*, fit_view_nodes_in_bounds*,
fit_canvas_rect_in_bounds*, screen_to_canvas, canvas_to_screen}` family or the lower-level
`NodeGraphController::{set_viewport*, set_center_in_bounds*, fit_view_nodes_in_bounds*,
fit_canvas_rect_in_bounds*, screen_to_canvas, canvas_to_screen}` surface, while declarative
action hooks should prefer the matching `NodeGraphSurfaceBinding::*_action_host(...)` helpers over
owning raw transport queues.
Raw edit/view queue transport is no longer a current public concept in this crate; new helper work
should stay on binding/controller/store APIs.

`fret_node::ui::advanced::*` is now deleted, and root `fret_node::ui::*` no longer exposes the raw
queue/helper surfaces. First-party demos stay controller/binding-first, while retained/test callers
use explicit crate-internal module paths as needed; viewport option types still stay on the root
`ui::*` surface without exposing the raw view queue itself, but those root types now come from a
dedicated store-first module and no longer expose queue-era animation fields that only retained
transport still consumes internally.
Because this repo does not need a public compatibility window, the old root queue/helper aliases are
removed outright instead of going through a deprecation cycle.
Current controller-facing XyFlow mapping (review helper, not a final contract):

- viewport read:
  - XyFlow mental model: `useReactFlow().getViewport()`
  - current Fret surface: `NodeGraphController::viewport`
- viewport set/reset:
  - XyFlow mental model: `setViewport`, `setCenter`
  - current Fret surface: `NodeGraphController::set_viewport*`, `set_center_in_bounds*`
- fit view:
  - XyFlow mental model: `fitView`, `fitBounds`
  - current Fret surface:
    `NodeGraphController::fit_view_nodes_in_bounds*`, `fit_canvas_rect_in_bounds*`
- coordinate projection:
  - XyFlow mental model: `screenToFlowPosition`, `flowToScreenPosition`
  - current Fret surface:
    `NodeGraphController::screen_to_canvas`, `canvas_to_screen`
- node / handle connections:
  - XyFlow mental model: `getNodeConnections`, `getHandleConnections`
  - current Fret surface: `NodeGraphController::node_connections`, `port_connections`
- node / edge metadata updates:
  - XyFlow mental model: `updateNode`, `updateEdge`
  - current Fret surface: `NodeGraphController::update_node*`, `update_edge*`
  - contract note: these helpers expose `NodeGraphNodeUpdate` / `NodeGraphEdgeUpdate` drafts
    rather than raw `Node` / `Edge`, so structural port ordering and edge endpoint rewiring stay on
    explicit transactions
- graph replacement / transaction-safe updates:
  - XyFlow mental model: imperative instance/store writes
  - current Fret surface: `replace_graph`, `submit_transaction*`,
    `submit_transaction_and_sync_*`, `dispatch_transaction*`
- still open:
  - whether diff-first controlled sync earns a public helper beyond the full-replace-first contract.

### P5. The workstream itself must stay reviewable

The previous workstream captured a lot of useful implementation evidence, but parts of it became
too granular. The updated docs should keep the important evidence and gates while focusing reviewers
on the next architectural decisions.

## Target architecture

The target architecture remains aligned with ADR 0126, but with sharper boundaries.

### A. Asset layer: long-lived graph document + reversible edits

This remains the non-negotiable center:

- `Graph`
- `GraphOp`
- `GraphTransaction`
- `GraphHistory`
- `rules`
- `profile`
- diagnostics

This layer should stay portable, serializable, and editor-agnostic.

### B. Runtime layer: editor state + controller + queries

This layer should converge on:

- `NodeGraphStore` as the transaction-aware state owner,
- `NodeGraphLookups` as the canonical fast-query substrate,
- the `NodeGraphSurfaceBinding` + `NodeGraphController` pair as the ergonomic app-facing facade,
- explicit controlled-mode helpers, with full replace as today's canonical sync path and diff-driven replace deferred.

Target state split:

- `NodeGraphViewState`
  - pan
  - zoom
  - selected nodes/edges/groups
  - draw order
- `NodeGraphInteractionConfig`
  - selection/drag/connect key policy
  - connection mode
  - pan/zoom activation settings
  - editor interaction toggles
- `NodeGraphRuntimeTuning`
  - spatial index tuning
  - cache pruning
  - expensive runtime knobs

Exact names may change during implementation, but the split itself is part of the workstream.

### C. Surface layer: declarative-first public UI

The intended public story is:

- apps compose a declarative node-graph surface,
- the surface talks to the controller/store,
- visible node content uses portal-based composition over time,
- overlays and editor chrome stay explicit and testable.

This means the final recommended surface should look like a declarative element-first entrypoint,
not a retained widget constructor.

### D. Compatibility layer: retained internal engine, feature-gated

The retained path remains acceptable only when it satisfies all of the following:

- it is hidden behind declarative entrypoints where possible,
- it remains feature-gated,
- it does not expand the long-term public API footprint,
- it has explicit exit criteria.

### E. Policy layer: recipes stay out of mechanism by default

`fret-node` may keep editor-specific mechanisms, but:

- overlay dismissal policy,
- design-system row sizing,
- recipe spacing/padding,
- shadcn/material chrome defaults,

should still live in the proper ecosystem policy/recipe layers unless there is a clear contract case
for keeping them local.

## Recommended authoring posture

This is the part downstream authors should be able to follow without reading the whole repo.

### Recommended today for shipping editor-grade workflows

Use a **declarative root surface** backed by the authoritative store/controller/binding surfaces.

Concretely:

- prefer declarative composition at the app boundary,
- prefer binding-first declarative integration (`NodeGraphSurfaceBinding` + `node_graph_surface(...)`),
- prefer controller-driven commands and store-backed transactions,
- do not add new public escape hatches that expose raw mutable graph/view-state ownership or stale
  retained compatibility transport.

### Golden path for new app code

For new editor surfaces, teach and copy this shape first:

1. create one `NodeGraphSurfaceBinding::new(models, graph, view_state, editor_config)`,
2. render `node_graph_surface(cx, binding.surface_props())` for the default surface props,
3. use the binding itself for common app-facing helpers (`viewport`, `graph_snapshot`,
   `view_state_snapshot`, `set_viewport`, `set_center_in_bounds`, `fit_view_nodes_in_bounds`,
   `dispatch_transaction`, `submit_transaction`, `replace_document`, `replace_graph`,
   `replace_view_state`, `set_selection`, `outgoers`, `incomers`, `connected_edges`,
   `port_connections`, `node_connections`, `undo`, `redo`),
4. when retained/compat composition or lower-level controller APIs are required, construct
   `NodeGraphController::new(binding.store_model())` explicitly,
5. treat raw graph/view models as advanced seams rather than the default teaching surface.

This is the public teaching surface now used by `apps/fret-examples/src/node_graph_demo.rs`.
When retained composition is still required, keep the controller explicit at the composition site;
do not hide that advanced seam behind routine binding helpers.

For controlled sync, the current canonical posture is **full replace first**: use
`binding.replace_document(...)` (or the controller's sync helper) when external authority swaps the
whole graph document and wants a fresh history boundary; keep `replace_graph(...)` as the graph-only
helper when view/history policy should stay caller-controlled, and treat diff-first replace helpers
as a later optimization rather than the starting contract.

### Recommended target posture

Ship a declarative editor-grade surface whose committed edits flow through transactions/store and
whose node content progressively moves toward portal-based declarative composition.

### Removed compatibility posture

The old retained compatibility feature and widget surface are no longer current public targets for
this workstream. Remaining retained/compatibility references in the long-form history should be read
as historical evidence, not as implementation guidance.

Current requirements:

- `NodeGraphSurfaceBinding` + `node_graph_surface(...)` remains the default documented app-facing
  path,
- committed declarative edits and viewport changes stay transaction-safe and controller/store-driven,
- declarative gates cover editor-grade overlay, portal, minimap, controls, blackboard, and rename
  flows without requiring a retained compatibility feature,
- no new public app-facing APIs depend on retained widget types, raw queue ownership, or raw mutable
  view-state references.

### Current follow-up blockers

The remaining blockers should be tracked as concrete public-surface or behavior gates:

- whether the remaining broad `NodeGraphPresenter` responsibilities should split into narrower
  default-path label/geometry/menu/search contracts, or stay advanced-only,
- whether broader EdgeWrapper lifecycle policy should be the next edge-wrapper parity slice,
- whether paint/style tokens still leak geometry or hit-testing policy,
- whether `prepare_surface_frame` should be split further around frame plan, portal measurement,
  a11y/internals publication, and diagnostics.

### Comparison checklist: declarative vs compatibility retained

Review these flows whenever a change claims declarative parity or adds retained-only work:

- viewport interactions: pan / wheel zoom / pinch / fit-view,
- transaction-safe node drag and committed selection / marquee flows,
- portal bounds, hover anchors, and fit-to-portals behavior,
- rename / blackboard / toolbar / minimap editor chrome,
- diagnostics and conformance coverage for the same user-visible behavior.

### API red lines

Do not add or normalize any of the following as long-term best practice:

- public constructors that require retained types,
- editor-grade interactions that commit by mutating `Graph` directly,
- new UI-policy defaults hidden in mechanism code,
- tutorial/demo guidance that implies retained authoring is the normal downstream path.

Any new retained-only addition should document:

- why `node_graph_surface(...)` cannot host it yet,
- which gate or parity test will track the gap,
- what the exit path is back to the declarative teaching surface.

## Reviewer checklist

A reviewer should be able to answer "yes" to all of these in under five minutes.

- Is the recommended **today** posture clear: declarative root surface first, controller/store
  integration first, with no public retained compatibility escape hatch?
- Is the **target** posture clear: declarative editor-grade surface with transaction-safe commits and
  progressively more declarative node content?
- Does the change avoid reintroducing direct retained widget authoring, raw queue transport, or raw
  mutable store access as a downstream story?
- If a declarative gesture commits graph or view-state data, does it route through
  controller/store/transaction entry points instead of mutating `Graph` directly?
- Does the change avoid pushing more interaction policy or runtime tuning into
  `NodeGraphViewState`?
- Do examples/docs keep the binding-first declarative surface as the only recommended downstream
  entrypoint?

## Wording audit snapshot

- `docs/workstreams/xyflow-gap-analysis.md` is aligned with this workstream's public recommendation.
- First-party examples and source-policy tests keep direct retained widget authoring out of the
  recommended downstream recipe.

## Deliverables expected from this workstream

This workstream is complete only when it leaves behind:

1. **Clear documentation**
   - one canonical authoring recommendation,
   - one milestone plan reviewers can evaluate,
   - one TODO list that is small enough to execute in slices.

2. **Architectural closure**
   - state boundaries are explicit,
   - controller surface is explicit,
   - compatibility retained path is clearly bounded.

3. **Regression protection**
   - keep the existing useful cache/portal/interaction gates,
   - add transaction-safe declarative gates as the new behavior lands,
   - preserve editor-grade correctness under undo/redo and controlled sync.

## What this workstream intentionally does not do

- It does **not** propose rewriting the graph model away from map-based, long-lived documents.
- It does **not** propose splitting `fret-node` into multiple crates immediately.
- It does **not** require deleting all retained code before declarative architecture is ready.
- It does **not** try to solve all visual recipe/theming work in the same pass.

## Primary references

- Node graph contract: `docs/adr/0126-node-graph-editor-and-typed-connections.md`
- Declarative runtime direction: `docs/adr/0028-declarative-elements-and-element-state.md`
- Component authoring direction: `docs/adr/0039-component-authoring-model-render-renderonce-and-intoelement.md`
- Architecture overview: `docs/architecture.md`
- Node graph roadmap: `docs/node-graph-roadmap.md`
- XyFlow parity map: `docs/node-graph-xyflow-parity.md`
- XYFlow gap analysis: `docs/workstreams/standalone/xyflow-gap-analysis.md`

## Evidence anchors to preserve while refactoring

- `ecosystem/fret-node/src/runtime/store.rs`
- `ecosystem/fret-node/src/runtime/changes.rs`
- `ecosystem/fret-node/src/runtime/lookups.rs`
- `ecosystem/fret-node/src/surface_policy_tests.rs`
- `ecosystem/fret-node/src/ui/declarative/paint_only.rs`
- `ecosystem/fret-node/src/ui/declarative/paint_only/surface_frame.rs`
- `ecosystem/fret-node/src/ui/declarative/paint_only/transactions.rs`
- `ecosystem/fret-node/src/ui/portal_commands.rs`
- `ecosystem/fret-node/src/ui/overlays/blackboard_declarative.rs`
- `apps/fret-examples/src/node_graph_demo.rs`
- `tools/diag-scripts/node-graph/`

## Minimal runnable targets and gates

Canonical runnable targets:

- default declarative demo: `cargo run -p fretboard-dev -- dev native --bin node_graph_demo`

### Compact gate matrix

| Gate | Command | Why it stays |
| --- | --- | --- |
| package conformance | `cargo nextest run -p fret-node` | keeps the public node-graph substrate, declarative surface, and source-policy gates green together |
| headless runtime | `cargo nextest run -p fret-node --no-default-features runtime` | protects store/change/history behavior without default UI features |
| optional integration compile | `cargo check -p fret-node --all-features --tests` | keeps UI-enabled and optional integration test targets compiling against the current public surface |
| example wiring smoke | `cargo check -p fret-examples` | keeps `node_graph_demo` compiling against the current public teaching surface |
| paint-only diagnostics | `cargo run -p fretboard-dev -- diag suite fret-examples-node-graph-paint-only --dir target/fret-diag-node-graph --launch -- cargo run -p fret-demo --bin node_graph_demo --features node-graph-demos` | protects cache, portal-bounds, hover-anchor, and paint-only scripted regressions |
| layering | `python tools/check_layering.py` | catches accidental boundary drift while the surface is still moving |

The TODO tracker defines the next gate additions still required for full transaction-safe declarative parity.
