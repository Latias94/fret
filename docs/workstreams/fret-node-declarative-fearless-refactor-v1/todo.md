# `fret-node` Fearless Refactor (v1) - TODO

This tracker is intentionally biased toward small, reviewable slices. Keep items concrete enough to
land in code review; move design discussion back to `README.md` if a TODO turns into prose.

Execution companion: `design.md` (surface map + next worktree order).

## Cross-cutting guardrails

- [x] Keep `compat-retained-canvas` out of default features.
- [x] Keep the compatibility retained path feature-gated and explicitly named.
- [x] Keep the default lightweight declarative demo path (`node_graph_demo`).
- [x] Keep the workstream docs aligned with the actual public recommendation after each milestone.
- [x] Keep diagnostics-driven example hosts aligned with the current `UiDiagnosticsService`
      script-driving surface when syncing from `main`.
- [x] Publish and maintain an L0 crate audit note (`docs/workstreams/crate-audits/fret-node.l0.md`).
- [x] Update ADR alignment or ADR text when a hard contract changes rather than hiding the change in
      implementation-only diffs.

## Cross-cutting red lines

- [ ] Do not add new long-term public APIs that require retained widget types.
- [ ] Do not bless direct `Graph` mutation as the editor-grade commit path for declarative surfaces.
- [ ] Do not expand `NodeGraphViewState` with more policy or tuning fields.
- [ ] Do not solve recipe/policy gaps by smuggling new defaults into mechanism code.

## M0 - Decision gates and internal seam map

- [x] Reframe the workstream docs around architecture closure rather than a paint-only lab log.
- [x] Preserve the historical folder path to avoid breaking references.
- [x] Add one short "current hazards" section with evidence anchors for:
  - direct graph mutation in the declarative path,
  - overgrown `NodeGraphViewState`,
  - missing controller facade.
- [x] Add one short reviewer checklist to the README so a reviewer can verify the intended posture
      in under five minutes.

## M1 - Public posture and API closure

- [x] State explicitly in docs which path is recommended **today** for shipping editor-grade usage.
- [x] State explicitly in docs which path is the **target** posture after this refactor.
- [x] State explicitly in docs when `compat-retained-canvas` is acceptable.
- [x] Document that direct retained `NodeGraphCanvas` authoring is internal-only except for tests,
      diagnostics, or temporary compatibility harnesses.
- [x] Audit examples and internal guides for wording that still suggests retained authoring is the
      normal downstream path.
- [x] Introduce the canonical wrapper naming: `NodeGraphSurfaceProps` + `node_graph_surface(...)`,
      while keeping `node_graph_surface_compat_retained(...)` explicit as the compatibility path.

## M2 - State boundary split

- [x] Shrink `NodeGraphViewState` to true view state only in non-test/runtime builds:
  - pan
  - zoom
  - selected nodes/edges/groups
  - draw order
- [x] Introduce a separate interaction config type for:
  - selection/connect/drag toggles
  - connection mode
  - key semantics and activation policy
  - interaction defaults that are not pure view state
- [x] Introduce a separate runtime tuning type for:
  - spatial index tuning
  - cache prune tuning
  - expensive runtime knobs
- [x] Land the first persisted split slice:
  - `NodeGraphViewStateFileV1.state` now stores pure `NodeGraphViewState`
  - wrapper-owned `interaction` stores `NodeGraphInteractionConfig`
  - wrapper-owned `runtime_tuning` stores `NodeGraphRuntimeTuning`
  - runtime/widget code resolves `NodeGraphInteractionState` from explicit editor-config seams
  - legacy serialized `interaction` payloads still migrate at load time
- [x] Decide where these new types live and who owns persistence for them.
- [x] Design the migration/compat strategy for existing serialized `NodeGraphViewState` payloads.
- [x] Update store code and tests so the new boundary is explicit in subscriptions and controlled
      synchronization.
- [x] Add focused tests for serialization migration and store behavior after the split.
- [x] Move app/example persistence and overlay authoring to the explicit `NodeGraphEditorConfig`
      seam.
  - `node_graph_legacy_demo` / `node_graph_domain_demo` now load and save
    `NodeGraphViewStateFileV1` through `new(...)`.
  - `NodeGraphTuningOverlay` now reads/writes `NodeGraphEditorConfig` instead of mutating
    `NodeGraphViewState`.
  - `NodeGraphControlsOverlay`, retained canvas, and declarative bindings now consume an explicit
    editor-config mirror where the app surface owns one.
  - `NodeGraphControlsOverlay::new(...)` now requires the editor-config model directly instead of
    teaching a follow-up `with_editor_config_model(...)` builder.
- [x] Keep retained public compatibility constructors explicit about editor-config ownership.
  - `NodeGraphCanvas::new(...)` / `new_with_middleware(...)` now require
    `Model<NodeGraphEditorConfig>`.
  - retained test fixtures now also inject editor-config ownership at construction time instead of
    relying on a follow-up retained-widget builder seam.
  - Retained test fixtures now insert host-owned default editor-config models explicitly instead of
    relying on widget-local fallbacks.
  - Retained widget test harnesses now also expose combined `graph + view + editor_config` helpers,
    so focused conformance gates can stay explicit about editor-config ownership without repeating
    the same host/model setup block in every file.
  - Overlay-only retained harnesses now also expose combined `view + editor_config` helpers, so
    controls/minimap-style gates can reuse the same explicit editor-config seam without
    re-assembling view/config bootstrap in each test.
  - Edge label/marker/cache, edge-insert, paint-overrides, skin, invalidation,
    selection/preview/semantic-zoom, measured/spatial, and a11y/fit-view/connection-validity
    retained conformance gates now also route their `graph + view + editor_config` bootstrap
    through the shared harness seam, so renderer-focused, edge-insert interaction, paint-only
    parity, skin-chrome, invalidation, selection/preview, measured/spatial, and a11y/fit-view
    tests keep explicit editor-config ownership without teaching more local setup variants.
  - Background-style/color-mode, custom-edge-path, derived-geometry invalidation,
    edit-command-availability, escape-cancel, and insert-node-drag retained conformance gates now
    also route their `graph + view + editor_config` bootstrap through the shared harness seam, so
    canvas chrome/theme sync, custom-path hit-testing, derived-geometry cache invalidation,
    command-availability gating, pointer-capture cancel, and insert-node threshold coverage stop
    teaching the implicit default editor-config path.
  - Drag-preview, node-resize-preview, overlay invalidation, and overlay menu/searcher retained
    conformance gates now also route their `graph + view + editor_config` bootstrap through the
    shared harness seam, so preview cache reuse, preview geometry/index drift, overlay-only
    invalidation, and overlay clamp coverage stay on the same explicit editor-config contract.
  - Callback-oriented retained conformance gates now also route connect/reconnect, pan, and
    node-drag callback bootstrap through the shared `graph + view + editor_config` harness seam,
    so gesture/view callback coverage no longer teaches implicit default editor-config ownership.
  - Hit-testing and internals retained conformance gates now also route target-port picking,
    edge/anchor hit resolution, internals snapshot publication, and internals/measured-output
    stability coverage through the shared `graph + view + editor_config` harness seam, so those
    geometry-facing tests stop teaching implicit default editor-config ownership.
  - Insert-node-drag-drop, middleware, op-batching determinism, portal measured-internals,
    set-viewport queue, and perf-cache-prune retained conformance gates now also route their
    `graph + view + editor_config` bootstrap through the shared harness seam, so drag-drop,
    middleware rejection, group-op batching, portal-measurement-to-internals, view-queue viewport,
    and cache-prune coverage stop teaching implicit default editor-config ownership.
  - Perf-cache retained coverage now also routes static node/edge cache reuse, tile-boundary
    reuse, incremental edge-label/marker warmup, and repeated-label auto-measure bootstrap through
    the shared `graph + view + editor_config` harness seam, so renderer-cache-focused tests stop
    teaching the implicit default editor-config path.
  - Interaction-conformance and the remaining root retained widget tests now also route their
    `graph + view + editor_config` bootstrap through the shared harness seam, and the now-unused
    implicit `make_host_graph_view(...)` helper plus the test-only 3-arg `new_canvas!(...)` arm
    are deleted from the retained test harness, so this lane no longer leaves any retained
    conformance teaching surface on the implicit default editor-config path.
- [x] Delete the temporary `cfg(test)` bridge that mirrors `NodeGraphEditorConfig` back into
      `NodeGraphViewState` for old tests.
  - `NodeGraphViewState` test helpers now stay pure view-state.
  - Store/controller sync tests now bind explicit editor-config mirrors.
  - Retained/declarative fixtures no longer read `view_state.interaction` as a compatibility path.
  - `ui/canvas/widget/view_state/sync.rs` no longer keeps a `cfg(test)` fallback that reconstructs
    `NodeGraphEditorConfig` from `NodeGraphViewState`; retained runtime/tests and
    `--all-features` builds now all resolve editor config from the explicit mirror seam only.

## M3 - Controller / instance facade

- [x] Introduce a first thin controller facade in `ecosystem/fret-node/src/ui/controller.rs`.
- [x] Make the first landing slice responsible for an ergonomic app-facing surface for:
  - viewport read / set-viewport glue
  - controlled graph replacement / synchronization
  - common graph queries (`outgoers`, `incomers`, `connected_edges`)
  - canonical transaction-safe update entry points
- [x] Wire the default declarative demo to use `NodeGraphController` instead of teaching raw store
      plumbing directly.
- [x] Collapse declarative surface wiring to one app-facing bundle (`NodeGraphSurfaceBinding`) so
      demos and docs stop teaching raw `graph + view_state + controller` triplets.
- [x] Extend the controller surface with the first bounds-aware viewport helpers:
  - `set_center_in_bounds` / `set_center_in_bounds_with_options`
  - `fit_view_nodes_in_bounds` / `fit_view_nodes_in_bounds_with_options`
  - public fit-view now stays bounds-aware only; queue-driven unbounded fit-view helpers are
    removed instead of surviving as misleading no-op facade surface
- [x] Start routing declarative viewport interactions through controller/store-backed view-state
      replacement when a controller/store exists:
  - keyboard zoom
  - wheel zoom / pinch zoom
  - drag-pan viewport updates
  - diagnostics viewport normalization hotkeys
  - deferred `fit-to-portals` viewport application in the render/portal pass
- [x] Start routing declarative selection commits through controller/store-backed selection helpers
      when a controller/store exists:
  - click-to-select / toggle-select
  - empty-click clear selection
  - marquee preview selection updates
  - marquee cancel / escape restore
- [x] Stop teaching diagnostics-only direct graph mutation in `paint_only` hotkeys:
  - the `Digit3/4/5` deterministic graph tweaks now build transactions from `graph_diff`
  - those transactions now commit through the same controller/store transaction path when present
- [x] Extend the controller surface further for broader imperative viewport choreography beyond the
      first bounds-aware helper set.
  - Progress: controller/binding now also expose `fit_canvas_rect_in_bounds*`, lifting the
    declarative-only fit-rect viewport math into a store-first runtime/controller helper so
    XyFlow-style `fitBounds` framing no longer depends on paint-only-local reducers.
  - Progress: controller/binding now also expose `screen_to_canvas` / `canvas_to_screen`, so
    app-facing viewport choreography no longer has to reach into retained/declarative-local view
    math just to project pointer positions or anchors.
  - Progress: focused controller/binding gates now also lock viewport read/projection plus
    `set_viewport*`, `set_center_in_bounds*`, `fit_view_nodes_in_bounds*`, and
    `fit_canvas_rect_in_bounds*`, so the current controller-facing XyFlow viewport mapping is no
    longer an open-ended breadth item; future helper additions should land as separate,
    evidence-backed follow-ons.
- [x] Split the public viewport option surface from retained queue-era animation transport.
  - Progress: root `ui::*` now re-exports `NodeGraphFitViewOptions` /
    `NodeGraphSetViewportOptions` from a dedicated store-first module, while `view_queue.rs` keeps
    queue-only animation overrides crate-internal.
- [x] Keep `NodeGraphSurfaceBinding` complete enough for routine instance-style viewport authoring.
  - Progress: binding now mirrors `set_viewport*`, `set_center_in_bounds*`,
    `fit_view_nodes_in_bounds*`, and `fit_canvas_rect_in_bounds*` with both option-bearing and
    action-host variants, so ordinary viewport hooks no longer need to drop to explicit controller
    construction.
- [x] Keep `NodeGraphSurfaceBinding` complete enough for routine instance-style bound-store
      edit/sync/history hooks.
  - Progress: binding now mirrors `dispatch_transaction*`, `submit_transaction*`,
    `replace_*_action_host`, `set_selection_action_host`, `undo_action_host`, and
    `redo_action_host`, so object-safe app hooks no longer need to bypass the binding for routine
    bound-model synchronization.
- [x] Require explicit editor-config ownership even on the advanced mirror-owned binding seam.
  - Progress: `NodeGraphSurfaceBinding::from_models_and_controller(...)` now requires
    `graph + view_state + editor_config + controller`, `editor_config_model()` is always present,
    and store-sync / replace-document helpers no longer fall back to `NodeGraphEditorConfig::default()`.
- [x] Split `NodeGraphSurfaceBinding` by responsibility instead of letting `binding.rs` regrow as a
      god file.
  - Progress: viewport helpers now live in `binding_viewport.rs`, queries in
    `binding_queries.rs`, store-sync/history helpers in `binding_store_sync.rs`, while source-policy
    tests aggregate the full binding surface so the public contract is no longer tied to single-file
    placement.
- [x] Start converging declarative `paint_only` routine hooks on the same binding-first facade
      taught to app code.
  - Progress: `paint_only` transaction commit, selection commit, diagnostics preset/keyboard zoom,
    pointer release/move helpers, and fit-to-portals viewport updates now consume
    `NodeGraphSurfaceBinding`, so internal declarative orchestration no longer rethreads `graph +
    view_state + controller` for ordinary bound-surface work.
- [x] Decide whether `view_queue` stays as the transport for imperative viewport requests or becomes
      an internal retained compatibility detail.
  - Decision: raw viewport queue transport now lives under
    `ui/canvas/widget/view_queue.rs`, so it is retained-canvas-local compatibility plumbing rather
    than a root `ui` concept or app-facing controller surface.
  - Progress: retained canvas / minimap composition can now bind through `NodeGraphController`, so
    new app/UI glue no longer needs to teach raw queue mutation first.
  - Progress: public viewport options no longer expose retained-only `duration/ease/interpolate`
    knobs, so app-facing viewport authors stay on controller/binding-first store-backed options.
- [x] Decide whether `edit_queue` stays public, becomes controller-owned, or is limited to internal
      composition seams.
  - Decision: raw edit transport is now crate-internal only; first-party demos no longer own
    `NodeGraphEditQueue`, and controller-first submission is the only taught app-facing path.
  - Progress: the remaining raw edit transport now lives under `ui/compat_transport.rs`, making it
    an explicit retained compatibility detail instead of a root `ui` module concept.
- [x] Collapse the remaining legacy-demo-only raw `edit_queue` command hotkey (`Bump Float Value`)
      into controller-owned submission helpers so example code stops teaching ad-hoc queue mutation.
- [x] Normalize retained controller binding APIs toward `new + with_*` composition where it improves
      teaching clarity, instead of growing more parallel constructor names.
  - Progress: retained edit glue now prefers controller-owned submission helpers when a controller is
    available:
    - `NodeGraphCanvas::with_controller` now carries store-backed controller state only; explicit
      queue transport stays on crate-private retained seams.
    - `NodeGraphPortalHost::with_controller` submits transactions through the controller before
      falling back to raw queue transport.
    - `NodeGraphOverlayHost::new(...).with_controller(...)` and `compat_retained` now teach
      controller-first rename / portal composition instead of requiring raw queue mutation at the
      app boundary, and `compat_retained` now takes a controller binding instead of exposing public
      `edit_queue` / `view_queue` transport props.
    - `NodeGraphSurfaceCompatRetainedProps::new(...)` now also takes the editor-config model, so
      the compatibility shell keeps the explicit editor-config seam when it hosts the retained
      canvas internally.
    - `NodeGraphBlackboardOverlay` now supports controller-first retained symbol actions, while raw
      queue fallback remains crate-internal for compatibility harnesses and focused retained tests.
    - `apps/fret-examples/src/node_graph_legacy_demo.rs` now uses the same controller-first canvas /
      overlay / blackboard / portal / minimap wiring and no longer keeps a demo-owned
      `NodeGraphEditQueue`; remaining queue ownership is limited to crate-internal compatibility
      transport seams that still need explicit queue binding.
- [x] Land the first XyFlow-style connection-query mapping on the controller surface:
  - `NodeGraphController::node_connections`
  - `NodeGraphController::port_connections` (XyFlow `getHandleConnections` analogue)
- [x] Document the current XyFlow-style viewport/controller mapping in the workstream README.
- [x] Add a clear mapping from the remaining XyFlow-style expectations to the controller API:
  - Landed via `NodeGraphController::{update_node*, update_edge*}` and the matching
    `NodeGraphSurfaceBinding` helpers.
  - Contract: those helpers now use `NodeGraphNodeUpdate` / `NodeGraphEdgeUpdate` drafts rather
    than raw `Node` / `Edge`, so structural port edits and endpoint rewires still require explicit
    transactions.
- [x] Decide the long-term public naming/ownership story before widening the teaching surface
      further.
  - Decision: `NodeGraphSurfaceBinding` is the instance-style app-facing bundle, while
    `NodeGraphController` remains the lower-level imperative/runtime facade. Advanced retained
    composition stays explicit via `NodeGraphController::new(binding.store_model())` plus
    `with_controller(...)`; the binding no longer hides controller ownership.

### Retained transport seam audit (snapshot 2026-03-06)

- **Retained compatibility seams now stay crate-internal**
  - `NodeGraphCanvas::with_view_queue`
    (`ecosystem/fret-node/src/ui/canvas/widget.rs`): still needed because the retained canvas is the
    compatibility root that drains queue transport directly, and several focused retained tests still
    exercise that path; the methods are now crate-private so they stop reading like public app-facing API.
  - `NodeGraphPortalHost::with_edit_queue`
    (`ecosystem/fret-node/src/ui/portal.rs`),
    `NodeGraphOverlayHost::with_edit_queue`
    (`ecosystem/fret-node/src/ui/overlays/group_rename.rs`), and
    `NodeGraphBlackboardOverlay::with_edit_queue`
    (`ecosystem/fret-node/src/ui/overlays/blackboard.rs`): remain as fallback bindings for retained-only
    compatibility harnesses that still do not own a store/controller, but are no longer public API.
  - `NodeGraphMiniMapOverlay`
    (`ecosystem/fret-node/src/ui/overlays/minimap.rs`): now only teaches `Default`, `Disabled`,
    or `Controller` navigation; queue-owned viewport transport must flow through a controller if an
    integration still needs it.

- **Landable follow-ups from this audit**
  - [x] Delete `NodeGraphViewportHelper`; viewport authoring now stays on controller/binding
        surfaces.
  - [x] Remove the temporary public `fret_node::ui::advanced::*` edit seam; raw edit/view
        transport now stay crate-internal.
  - [x] Make raw view-queue transport crate-internal; root `ui::*` now only re-exports viewport
        option types.
  - [x] Demote retained widget / overlay queue binding methods to crate-private compatibility seams.
  - [x] Remove root `fret_node::ui::*` queue/helper re-exports from the public surface.
  - [x] Add one short README/workstream note that raw queue APIs are crate-internal compatibility
        seams, not the default app-facing integration surface.
  - [x] Clear first-party demo uses of raw edit transport (`node_graph_domain_demo`,
        `node_graph_legacy_demo`).
  - [x] Stop teaching raw view queues in the workflow gallery snippet; use
        `NodeGraphSurfaceBinding::*_action_host(...)` instead.
  - [x] Delete the minimap-specific `ViewQueue` navigation mode; queue transport now flows through
        `NodeGraphController` when explicitly needed.
  - [x] Clear in-tree uses of root queue/helper aliases (apps, gallery snippet, crate-internal retained/tests).
  - [x] Skip the external compatibility/deprecation phase and remove the old root aliases directly.
  - [x] Add source-policy coverage that retained public widget/overlay composition stays
        controller-only, while raw queue fallback remains crate-internal compatibility plumbing.





## M3 - Transaction-safe declarative commits

- [x] Land the first declarative transaction-safe commit slice:
  - `ecosystem/fret-node/src/ui/declarative/paint_only.rs` node-drag commit now builds a
    `GraphTransaction` instead of mutating `Graph` in place.
  - The commit now dispatches through `NodeGraphController` (store-backed) and syncs graph/view
    models back from store.
  - `NodeGraphSurfaceProps` now takes a single `NodeGraphSurfaceBinding`.
    `NodeGraphSurfaceProps.store` and the no-controller fallback path remain removed.
- [x] Wire `apps/fret-examples/src/node_graph_demo.rs` to provide a `NodeGraphSurfaceBinding` so
      the default declarative demo path exercises the transaction-safe architecture without
      teaching raw graph/view/controller plumbing.
- [x] Add a focused regression test for the drag transaction builder used by the declarative path.
- [x] Lock declarative `paint_only` runtime source ownership to the authoritative store-backed
      seam.
  - Progress: source-policy coverage now scans `paint_only.rs` plus the private runtime submodules
    and forbids `binding.graph_model()`, `binding.view_state_model()`, and
    `binding.editor_config_model()` access there; bound surfaces must read/write authoritative
    graph/view/config through `binding.store_model()` instead.
- [x] Keep future declarative graph-edit commit paths on the same transaction-backed seam used by
      node-drag and diagnostics, rather than reintroducing direct graph mutation when new gestures
      land.
  - Status note: the currently landed declarative graph-edit commit paths are node-drag plus the
    diagnostics graph-diff actions.
  - Progress: authoritative store-first source ownership is now locked, and graph-edit commit
    authority is now also locked to `paint_only/transactions.rs`; runtime files cannot replace the
    graph/document directly or dispatch/submit transactions outside that seam.
- [x] Keep ephemeral drag/hover session state local where that improves ergonomics, but route final
      commits through transactions.
  - Progress: marquee, pending-selection, node-drag, and hover session state now stay local until
    commit/cancel, while final selection commits route through controller/store-backed selection
    seams and final graph edits route through `GraphTransaction` submission helpers.
- [x] Add undo/redo coverage for the declarative path once commits stop mutating `Graph` directly.
  - Landed via focused paint-only coverage: controller-backed node-drag commit now proves
    history records the transaction, undo re-syncs graph/view models, and redo restores the same
    committed graph state.
- [x] Add at least one gate proving that a declarative drag or marquee commit produces a
      transaction-safe update path.
  - Landed via `paint_only.rs` callback gates: controller-backed node-drag commit proves it
    dispatches through store commit callbacks, and controller-backed pending-selection / marquee
    commits prove they dispatch through store selection callbacks rather than only syncing local
    view models.
- [x] Define the policy for full replace vs diff-based replace in controlled mode.
  - Landed policy: full replace is the current canonical external-to-store sync path; diff-first
    helpers remain deferred until a concrete workload proves they are needed.
  - Landed helper: `NodeGraphSurfaceBinding::replace_document(...)` /
    `NodeGraphController::replace_document_and_sync_models(...)` now make whole-document reset
    semantics explicit (graph + view-state replace + history clear + mirror sync).
- [ ] Consider adding `replace_graph_with_diff` or equivalent if full reset semantics are not enough
      for editor-grade controlled integrations.

## M3 - Callback surface split

- [x] Split callback surfaces so reviewers can clearly distinguish:
  - headless/store commit callbacks,
  - view-state callbacks,
  - UI gesture lifecycle callbacks.
  - Landed as `NodeGraphCommitCallbacks`, `NodeGraphViewCallbacks`, and
    `NodeGraphGestureCallbacks` in `ecosystem/fret-node/src/runtime/callbacks.rs`.
- [x] Keep the main adapter seams stable where useful instead of reopening transport cleanup.
  - `install_callbacks(...)` and `NodeGraphCanvas::with_callbacks(...)` still consume the composite
    `NodeGraphCallbacks` surface.
- [x] Add one small note explaining which callback layer should be used by apps vs internal UI glue.
  - Controlled-mode docs and parity docs now teach app code = commit/view, retained glue = gesture.

## M4 - Declarative interaction closure

- [x] Migrate selection/marquee state machines toward declarative reducers with explicit commit and
      cancel semantics.
  - Landed first marquee slice: preview selection is now local transient state and pointer-up commits
    through the controller/store-backed selection seam.
  - Landed click-selection follow-up: hit-node selection and empty-space clear now also stay local
    until pointer-up (or node-drag activation).
  - Landed node-drag phase follow-up: local drag state now uses explicit `Armed` / `Active` /
    `Canceled` phases, threshold crossing commits pending selection exactly once, and pointer-up only
    emits a drag transaction for active non-zero delta drags.
  - Landed cancel/release follow-up: selection-only node-drag release now has a dedicated gate, and
    escape / pointer-cancel now have focused helper-backed tests (including the pending-selection-only
    escape case).
  - Landed left-release reducer follow-up: node-drag release, pending-selection commit, and
    marquee release now route through explicit helpers, with focused tests covering node-drag,
    pending-only, inactive-toggle-marquee, and no-state branch cleanup/commit semantics.
  - Landed pointer-session event follow-up: pointer-up and pointer-cancel now route
    through explicit helpers, with focused tests covering left-release finish, non-left ignore,
    pan-release cleanup, and cancel-finish semantics.
  - Landed declarative test-fixture follow-up: controller/store callback setup and
    pointer-session finish assertions now share small fixtures/helpers, reducing repetition in
    release/cancel coverage tests.
  - Landed private submodule follow-up: release/cancel/session host helpers now live in
    `paint_only/pointer_session.rs`, keeping the main paint-only surface file focused on
    orchestration while tests keep the same behavior gates.
  - Landed pointer-move submodule follow-up: drag/marquee/hover move helpers and their
    outcomes now live in `paint_only/pointer_move.rs`, shrinking the main surface file without
    changing the focused move-path gates.
  - Landed cancel/effects follow-up: Escape and pointer-cancel now share a mode-aware transient
    cleanup reducer, pointer session finish effects are routed through shared helpers, and focused
    tests lock the already-canceled node-drag divergence between Escape and pointer-cancel.
  - Landed keydown dispatch follow-up: `Escape`, diagnostics digits, and keyboard zoom now route
    through explicit action helpers, with focused tests for diag-key parsing, diag view presets,
    portal-disable cleanup, zoom reset, and paint-override toggling.
  - Landed pointer-down reducer follow-up: pan start, hit-node preview/drag arming, marquee
    arming, and empty-space clear now route through explicit snapshot/reducer helpers, with
    focused tests covering the pan, hit-node, marquee, and clear-selection branches.
  - Landed pointer-move reducer follow-up: node-drag activation/update, marquee preview, and
    hover hit-testing now route through explicit helpers, with focused tests covering drag
    activation, canceled drag cleanup, marquee preview/cancel, and hover hit updates.
  - Landed local-surface-state follow-up: drag/marquee/pending-selection/cache boundary types now
    live in the private `paint_only/surface_state.rs` seam, so the main declarative surface file
    keeps moving toward orchestration-only ownership instead of re-embedding every local session
    type inline.
  - Landed authoritative interaction-read follow-up: pointer-down snapshot, node-drag activation
    threshold, marquee preview, and hover hit-testing now read interaction/view baselines from the
    authoritative store via `NodeGraphSurfaceBinding`, with focused stale-bound tests proving local
    bound view mirrors no longer decide declarative interaction math.
  - Landed authoritative frame-view follow-up: frame preparation and canvas paint observation now
    also read view-state baselines from the authoritative store, so pan/zoom/selection-driven
    paint/layout paths no longer depend on the bound `view_state` mirror staying fresh.
  - Landed authoritative graph-read follow-up: paint-only frame assembly, cache rebuilds, portal
    hosting, hover-tooltip diagnostics, and visible-edge diagnostics now read the graph document
    from the authoritative store via `NodeGraphSurfaceBinding`, while graph-dependent cache/frame
    invalidation now keys off `NodeGraphStore::graph_revision()` instead of a stale bound
    `graph_model()` mirror.
  - Status note: the remaining declarative closure work is no longer reducer extraction itself;
    follow-on slices should target any still-missing transaction-backed graph edit path or the
    portal/overlay closure work, rather than reopening selection/marquee reducer structure.
- [x] Keep pointer-capture and cancel behavior as a first-class regression target while doing this.
  - Landed initial declarative gates for selection-only release, escape cancel, and pointer-cancel
    cleanup in `paint_only.rs`.
- [x] Decide which interaction pieces remain local surface state vs store-backed editor state.
  - Landed boundary: committed graph edits + viewport/selection/draw-order stay store-backed,
    while pan-drag / node-drag preview / marquee preview / pending click-selection / hover /
    hit-test scratch stay local to the declarative paint-only surface until commit/cancel.
  - Landed paint precedence helper: active marquee preview overrides pending selection preview,
    which overrides committed selection for paint/layout only.
- [x] Ensure new declarative interaction work does not regress cache discipline.
  - Landed paint-cache key gates: selection-only authoritative updates keep grid / derived / node / edge
    cache keys stable, while graph replacement invalidates only graph-dependent caches.
- [x] Add at least one parity gate meaningful to real editor usage, not just synthetic paint-only
      counters.
  - Landed authoritative-boundary gate: when controlled-mode authority replaces the graph document,
    declarative local transient state (pan / node-drag / marquee / pending-selection / hover /
    portal anchors) is cleared on the next frame; selection-only authoritative changes clear only
    selection-scoped preview state so viewport/hover caches do not regress.

## M4 - Portal and overlay closure

- [x] Move from portal/bounds experimentation toward a declared editor-grade portal hosting path for
      the visible subset.
  - Progress: declarative portal hosting now routes visible-subset selection through
    `collect_portal_label_infos_for_visible_subset(...)`, which locks draw-order/cap semantics and
    uses dragged node rects (not stale pre-drag rects) for viewport culling.
  - Progress: `NodeGraphSurfaceProps` now carries `NodeGraphVisibleSubsetPortalConfig` instead of
    loose portal booleans, so the public declarative surface explicitly names the visible-subset
    hosting contract and its cap semantics.
- [x] Clarify how node content subtrees publish measured geometry into derived stores.
  - Progress: portal subtree bounds harvest now routes through
    `sync_portal_canvas_bounds_in_models(...)`, giving `LayoutQueryRegion` publish semantics an
    explicit epsilon-filtered seam instead of ad-hoc inline store writes.
  - Progress: declarative surfaces can now opt into a shared `MeasuredGeometryStore` through
    `NodeGraphSurfaceProps.measured_geometry`, while portal subtree measurement publication routes
    through `record_portal_measured_node_size_in_state(...)` +
    `flush_portal_measured_geometry_state(...)` instead of inline ad-hoc store writes.
  - Progress: derived geometry cache keys now include presenter revision, and the declarative
    geometry build path uses `MeasuredNodeGraphPresenter` when measured geometry is present, so
    measured node-size updates invalidate geometry/index caches deterministically.
  - Progress: portal bounds + measured-geometry publish helpers now live under the private
    `paint_only/portal_measurement.rs` seam, so the main declarative surface stays focused on
    orchestration instead of re-embedding `LayoutQueryRegion`/store sync details inline.
- [x] Clarify how portal-hosted controls emit edits without bypassing the transaction architecture.
  - `NodeGraphPortalHost::with_controller` now prefers
    `NodeGraphController::submit_transaction_and_sync_models`.
  - `NodeGraphOverlayHost::new(...).with_controller(...)` now prefers
    `NodeGraphController::submit_transaction_and_sync_graph_model`.
  - Raw `edit_queue` now remains only as a compatibility transport seam for retained-only
    widget bindings; the `compat_retained` declarative surface now binds through a controller and
    no longer exposes public queue transport props.
- [ ] Move overlay/menu/toolbar policy to the right ecosystem surfaces where that boundary is
      currently blurry.
  - Progress: diagnostics-only tooltip + marquee overlay composition now routes through the private
    `paint_only/overlay_elements.rs` seam, so the main declarative surface only computes overlay
    specs / orchestration instead of re-embedding element composition inline.
  - Progress: diagnostics hover-tooltip overlay lookup no longer imports the portal-hosting module
    for node label/port summaries; that shared summary helper now lives under the neutral
    `paint_only/surface_support.rs` seam instead.
  - Progress: declarative diagnostics hotkeys and diagnostics-only hover tooltip policy now come
    from explicit `NodeGraphDiagnosticsConfig` surface props; the mechanism layer no longer reads
    `FRET_DIAG` directly, and the demo owns that env-to-config choice instead.
  - Progress: root `fret_node::ui::*` now also re-exports
    `NodeGraphDiagnosticsConfig` / `NodeGraphVisibleSubsetPortalConfig`, so app-facing
    declarative authoring does not have to split imports across `ui` and `ui::declarative` to
    configure `NodeGraphSurfaceProps`.
  - Progress: focused paint-only gates now lock tooltip flip-below fallback and marquee clamp
    behavior, giving the next toolbar/menu policy split a small correctness baseline before moving
    composition into broader overlay surfaces.
  - Progress: retained toolbar target-selection and visibility rules now route through the private
    `ui/overlays/toolbar_policy.rs` seam, so node/edge toolbar widgets reuse one target fallback
    policy instead of embedding the same "explicit target vs first selected target" branches twice.
  - Progress: the public toolbar policy types now also live with that seam:
    `NodeGraphToolbarVisibility` / `NodeGraphToolbarPosition` / `NodeGraphToolbarAlign` /
    `NodeGraphToolbarSize` are no longer declared inside `toolbars.rs`, so the widget file keeps
    anchor layout ownership while public policy types stay beside toolbar target/visibility policy.
  - Progress: retained rename overlays now also route active-session choice, seed-text loading,
    focus-loss cancel policy, and commit-transaction planning through the private
    `ui/overlays/rename_policy.rs` seam, so `NodeGraphOverlayHost` no longer duplicates
    group-vs-symbol rename branches or commits a hidden second session when overlay state drifts.
  - Progress: retained controls overlays now also route button roster order, default command
    mapping, override-resolution, a11y labels, and display labels through the private
    `ui/overlays/controls_policy.rs` seam, so layout, keyboard navigation, and activation all
    consume one authority table instead of repeating the controls button list in multiple helpers.
  - Progress: the public controls binding types now also live with that policy seam:
    `NodeGraphControlsCommandBinding` / `NodeGraphControlsBindings` are no longer declared inside
    `controls.rs`, so the widget file keeps implementation ownership while public policy types stay
    next to command-resolution logic.
  - Progress: retained blackboard overlays now also route action roster order, keyboard
    navigation policy, action labels, default symbol naming, transaction planning, and
    symbol-rename opening through the private `ui/overlays/blackboard_policy.rs` seam, so
    `blackboard.rs` now keeps layout/paint/event orchestration while action policy no longer
    sprawls through the widget body.
  - Progress: retained minimap overlays now also route keyboard action mapping, pan/zoom step
    policy, zoom clamp, and center-based zoom planning through the private
    `ui/overlays/minimap_policy.rs` seam, so `minimap.rs` now keeps internals sampling, pointer
    drag handling, and viewport application while keyboard navigation policy stops living inline in
    the widget event branch.
  - Progress: retained minimap overlays now also route viewport-update ownership and zoom
    normalization through the private `ui/overlays/minimap_navigation_policy.rs` seam, so
    controller/store/default navigation binding resolution no longer stays embedded in
    `minimap.rs`.
  - Progress: `NodeGraphMiniMapNavigationBinding` / `NodeGraphMiniMapBindings` now also live with
    that seam instead of being declared inside `minimap.rs`, so the widget file no longer owns the
    public minimap policy types.
  - Progress: compat-retained window-space placement math now routes through the shared
    `ui/screen_space_placement.rs` seam, so toolbars, panel placement, blackboard, rename,
    controls, and minimap all reuse the same clamp / anchor math instead of keeping subtly
    duplicated per-widget geometry branches.
  - Progress: canvas menu/searcher host-state construction now routes through the private
    `canvas/widget/menu_session.rs` seam, and `SearcherState` now carries explicit
    `SearcherRowsMode` policy instead of inferring flat-vs-catalog row building from
    `ContextMenuTarget` branches.
  - Progress: menu/searcher overlay-session policy types now also live on the dedicated private
    `ui/canvas/state/state_overlay_policy.rs` seam, so `ContextMenuTarget` and
    `SearcherRowsMode` no longer stay declared inside `state_overlay_sessions.rs`.
  - Progress: searcher picker opener policy now also lives on the dedicated private
    `ui/canvas/widget/searcher_picker/request.rs` seam, so `SearcherPickerRequest` owns
    `rows_mode`, and background/connection insert pickers, edge-insert pickers, and conversion
    picker openers no longer hand-assemble `Catalog` / `Flat` request policy separately.
  - Progress: searcher row activation now also reuses the insert-candidate menu authority in
    `ui/canvas/widget/insert_candidates/menu.rs`, so candidate-row activation no longer
    hand-assembles `NodeGraphContextMenuAction::InsertNodeCandidate(...)` outside the same seam
    used to build context-menu candidate items.
  - Progress: searcher row activation now also reuses selectable-row policy from
    `ui/canvas/widget/searcher_rows.rs`, so activation gating no longer keeps a second implicit
    "candidate + enabled" rule separate from active-row selection and keyboard navigation.
  - Progress: context-menu target dispatch now also routes non-command activation through the
    private `ui/canvas/widget/context_menu/activate/target.rs` seam, so `activate.rs` keeps the
    command-vs-target action-kind split while background/connection/edge/conversion target routing
    becomes a named, focused-testable authority instead of an unowned inline match.
  - Progress: command context-menu activation now also routes target-scoped selection side effects
    through the private `ui/canvas/widget/context_menu/activate/command.rs` seam, so the
    group-selection-vs-ignore policy becomes explicit and command dispatch no longer keeps that
    target-specific selection sync inline.
  - Progress: edge context-menu activation now also routes edge-action planning through the
    private `ui/canvas/widget/context_menu/edge_execution.rs` seam, so insert-picker / reroute /
    delete / custom edge actions no longer stay as an unowned inline match before delegating to
    their executor modules.
  - Progress: right-click context-menu opening now also routes target-hit priority through the
    private `ui/canvas/widget/context_menu/opening.rs` seam, so group-vs-edge-vs-background
    precedence becomes a named, focused-testable authority while the group/edge opener modules keep
    only already-resolved target presentation work.
  - Progress: context-menu presentation now also routes open-event state effects through the
    private `ui/canvas/widget/context_menu/ui.rs` seam, so menu install, hover-edge cleanup, focus
    request, and event-finish invalidation no longer stay embedded in `show_context_menu(...)`, and
    the open-path hover-edge behavior now uses an explicit policy type instead of a bare boolean.
  - Progress: context-menu presentation lifecycle now also mirrors the searcher split:
    `ui/canvas/widget/context_menu/ui/overlay.rs` owns state install/restore/take/clear plus
    hover-edge cleanup policy, `ui/canvas/widget/context_menu/ui/event.rs` owns
    open/restore/dismiss event tails plus finish/invalidation, and the root `ui.rs` now stays a
    thin wrapper surface instead of mixing state and event responsibilities.
  - Progress: searcher overlay install now also has an explicit replacement seam in
    `ui/canvas/widget/searcher_ui/overlay.rs`, so the "clear context menu, then install/replace
    searcher state" rule becomes a named, focused-testable helper instead of staying hidden in the
    root install path.
  - Progress: context-menu/searcher event tails now also share the retained widget runtime finish
    helper, so `ui/canvas/widget/context_menu/ui/event.rs` and
    `ui/canvas/widget/searcher_ui/event.rs` stop re-embedding the same stop-propagation plus paint
    invalidation tail logic inline.
  - Progress: active menu-session occupancy now also routes through the private
    `canvas/widget/menu_session.rs` seam, so window-focus deferral, space-to-pan gating,
    Tab-navigation suppression, edge double-click preflight, motion/auto-pan tick guards, and
    retained `view_interacting(...)` all reuse one `context_menu || searcher` authority instead of
    re-embedding that overlay-session policy across multiple runtime files.
  - Progress: retained portal/overlay transaction fallback now also routes through the private
    `ui/retained_submit.rs` seam, so portal command commits plus blackboard/group-rename overlays
    share one controller-first vs edit-queue fallback policy instead of duplicating that
    compatibility branch inline.
  - Progress: retained action-panel keyboard routing now also routes through the private
    `ui/overlays/panel_navigation_policy.rs` seam, so controls and blackboard overlays share one
    Arrow/Home/End/Enter/Escape navigation authority instead of each embedding the same keyboard
    roster policy inline.
  - Progress: retained toolbar child layout lifecycle now also routes through the private
    `ui/overlays/toolbars_layout.rs` seam, so node and edge toolbars share one child measurement,
    hide-and-release-focus, and child paint authority while the root widget file keeps only
    target-specific anchor resolution.
  - Progress: retained overlay/portal handled-event tails now also route through the private
    `ui/retained_event_tail.rs` seam, so portal commands plus
    controls/blackboard/minimap/group-rename overlays share one authority for focus-to-canvas,
    stop-propagation, redraw, and paint/layout invalidation tails instead of duplicating those
    handled-event endings inline.
  - Progress: retained action-panel pointer state now also routes through the private
    `ui/overlays/panel_pointer_policy.rs` seam, so controls and blackboard overlays share one
    hover sync plus press-on-down / activate-on-matching-up authority instead of each
    re-embedding that pointer-state policy inline.
  - Progress: retained minimap projection math now also routes through the private
    `ui/overlays/minimap_projection.rs` seam, so world-bounds union, project/unproject
    transforms, and center-pan math live behind one focused authority instead of staying embedded
    in the overlay widget file.
  - Progress: retained blackboard layout and hit-testing now also route through the private
    `ui/overlays/blackboard_layout.rs` seam, so panel/header/row geometry plus action hit
    detection live behind one focused authority instead of staying embedded in the overlay widget
    file.
  - Progress: retained controls layout and hit-testing now also route through the private
    `ui/overlays/controls_layout.rs` seam, so panel geometry plus button hit detection live
    behind one focused authority instead of staying embedded in the overlay widget file.
  - Progress: retained action-panel item state now also routes through the private
    `ui/overlays/panel_item_state.rs` seam, so controls and blackboard overlays share one
    authority for keyboard selection resets, pointer-to-keyboard promotion, and visible
    item-state evaluation instead of each re-embedding that panel-state policy inline.
  - Progress: retained rename host layout lifecycle now also routes through the private
    `ui/overlays/rename_host_layout.rs` seam, so hidden/cancelled/active overlay layout planning
    lives behind one focused authority instead of staying embedded in the rename host widget file.
  - Progress: retained rename host key handling now also routes through the private
    `ui/overlays/rename_host_event.rs` seam, so Enter/Escape commit-vs-close routing plus
    controller-first submit/close ordering live behind one focused authority instead of staying
    embedded in the rename host widget file.
  - Progress: retained minimap drag planning now also routes through the private
    `ui/overlays/minimap_drag_policy.rs` seam, so pointer-down recentering and drag-pan delta
    planning live behind one focused authority instead of staying embedded in the overlay widget
    file.
  - Progress: the `menu_session.rs` wrapper now delegates `build_searcher_rows(...)` directly to
    `canvas/widget/menu_session/searcher.rs`, so flat-vs-catalog row policy has one authority seam
    instead of staying duplicated in both the wrapper and the searcher submodule.
  - Progress: insert-node family candidate sourcing now routes through the private
    `canvas/widget/insert_candidates.rs` seam, so background / connection / edge pickers share the
    same `Reroute` prepend rule and edge-insert menus reuse one candidate-to-menu-item mapping
    instead of rebuilding those lists in each opener.
  - Progress: insert-node execution policy now routes through the private
    `canvas/widget/insert_execution.rs` seam, so background menus, connection menus, drag-drop, and
    reroute-focused commands reuse one `Reroute` create-op path plus one inserted-node selection
    reducer instead of scattering those execution details across multiple widget entrypoints.
  - Progress: split-edge reroute execution now routes through the private
    `canvas/widget/split_edge_execution.rs` seam, so edge context actions, double-click gestures, and
    command-open flows share one reroute split planner, one rejection-toast fallback, and one
    post-commit selection path instead of repeating that edge-specific transaction wiring inline.
  - Progress: the private `canvas/widget/insert_execution.rs` seam now also owns split-edge
    candidate preview/plan helpers, so edge-insert direct actions and insert-node drag preview/drop
    reuse one candidate-aware split planner and one rejection-toast fallback instead of re-deriving
    reroute positions and `plan_split_edge_candidate` branches in each entrypoint.
  - Progress: connection picker activation now routes through the private
    `canvas/widget/context_menu/connection_execution.rs` seam, so connection-insert and conversion
    picker actions now share picker activation, planner/result helpers, and suspended-wire resume
    policy instead of keeping that orchestration inline in `context_menu/activate.rs`.
  - Progress: group target selection and edge-target context actions now route through the private
    `canvas/widget/context_menu/{target_selection,edge_execution}.rs` seams, so group selection
    sync and edge action execution no longer stay duplicated between right-click setup and activation
    dispatch branches.
  - Progress: background insert picker activation now also routes through the private
    `canvas/widget/context_menu/background_execution.rs` seam, so background insert planning,
    commit/selection, and rejection-toast handling no longer stay inlined inside
    `context_menu/activate.rs`.
  - Progress: edge-insert picker activation now also routes through the `canvas/widget/edge_insert`
    seam, so `context_menu/activate.rs` no longer owns candidate lookup plus handoff for the
    edge-insert searcher target.
  - Progress: keyboard and pointer menu-item activation now route through the private
    `canvas/widget/context_menu/selection_activation.rs` seam, so enabled-item lookup and payload
    cloning no longer stay duplicated between `context_menu/input.rs` and
    `context_menu/pointer.rs`.
  - Progress: keyboard context-menu navigation and typeahead now route through the private
    `canvas/widget/context_menu/key_navigation.rs` seam, so enabled-item stepping and typeahead
    fallback rules no longer stay embedded in `context_menu/input.rs`.
  - Progress: group bring-to-front / send-to-back command reducers now route through the private
    `canvas/widget/group_draw_order.rs` seam, so selected-group ordering and missing-group merge
    rules no longer stay duplicated inside `command_open.rs`.
  - Progress: right-click menu presentation and edge-target selection now route through the private
    `canvas/widget/context_menu/opening.rs` plus `context_menu/target_selection.rs` seams, so menu
    state presentation and edge selection sync no longer stay duplicated in `right_click.rs`.
  - Progress: static group/background/edge context-menu items now route through the private
    `canvas/widget/context_menu/item_builders.rs` seam, so command-item construction no longer
    stays duplicated between `right_click.rs` and `context_menu/opening.rs`.
  - Progress: split-edge reroute outcome handling now routes through the private
    `canvas/widget/split_edge_execution.rs` seam, so command-open, double-click, and edge context
    actions no longer duplicate the same outcome/toast/application branches.
  - Progress: right-click group/edge target hit testing now routes through the private
    `canvas/widget/context_menu/target_hit.rs` seam, so `right_click.rs` no longer owns the raw
    group-header/group-resize/edge hit-test traversal inline.
  - Progress: command-open UI orchestration now routes through the private
    `canvas/widget/command_ui.rs` seam, so transient dismissal, invoked-at fallback, and common
    paint invalidation no longer stay repeated across `command_open.rs` entrypoints.
  - Progress: searcher overlay UI orchestration now routes through the private
    `canvas/widget/searcher_ui.rs` seam, so overlay install/open/dismiss handling and shared
    event-finish paint invalidation no longer stay repeated across `searcher.rs`,
    `searcher_logic.rs`, `command_open.rs`, and `edge_insert/picker.rs`.
  - Progress: searcher row activation and pending-drag arming now route through the private
    `canvas/widget/searcher_activation.rs` seam, so pointer hit resolution, active-row sync, and
    pointer-up activation/dismiss fallback no longer stay repeated inline in `searcher.rs`.
  - Progress: searcher keyboard navigation and query mutation now route through the private
    `canvas/widget/searcher_input.rs` seam, so active-row stepping, Enter activation handoff, and
    query-edit rebuild triggers no longer stay embedded in `searcher.rs`.
  - Progress: searcher pointer hover feedback and wheel scroll state now route through the
    private `canvas/widget/searcher_pointer.rs` seam, so hovered-row sync, hover-driven active-row
    promotion, and wheel scroll clamping no longer stay embedded in `searcher.rs`.
  - Progress: `searcher.rs` now acts as a thin retained event router, while escape / key /
    pointer-down / pointer-up / pointer-move / wheel behavior each delegate to their owning
    private seam instead of keeping event glue in one file.
  - Progress: context-menu event glue now also routes through the private
    `canvas/widget/context_menu/ui.rs` seam, so dismiss / restore / paint invalidation rules no
    longer stay duplicated between `context_menu/input.rs` and `context_menu/pointer.rs`.
  - Progress: `context_menu/input.rs` and `context_menu/pointer.rs` now act as thin retained event
    routers, delegating key and pointer behavior to `key_navigation.rs` and
    `selection_activation.rs` instead of keeping event glue inline.
  - Progress: `right_click.rs` now also acts as a thin retained event router, while pending
    right-click click-vs-drag threshold checks are shared between `event_pointer_move.rs` and
    `event_pointer_up.rs`, and context-menu opening delegates into `context_menu/opening.rs`.
  - Progress: `left_click/handlers.rs` now routes group-resize / group-header / background
    branches through the private `canvas/widget/left_click/group_background.rs` seam, so group
    selection sync, pending drag/resize arming, and background marquee/pan fallback no longer stay
    embedded in the main hit-dispatch match.
  - Progress: `left_click/handlers.rs` now also routes port / edge-anchor branches through the
    private `canvas/widget/left_click/connection_hits.rs` seam, so connect-on-click resolution,
    reconnect drag arming, and edge-anchor selection sync no longer stay embedded in the main
    hit-dispatch match.
  - Progress: `left_click/handlers.rs` now also routes resize / node / edge branches through the
    private `canvas/widget/left_click/element_hits.rs` seam, so node selection sync, drag-handle
    gating, resize arming, and edge alt-insert arming no longer stay embedded in the main
    hit-dispatch match; the file now behaves as a thin retained hit router.
  - Progress: `pointer_up.rs` now routes node-resize / group-resize / group-drag / node-drag
    commit branches through the private `canvas/widget/pointer_up_commit.rs` seam, so graph-op
    commit assembly and drag-end outcome labeling no longer stay embedded in the retained pointer
    release router.
  - Progress: `pointer_up.rs` now also routes pending group / node / wire release branches through
    the private `canvas/widget/pointer_up_pending.rs` seam, so click-distance selection toggles,
    click-connect wire re-arming, and pointer-capture release cleanup no longer stay embedded in
    the retained pointer release router.
  - Progress: `event_pointer_down.rs` now routes background-zoom / edge-insert / reroute
    double-click branches through the private `canvas/widget/pointer_down_double_click.rs` seam,
    so repeated edge/background hit filtering and double-click orchestration no longer stay
    embedded in the retained pointer-down router.
  - Progress: `pointer_up.rs` now routes pointer-up state sync, sticky-wire
    ignore handling, pan-release unwind, and left-button release ordering through the
    private `canvas/widget/pointer_up_state.rs` and
    `canvas/widget/pointer_up_left_route.rs` seams, so retained pointer release routing
    no longer keeps state sync and left-tail fallback ordering embedded in one surface.
  - Progress: `event_pointer_down.rs` now also routes close-button dispatch, pending right-click
    arming, sticky-wire activation, and pan-start branches through the private
    `canvas/widget/pointer_down_gesture_start.rs` seam, so gesture-start ordering no longer stays
    embedded in the retained pointer-down router.
  - Progress: `event_pointer_down.rs` now routes pointer-down interaction priming
    and final left/right/ignore tail dispatch through the private
    `canvas/widget/event_pointer_down_state.rs` and
    `canvas/widget/event_pointer_down_route.rs` seams, so retained pointer-down routing
    no longer keeps timer-stop/state-sync setup and tail button fallback embedded in one surface.
  - Progress: `focus_nav.rs` now routes edge / node / port traversal through the private
    `canvas/widget/focus_nav_traversal.rs` seam, so selection/focus cycling order and auto-pan on
    node focus no longer stay embedded in the port-hint / activation file.
  - Progress: `focus_nav.rs` now also routes focused-port validity refresh, canvas-center lookup,
    and focused-port activation through the private `canvas/widget/focus_nav_ports.rs` seam, so
    connect-preview simulation and click-connect activation no longer stay embedded in the thin
    focus-nav router.
  - Progress: `event_pointer_move.rs` now routes missing pointer-up inference, pending right-click
    pan threshold checks, and retained move-handler dispatch through the private
    `canvas/widget/pointer_move_release.rs` and `canvas/widget/pointer_move_dispatch.rs` seams, so
    release synthesis and move arbitration no longer stay embedded in one retained pointer-move
    router.
  - Progress: `event_pointer_move.rs` now routes modifier/multi-select state sync,
    last-pointer seeding, and cursor/auto-pan tail work through the private
    `canvas/widget/event_pointer_move_state.rs` and
    `canvas/widget/event_pointer_move_tail.rs` seams, so retained pointer-move routing no
    longer keeps move-state priming and tail post-dispatch sync embedded in one surface.
  - Progress: `event_pointer_wheel.rs` now routes wheel zoom / pan and pinch viewport
    motion through the private `canvas/widget/pointer_wheel_viewport.rs` seam, so
    viewport-motion cancellation, wheel pan math, and pinch zoom math no longer stay
    embedded in the retained wheel router.
  - Progress: `event_pointer_wheel.rs` now routes wheel modifier state sync
    and scroll/pinch event dispatch through the private
    `canvas/widget/event_pointer_wheel_state.rs` and
    `canvas/widget/event_pointer_wheel_route.rs` seams, so retained wheel routing no
    longer keeps modifier priming and scroll/pinch entry dispatch embedded in one surface.
  - Progress: `event_timer.rs` now routes timer-driven viewport and auto-pan
    motion through the private `canvas/widget/timer_motion.rs` seam, so pan inertia,
    viewport animation, auto-pan replay, and move-end debounce no longer stay embedded
    in the retained timer router.
  - Progress: `event_timer.rs` now also routes toast expiry cleanup and timer-motion
    sequencing through the private `canvas/widget/event_timer_toast.rs` and
    `canvas/widget/event_timer_route.rs` seams, so retained timer handling no longer
    keeps toast dismissal and motion/debounce dispatch ordering embedded in one surface.
  - Progress: `event_router.rs` now routes non-pointer lifecycle dispatch and
    pointer-variant dispatch through the private
    `canvas/widget/event_router_system.rs` and
    `canvas/widget/event_router_pointer.rs` seams, so clipboard/focus cancel,
    internal-drag/timer/keyboard routing, and pointer-variant branching no longer stay
    embedded in one retained event router surface.
  - Progress: `event_router_system.rs` now routes lifecycle/system events and keyboard
    input dispatch through the private
    `canvas/widget/event_router_system_lifecycle.rs` and
    `canvas/widget/event_router_system_input.rs` seams, so retained non-pointer routing no
    longer keeps clipboard/focus/timer/internal-drag handling and keyboard dispatch in
    one surface.
  - Progress: `event_router_pointer.rs` now routes button-pointer dispatch and
    wheel/pinch dispatch through the private
    `canvas/widget/event_router_pointer_button.rs` and
    `canvas/widget/event_router_pointer_wheel.rs` seams, so retained pointer routing no
    longer keeps down/move/up branching and wheel/pinch branching in one surface.
  - Progress: `event_clipboard.rs` now routes pending-paste token resolution and
    clipboard feedback side effects through the private
    `canvas/widget/event_clipboard_pending.rs` and
    `canvas/widget/event_clipboard_feedback.rs` seams, so retained clipboard event
    handling no longer keeps token matching/requeue logic and toast/redraw feedback
    embedded in one surface.
  - Progress: `event_keyboard.rs` now routes escape / overlay / modifier shortcut /
    tab / nudge / delete handling through the private
    `canvas/widget/keyboard_shortcuts.rs` seam, so key-driven command dispatch and
    overlay-aware keyboard exits no longer stay embedded in the retained keyboard router.
  - Progress: `event_keyboard.rs` now also routes pan-activation hold/release through the
    private `canvas/widget/keyboard_pan_activation.rs` seam, so space-to-pan arming,
    release, and paint invalidation no longer stay embedded in the retained keyboard router.
  - Progress: `event_keyboard.rs` now routes text-input gating, multi-selection
    modifier sync, and keydown/up dispatch ordering through the private
    `canvas/widget/event_keyboard_state.rs` and
    `canvas/widget/event_keyboard_route.rs` seams, so retained keyboard entry handling no
    longer keeps state priming and top-level key routing embedded in one surface.
    release, and paint invalidation no longer stay embedded in the retained keyboard router.
  - Progress: `retained_widget.rs` now routes semantics / layout / prepaint through the
    private `canvas/widget/retained_widget_frame.rs` seam, so viewport semantics value
    assembly, diagnostics-anchor child layout, queue drain-on-layout, and cull-window
    tracking no longer stay embedded in the main retained widget trait router.
  - Progress: `retained_widget.rs` now also routes command / event / paint runtime
    bridge work through the private `canvas/widget/retained_widget_runtime.rs` seam, so
    style/skin/paint-override sync, text-input command deferral, middleware handoff, and
    middleware-handled redraw/invalidation no longer stay embedded in the main trait router.
  - Progress: `retained_widget.rs` now also routes command availability through the
    private `canvas/widget/retained_widget_command_availability.rs` seam, so clipboard
    capability gating and selection/content availability checks no longer stay embedded in
    the main retained widget trait router.
  - Progress: `retained_widget_command_availability.rs` now routes focus/clipboard gating
    and graph/view-state availability queries through the private
    `canvas/widget/retained_widget_command_availability_gate.rs` and
    `canvas/widget/retained_widget_command_availability_query.rs` seams, so retained edit
    command availability no longer keeps capability checks and selection/content reads
    embedded in one surface.
  - Progress: `node_drag.rs` now routes snapline arbitration and preview planning
    through the private `canvas/widget/node_drag_snap.rs` and
    `canvas/widget/node_drag_preview.rs` seams, so snap-guides math and drag-preview
    node/group projection no longer stay embedded in the retained drag router.
  - Progress: `node_drag.rs` now also routes anchor clamp / extent union /
    multi-drag extent clamp math through the private
    `canvas/widget/node_drag_constraints.rs` seam, so node/group constraint math no longer
    stays embedded in the retained drag router.
  - Progress: `paint_grid.rs` now routes tile scene-op generation through the
    private `canvas/widget/paint_grid_tiles.rs` seam, so grid line/dot/cross emission
    and focused pattern tests no longer stay embedded in the retained grid cache/router
    surface.
  - Progress: `pointer_up_commit.rs` now routes node-drag release commit
    assembly through the private `canvas/widget/pointer_up_node_drag.rs` seam and shares
    pointer-capture teardown via `canvas/widget/pointer_up_finish.rs`, so retained
    pointer-up finalize logic no longer stays duplicated across commit/pending reducers.
  - Progress: `focus_nav_traversal.rs` now routes edge/node/port cycle
    traversal through the private `canvas/widget/focus_nav_traversal_edge.rs`,
    `canvas/widget/focus_nav_traversal_node.rs`, and
    `canvas/widget/focus_nav_traversal_port.rs` seams, so retained focus-cycle reducers
    no longer stay embedded in a single traversal surface.
  - Progress: `focus.rs` now routes focused-edge repair, draw-order fingerprinting, and
    directional port navigation through the private `canvas/widget/focus_edge_repair.rs`,
    `canvas/widget/focus_draw_order.rs`, and `canvas/widget/focus_port_direction.rs` seams,
    so retained focus-maintenance helpers no longer stay embedded in one mixed utility surface.
  - Progress: `callbacks.rs` now routes graph commit/delete dispatch, connect lifecycle
    callbacks, and viewport/node-drag/view-change emissions through the private
    `canvas/widget/callbacks_graph.rs`, `canvas/widget/callbacks_connect.rs`, and
    `canvas/widget/callbacks_view.rs` seams, so retained callback orchestration no longer stays
    embedded in one mixed surface.
  - Progress: `clipboard.rs` now routes paste-anchor math, clipboard host effects,
    and paste/duplicate transaction assembly through the private
    `canvas/widget/clipboard_anchor.rs`, `canvas/widget/clipboard_transfer.rs`, and
    `canvas/widget/clipboard_paste.rs` seams, so retained clipboard reducers no longer stay
    embedded in one mixed surface.
  - Progress: `marquee.rs` now routes background-marquee arming, active selection
    updates, threshold/pan arbitration, and pointer-up cleanup through the private
    `canvas/widget/marquee_begin.rs`, `canvas/widget/marquee_selection.rs`,
    `canvas/widget/marquee_pending.rs`, and `canvas/widget/marquee_finish.rs` seams,
    so retained marquee reducers no longer stay embedded in one mixed surface.
  - Progress: `cancel.rs` now routes gesture-state teardown, viewport-motion
    cancellation, and final cleanup through the private `canvas/widget/cancel_gesture_state.rs`,
    `canvas/widget/cancel_viewport_state.rs`, and `canvas/widget/cancel_cleanup.rs` seams,
    so retained cancel reducers no longer stay embedded in one mixed surface.
  - Progress: `pan_zoom.rs` now routes zoom cache mutation, pan-start arbitration,
    and pan-move velocity updates through the private `canvas/widget/pan_zoom_zoom.rs`,
    `canvas/widget/pan_zoom_begin.rs`, and `canvas/widget/pan_zoom_move.rs` seams,
    so retained pan/zoom reducers no longer stay embedded in one mixed surface.
  - Progress: `insert_execution.rs` now routes candidate point resolution, graph-op
    planning, and insertion feedback through the private
    `canvas/widget/insert_execution_point.rs`, `canvas/widget/insert_execution_plan.rs`, and
    `canvas/widget/insert_execution_feedback.rs` seams, so retained insert execution reducers no
    longer stay embedded in one mixed surface.
  - Progress: `pointer_down_double_click.rs` now routes background zoom and edge
    double-click actions through the private `canvas/widget/pointer_down_double_click_background.rs`
    and `canvas/widget/pointer_down_double_click_edge.rs` seams, so retained double-click reducers no
    longer stay embedded in one mixed surface.
  - Progress: `command_open.rs` now routes insert-picker positioning, group command reducers,
    split-edge open/reroute actions, and conversion-picker presentation through the private
    `canvas/widget/command_open_insert.rs`, `canvas/widget/command_open_group.rs`,
    `canvas/widget/command_open_edge.rs`, and `canvas/widget/command_open_conversion.rs` seams,
    so retained command-open reducers no longer stay embedded in one mixed surface.
  - Progress: `focus_nav_ports.rs` now routes focused-port validation hints, port-center
    lookup, and click-connect activation handoff through the private
    `canvas/widget/focus_nav_ports_hints.rs`, `canvas/widget/focus_nav_ports_center.rs`, and
    `canvas/widget/focus_nav_ports_activation.rs` seams, so retained focused-port reducers no
    longer stay embedded in one mixed surface.
  - Progress: `paint_grid.rs` now routes grid paint planning, tile-cache warmup, and
    cache stats publication through the private `canvas/widget/paint_grid_plan.rs`,
    `canvas/widget/paint_grid_cache.rs`, and `canvas/widget/paint_grid_stats.rs` seams,
    so retained grid-paint orchestration no longer stays embedded in one mixed surface.
  - Progress: `focus_port_direction.rs` now routes wire-drag direction filtering and
    directional candidate ranking through the private
    `canvas/widget/focus_port_direction_candidate.rs` seam, and focus/view-state application
    through `canvas/widget/focus_port_direction_apply.rs`, so retained directional port-focus
    reducers no longer stay embedded in one mixed surface.
  - Progress: `sticky_wire.rs` now routes connect-target planning/reject handling and
    non-port picker routing through the private `canvas/widget/sticky_wire_connect.rs` and
    `canvas/widget/sticky_wire_targets.rs` seams, so retained sticky-wire pointer reducers no
    longer stay embedded in one mixed surface.
  - Progress: `pointer_move_release.rs` now routes pan-release cleanup, right-click
    pan arming, missing-left-release finalization, and last-pointer-state sync through the
    private `canvas/widget/pointer_move_release_pan.rs`,
    `canvas/widget/pointer_move_release_left.rs`, and
    `canvas/widget/pointer_move_pointer_state.rs` seams, so retained move-release reducers no
    longer stay embedded in one mixed surface.
  - Progress: `pointer_wheel_viewport.rs` now routes wheel/pinch motion stop,
    zoom application, and scroll-pan updates through the private
    `canvas/widget/pointer_wheel_motion.rs`, `canvas/widget/pointer_wheel_zoom.rs`, and
    `canvas/widget/pointer_wheel_pan.rs` seams, so retained wheel-viewport reducers no longer
    stay embedded in one mixed surface.
  - Progress: `searcher_logic.rs` now routes recent-kind/row rebuild helpers, row
    activation handoff, and picker-opening orchestration through the private
    `canvas/widget/searcher_rows.rs`, `canvas/widget/searcher_row_activation.rs`, and
    `canvas/widget/searcher_picker.rs` seams, so retained searcher logic no longer stays
    embedded in one mixed surface.
  - Progress: `command_focus.rs` now routes cycle commands and directional/activate
    commands through the private `canvas/widget/command_focus_cycle.rs` and
    `canvas/widget/command_focus_port.rs` seams, so retained focus command wrappers no longer
    stay embedded in one mixed surface.
  - Progress: `retained_widget_frame.rs` now routes semantics sync, layout/update
    orchestration, and prepaint cull-window tracking through the private
    `canvas/widget/retained_widget_semantics.rs`,
    `canvas/widget/retained_widget_layout.rs`, and
    `canvas/widget/retained_widget_cull_window.rs` seams, so retained widget frame
    orchestration no longer stays embedded in one mixed surface.
  - Progress: `retained_widget_semantics.rs` now routes active-descendant lookup and
    semantics value assembly through the private
    `canvas/widget/retained_widget_semantics_focus.rs` and
    `canvas/widget/retained_widget_semantics_value.rs` seams, so retained semantics sync
    no longer keeps descendant arbitration and accessibility value string assembly in one
    surface.
  - Progress: `retained_widget_layout.rs` now routes model observation, diagnostics
    publish, child layout, and post-layout queue drain through the private
    `canvas/widget/retained_widget_layout_observe.rs`,
    `canvas/widget/retained_widget_layout_publish.rs`,
    `canvas/widget/retained_widget_layout_children.rs`, and
    `canvas/widget/retained_widget_layout_drain.rs` seams, so retained layout sync no
    longer keeps mixed observation, diagnostics, child placement, and queue drain logic in
    one surface.
  - Progress: `retained_widget_cull_window.rs` now routes cull-window gating/key
    derivation and key-shift application through the private
    `canvas/widget/retained_widget_cull_window_key.rs` and
    `canvas/widget/retained_widget_cull_window_shift.rs` seams, so retained prepaint cull
    tracking no longer keeps visibility gating, tile-key math, and shift reporting in one
    surface.
  - Progress: `delete.rs` now routes delete-op construction, removable-id
    collection, and deletable predicates through the private
    `canvas/widget/delete_ops_builder.rs`,
    `canvas/widget/delete_removed_ids.rs`, and
    `canvas/widget/delete_predicates.rs` seams, so retained deletion helpers
    no longer stay embedded in one mixed surface.
  - Progress: `clipboard_paste.rs` now routes clipboard parsing/offset
    derivation, paste-transaction construction, and inserted-selection replay
    through the private `canvas/widget/clipboard_paste_parse.rs`,
    `canvas/widget/clipboard_paste_transaction.rs`, and
    `canvas/widget/clipboard_paste_selection.rs` seams, so retained clipboard
    paste helpers no longer stay embedded in one mixed surface.
  - Progress: `keyboard_shortcuts.rs` now routes overlay escape/key-down
    handling and modifier/navigation shortcut dispatch through the private
    `canvas/widget/keyboard_shortcuts_overlay.rs` and
    `canvas/widget/keyboard_shortcuts_commands.rs` seams, so retained keyboard
    shortcut reducers no longer stay embedded in one mixed surface.
  - Progress: `pointer_up_node_drag.rs` now routes parent-group resolution and
    release-op/commit orchestration through the private
    `canvas/widget/pointer_up_node_drag_parent.rs` and
    `canvas/widget/pointer_up_node_drag_ops.rs` seams, so retained node-drag
    release reducers no longer stay embedded in one mixed surface.
  - Progress: `node_drag_constraints.rs` now routes anchor/rect clamping and
    multi-drag extent constraint helpers through the private
    `canvas/widget/node_drag_constraints_anchor.rs` and
    `canvas/widget/node_drag_constraints_extent.rs` seams, so retained node-drag
    geometry helpers no longer stay embedded in one mixed surface.
  - Progress: `command_edit.rs` now routes cut/delete removal orchestration and
    removed-selection cleanup through the private
    `canvas/widget/command_edit_remove.rs` seam, so retained edit command
    reducers no longer keep destructive edit flows embedded in one surface.
  - Progress: `paint_overlay_feedback.rs` now routes toast overlay painting
    and wire-drag hint painting through the private
    `canvas/widget/paint_overlay_toast.rs` and
    `canvas/widget/paint_overlay_wire_hint.rs` seams, so retained overlay
    feedback paint helpers no longer stay embedded in one mixed surface.
  - Progress: `auto_measure.rs` now routes node measurement input collection
    and measured-size computation/store updates through the private
    `canvas/widget/auto_measure_collect.rs` and
    `canvas/widget/auto_measure_apply.rs` seams, so retained auto-measure
    reducers no longer keep collection and apply phases embedded in one surface.
  - Progress: `paint_grid_tiles.rs` now routes line, dot, and cross tile-op
    painting through the private `canvas/widget/paint_grid_tiles_lines.rs`,
    `canvas/widget/paint_grid_tiles_dots.rs`, and
    `canvas/widget/paint_grid_tiles_cross.rs` seams, so retained grid tile
    painters no longer keep all background patterns embedded in one surface.
  - Progress: `group_resize.rs` now routes preview-rect computation, child-bounds
    clamping, and resize-handle hit helpers through the private
    `canvas/widget/group_resize_apply.rs` and
    `canvas/widget/group_resize_hit.rs` seams, so retained group-resize reducers
    no longer keep geometry math and hit testing embedded in one surface.
  - Progress: `marquee_selection.rs` now routes marquee query/edge-derivation
    and selection-state writes through the private
    `canvas/widget/marquee_selection_query.rs` and
    `canvas/widget/marquee_selection_apply.rs` seams, so retained marquee reducers
    no longer keep box-selection geometry and state writes embedded in one surface.
  - Progress: `paint_grid_plan.rs` now routes canvas chrome hint lookup,
    grid metrics/tile scratch preparation, and pattern-size validation through the private
    `canvas/widget/paint_grid_plan_support.rs` seam, so retained grid-plan reducers
    no longer keep paint planning helpers embedded in one surface.
  - Progress: `cursor.rs` now routes interaction gating and concrete resize/edge-anchor
    cursor resolution through the private `canvas/widget/cursor_gate.rs` and
    `canvas/widget/cursor_resolve.rs` seams, so retained cursor reducers
    no longer keep cursor eligibility checks and hit resolution embedded in one surface.
  - Progress: `pointer_up_commit.rs` now routes node/group resize commit op assembly
    and group-drag release ops through the private
    `canvas/widget/pointer_up_commit_resize.rs` and
    `canvas/widget/pointer_up_commit_group_drag.rs` seams, so retained pointer-up reducers
    no longer keep multiple commit builders embedded in one surface.
  - Progress: `paint_searcher.rs` now routes searcher query chrome and row list painting
    through the private `canvas/widget/paint_searcher_query.rs` and
    `canvas/widget/paint_searcher_rows.rs` seams, so retained searcher paint reducers
    no longer keep all query/list paint phases embedded in one surface.
  - Progress: `view_math.rs` now routes viewport/pan-zoom conversion helpers and
    rect/hit/resize-handle geometry through the private
    `canvas/widget/view_math_viewport.rs` and `canvas/widget/view_math_rect.rs` seams,
    so retained view-math utilities no longer keep mixed viewport and hit geometry in one surface.
  - Progress: `rect_math.rs` now routes base rect set-ops and path/edge bounds helpers
    through the private `canvas/widget/rect_math_core.rs` and
    `canvas/widget/rect_math_path.rs` seams, so retained rect math utilities
    no longer keep mixed hit-rect and edge/path bounds helpers in one surface.
  - Progress: `focus_port_direction_candidate.rs` now routes wire-drag required-direction
    lookup and directional port ranking through the private
    `canvas/widget/focus_port_direction_wire.rs` and
    `canvas/widget/focus_port_direction_rank.rs` seams, so retained focus-navigation reducers
    no longer keep input-context lookup and ranking math embedded in one surface.
  - Progress: `keyboard_shortcuts_commands.rs` now routes shortcut eligibility gates and
    command lookup tables through the private `canvas/widget/keyboard_shortcuts_gate.rs`
    and `canvas/widget/keyboard_shortcuts_map.rs` seams, so retained keyboard shortcut
    reducers no longer keep mixed gating and command mapping embedded in one surface.
  - Progress: `command_router.rs` now routes nudge vector lookup and
    align/distribute mode mapping through the private
    `canvas/widget/command_router_nudge.rs` and
    `canvas/widget/command_router_align.rs` seams, so retained command routing no
    longer keeps repeated movement/alignment command tables embedded in one surface.
  - Progress: `retained_widget_runtime.rs` now routes retained command/event/paint
    bridge work through the private
    `canvas/widget/retained_widget_runtime_command.rs`,
    `canvas/widget/retained_widget_runtime_event.rs`,
    `canvas/widget/retained_widget_runtime_paint.rs`, and
    `canvas/widget/retained_widget_runtime_shared.rs` seams, so runtime theme sync,
    middleware context assembly, text-input command deferral, and handled invalidation
    no longer stay embedded in one retained runtime surface.
  - Progress: `paint_overlay_elements.rs` now routes context-menu chrome,
    marquee/snap-guide primitives, and toast/wire-drag hint feedback through the private
    `canvas/widget/paint_overlay_menu.rs`, `canvas/widget/paint_overlay_guides.rs`, and
    `canvas/widget/paint_overlay_feedback.rs` seams, so retained overlay paint helpers no
    longer stay embedded in one surface file.
  - Progress: `viewport_timers.rs` now routes animation/debounce, inertia,
    and auto-pan timer orchestration through the private
    `canvas/widget/viewport_timer_animation.rs`,
    `canvas/widget/viewport_timer_inertia.rs`, and
    `canvas/widget/viewport_timer_auto_pan.rs` seams, so retained viewport timer helpers
    no longer stay embedded in one surface file.
  - Progress: `timer_motion.rs` now routes pan-inertia ticks, viewport
    animation/debounce ticks, and auto-pan motion replay through the private
    `canvas/widget/timer_motion_pan_inertia.rs`,
    `canvas/widget/timer_motion_viewport.rs`, and
    `canvas/widget/timer_motion_auto_pan.rs` seams with shared invalidation in
    `canvas/widget/timer_motion_shared.rs`, so retained timer-driven motion reducers no
    longer stay embedded in one surface file.
  - Progress: the searcher input/pointer activation trio now routes hit
    testing, drag arming, key-step/query reducers, hover sync, and wheel scroll through
    the private `canvas/widget/searcher_activation_hit.rs`,
    `canvas/widget/searcher_activation_state.rs`,
    `canvas/widget/searcher_input_nav.rs`, `canvas/widget/searcher_input_query.rs`,
    `canvas/widget/searcher_pointer_hover.rs`, and
    `canvas/widget/searcher_pointer_wheel.rs` seams, so retained searcher reducers no
    longer stay embedded in three medium-sized surface files.
- [x] Add at least one gate that exercises portal + overlay anchoring under motion.
  - Progress: the feature-gated retained conformance files now include controller-first rename and
    portal commit scenarios (`overlay_group_rename_conformance.rs`,
    `portal_lifecycle_conformance.rs`), and those retained gates now run again after the harness was
    updated for the latest `EventCx` / `LayoutCx` contract.
  - Progress: declarative paint-only tooltip anchoring now has focused correctness gates proving
    `portal_bounds_store` wins when hosted bounds exist, while `hover_anchor_store` remains the
    fallback when portals are disabled or unavailable.
  - Progress: portal visible-subset hosting now also has a motion gate proving a dragged node that
    crosses into the viewport is hosted using its dragged rect rather than its stale pre-drag rect.
  - Progress: declarative hover anchoring now also has a motion gate proving drag-adjusted hover
    rects update `hover_anchor_store`, and a portal-vs-hover precedence gate proving dragged portal
    bounds still win over stale hover anchors when the hovered node moves.
  - Progress: hover anchor state/tooltip anchor helpers now live under the private
    `paint_only/hover_anchor.rs` seam, reducing how much motion-anchoring policy stays in the main
    surface file while keeping the same focused gates.
  - Progress: `paint_only.rs` now routes controller-backed transaction/view-state helpers and
    selection preview/commit reducers through the private `paint_only/transactions.rs` and
    `paint_only/selection.rs` seams, so declarative paint-only orchestration no longer keeps
    transaction plumbing and selection-state writes embedded in the main surface file.
  - Progress: `paint_only.rs` now also routes left-pointer down snapshot/arming logic through the
    private `paint_only/pointer_down.rs` seam, so declarative paint-only pointer-entry handling no
    longer keeps pan arming, hit snapshot reads, and selection/marquee arming embedded inline.
  - Progress: `paint_only.rs` now also routes diagnostics hotkeys, preset application, and
    keyboard zoom reducers through the private `paint_only/diag.rs` seam, so declarative
    paint-only keyboard/diagnostics branching no longer stays embedded in the main surface file.
  - Progress: `paint_only.rs` now routes grid/derived/nodes/edges cache rebuild helpers through the
    private `paint_only/cache.rs` seam, so declarative paint-only surface assembly no longer keeps
    cache warming and draw-cache rebuild sequencing embedded inline.
  - Progress: `paint_only.rs` now also routes local uncontrolled-model/bootstrap wiring through the
    private `paint_only/surface_models.rs` seam, so declarative paint-only surface assembly no
    longer keeps all local state bundle construction embedded inline.
  - Progress: `paint_only.rs` now also routes visible-subset portal hosting and deferred
    `fit-to-portals` viewport application through the private `paint_only/portals.rs` seam, so
    declarative paint-only surface assembly no longer keeps portal subtree hosting, bounds-store
    pruning, and pending-fit orchestration embedded inline.
  - Progress: `paint_only.rs` now also routes diagnostics hover-tooltip overlay orchestration
    through the private `paint_only/overlays.rs` seam, so declarative paint-only surface assembly
    no longer keeps hover-anchor reads, portal-bounds fallback, and tooltip element wiring
    embedded inline.
  - Progress: `paint_only.rs` now also routes marquee overlay append and final overlay-layer
    wrapping through the same private `paint_only/overlays.rs` seam, so declarative paint-only
    surface assembly no longer keeps overlay child flush/wrap plumbing embedded inline.
  - Progress: `paint_only.rs` now also routes edge/portal diagnostics aggregation and semantics
    value assembly through the private `paint_only/semantics.rs` seam, so declarative paint-only
    surface assembly no longer keeps observability counters and long semantics formatting embedded
    inline.
  - Progress: `paint_only.rs` now also routes key/pointer/wheel/pinch handler construction through
    the private `paint_only/input_handlers.rs` seam, so declarative paint-only surface assembly no
    longer keeps the full event closure builder set embedded inline.
  - Progress: `paint_only.rs` now also routes grid/derived/node/edge cache key generation, draw
    model construction, and canvas paint helpers through the private `paint_only/cache.rs` seam,
    so declarative paint-only surface assembly no longer keeps the retained-like cache/paint
    implementation blocks embedded inline.
  - Progress: `paint_only.rs` now also routes surface state snapshots, authoritative-boundary
    sync, portal measured-geometry flush, cache refresh, and semantics preparation through the
    private `paint_only/surface_frame.rs` seam, so declarative paint-only surface assembly no
    longer keeps the full pre-render context preparation block embedded inline.
  - Progress: `paint_only.rs` now also routes canvas paint closure wiring, portal hosting, hover
    anchor sync, hover tooltip append, fit-to-portals replay, marquee overlay append, and final
    overlay flush through the private `paint_only/surface_content.rs` seam, so declarative
    paint-only surface assembly no longer keeps the full post-handler render/output block embedded
    inline.
  - Progress: `paint_only.rs` now also routes bounds sync, keyboard/pointer gesture handler
    wiring, and pointer-region shell assembly through the private `paint_only/surface_shell.rs`
    seam, so declarative paint-only surface assembly no longer keeps the full
    `semantics_with_id(..., move |cx, element| { ... })` shell block embedded inline.
  - Progress: `paint_only.rs` now also routes geometry quantization, rectangle helpers, marquee
    math, node-drag delta/commit helpers, and point hit-testing through the private
    `paint_only/surface_math.rs` seam, so declarative paint-only surface assembly no longer keeps
    the shared geometry and gesture math helper set embedded inline.
  - Progress: `paint_only.rs` now also routes uncontrolled-model bootstrap, mouse-button/hash
    helpers, and authoritative surface-boundary snapshot/sync through the private
    `paint_only/surface_support.rs` seam, while diagnostic visible-node transaction builders now
    live beside the rest of the diagnostic policy in `paint_only/diag.rs`, so the main paint-only
    surface no longer keeps these support and diagnostic helper blocks embedded inline.
  - Progress: `ui/controller.rs` now also routes controller queries, viewport/fit-view helpers,
    and store-sync/replace/selection transport through the private
    `controller_queries.rs`, `controller_viewport.rs`, and `controller_store_sync.rs` seams, so
    the app-facing `NodeGraphController` surface no longer keeps the full query + viewport +
    queue/store orchestration implementation embedded inline.
  - Progress: `ui/canvas/widget.rs` now routes the retained canvas surface impl through the private
    `canvas/widget/widget_surface.rs` seam, so the root widget module now mainly holds the module
    map and shared type definitions while constructor/config/style-sync/cull helper orchestration
    lives beside the rest of the retained widget internals.
  - Progress: repeated `InteractionState` gate predicates for cursor/hover/edge-cache/pan-inertia
    now route through the private `canvas/widget/interaction_gate.rs` seam, so widget submodules no
    longer duplicate the same busy/idle interaction checks inline.
  - Progress: repeated focus-session mutations for focused edge/node/port transitions and
    selection-only sync now route through the private `canvas/widget/focus_session.rs` seam, so
    focus navigation helpers no longer duplicate the same focus-reset and selection-update blocks.
  - Progress: cancel-path residual cleanup, hover/focus reset, and pan-drag reset now route
    through the private `canvas/widget/cancel_session.rs` seam, so cancel/pointer-up helpers no
    longer duplicate the same interaction cleanup blocks inline.
  - Progress: left-click hit routes, pan-zoom start, marquee selection, and wire-commit cleanup
    now reuse the expanded private `canvas/widget/focus_session.rs` seam, so pointer-down helpers
    no longer re-embed the same edge-focus and hover-port hint resets inline.
  - Progress: left-click pointer-down preparation and pan-start competing-session cleanup now route
    through the private `canvas/widget/press_session.rs` seam, so retained widget hit handlers no
    longer re-embed the same pending-drag / marquee / edge-insert reset blocks inline.
  - Progress: pending pointer-up release finish helpers and node-drag release residual cleanup now
    route through the private `canvas/widget/pointer_up_session.rs` seam, so pointer-up handlers no
    longer re-embed the same pending-slot finish and snap-guide cleanup blocks inline.
  - Progress: pointer-up commit releases, marquee finish cleanup, and pending resize activation now
    also route through the private `canvas/widget/pointer_up_session.rs` and
    `canvas/widget/pending_resize_session.rs` seams, so release/activation helpers no longer
    re-embed the same companion-slot clearing and resize-activation blocks inline.
  - Progress: pending node/group drag activation and early-abort cleanup now also route through the
    private `canvas/widget/pending_drag_session.rs` seam, so pending drag helpers no longer
    re-embed the same pending-slot abort and activation-state construction blocks inline.
  - Progress: pending insert-node drag abort/finish and pending wire/edge-insert activation now
    also route through the private `canvas/widget/insert_node_drag/session.rs` and
    `canvas/widget/pending_connection_session.rs` seams, so insert/connection pending helpers no
    longer re-embed the same capture-release and pending-to-active construction blocks inline.
  - Progress: insert-node drag internal enter/leave/drop handling now also routes preview
    invalidation and drag-event finish through the private `canvas/widget/insert_node_drag/session.rs`
    seam, so `internal_move.rs`, `internal_drop.rs`, and `insert_node_drag/mod.rs` no longer
    re-embed the same preview repaint and propagation-stop tail blocks inline.
  - Progress: searcher overlay dismissal and row-drag release now also route through the private
    `canvas/widget/searcher_activation_state.rs` seam, so `searcher_activation.rs` and
    `searcher_ui.rs` no longer re-embed the same pending insert-drag clearing and capture-release
    state transitions inline.
  - Progress: command-driven transient dismissal now also routes searcher close through the private
    `canvas/widget/searcher_activation_state.rs` seam, so `command_ui.rs` no longer clears the
    searcher overlay without also clearing pending searcher row-drag state.
  - Progress: cancel gesture cleanup for insert-node drag now also routes through the private
    `canvas/widget/insert_node_drag/session.rs` seam, so `cancel_gesture_state.rs` no longer
    re-embeds pending-insert and preview-slot clearing inline.
  - Progress: context-menu close/restore state now also routes through the private
    `canvas/widget/context_menu/ui.rs` seam, so `command_ui.rs`, `searcher_ui.rs`,
    `context_menu/activate.rs`, and conversion-picker handoff in `wire_drag/commit/new_wire.rs`
    no longer re-embed the same context-menu slot clearing inline.
  - Progress: context-menu slot take/restore now also routes through the private
    `canvas/widget/context_menu/ui.rs` seam, so `context_menu/opening.rs`,
    `context_menu/selection_activation.rs`, and `context_menu/key_navigation.rs` no longer
    re-embed the same menu-slot mutation inline.
  - Progress: command redraw tails now also route through the private `canvas/widget/command_ui.rs`
    seam, so `command_history.rs`, `command_mode.rs`, `command_selection.rs`,
    `command_view.rs`, `command_move.rs`, `command_edit.rs`, `command_edit_remove.rs`,
    `command_focus_cycle.rs`, and `command_focus_port.rs` no longer re-embed the same
    redraw-plus-paint-invalidation tail blocks inline.
  - Progress: event/timer paint invalidation now also routes through the private
    `canvas/widget/paint_invalidation.rs` seam, so `event_clipboard_feedback.rs`,
    `event_timer_toast.rs`, `timer_motion_shared.rs`, `keyboard_pan_activation.rs`,
    `pointer_wheel_pan.rs`, and `pointer_wheel_zoom.rs` no longer re-embed the same
    event-scope redraw-plus-paint-invalidation tail blocks inline.
  - Progress: edge-drag / edge-insert drag / double-click / pointer-up event tails now also
    route through the same private `canvas/widget/paint_invalidation.rs` seam, so
    `edge_drag/move_start.rs`, `edge_drag/pointer_up.rs`, `edge_insert_drag/drag.rs`,
    `edge_insert_drag/pending.rs`, `pointer_down_double_click_background.rs`,
    `pointer_down_double_click_edge.rs`, `pointer_down_gesture_start.rs`, and
    `pointer_up_finish.rs` no longer re-embed the same event-scope
    redraw-plus-paint-invalidation tail blocks inline.
  - Progress: pan/marquee/group-drag/hover wire-drag event tails now also route through the same
    private `canvas/widget/paint_invalidation.rs` seam, so `pan_zoom_begin.rs`,
    `pan_zoom_move.rs`, `marquee_begin.rs`, `group_drag.rs`, `group_resize.rs`, `hover.rs`, and
    `wire_drag_helpers.rs` no longer re-embed the same event-scope
    redraw-plus-paint-invalidation tail blocks inline.
  - Progress: pointer-up / left-click / marquee-selection / node-drag / sticky-wire event tails
    now also route through the same private `canvas/widget/paint_invalidation.rs` seam, so
    `pointer_up_left_route.rs`, `pointer_up_state.rs`, `left_click/group_background.rs`,
    `left_click/connection_hits.rs`, `left_click/element_hits.rs`, `marquee_selection.rs`,
    `node_drag.rs`, `node_resize/move_update.rs`, `sticky_wire_connect.rs`, and
    `sticky_wire_targets.rs` no longer re-embed the same event-scope
    redraw-plus-paint-invalidation tail blocks inline.
  - Progress: cancel / context-menu / searcher / insert-node-drag event tails now also route
    through the same private `canvas/widget/paint_invalidation.rs` seam, so
    `cancel_cleanup.rs`, `context_menu/ui.rs`, `context_menu/opening.rs`, `searcher_ui.rs`, and
    `insert_node_drag/session.rs` no longer re-embed the same event-scope
    redraw-plus-paint-invalidation tail blocks inline.
  - Progress: command / retained-runtime / wire-commit paint tails now also route through small
    private helper seams, so `command_ui.rs`, `retained_widget_runtime_shared.rs`,
    `wire_drag/commit_cx.rs`, `wire_drag/commit/mod.rs`, and `wire_drag/move_update/mod.rs`
    no longer re-embed the same redraw-plus-paint-invalidation tail blocks inline.
  - Progress: paint/layout redraw requests now also route through the private
    `canvas/widget/redraw_request.rs` seam, so `paint_grid_cache.rs`, `paint_edges/main.rs`,
    `paint_root/cached_edges/single_rect.rs`, `paint_root/cached_edges/tile_path.rs`,
    `retained_widget_layout_drain.rs`, and `wire_drag/commit_cx.rs` no longer re-embed the same
    next-frame redraw request blocks inline.
  - Progress: `paint_edges/main.rs` now routes hash/glow-bounds helper logic through the private
    `canvas/widget/paint_edges/support.rs` seam, so the root edge-paint surface no longer keeps
    the full helper set for stable cache keys and glow bounds math embedded inline.
  - Progress: `paint_edges/main.rs` now also routes drop-marker drawing and wire-drag preview
    style/path emission through the private `canvas/widget/paint_edges/preview.rs` seam, so the
    root edge-paint surface no longer re-embeds preview marker geometry and preview wire paint
    orchestration inline while keeping the same preview behavior.
  - Progress: `paint_edges/main.rs` now also routes selected/base outline paint, selected glow
    effect setup, and selected/hovered highlight resolution through the private
    `canvas/widget/paint_edges/chrome.rs` seam, so the root edge-paint surface no longer
    re-embeds edge chrome orchestration inline while keeping the same wire/marker draw behavior.
  - Progress: `paint_edges/main.rs` now also routes edge paint batch preparation plus
    edge-insert/insert-node-drop marker projection through the private
    `canvas/widget/paint_edges/prepare.rs` seam, so the root edge-paint surface no longer
    re-embeds edge width classification and marker projection setup inline.
  - Progress: `paint_edges/main.rs` now also routes static edge-label drawing plus label/marker
    budget stats publication through the private `canvas/widget/paint_edges/labels.rs` seam, so
    the root edge-paint surface no longer re-embeds label tail orchestration and budget registry
    publication inline.
  - Progress: `paint_edges/main.rs` now also routes the main edge wire/marker paint pass plus
    paint-budget bookkeeping through the private `canvas/widget/paint_edges/pass.rs` seam, so the
    root edge-paint surface no longer re-embeds the full edge iteration loop and redraw-budget
    bookkeeping inline.
  - Progress: cached-edge single-rect/tiled label replay and single-rect label build orchestration
    now also route through the private `canvas/widget/paint_root/cached_edges/labels.rs` seam, so
    `paint_root/cached_edges/single_rect.rs` and `paint_root/cached_edges/tile_path.rs` no longer
    re-embed the same label-cache replay closure or the single-rect label build tail inline.
  - Progress: cached-edge single-rect edge replay/build plus tiled edge-cache and tiled label-cache
    orchestration now also route through the private
    `canvas/widget/paint_root/cached_edges/edges.rs` and
    `canvas/widget/paint_root/cached_edges/labels.rs` seams, so
    `paint_root/cached_edges/single_rect.rs` and `paint_root/cached_edges/tile_path.rs` now mainly
    choose cache mode, fall back to uncached paint when needed, and keep overlay ordering explicit.
  - Progress: cached-edge tile geometry plus cached render-data/build-state initialization now also
    route through the private `canvas/widget/paint_root/cached_edges/geometry.rs` and
    `canvas/widget/paint_root/cached_edges/build_state.rs` helpers, so the edge/label cache seams
    stop re-embedding tile-rect math, cull inflation, and render-data collection boilerplate.
  - Progress: root edge-anchor target selection now also routes through the private
    `canvas/widget/paint_root/edge_anchor.rs` seam, so `paint_root/immediate.rs` and
    `paint_root/cached_edges/mod.rs` stop re-embedding the same reconnectability gate and anchor
    target resolution logic while keeping cached-vs-immediate data sourcing explicit.
  - Progress: static scene cache tile/window planning now also routes through the private
    `canvas/widget/static_scene_cache_plan.rs` seam, so `paint_root/cached.rs` and
    `retained_widget_cull_window_key.rs` stop re-embedding the same power-of-two tile sizing and
    centered single-tile window math inline.
  - Progress: root frame/bootstrap orchestration now also routes through the private
    `canvas/widget/paint_root/frame.rs` seam, so `paint_root/cached.rs` stops re-embedding cache
    begin-frame bookkeeping, path-cache diagnostics publication, viewport/cull setup, canvas
    background fill, and grid paint bootstrap inline.
  - Progress: root cache-plan orchestration now also routes through the private
    `canvas/widget/paint_root/cache_plan.rs` seam, so `paint_root/cached.rs` stops re-embedding
    hovered-edge resolution, derived geometry publication, static cache eligibility, tile sizing,
    cache-rect selection, and style/base-key planning inline.
  - Progress: cached-path render tail orchestration now also routes through the private
    `canvas/widget/paint_root/cached_pass.rs` seam, so `paint_root/cached.rs` stops re-embedding
    the groups/edges/nodes cached pass ordering, anchor tail, overlay tail, prune tail, and clip
    pop inline.
  - Progress: immediate-path render pass plus shared paint-root finish tail now also route through
    the private `canvas/widget/paint_root/immediate_pass.rs` and
    `canvas/widget/paint_root/tail.rs` seams, so `paint_root/immediate.rs` and
    `paint_root/cached_pass.rs` stop re-embedding the immediate draw ordering plus the shared
    anchors/overlays/prune/pop-clip tail inline.
  - Progress: cached edge build-state initialization and budget-step tails now route through
    smaller private helpers in `canvas/widget/paint_root/cached_edges/build_state.rs`, so the
    edge-vs-label cached build path keeps only the budget function choice and state-specific fields
    at the root instead of re-embedding the same clip-op setup and next-edge replay tail inline.
  - Progress: cached edge root-shell uncached fallback and tile preparation now also route through
    smaller helpers in `canvas/widget/paint_root/cached_edges/edges.rs` and
    `canvas/widget/paint_root/cached_edges/geometry.rs`, so `single_rect.rs` and `tile_path.rs`
    mainly keep cache-mode choice, overlay ordering, and label-pass orchestration at the root.
  - Progress: cached edge-label replay and finished-store tails now also route through smaller
    helpers in `canvas/widget/paint_root/cached_edges/labels.rs`, so the label cache paths stop
    re-embedding the same translated replay and empty-vs-populated finished-store bookkeeping.
  - Progress: cached edge replay and finished-store tails now also route through smaller helpers in
    `canvas/widget/paint_root/cached_edges/edges.rs`, so the edge cache paths stop re-embedding the
    same translated replay and finished-store bookkeeping when single-rect and tiled passes share
    the same partially built state.
  - Progress: cached static group/node replay and store tails now also route through the private
    `canvas/widget/paint_root/static_cache.rs` seam, so `cached_groups.rs` and `cached_nodes.rs`
    stop re-embedding the same cache replay/store-and-replay bookkeeping while keeping the
    render-data collection and static paint bodies explicit at the root.
  - Progress: cached static group/node layer-key planning now also routes through the same private
    `canvas/widget/paint_root/static_cache.rs` seam, so `cached_groups.rs` and `cached_nodes.rs`
    stop re-embedding the same base-key/style-key/tile-origin cache key assembly inline.
  - Progress: paint-root cache prune tails now also route through smaller private helpers in
    `canvas/widget/paint_root/prune.rs`, so the root prune entry keeps static tile-cache cleanup
    and dynamic paint-cache cleanup as explicit, separately reviewable responsibilities.
  - Progress: selected-node overlay and dynamic-node paint tails now also route through the private
    `canvas/widget/paint_root/node_layers.rs` seam, so `cached_nodes.rs` and
    `immediate_pass.rs` stop re-embedding the same selected-node replay and dynamic-node overlay
    tail while keeping static node paint ordering explicit at the root.
  - Progress: selected-group overlay rect replay now also routes through shared helpers in
    `ui/canvas/widget/paint_groups.rs`, so `paint_root/cached_groups.rs` stops re-embedding the
    same selected-group rect collection and quad replay tail inline while keeping static group
    cache orchestration explicit at the root.
  - Progress: widget-surface color-mode / skin / paint-override sync now also routes through the
    private `ui/canvas/widget/widget_surface/sync.rs` seam, so `widget_surface.rs` stops
    re-embedding the same geometry-reset and scene-cache/build-state invalidation tails inline
    while keeping construction and builder-style surface composition explicit at the root.
  - Progress: widget-surface fit-view-on-mount builder/runtime now also routes through the private
    `ui/canvas/widget/widget_surface/fit_view.rs` seam, so `widget_surface.rs` stops re-embedding
    the same fit-on-mount option setup, node-id collection, and one-shot framing tail inline while
    keeping the public builder surface unchanged.
  - Progress: widget-surface style/transport builders now also route through the private
    `ui/canvas/widget/widget_surface/builders.rs` seam, so `widget_surface.rs` stops re-embedding
    the same style-reset, geometry-reset, and transport-key reset tails inline while keeping the
    app-facing builder API unchanged.
  - Progress: widget-surface construction and middleware transplant now also route through the
    private `ui/canvas/widget/widget_surface/construct.rs` seam, so `widget_surface.rs` stops
    re-embedding the same default state allocation and cross-middleware field transplant block
    inline while keeping the public constructor/composition API unchanged.
  - Progress: widget-surface runtime helpers now also route through the private
    `ui/canvas/widget/widget_surface/runtime.rs` seam, so `widget_surface.rs` stops re-embedding
    the same render-cull, debug-metrics, interaction-state, and edge-path helper bodies inline.
  - Progress: widget-surface output/diagnostics builders now also route through the same private
    `ui/canvas/widget/widget_surface/builders.rs` seam, so `widget_surface.rs` no longer keeps the
    measured-output, internals, and diagnostics-anchor builder tails inline.
  - Progress: paint-render-data node visibility and payload assembly now also route through the
    private `ui/canvas/widget/paint_render_data/nodes.rs` seam, so `collect.rs` and
    `selected_nodes.rs` stop re-embedding the same node chrome/ports payload build tail and
    visible-node ordering logic inline.
  - Progress: paint-render-data group collection now also routes through the private
    `ui/canvas/widget/paint_render_data/groups.rs` seam, so `collect.rs` stops re-embedding the
    same group ordering, preview-rect projection, cull filtering, and metrics bookkeeping inline.
  - Progress: paint-render-data edge candidate selection, hint resolution, cull filtering, and
    render payload assembly now also route through the private
    `ui/canvas/widget/paint_render_data/edges.rs` seam, so `collect.rs` stops re-embedding the
    same edge iteration, override application, bounds rejection, rank calculation, and stable sort
    tail inline.
  - Progress: full node-paint insert-preview, node chrome/body, and port/pin tails now also route
    through the private `ui/canvas/widget/paint_nodes/full_preview.rs`,
    `ui/canvas/widget/paint_nodes/full_nodes.rs`, and
    `ui/canvas/widget/paint_nodes/full_ports.rs` seams, so `paint_nodes/full.rs` now mainly keeps
    shared paint setup, skin hint collection, and top-level draw ordering explicit.
  - Progress: dynamic selected-node chrome/ring logic and port-adorners now also route through the
    private `ui/canvas/widget/paint_nodes/dynamic_nodes.rs` and
    `ui/canvas/widget/paint_nodes/dynamic_ports.rs` seams, while
    `paint_nodes/dynamic_from_geometry.rs` reuses the shared insert-preview helper and now mainly
    keeps transient paint setup plus top-level orchestration explicit.
  - Progress: static node chrome/text and static port-label/shape paint now also route through the
    private `ui/canvas/widget/paint_nodes/static_node_chrome.rs` and
    `ui/canvas/widget/paint_nodes/static_ports.rs` seams, so `paint_nodes/static_nodes.rs` now
    mainly keeps shared paint setup plus top-level node/port pass ordering explicit.
  - Progress: context-menu connection insert/conversion execution now also routes through the
    private `ui/canvas/widget/context_menu/connection_execution_insert.rs` and
    `ui/canvas/widget/context_menu/connection_execution_conversion.rs` seams, so
    `context_menu/connection_execution.rs` now mainly keeps the plan enums and focused tests.
  - Progress: edge marker-path planning and wire/highlight replay helpers now also route through the
    private `ui/canvas/widget/paint_edges/markers_support.rs` seam, so
    `paint_edges/markers.rs` now mainly keeps the regular-vs-custom marker orchestration explicit.
  - Progress: `ui/canvas/paint.rs` now routes wire-path prep, port-shape factories, edge-marker
    factories, and text cache helpers through the private `canvas/paint/paint_wire.rs`,
    `canvas/paint/paint_ports.rs`, `canvas/paint/paint_markers.rs`, and
    `canvas/paint/paint_text.rs` seams, so the root paint module now mainly holds cache state,
    shared key types, and lifecycle/prune orchestration.
  - Progress: `ui/canvas/spatial.rs` now routes coarse index construction, port-edge adjacency,
    and derived spatial wrapper helpers through the private `canvas/spatial/spatial_index.rs`,
    `canvas/spatial/spatial_adjacency.rs`, and `canvas/spatial/spatial_derived.rs` seams, so the
    root spatial module now mainly holds shared type definitions plus tests.
  - Progress: `ui/canvas/state.rs` now routes paste-series stepping, viewport easing, and geometry
    cache preview/key helpers through the private `canvas/state/state_paste_series.rs`,
    `canvas/state/state_viewport_animation.rs`, and `canvas/state/state_geometry_cache.rs` seams,
    so the root state module now mainly holds shared state/data types plus tests.
  - Progress: `ui/canvas/state.rs` now also routes menu/searcher/toast/paste session structs
    through the private `canvas/state/state_overlay_sessions.rs` seam, and derived geometry cache
    key / preview cache structs through the private `canvas/state/state_preview_cache.rs` seam, so
    the root state module keeps shrinking toward a pure state shell without changing state paths.
  - Progress: `ui/canvas/state.rs` now also routes insert/node/group/marquee/wire/edge drag
    session structs through the private `canvas/state/state_drag_sessions.rs` seam, so the root
    state module no longer re-embeds the full drag-session data inventory inline.
  - Progress: `ui/canvas/workflow.rs` now routes wire-drop insert planning through the private
    `canvas/workflow/workflow_insert.rs` seam, so the root workflow module now mainly holds the
    shared plan type, root re-export, and tests.
  - Progress: `ui/canvas/searcher.rs` now routes query scoring/normalization and row builders
    through the private `canvas/searcher/searcher_score.rs` and
    `canvas/searcher/searcher_build.rs` seams, so the root searcher module now mainly holds shared
    row types, constants, and root re-exports.
  - Progress: `ui/canvas/middleware.rs` now routes middleware chaining and transaction validation
    adapters through the private `canvas/middleware/middleware_chain.rs` and
    `canvas/middleware/middleware_validation.rs` seams, so the root middleware module now mainly
    holds shared trait/context/outcome types plus root re-exports.
  - Progress: `ui/canvas/route_math.rs` now routes curve primitives and route tangent helpers
    through the private `canvas/route_math/route_math_curve.rs` and
    `canvas/route_math/route_math_tangent.rs` seams, while `ui/canvas/conversion.rs` now routes
    conversion candidate building and insert-plan helpers through the private
    `canvas/conversion/conversion_candidates.rs` and
    `canvas/conversion/conversion_plan.rs` seams, so both root modules now mainly hold root
    re-exports and shared imports.
  - Progress: `ui/canvas/snaplines.rs` now routes snap-anchor extraction and best-guide delta
    selection through the private `canvas/snaplines/snaplines_align.rs` seam, so the root
    snaplines module now mainly holds shared result types plus tests.
  - Progress: align/distribute planning now also routes element collection, per-mode delta
    planning, extent-shift computation, and group/node op application through the private
    `ui/canvas/widget/move_ops/align_distribute/support.rs` seam, so
    `move_ops/align_distribute/plan.rs` now mainly keeps the top-level planning orchestration
    explicit.
  - Progress: nudge move planning now also routes moved-set collection, shared extent clamps, and
    group/node op application through the private
    `ui/canvas/widget/move_ops/nudge_support.rs` seam, so `move_ops/nudge.rs` now mainly keeps
    delta normalization, snap-to-grid primary selection handling, and top-level orchestration
    explicit.
  - Progress: node-resize math now also routes rect utilities and resize-handle geometry/clamp
    flow through the private `ui/canvas/widget/node_resize/math/rects.rs` and
    `ui/canvas/widget/node_resize/math/resize.rs` seams, so `node_resize/math.rs` now mainly
    keeps the root re-exports and focused resize conformance tests explicit.
  - Progress: press-session preparation now also routes session clearing helpers and hit-specific
    preparation profiles through the private `ui/canvas/widget/press_session/clear.rs` and
    `ui/canvas/widget/press_session/prepare.rs` seams, so `press_session.rs` now mainly keeps the
    root re-exports and focused interaction-state fixture tests explicit.
  - Progress: pending pointer-up release routing now also routes click-selection, generic pending
    release, and click-connect promotion through the private
    `ui/canvas/widget/pointer_up_pending/click_select.rs`,
    `ui/canvas/widget/pointer_up_pending/release.rs`, and
    `ui/canvas/widget/pointer_up_pending/wire_drag.rs` seams, so `pointer_up_pending.rs` now
    mainly keeps the root re-exports explicit while the click-threshold and click-connect policy
    helpers gain focused unit coverage.
  - Progress: pointer-up resize commit op building now also routes node resize and group resize
    planners through the private `ui/canvas/widget/pointer_up_commit_resize/node.rs` and
    `ui/canvas/widget/pointer_up_commit_resize/group.rs` seams, so
    `pointer_up_commit_resize.rs` now mainly keeps the root re-exports explicit while each resize
    planner keeps its own focused unit coverage.
  - Progress: left-button pointer-up routing now also routes edge-insert double-click activation
    and the release arbitration chain through the private
    `ui/canvas/widget/pointer_up_left_route/double_click.rs` and
    `ui/canvas/widget/pointer_up_left_route/dispatch.rs` seams, so
    `pointer_up_left_route.rs` now mainly keeps stop-auto-pan plus top-level orchestration
    explicit while the plain-double-click gate keeps focused unit coverage.
  - Progress: committed pointer-up release handling now also routes resize and group-drag commit
    branches through the private `ui/canvas/widget/pointer_up_commit/resize.rs` and
    `ui/canvas/widget/pointer_up_commit/group_drag.rs` seams, so `pointer_up_commit.rs` now mainly
    keeps root re-exports plus node-drag delegation explicit while the commit wrappers stop
    accumulating inline orchestration.
  - Progress: pointer-up state synchronization and release guards now also route through the
    private `ui/canvas/widget/pointer_up_state/sync.rs` and
    `ui/canvas/widget/pointer_up_state/release.rs` seams, so `pointer_up_state.rs` now mainly
    keeps root re-exports explicit while pointer-state projection and sticky-wire/pan release
    branches stop living inline together.
  - Progress: pointer-up session helpers now also route generic release-slot handling and
    interaction cleanup through the private `ui/canvas/widget/pointer_up_session/release.rs` and
    `ui/canvas/widget/pointer_up_session/cleanup.rs` seams, so `pointer_up_session.rs` now mainly
    keeps root re-exports explicit while pending-release and snap-guide cleanup helpers stop
    sharing one inline module body.
  - Progress: focus-session helpers now also route hint clearing, focus transitions, and
    selection-only view-state updates through the private
    `ui/canvas/widget/focus_session/hints.rs`,
    `ui/canvas/widget/focus_session/focus.rs`, and
    `ui/canvas/widget/focus_session/selection.rs` seams, so `focus_session.rs` now mainly keeps
    root re-exports explicit while edge/port/node focus bookkeeping stops sharing one inline
    helper file.
  - Progress: cancel-session helpers now also route residual interaction cleanup and pan-release
    state helpers through the private `ui/canvas/widget/cancel_session/residuals.rs` and
    `ui/canvas/widget/cancel_session/pan.rs` seams, so `cancel_session.rs` now mainly keeps root
    re-exports explicit while sticky-wire/right-click cleanup and pan-release matching stop sharing
    one inline helper file.
  - Progress: gesture-cancel handling now also routes wire-drag cancel callbacks and the remaining
    session clears through the private `ui/canvas/widget/cancel_gesture_state/wire.rs` and
    `ui/canvas/widget/cancel_gesture_state/sessions.rs` seams, so
    `cancel_gesture_state.rs` now mainly keeps top-level orchestration explicit while the bulk
    session reset logic gains focused state-only coverage.
  - Progress: interaction gating now also routes cursor-detail, edge-hover, cache, and
    pan-inertia predicates through the private `ui/canvas/widget/interaction_gate/detail.rs`,
    `ui/canvas/widget/interaction_gate/hover.rs`, `ui/canvas/widget/interaction_gate/cache.rs`,
    and `ui/canvas/widget/interaction_gate/motion.rs` seams, so `interaction_gate.rs` now mainly
    keeps the gate surface explicit while each predicate family gains focused unit coverage.
  - Progress: reconnect helpers now also route port-edge yank logic and reconnectable flag
    predicates through the private `ui/canvas/widget/reconnect/edges.rs` and
    `ui/canvas/widget/reconnect/flags.rs` seams, so `reconnect.rs` now mainly keeps the module
    split explicit while reconnect eligibility and endpoint derivation gain focused unit coverage.
  - Progress: selection helpers now also route marquee edge-derivation and selectable predicates
    through the private `ui/canvas/widget/selection/box_edges.rs` and
    `ui/canvas/widget/selection/selectable.rs` seams, so `selection.rs` now mainly keeps the
    module split explicit while box-select edge modes and selectable overrides gain focused unit
    coverage.
  - Progress: interaction policy helpers now also route node drag/connectable predicates plus
    port connectable/bundle checks through the private
    `ui/canvas/widget/interaction_policy/node.rs` and
    `ui/canvas/widget/interaction_policy/port.rs` seams, so `interaction_policy.rs` now mainly
    keeps the module split explicit while per-node and per-port policy overrides gain focused unit
    coverage.
  - Progress: view commands now also route frame-all selection collection plus reset/zoom viewport
    helpers through the private `ui/canvas/widget/command_view/frame.rs` and
    `ui/canvas/widget/command_view/zoom.rs` seams, so `command_view.rs` now mainly keeps the
    module split explicit while frame-node collection and reset/zoom helper behavior gain focused
    unit coverage.
  - Progress: hover-edge updates now also route target-edge resolution, hover hit queries, and
    hover-state sync through the private `ui/canvas/widget/hover/target.rs`,
    `ui/canvas/widget/hover/hit.rs`, and `ui/canvas/widget/hover/state.rs` seams, so `hover.rs`
    now mainly keeps the orchestration explicit while edge-target precedence and hover-state diff
    behavior gain focused unit coverage.
  - Progress: command routing now also routes string-to-command dispatch through the private
    `ui/canvas/widget/command_router/dispatch.rs` seam, so `command_router.rs` now mainly keeps
    execution dispatch explicit while direct command aliases and canonical route mapping gain
    focused unit coverage.
  - Progress: graph construction helpers now also route reroute-node op assembly and group-create
    selection/update helpers through the private `ui/canvas/widget/graph_construction/node.rs` and
    `ui/canvas/widget/graph_construction/group.rs` seams, so `graph_construction.rs` now mainly
    keeps the module split explicit while reroute/group construction helpers gain focused unit
    coverage.
  - Progress: pending drag session helpers now also route group/node activation and node-abort
    behavior through the private `ui/canvas/widget/pending_drag_session/group.rs` and
    `ui/canvas/widget/pending_drag_session/node.rs` seams, so `pending_drag_session.rs` now mainly
    keeps the re-export surface explicit while pending drag activation helpers gain focused unit
    coverage.
  - Progress: group paint helpers now also route static chrome/text layout and selected overlay
    filtering through the private `ui/canvas/widget/paint_groups/chrome.rs` and
    `ui/canvas/widget/paint_groups/overlay.rs` seams, so `paint_groups.rs` now mainly keeps the
    module split explicit while zoom-scaled group chrome and selected-overlay filtering gain
    focused unit coverage.
  - Progress: press-session prepare helpers now also route target-hit and surface/pan preparation
    through the private `ui/canvas/widget/press_session/prepare/target.rs` and
    `ui/canvas/widget/press_session/prepare/surface.rs` seams, so `press_session/prepare.rs` now
    mainly keeps the re-export surface explicit while pointer-session clearing variants gain
    focused unit coverage.
  - Progress: wire-drag hint paint helpers now also route hint message and border-color semantics
    through the private `ui/canvas/widget/paint_overlay_wire_hint/message.rs` and
    `ui/canvas/widget/paint_overlay_wire_hint/style.rs` seams, so `paint_overlay_wire_hint.rs`
    now mainly keeps the paint orchestration explicit while invalid-hover diagnostics and bundle/yank
    hint semantics gain focused unit coverage.
  - Progress: toast overlay paint helpers now also route zoom-scaled layout and severity/style
    semantics through the private `ui/canvas/widget/paint_overlay_toast/layout.rs` and
    `ui/canvas/widget/paint_overlay_toast/style.rs` seams, so `paint_overlay_toast.rs` now mainly
    keeps the paint orchestration explicit while toast placement and severity color mapping gain
    focused unit coverage.
  - Progress: pointer-down routing now also routes double-click arbitration and tail-lane dispatch
    through the private `ui/canvas/widget/event_pointer_down_route/double_click.rs` and
    `ui/canvas/widget/event_pointer_down_route/dispatch.rs` seams, so
    `event_pointer_down_route.rs` now mainly keeps early-return orchestration explicit while button
    lane selection retains focused unit coverage.
  - Progress: grid-tile paint helpers now also route tile-index projection and pattern-density
    capacity estimation through the private `ui/canvas/widget/paint_grid_tiles/support.rs` seam, so
    `paint_grid_tiles.rs` now mainly keeps pattern-to-painter orchestration explicit while tile
    bounds projection and capacity heuristics gain focused unit coverage.
  - Progress: keyboard-shortcut mapping now also routes modifier/history bindings and tab/arrow
    navigation bindings through the private `ui/canvas/widget/keyboard_shortcuts_map/modifier.rs`
    and `ui/canvas/widget/keyboard_shortcuts_map/navigation.rs` seams, so
    `keyboard_shortcuts_map.rs` now mainly keeps the re-export surface explicit while shortcut
    family mapping retains focused unit coverage.
  - Progress: keyboard-shortcut gating now also routes modifier, navigation, and delete-binding
    predicates through the private `ui/canvas/widget/keyboard_shortcuts_gate/modifier.rs`,
    `ui/canvas/widget/keyboard_shortcuts_gate/navigation.rs`, and
    `ui/canvas/widget/keyboard_shortcuts_gate/editing.rs` seams, so
    `keyboard_shortcuts_gate.rs` now mainly keeps the re-export surface explicit while per-family
    gate predicates retain focused unit coverage.
  - Progress: overlay hit helpers now also route context-menu geometry/item hit-testing and
    searcher geometry/row hit-testing through the private
    `ui/canvas/widget/overlay_hit/context_menu.rs` and
    `ui/canvas/widget/overlay_hit/searcher.rs` seams, so `overlay_hit.rs` now mainly keeps the
    re-export surface explicit while overlay hit geometry retains focused unit coverage.
  - Progress: viewport math helpers now also route viewport construction/clamp helpers and canvas
    snap helpers through the private `ui/canvas/widget/view_math_viewport/viewport.rs` and
    `ui/canvas/widget/view_math_viewport/snap.rs` seams, so `view_math_viewport.rs` now mainly
    keeps the re-export surface explicit while viewport construction equivalence and snap behavior
    retain focused unit coverage.
  - Progress: delete-op building now also routes group, node, and edge removal planners through the
    private `ui/canvas/widget/delete_ops_builder/group.rs`,
    `ui/canvas/widget/delete_ops_builder/node.rs`, and
    `ui/canvas/widget/delete_ops_builder/edge.rs` seams, so `delete_ops_builder.rs` now mainly
    keeps top-level delete orchestration explicit while edge de-duplication across node removal
    retains focused unit coverage.
  - Progress: delete command helpers now also route remove-op collection and selection/view cleanup
    through the private `ui/canvas/widget/command_edit_remove/collect.rs` and
    `ui/canvas/widget/command_edit_remove/apply.rs` seams, so `command_edit_remove.rs` now mainly
    keeps cut/delete command orchestration explicit while remove-op collection and commit/view-state
    cleanup stop sharing one inline tail.
  - Progress: right-click helpers now also route pending-release handling and click-threshold
    predicates through the private `ui/canvas/widget/right_click/pending.rs` and
    `ui/canvas/widget/right_click/threshold.rs` seams, so `right_click.rs` now mainly keeps the
    public helper surface explicit while pending click-threshold behavior retains focused unit
    coverage.
  - Progress: searcher activation hit helpers now also route pointer-hit geometry and candidate-row
    lookup through the private `ui/canvas/widget/searcher_activation_hit/hit.rs` and
    `ui/canvas/widget/searcher_activation_hit/candidate.rs` seams, so
    `searcher_activation_hit.rs` now mainly keeps the re-export surface explicit while candidate-row
    mapping retains focused unit coverage.
  - Progress: searcher activation state helpers now also route clear/dismiss, row-arm, and
    release/activation tails through the private `ui/canvas/widget/searcher_activation_state/clear.rs`,
    `ui/canvas/widget/searcher_activation_state/arm.rs`, and
    `ui/canvas/widget/searcher_activation_state/release.rs` seams, so
    `searcher_activation_state.rs` now mainly keeps the re-export surface explicit while searcher
    overlay clearing retains focused unit coverage.
  - Progress: searcher wheel helpers now also route scroll-delta application through the private
    `ui/canvas/widget/searcher_pointer_wheel/delta.rs` seam, so
    `searcher_pointer_wheel.rs` now mainly keeps canvas-level wheel routing explicit while scroll
    clamping behavior retains focused unit coverage.
  - Progress: searcher hover helpers now also route hovered-row state sync through the private
    `ui/canvas/widget/searcher_pointer_hover/state.rs` seam, so
    `searcher_pointer_hover.rs` now mainly keeps pointer-position to hovered-row orchestration
    explicit while hovered-row promotion behavior retains focused unit coverage.
  - Progress: searcher navigation helpers now also route active-row step planning through the
    private `ui/canvas/widget/searcher_input_nav/step.rs` seam, so
    `searcher_input_nav.rs` now mainly keeps canvas-level active-row update orchestration explicit
    while selectable-row step planning retains focused unit coverage.
  - Progress: view/gesture callback helpers now also route viewport lifecycle, node-drag gesture,
    and view-change fanout through the private `ui/canvas/widget/callbacks_view/viewport.rs`,
    `ui/canvas/widget/callbacks_view/node_drag.rs`, and
    `ui/canvas/widget/callbacks_view/view_change.rs` seams, so `callbacks_view.rs` now mainly
    keeps the re-export surface explicit while retained callback emission stops accumulating
    unrelated gesture/view tails inline.
  - Progress: auto-measure sizing helpers now also route text-metric and width-planning logic
    through the private `ui/canvas/widget/auto_measure_apply/measure.rs` seam, so
    `auto_measure_apply.rs` now mainly keeps size-apply synchronization explicit while measured
    width planning stops sharing the same inline helper body.
  - Progress: retained callback connect/graph helpers now also route wire-drag kind mapping and
    committed connection/delete fanout through the private
    `ui/canvas/widget/callbacks_connect/kind.rs`,
    `ui/canvas/widget/callbacks_graph/connection.rs`, and
    `ui/canvas/widget/callbacks_graph/delete.rs` seams, so `callbacks_connect.rs` and
    `callbacks_graph.rs` now mainly keep lifecycle orchestration explicit while callback payload
    mapping stops accumulating inline in the root helpers.
  - Progress: auto-measure cache-key, per-node collect, and measured-size apply tails now also
    route through the private `ui/canvas/widget/auto_measure/key.rs`,
    `ui/canvas/widget/auto_measure_collect/input.rs`, and
    `ui/canvas/widget/auto_measure_apply/apply.rs` seams, so the `auto_measure*` roots now mainly
    keep cache invalidation and pipeline orchestration explicit while the collect/apply tails stop
    sharing root helper bodies.
  - Progress: searcher query-edit and row-state helpers now also route key-to-query mutation plus
    recent-kind and active-row/scroll maintenance through the private
    `ui/canvas/widget/searcher_input_query/query.rs`,
    `ui/canvas/widget/searcher_rows/recent.rs`, and
    `ui/canvas/widget/searcher_rows/active.rs` seams, so `searcher_input_query.rs` and
    `searcher_rows.rs` now mainly keep canvas-level orchestration explicit while query mutation and
    row-state tails stop accumulating in the root helpers.
  - Progress: searcher pointer activation now also routes pointer-down and pointer-up event tails
    through the private `ui/canvas/widget/searcher_activation/pointer_down.rs` and
    `ui/canvas/widget/searcher_activation/pointer_up.rs` seams, so `searcher_activation.rs` now
    mainly keeps shared hit shape plus activation-state façade methods explicit while event tails
    stop sharing the same root helper body.
  - Progress: searcher picker and row-activation helpers now also route picker-request assembly,
    overlay-open tails, and activation-item mapping through the private
    `ui/canvas/widget/searcher_picker/catalog.rs`,
    `ui/canvas/widget/searcher_picker/overlay.rs`, and
    `ui/canvas/widget/searcher_row_activation/item.rs` seams, so `searcher_picker.rs` and
    `searcher_row_activation.rs` now mainly keep canvas-level orchestration explicit while picker
    request shaping and activation-item validation gain their own helper boundaries.
  - Progress: searcher keyboard/input and overlay UI helpers now also route key dispatch plus
    overlay install/open and dismiss/finish tails through the private
    `ui/canvas/widget/searcher_input/dispatch.rs`,
    `ui/canvas/widget/searcher_ui/overlay.rs`, and
    `ui/canvas/widget/searcher_ui/event.rs` seams, so `searcher_input.rs` and `searcher_ui.rs`
    now mainly keep façade methods explicit while key routing and overlay event tails stop
    accumulating in the root files.
  - Progress: searcher pointer helpers now also route pointer-move and wheel event tails through
    the private `ui/canvas/widget/searcher_pointer/move_event.rs` and
    `ui/canvas/widget/searcher_pointer/wheel_event.rs` seams, so `searcher_pointer.rs` now mainly
    keeps façade forwarding explicit while pointer invalidation tails stop sharing the same root
    helper body.
  - Progress: menu/searcher session builders now also route context-menu state assembly and
    searcher state/row builders through the private
    `ui/canvas/widget/menu_session/context_menu.rs` and
    `ui/canvas/widget/menu_session/searcher.rs` seams, so `menu_session.rs` now mainly keeps the
    shared session-builder surface explicit while context-menu and searcher state assembly stop
    sharing one root helper body.
  - Progress: insert-candidate helpers now also route reroute candidate synthesis, menu-item
    mapping, and presenter-backed candidate list loading through the private
    `ui/canvas/widget/insert_candidates/reroute.rs`,
    `ui/canvas/widget/insert_candidates/menu.rs`, and
    `ui/canvas/widget/insert_candidates/list.rs` seams, so `insert_candidates.rs` now mainly
    keeps the façade surface explicit while candidate synthesis and list loading stop sharing the
    same root helper body.
  - Progress: group open-command helpers now also route create, draw-order, and rename overlay
    tails through the private `ui/canvas/widget/command_open_group/create.rs`,
    `ui/canvas/widget/command_open_group/order.rs`, and
    `ui/canvas/widget/command_open_group/rename.rs` seams, so `command_open_group.rs` now mainly
    keeps the command façade surface explicit while group command tails stop sharing one root
    helper body.
  - Progress: insert/edge/conversion open-command helpers now also route insert fallback math,
    edge picker/reroute command tails, and conversion overlay open tails through the private
    `ui/canvas/widget/command_open_insert/fallback.rs`,
    `ui/canvas/widget/command_open_edge/picker.rs`,
    `ui/canvas/widget/command_open_edge/reroute.rs`, and
    `ui/canvas/widget/command_open_conversion/overlay.rs` seams, so the remaining
    `command_open_*` roots now mainly keep façade forwarding explicit.
  - Progress: context-menu activation dispatch now also routes command actions and target-specific
    activation branches through the private `ui/canvas/widget/context_menu/activate/command.rs`
    and `ui/canvas/widget/context_menu/activate/target.rs` seams, so
    `context_menu/activate.rs` now mainly keeps the top-level dispatch surface explicit.
  - Progress: context-menu item builders now also route shared command-item construction plus
    background/group/edge item families through the private
    `ui/canvas/widget/context_menu/item_builders/command_item.rs`,
    `ui/canvas/widget/context_menu/item_builders/background.rs`,
    `ui/canvas/widget/context_menu/item_builders/group.rs`, and
    `ui/canvas/widget/context_menu/item_builders/edge.rs` seams, so
    `context_menu/item_builders.rs` now mainly keeps the public builder surface explicit.
  - Progress: context-menu selection activation now also routes activation-payload assembly and
    pointer-down activation tails through the private
    `ui/canvas/widget/context_menu/selection_activation/payload.rs` and
    `ui/canvas/widget/context_menu/selection_activation/pointer_down.rs` seams, so
    `context_menu/selection_activation.rs` now mainly keeps the selection façade explicit.
  - Progress: context-menu opening now also routes group-hit, edge-hit, and background fallback
    branches through the private `ui/canvas/widget/context_menu/opening/group.rs`,
    `ui/canvas/widget/context_menu/opening/edge.rs`, and
    `ui/canvas/widget/context_menu/opening/background.rs` seams, so
    `context_menu/opening.rs` now mainly keeps the right-click orchestration explicit.
  - Progress: keyboard context-menu navigation now also routes active-item stepping, typeahead,
    hover sync, key handling, and pointer-move tails through the private
    `ui/canvas/widget/context_menu/key_navigation/active_item.rs`,
    `ui/canvas/widget/context_menu/key_navigation/typeahead.rs`,
    `ui/canvas/widget/context_menu/key_navigation/hover.rs`,
    `ui/canvas/widget/context_menu/key_navigation/key_down.rs`, and
    `ui/canvas/widget/context_menu/key_navigation/pointer_move.rs` seams, so
    `context_menu/key_navigation.rs` now mainly keeps the navigation façade explicit.
  - Progress: background context-menu execution now also routes insert planning, plan application,
    and action activation through the private
    `ui/canvas/widget/context_menu/background_execution/plan.rs`,
    `ui/canvas/widget/context_menu/background_execution/apply.rs`, and
    `ui/canvas/widget/context_menu/background_execution/activate.rs` seams, so
    `context_menu/background_execution.rs` now mainly keeps the plan enum plus execution façade
    explicit.
  - Progress: connection insert/conversion menu execution now also routes activation, planning,
    plan application, and wire-drag recovery through the private
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
  - Progress: context target hit/selection now also routes group-vs-edge hit tests plus
    group-vs-edge selection reducers through the private
    `ui/canvas/widget/context_menu/target_hit/group.rs`,
    `ui/canvas/widget/context_menu/target_hit/edge.rs`,
    `ui/canvas/widget/context_menu/target_selection/group.rs`, and
    `ui/canvas/widget/context_menu/target_selection/edge.rs` seams, so
    `context_menu/target_hit.rs` and `context_menu/target_selection.rs` now mainly keep the
    façade surfaces explicit.
  - Progress: edge context-menu execution now also routes open-picker, reroute, delete, and custom
    action branches through the private
    `ui/canvas/widget/context_menu/edge_execution/open_insert.rs`,
    `ui/canvas/widget/context_menu/edge_execution/reroute.rs`,
    `ui/canvas/widget/context_menu/edge_execution/delete.rs`, and
    `ui/canvas/widget/context_menu/edge_execution/custom_action.rs` seams, so
    `context_menu/edge_execution.rs` now mainly keeps the edge action dispatch explicit.
  - Progress: split-edge reroute execution now also routes reroute planning, rejection-toast
    mapping, commit/apply tails, and outcome execution through the private
    `ui/canvas/widget/split_edge_execution/plan.rs`,
    `ui/canvas/widget/split_edge_execution/toast.rs`,
    `ui/canvas/widget/split_edge_execution/apply.rs`, and
    `ui/canvas/widget/split_edge_execution/execute.rs` seams, so
    `split_edge_execution.rs` now mainly keeps the public execution façade explicit.
  - Progress: edge double-click handling now also routes insert-picker opening, reroute execution,
    double-click target hit-testing, and finish tails through the private
    `ui/canvas/widget/pointer_down_double_click_edge/insert_picker.rs`,
    `ui/canvas/widget/pointer_down_double_click_edge/reroute.rs`,
    `ui/canvas/widget/pointer_down_double_click_edge/target.rs`, and
    `ui/canvas/widget/pointer_down_double_click_edge/finish.rs` seams, and the insert-picker path
    now reuses the shared `select_edge_context_target` reducer instead of duplicating edge
    selection updates inline.
  - Progress: pointer-down gesture start handling now also routes close-button dispatch,
    pending-right-click startup, and pan-start gating through the private
    `ui/canvas/widget/pointer_down_gesture_start/close_button.rs`,
    `ui/canvas/widget/pointer_down_gesture_start/pending_right_click.rs`, and
    `ui/canvas/widget/pointer_down_gesture_start/pan_start.rs` seams, so
    `pointer_down_gesture_start.rs` now mainly keeps the gesture-start façade explicit while
    preserving the existing context-menu and sticky-wire delegation.
  - Progress: node-drag release op building now also routes release-op assembly, group-rect
    mapping, and commit-label selection through the private
    `ui/canvas/widget/pointer_up_node_drag_ops/build.rs`,
    `ui/canvas/widget/pointer_up_node_drag_ops/group_rect.rs`, and
    `ui/canvas/widget/pointer_up_node_drag_ops/commit_label.rs` seams, so
    `pointer_up_node_drag_ops.rs` now mainly keeps the public release-op façade explicit.
  - Progress: node-drag preview now also routes preview-position computation and preview-state
    revision updates through the private `ui/canvas/widget/node_drag_preview/compute.rs` and
    `ui/canvas/widget/node_drag_preview/state.rs` seams, so `node_drag_preview.rs` now mainly
    keeps the node-drag preview façade explicit while the heavy preview calculation stops living in
    one monolithic root function.
  - Progress: overlay painting now also routes close-button chrome/text paint and overlay-layer
    dispatch through the private `ui/canvas/widget/paint_overlays/close_button.rs` and
    `ui/canvas/widget/paint_overlays/layers.rs` seams, so `paint_overlays.rs` now mainly keeps the
    overlay paint orchestration explicit.
  - Progress: multi-node extent clamping now also routes dragged-bounds collection and extent-delta
    clamping through the private `ui/canvas/widget/node_drag_constraints_extent/bounds.rs` and
    `ui/canvas/widget/node_drag_constraints_extent/clamp_delta.rs` seams, so
    `node_drag_constraints_extent.rs` now mainly keeps the extent-clamp entrypoint explicit.
  - Progress: group resize planning now also routes pointer-to-rect mapping, child-size minimums,
    and snap-to-grid sizing through the private
    `ui/canvas/widget/group_resize_apply/pointer_rect.rs`,
    `ui/canvas/widget/group_resize_apply/children_min.rs`, and
    `ui/canvas/widget/group_resize_apply/snap.rs` seams, so
    `group_resize_apply.rs` now mainly keeps the resize-planning entrypoint explicit.
  - Progress: viewport timer motion now also routes animation tick advancement and move-end
    debounce handling through the private
    `ui/canvas/widget/timer_motion_viewport/animation.rs` and
    `ui/canvas/widget/timer_motion_viewport/debounce.rs` seams, so
    `timer_motion_viewport.rs` now mainly keeps the viewport timer-motion façades explicit.
  - Progress: pan-inertia timer motion now also routes stop guards and per-frame advancement
    through the private `ui/canvas/widget/timer_motion_pan_inertia/guards.rs` and
    `ui/canvas/widget/timer_motion_pan_inertia/advance.rs` seams, so
    `timer_motion_pan_inertia.rs` now mainly keeps the inertia tick orchestration explicit.
  - Progress: viewport auto-pan timers now also route delta calculation, tick policy, and timer
    start/stop through the private `ui/canvas/widget/viewport_timer_auto_pan/delta.rs`,
    `ui/canvas/widget/viewport_timer_auto_pan/policy.rs`, and
    `ui/canvas/widget/viewport_timer_auto_pan/timer.rs` seams, so
    `viewport_timer_auto_pan.rs` now mainly keeps the auto-pan timer orchestration explicit.
  - Progress: auto-pan motion ticks now also route drag-move dispatch through the private
    `ui/canvas/widget/timer_motion_auto_pan/dispatch.rs` seam, so
    `timer_motion_auto_pan.rs` now mainly keeps the auto-pan tick orchestration explicit.
  - Progress: viewport animation timers now also route animation start/stop and move-end debounce
    handling through the private `ui/canvas/widget/viewport_timer_animation/animation.rs` and
    `ui/canvas/widget/viewport_timer_animation/debounce.rs` seams, so
    `viewport_timer_animation.rs` now mainly keeps the viewport timer façades explicit.
  - Progress: sticky-wire connect handling now also routes target filtering / outcome planning and
    pointer-down finish cleanup through the private `ui/canvas/widget/sticky_wire_connect/plan.rs`
    and `ui/canvas/widget/sticky_wire_connect/finish.rs` seams, so
    `sticky_wire_connect.rs` now mainly keeps the sticky-wire connect orchestration explicit.
  - Progress: node-drag move handling now also routes drag-delta planning and move-tail pan /
    callback / invalidation through the private `ui/canvas/widget/node_drag/delta.rs` and
    `ui/canvas/widget/node_drag/tail.rs` seams, so `node_drag.rs` now mainly keeps the node-drag
    move orchestration explicit.
  - Progress: marquee selection queries now also route node hit collection and connected-edge
    selection through the private `ui/canvas/widget/marquee_selection_query/nodes.rs` and
    `ui/canvas/widget/marquee_selection_query/edges.rs` seams, so
    `marquee_selection_query.rs` now mainly keeps the marquee query orchestration explicit.
  - Progress: group draw-order reducers now also route selected-group ordering and front/back
    application through the private `ui/canvas/widget/group_draw_order/selection.rs` and
    `ui/canvas/widget/group_draw_order/apply.rs` seams, so `group_draw_order.rs` now mainly keeps
    the group draw-order orchestration explicit.
  - Progress: cursor resolution now also routes resize-handle hit resolution and edge-anchor target
    selection through the private `ui/canvas/widget/cursor_resolve/resize.rs` and
    `ui/canvas/widget/cursor_resolve/edge.rs` seams, so `cursor_resolve.rs` now mainly keeps the
    cursor-resolution façades explicit.
  - Progress: pointer-move pan-release handling now also routes missing pan-release recovery and
    pending right-click pan-start arming through the private
    `ui/canvas/widget/pointer_move_release_pan/missing_release.rs` and
    `ui/canvas/widget/pointer_move_release_pan/pending_right_click.rs` seams, so
    `pointer_move_release_pan.rs` now mainly keeps the pan-release orchestration explicit.
  - Progress: pending resize-session activation now also routes group/node activation state
    assembly through the private `ui/canvas/widget/pending_resize_session/group.rs` and
    `ui/canvas/widget/pending_resize_session/node.rs` seams, so
    `pending_resize_session.rs` now mainly keeps the resize-session activation façades explicit.
  - Progress: drag-threshold checks now also route threshold normalization and squared-distance
    comparison through the private `ui/canvas/widget/threshold/normalize.rs` and
    `ui/canvas/widget/threshold/distance.rs` seams, so `threshold.rs` now mainly keeps the drag
    threshold façade explicit.
  - Progress: pending node-drag startup now also routes threshold/draggable gating and
    selection/start-node activation through the private `ui/canvas/widget/pending_drag/checks.rs`
    and `ui/canvas/widget/pending_drag/activate.rs` seams, so `pending_drag.rs` now mainly keeps
    the pending node-drag orchestration explicit.
  - Progress: group-drag move handling now also routes auto-pan / snap-aware delta planning and
    move-tail preview-state commit through the private `ui/canvas/widget/group_drag/delta.rs` and
    `ui/canvas/widget/group_drag/tail.rs` seams, so `group_drag.rs` now mainly keeps the
    group-drag move orchestration explicit.
  - Progress: pending group-drag startup now also routes threshold gating and grouped start-node
    discovery through the private `ui/canvas/widget/pending_group_drag/checks.rs` and
    `ui/canvas/widget/pending_group_drag/activate.rs` seams, so `pending_group_drag.rs` now mainly
    keeps the pending group-drag orchestration explicit.
  - Progress: group-resize move handling now also routes auto-pan-adjusted pointer planning and
    move-tail preview-state commit through the private `ui/canvas/widget/group_resize/pointer.rs`
    and `ui/canvas/widget/group_resize/tail.rs` seams, so `group_resize.rs` now mainly keeps the
    group-resize move orchestration and hit façades explicit.
  - Progress: pending group-resize startup now also routes threshold gating and activation
    forwarding through the private `ui/canvas/widget/pending_group_resize/checks.rs` and
    `ui/canvas/widget/pending_group_resize/activate.rs` seams, so
    `pending_group_resize.rs` now mainly keeps the pending group-resize orchestration explicit.
  - Progress: left-click group/background entry handling now also routes session prepare,
    group-selection updates, background interaction branching, and pending group drag/resize arming
    through the private `ui/canvas/widget/left_click/group_background/prepare.rs`,
    `ui/canvas/widget/left_click/group_background/selection.rs`,
    `ui/canvas/widget/left_click/group_background/background.rs`, and
    `ui/canvas/widget/left_click/group_background/start.rs` seams, so
    `left_click/group_background.rs` now mainly keeps group/background pointer-down orchestration
    explicit.
  - Progress: pointer-up resize commits now also route node/group release reducers through the
    private `ui/canvas/widget/pointer_up_commit/resize/node.rs` and
    `ui/canvas/widget/pointer_up_commit/resize/group.rs` seams, so
    `pointer_up_commit/resize.rs` now mainly keeps the resize-release façades explicit.
  - Progress: left-click node/edge/resize entry handling now also routes node, edge, and resize
    pointer-down reducers through the private `ui/canvas/widget/left_click/element_hits/node.rs`,
    `ui/canvas/widget/left_click/element_hits/edge.rs`, and
    `ui/canvas/widget/left_click/element_hits/resize.rs` seams, so
    `left_click/element_hits.rs` now mainly keeps the element-hit façades explicit.
  - Progress: left-click connection entry handling now also routes port and edge-anchor reducers
    through the private `ui/canvas/widget/left_click/connection_hits/port.rs` and
    `ui/canvas/widget/left_click/connection_hits/edge_anchor.rs` seams, so
    `left_click/connection_hits.rs` now mainly keeps the connection-hit façades explicit.
  - Progress: left-click hit-test routing now also routes connection hits, background/group hits,
    and node/resize hits through the private `ui/canvas/widget/left_click/hit/connection.rs`,
    `ui/canvas/widget/left_click/hit/background.rs`, and
    `ui/canvas/widget/left_click/hit/node.rs` seams, so `left_click/hit.rs` now mainly keeps the
    hit-test orchestration explicit.
  - Progress: left-button pointer-up routing now also routes committed-session reducers, pending
    session reducers, and active drag teardown through the private
    `ui/canvas/widget/pointer_up_left_route/dispatch/commit.rs`,
    `ui/canvas/widget/pointer_up_left_route/dispatch/pending.rs`, and
    `ui/canvas/widget/pointer_up_left_route/dispatch/active.rs` seams, so
    `pointer_up_left_route/dispatch.rs` now mainly keeps the left-release chain orchestration
    explicit.
  - Progress: pending pointer-up release wrappers now also route group/node pending-session teardown
    through the private `ui/canvas/widget/pointer_up_pending/release/group.rs` and
    `ui/canvas/widget/pointer_up_pending/release/node.rs` seams, so
    `pointer_up_pending/release.rs` now mainly keeps the pending-release façades explicit.
  - Progress: added direct left-release conformance coverage for pending group drag / resize teardown
    in `ui/canvas/widget/tests/interaction_conformance.rs`, so the new dispatch ordering is locked
    by focused pointer-up regressions.
  - Progress: pointer-move dispatch routing now also routes primary interaction handlers, active
    drag handlers, and overlay handlers through the private
    `ui/canvas/widget/pointer_move_dispatch/primary.rs`,
    `ui/canvas/widget/pointer_move_dispatch/secondary.rs`, and
    `ui/canvas/widget/pointer_move_dispatch/overlay.rs` seams, so
    `pointer_move_dispatch.rs` now mainly keeps the pointer-move routing order explicit.
  - Progress: retained pointer-move event handling now also routes modifier/pointer-state sync,
    release guards, and move-tail handoff through the private
    `ui/canvas/widget/event_pointer_move/pointer_state.rs`,
    `ui/canvas/widget/event_pointer_move/release.rs`, and
    `ui/canvas/widget/event_pointer_move/tail.rs` seams, so `event_pointer_move.rs` now mainly
    keeps the top-level pointer-move orchestration explicit.
  - Progress: pointer-move tail handling now also routes cursor updates and auto-pan timer sync
    through the private `ui/canvas/widget/event_pointer_move_tail/cursor.rs` and
    `ui/canvas/widget/event_pointer_move_tail/timer.rs` seams, so
    `event_pointer_move_tail.rs` now mainly keeps the pointer-move tail orchestration explicit.
  - Progress: pointer-up reducer entry now also routes state sync, non-left release guards, and
    left-button release handoff through the private `ui/canvas/widget/pointer_up/state.rs`,
    `ui/canvas/widget/pointer_up/release.rs`, and `ui/canvas/widget/pointer_up/left.rs` seams, so
    `pointer_up.rs` now mainly keeps the pointer-up entry orchestration explicit.
  - Progress: retained pointer-up event handling now also routes modifier-state sync, pre-dispatch
    guards, and final reducer dispatch through the private
    `ui/canvas/widget/event_pointer_up/prelude.rs` and
    `ui/canvas/widget/event_pointer_up/dispatch.rs` seams, so `event_pointer_up.rs` now mainly
    keeps the top-level pointer-up event orchestration explicit.
  - Progress: pointer button event routing now also routes pointer down / move / up event lanes
    through the private `ui/canvas/widget/event_router_pointer_button/down.rs`,
    `ui/canvas/widget/event_router_pointer_button/move_event.rs`, and
    `ui/canvas/widget/event_router_pointer_button/up.rs` seams, so
    `event_router_pointer_button.rs` now mainly keeps pointer-button event matching explicit.
  - Progress: retained pointer-down event handling now also routes state preparation and route
    dispatch through the private `ui/canvas/widget/event_pointer_down/prelude.rs` and
    `ui/canvas/widget/event_pointer_down/dispatch.rs` seams, so `event_pointer_down.rs` now mainly
    keeps the top-level pointer-down orchestration explicit.
  - Progress: pointer-down route handling now also routes preflight guards and gesture-start guards
    through the private `ui/canvas/widget/event_pointer_down_route/preflight.rs` and
    `ui/canvas/widget/event_pointer_down_route/starts.rs` seams, so
    `event_pointer_down_route.rs` now mainly keeps the pointer-down route ordering explicit.
  - Progress: pointer-down gesture-start wrappers now also route context-menu and sticky-wire
    pointer-down guards through the private `ui/canvas/widget/pointer_down_gesture_start/menu.rs`
    and `ui/canvas/widget/pointer_down_gesture_start/sticky.rs` seams, so
    `pointer_down_gesture_start.rs` now mainly keeps gesture-start façades explicit.
  - Progress: sticky-wire pointer-down handling now also routes session eligibility/new-wire
    extraction and target resolution through the private `ui/canvas/widget/sticky_wire/checks.rs`
    and `ui/canvas/widget/sticky_wire/target.rs` seams, so `sticky_wire.rs` now mainly keeps the
    sticky-wire pointer-down orchestration explicit while helper tests lock the branch predicates.
  - Progress: sticky-wire non-port target handling now also routes hit inspection and picker-open
    finalization through the private `ui/canvas/widget/sticky_wire_targets/inspect.rs` and
    `ui/canvas/widget/sticky_wire_targets/picker.rs` seams, so `sticky_wire_targets.rs` now mainly
    keeps the non-port target orchestration explicit.
  - Progress: pending connection session activation now also routes pending edge-insert and
    pending wire promotion through the private `ui/canvas/widget/pending_connection_session/edge.rs`
    and `ui/canvas/widget/pending_connection_session/wire.rs` seams, so
    `pending_connection_session.rs` now mainly keeps the retained session façades explicit.
  - Progress: pending wire-drag move handling now also routes drag-threshold preparation and
    activation side effects through the private `ui/canvas/widget/pending_wire_drag/checks.rs` and
    `ui/canvas/widget/pending_wire_drag/activate.rs` seams, while
    `ui/canvas/widget/pointer_up_pending/wire_drag.rs` now routes click-connect promotion
    predicates and activation through the private
    `ui/canvas/widget/pointer_up_pending/wire_drag/checks.rs` and
    `ui/canvas/widget/pointer_up_pending/wire_drag/activate.rs` seams, so both retained wire-entry
    reducers now mainly keep orchestration explicit.
  - Progress: pending edge-insert drag move handling now also routes drag-threshold preparation and
    activation through the private `ui/canvas/widget/edge_insert_drag/pending/checks.rs` and
    `ui/canvas/widget/edge_insert_drag/pending/activate.rs` seams, so
    `edge_insert_drag/pending.rs` now mainly keeps the retained move orchestration explicit.
  - Progress: left-click port-hit handling now also routes connectability resolution,
    click-connect target handling, and wire-kind arming through the private
    `ui/canvas/widget/left_click/connection_hits/port/connectable.rs`,
    `ui/canvas/widget/left_click/connection_hits/port/click_connect.rs`, and
    `ui/canvas/widget/left_click/connection_hits/port/kind.rs` seams, so
    `left_click/connection_hits/port.rs` now mainly keeps the connection-port orchestration
    explicit while the target-click predicate and yank kind planner gain focused helper coverage.
  - Progress: edge-insert pointer-up handling now also routes pending-release cleanup and active
    picker-opening tail handling through the private
    `ui/canvas/widget/edge_insert_drag/pointer_up/pending.rs` and
    `ui/canvas/widget/edge_insert_drag/pointer_up/active.rs` seams, so
    `edge_insert_drag/pointer_up.rs` now mainly keeps the left-up orchestration explicit.
  - Progress: left-click edge-anchor handling now also routes shared edge selectable/focus/selection
    reduction through the private `ui/canvas/widget/left_click/edge_selection.rs` seam and
    reconnect arming through `ui/canvas/widget/left_click/connection_hits/edge_anchor/arm.rs`, so
    `left_click/connection_hits/edge_anchor.rs` now mainly keeps the edge-anchor orchestration
    explicit while shared helper tests lock the selection-toggle and focus predicates.
  - Progress: left-click edge-hit handling now also routes shared edge
    selectable/focus/selection reduction through the private
    `ui/canvas/widget/left_click/edge_selection.rs` seam and drag-vs-insert arming through
    `ui/canvas/widget/left_click/element_hits/edge/drag.rs`, so
    `left_click/element_hits/edge.rs` now mainly keeps the edge-hit orchestration explicit.
  - Progress: left-click node-hit handling now also routes node selectable/draggable
    capability lookup, shared node-selection reducers, and pending-drag planning through the private
    `ui/canvas/widget/left_click/node_selection.rs`,
    `ui/canvas/widget/left_click/element_hits/node/capabilities.rs`,
    and `ui/canvas/widget/left_click/element_hits/node/drag.rs` seams, so
    `left_click/element_hits/node.rs` now mainly keeps the node-hit orchestration explicit.
  - Progress: left-click resize-hit handling now also routes shared
    node-selection reducers through the private `ui/canvas/widget/left_click/node_selection.rs`
    seam and pending-resize seed construction through
    `ui/canvas/widget/left_click/element_hits/resize/state.rs`, so
    `left_click/element_hits/resize.rs` now mainly keeps the resize-hit orchestration explicit.
  - Progress: pending node-resize move handling now also routes drag-threshold checks and
    activation through the private `ui/canvas/widget/pending_resize/checks.rs` and
    `ui/canvas/widget/pending_resize/activate.rs` seams, while active edge-insert drag move
    handling now routes drag-state writeback and invalidate tail handling through the private
    `ui/canvas/widget/edge_insert_drag/drag/state.rs` and
    `ui/canvas/widget/edge_insert_drag/drag/tail.rs` seams, so both retained move reducers now
    mainly keep façade-level orchestration explicit.
  - Progress: primary pointer-move dispatch now also routes surface pan/marquee, group drag/resize,
    node pending drag/resize, and pending connection move arbitration through the private
    `ui/canvas/widget/pointer_move_dispatch/primary/surface.rs`,
    `ui/canvas/widget/pointer_move_dispatch/primary/group.rs`,
    `ui/canvas/widget/pointer_move_dispatch/primary/node.rs`, and
    `ui/canvas/widget/pointer_move_dispatch/primary/connection.rs` seams, so
    `pointer_move_dispatch/primary.rs` now mainly keeps façade-level ordering explicit.
  - Progress: secondary pointer-move dispatch now also routes active node resize/drag, active
    connection drag arbitration, and pending insert-node drag promotion through the private
    `ui/canvas/widget/pointer_move_dispatch/secondary/node.rs`,
    `ui/canvas/widget/pointer_move_dispatch/secondary/connection.rs`, and
    `ui/canvas/widget/pointer_move_dispatch/secondary/insert.rs` seams, so
    `pointer_move_dispatch/secondary.rs` now mainly keeps façade-level ordering explicit.
  - Progress: `insert_execution.rs` now also routes reroute-candidate detection, plan delegates,
    feedback delegates, and local regression tests through the private
    `ui/canvas/widget/insert_execution/candidate.rs`,
    `ui/canvas/widget/insert_execution/plan.rs`,
    `ui/canvas/widget/insert_execution/feedback.rs`, and
    `ui/canvas/widget/insert_execution/tests.rs` seams, so the root insert-execution file now
    mainly keeps module wiring and façade exports explicit.
  - Progress: `command_router.rs` now also routes direct insert/group/view/focus/edit execution
    through the private `ui/canvas/widget/command_router/insert.rs`,
    `ui/canvas/widget/command_router/group.rs`,
    `ui/canvas/widget/command_router/view.rs`,
    `ui/canvas/widget/command_router/focus.rs`, and
    `ui/canvas/widget/command_router/edit.rs` seams, so the root command router now mainly keeps
    nudge/align precedence and direct-route orchestration explicit.
  - Progress: `paint_edge_anchors.rs` now also routes target-edge reconnectability lookup,
    hover/active interaction state resolution, anchor paint-style calculation, and scene-op push
    through the private `ui/canvas/widget/paint_edge_anchors/resolve.rs`,
    `ui/canvas/widget/paint_edge_anchors/state.rs`,
    `ui/canvas/widget/paint_edge_anchors/style.rs`, and
    `ui/canvas/widget/paint_edge_anchors/render.rs` seams, so the root edge-anchor painter now
    mainly keeps draw-order and endpoint iteration orchestration explicit.
  - Progress: `paint_root_helpers.rs` now also routes static-scene paint-token key writes and
    geometry-token key writes through the private `ui/canvas/widget/paint_root_helpers/paint.rs`
    and `ui/canvas/widget/paint_root_helpers/geometry.rs` seams, so the root paint-root helper
    file now mainly keeps static-scene style-key wiring explicit.
  - Progress: `paint_root/cache_plan.rs` now also routes hovered-edge resolution and static-scene
    cache eligibility / tile / rect planning through the private
    `ui/canvas/widget/paint_root/cache_plan/hover.rs` and
    `ui/canvas/widget/paint_root/cache_plan/tiles.rs` seams, so the root paint-root cache-plan
    file now mainly keeps derived-output publish and cache-plan orchestration explicit.
  - Progress: `widget_surface.rs` now also routes retained-only runtime constants through the
    private `ui/canvas/widget/widget_surface/constants.rs` seam, so the root widget-surface file
    now mainly keeps module wiring and the noop `new` constructor explicit while keeping the
    public widget contract unchanged.
  - Progress: `widget_surface/runtime.rs` now also routes render-cull/debug helpers, interaction
    activity grouping, and edge-path resolution through the private
    `ui/canvas/widget/widget_surface/runtime/render.rs`,
    `ui/canvas/widget/widget_surface/runtime/interaction.rs`, and
    `ui/canvas/widget/widget_surface/runtime/edge.rs` seams, so the runtime façade now mainly
    keeps retained helper routing explicit without changing the app-facing surface.
  - Progress: `paint_root/frame.rs` now also routes begin-frame cache bookkeeping, path-cache
    diagnostics publication, and canvas background fill through the private
    `ui/canvas/widget/paint_root/frame/cache.rs` and
    `ui/canvas/widget/paint_root/frame/background.rs` seams while reusing the shared
    `paint_grid_plan_support::resolve_canvas_chrome_hint` helper, so the root frame file now
    mainly keeps viewport/cull setup and grid bootstrap explicit.
  - Progress: `paint_root/cached_edges/edges.rs` now also routes uncached fallback rendering,
    cache replay/store helpers, single-rect cache build, and tiled cache build through the private
    `ui/canvas/widget/paint_root/cached_edges/edges/fallback.rs`,
    `ui/canvas/widget/paint_root/cached_edges/edges/replay.rs`,
    `ui/canvas/widget/paint_root/cached_edges/edges/single.rs`, and
    `ui/canvas/widget/paint_root/cached_edges/edges/tiled.rs` seams, so the root cached-edge file
    now mainly keeps the retained façade surface explicit.
  - Progress: `paint_root/cached_edges/labels.rs` now also routes label-cache replay/store
    helpers, single-rect label build, and tiled label build through the private
    `ui/canvas/widget/paint_root/cached_edges/labels/replay.rs`,
    `ui/canvas/widget/paint_root/cached_edges/labels/single.rs`, and
    `ui/canvas/widget/paint_root/cached_edges/labels/tiled.rs` seams, so the root cached-edge
    label file now mainly keeps the retained façade surface explicit.
  - Progress: `paint_root/cached_edges/build_state.rs` now also routes clip-op stack assembly,
    initial cached-edge/label state allocation, and budget-step advancement through the private
    `ui/canvas/widget/paint_root/cached_edges/build_state/ops.rs`,
    `ui/canvas/widget/paint_root/cached_edges/build_state/init.rs`, and
    `ui/canvas/widget/paint_root/cached_edges/build_state/step.rs` seams, so the root build-state
    file now mainly keeps the retained façade surface explicit.
  - Progress: static group/node scene-cache orchestration now also routes through the private
    `ui/canvas/widget/paint_root/static_layer.rs` seam, so `cached_groups.rs` and
    `cached_nodes.rs` stop re-embedding the same layer-target replay/store routing while keeping
    each layer's render-data collection, static paint body, and overlay tail explicit.
  - Progress: `paint_root/edge_anchor.rs` now also routes target edge-id selection,
    render-data target lookup, and geometry-based target reconstruction through the private
    `ui/canvas/widget/paint_root/edge_anchor/target_id.rs`,
    `ui/canvas/widget/paint_root/edge_anchor/render.rs`, and
    `ui/canvas/widget/paint_root/edge_anchor/geometry.rs` seams, so the root edge-anchor file now
    mainly keeps the retained façade surface explicit.
  - Progress: `paint_root/cached_edges/mod.rs` now also routes anchor-target resolution, tiled vs
    single-rect dispatch, and cache-disabled uncached fallback through the private
    `ui/canvas/widget/paint_root/cached_edges/anchor_target.rs`,
    `ui/canvas/widget/paint_root/cached_edges/dispatch.rs`, and
    `ui/canvas/widget/paint_root/cached_edges/fallback.rs` seams, so the root cached-edge
    orchestration file now mainly keeps retained cache routing explicit.
  - Progress: `paint_nodes/static_node_chrome.rs` now also routes node paint-style resolution,
    shadow effect setup, static chrome quads, and title/body text paint through the private
    `ui/canvas/widget/paint_nodes/static_node_chrome/style.rs`,
    `ui/canvas/widget/paint_nodes/static_node_chrome/shadow.rs`,
    `ui/canvas/widget/paint_nodes/static_node_chrome/quads.rs`, and
    `ui/canvas/widget/paint_nodes/static_node_chrome/text.rs` seams, so the root static-node
    chrome file now mainly keeps retained node-chrome orchestration explicit.
  - Progress: `paint_nodes/static_ports.rs` now also routes label text paint, inner-scale port
    geometry, fill-path fallback paint, stroke-path fallback paint, and top-level shape iteration
    through the private `ui/canvas/widget/paint_nodes/static_ports/labels.rs`,
    `ui/canvas/widget/paint_nodes/static_ports/geometry.rs`,
    `ui/canvas/widget/paint_nodes/static_ports/fill.rs`,
    `ui/canvas/widget/paint_nodes/static_ports/stroke.rs`, and
    `ui/canvas/widget/paint_nodes/static_ports/shapes.rs` seams, so the root static-port file now
    mainly keeps retained port-paint orchestration explicit without changing port chrome/path
    behavior.
  - Validation: `cargo check -p fret-node --features compat-retained-canvas --tests` plus focused
    `cargo nextest run -p fret-node --features compat-retained-canvas` coverage for
    `skin_port_chrome_hints_apply_fill_stroke_and_inner_scale_paint_only`,
    `skin_port_shape_hint_renders_path_ops_for_non_circle_shapes`, and
    `preset_exec_ports_use_triangle_shape_and_emit_path_ops`.
  - Progress: `paint_edges/markers_support.rs` now also routes marker paint-binding resolution,
    scene path replay, route-marker path preparation, and custom-marker/custom-wire path
    preparation through the private `ui/canvas/widget/paint_edges/markers_support/paint.rs`,
    `ui/canvas/widget/paint_edges/markers_support/route.rs`, and
    `ui/canvas/widget/paint_edges/markers_support/custom.rs` seams, so the root marker-support
    file now mainly keeps the retained helper façade explicit without changing marker budget or
    wire replay behavior.
  - Validation: focused `cargo nextest run -p fret-node --features compat-retained-canvas`
    coverage for `paint_overrides_can_drive_edge_marker_paint_binding`,
    `bezier_markers_align_with_bezier_start_end_tangents`, and
    `custom_edge_marker_falls_back_to_from_to_tangent_when_path_has_no_tangents`.
  - Progress: `paint_edges/cached_budgeted.rs` now also routes retained static-edge wire/marker
    warmup and label warmup through the private
    `ui/canvas/widget/paint_edges/cached_budgeted/wires.rs` and
    `ui/canvas/widget/paint_edges/cached_budgeted/labels.rs` seams, so the root cached-budgeted
    file now mainly keeps the cache-facing façade explicit without changing tile-budget or label
    placement behavior.
  - Validation: focused `cargo nextest run -p fret-node --features compat-retained-canvas`
    coverage for `cached_edge_paths_match_between_tiled_and_single_tile_cache_modes`,
    `paint_warms_edge_scene_cache_incrementally`, and
    `paint_warms_edge_label_scene_cache_incrementally_for_large_viewport_tiles`.
  - Progress: `paint_render_data/edges.rs` now also routes edge candidate enumeration, normalized
    hint/paint-override resolution, cull-bounds gating, and per-edge render payload append through
    the private `ui/canvas/widget/paint_render_data/edges/candidate.rs`,
    `ui/canvas/widget/paint_render_data/edges/hint.rs`,
    `ui/canvas/widget/paint_render_data/edges/cull.rs`, and
    `ui/canvas/widget/paint_render_data/edges/append.rs` seams, so the root edge render-data file
    now mainly keeps the top-level collection loop and stable sort explicit without changing edge
    payload semantics.
  - Validation: focused `cargo nextest run -p fret-node --features compat-retained-canvas`
    coverage for `edge_render_hint_is_resolved_in_stage_order_presenter_edge_types_skin`,
    `edge_label_border_uses_edge_render_hint_color_override`,
    `paint_reuses_static_edge_scene_cache_without_revisiting_presenter`, and
    `cached_edge_paths_match_between_tiled_and_single_tile_cache_modes`.
  - Progress: `paint_render_data/nodes.rs` now also routes node visibility filtering/order,
    label-overhead sizing, node tuple append, and port label/pin payload append through the private
    `ui/canvas/widget/paint_render_data/nodes/visible.rs`,
    `ui/canvas/widget/paint_render_data/nodes/overhead.rs`,
    `ui/canvas/widget/paint_render_data/nodes/append.rs`, and
    `ui/canvas/widget/paint_render_data/nodes/ports.rs` seams, so the root node render-data file
    now mainly keeps the retained façade entry points explicit without changing node/port payload
    semantics.
  - Validation: focused `cargo nextest run -p fret-node --features compat-retained-canvas`
    coverage for `elevate_nodes_on_select_draws_selected_node_body_last`,
    `skin_port_chrome_hints_apply_fill_stroke_and_inner_scale_paint_only`,
    `per_node_header_palette_draws_distinct_header_quads`, and
    `paint_reuses_static_node_scene_cache_without_revisiting_presenter`.
  - Progress: `paint_render_data/collect.rs` now also routes render-selection snapshot extraction
    and the graph-read collection body through the private
    `ui/canvas/widget/paint_render_data/collect/selection.rs` and
    `ui/canvas/widget/paint_render_data/collect/body.rs` seams, so the root collect file now
    mainly keeps the retained render-data façade explicit without changing collect-time grouping,
    visibility, or edge ordering semantics.
  - Validation: focused `cargo test -p fret-node --features compat-retained-canvas` coverage for
    `edge_render_hint_is_resolved_in_stage_order_presenter_edge_types_skin`,
    `elevate_nodes_on_select_draws_selected_node_body_last`,
    `skin_wire_outline_selected_draws_outline_path_before_core`, and
    `edges_are_sorted_by_endpoint_z_order`.
  - Progress: `paint_render_data/selected_nodes.rs` now also routes selected-node snapshot
    extraction and the graph-read selected-node append/sort body through the private
    `ui/canvas/widget/paint_render_data/selected_nodes/selection.rs` and
    `ui/canvas/widget/paint_render_data/selected_nodes/body.rs` seams, so the root selected-node
    render-data file now mainly keeps the retained façade explicit without changing elevate-on-
    select filtering, append, or rank-sort semantics.
  - Validation: focused `cargo test -p fret-node --features compat-retained-canvas` coverage for
    `elevate_nodes_on_select_draws_selected_node_body_last`,
    `selection_updates_do_not_rebuild_geometry_when_elevate_nodes_on_select_is_enabled`, and
    `paint_reuses_static_node_scene_cache_without_revisiting_presenter`.
  - Progress: `paint_overlay_menu.rs` now also routes menu frame/background layout and per-item
    highlight/text painting through the private `ui/canvas/widget/paint_overlay_menu/frame.rs` and
    `ui/canvas/widget/paint_overlay_menu/items.rs` seams, so the root context-menu paint file now
    mainly keeps the retained overlay façade explicit without changing overlay sizing, hover, or
    text placement semantics.
  - Validation: focused `cargo test -p fret-node --features compat-retained-canvas` coverage for
    `overlay_menu_searcher_conformance`,
    `right_click_cancels_wire_drag_and_opens_context_menu`,
    `right_pan_drag_does_not_open_context_menu`,
    `overlay_state_changes_do_not_rebuild_derived_geometry_or_spatial_index`, and
    `overlay_hover_and_scroll_updates_do_not_rebuild_derived_geometry_or_spatial_index`.
  - Progress: `paint_searcher.rs` now also routes searcher frame/background layout, text-style
    preparation, and shared text constraints through the private
    `ui/canvas/widget/paint_searcher/frame.rs` seam, so the root searcher paint file now mainly
    keeps query/row orchestration explicit without changing searcher sizing, query chrome, or row
    paint semantics.
  - Validation: focused `cargo test -p fret-node --features compat-retained-canvas` coverage for
    `overlay_menu_searcher_conformance`,
    `overlay_state_changes_do_not_rebuild_derived_geometry_or_spatial_index`, and
    `overlay_hover_and_scroll_updates_do_not_rebuild_derived_geometry_or_spatial_index`.
  - Progress: `paint_overlay_wire_hint.rs` now also routes wire-hint text/layout preparation and
    final quad/text scene emission through the private
    `ui/canvas/widget/paint_overlay_wire_hint/layout.rs` and
    `ui/canvas/widget/paint_overlay_wire_hint/draw.rs` seams, so the root wire-hint paint file now
    mainly keeps message/border resolution orchestration explicit without changing invalid-hover,
    bundle/yank hint, or wire-hint placement semantics.
  - Validation: focused `cargo test -p fret-node --features compat-retained-canvas` coverage for
    `invalid_hover_message_prefers_hover_diagnostic`,
    `hint_text_reports_bundle_and_yank_counts`,
    `resolved_hint_border_color_uses_context_border_for_valid_hover`,
    `diagnostic_hint_border_color_prefers_convertible_warning_color`, and
    `hover_state_updates_do_not_rebuild_canvas_derived_geometry_or_spatial_index`.
  - Progress: `paint_overlay_toast.rs` now also routes final toast quad/text scene emission through
    the private `ui/canvas/widget/paint_overlay_toast/draw.rs` seam, so the root toast paint file
    now mainly keeps style/layout orchestration explicit without changing toast border-color, box
    sizing, or viewport anchoring semantics.
  - Validation: focused `cargo test -p fret-node --features compat-retained-canvas` coverage for
    `toast_border_color_matches_diagnostic_severity`,
    `toast_rect_clamps_box_width_to_minimum`,
    `toast_rect_clamps_box_width_to_maximum`, and
    `toast_rect_places_box_at_viewport_bottom_left`.
  - Progress: `paint_grid_plan_support.rs` now also routes canvas chrome-hint lookup, grid width /
    thickness / tile-size metric resolution, tile scratch population, and pattern-size validation
    through the private `ui/canvas/widget/paint_grid_plan_support/hint.rs`,
    `ui/canvas/widget/paint_grid_plan_support/metrics.rs`,
    `ui/canvas/widget/paint_grid_plan_support/tiles.rs`, and
    `ui/canvas/widget/paint_grid_plan_support/validate.rs` seams, so the root grid-plan support
    file now mainly keeps the retained façade explicit without changing grid-plan preparation
    semantics.
  - Validation: focused `cargo test -p fret-node --features compat-retained-canvas` coverage for
    `background_style_updates_do_not_rebuild_canvas_derived_geometry`,
    `background_style_override_survives_color_mode_theme_sync`,
    `dots_pattern_emits_rounded_quads`, and
    `cross_pattern_emits_axis_aligned_segments`.
  - Progress: `pointer_down_double_click_background.rs` now also routes background zoom
    preflight gating, background hit testing, and double-click zoom application through the private
    `ui/canvas/widget/pointer_down_double_click_background/preflight.rs`,
    `ui/canvas/widget/pointer_down_double_click_background/hit.rs`, and
    `ui/canvas/widget/pointer_down_double_click_background/apply.rs` seams, so the root
    background-double-click reducer now mainly keeps the retained façade explicit without changing
    zoom-on-double-click gating, hit exclusion, or viewport move callback semantics.
  - Validation: focused `cargo test -p fret-node --features compat-retained-canvas` coverage for
    `double_click_background_zooms_in_about_pointer`,
    `shift_double_click_background_zooms_out_about_pointer`, and
    `double_click_background_zoom_emits_move_start_and_move_end`.
  - Progress: `paint_grid_cache.rs` now also routes warmup orchestration, per-tile op generation,
    and cache-key construction through the private
    `ui/canvas/widget/paint_grid_cache/warm.rs`,
    `ui/canvas/widget/paint_grid_cache/ops.rs`, and
    `ui/canvas/widget/paint_grid_cache/key.rs` seams, so the root grid-cache file now mainly keeps
    tile warmup stats plus the retained façade explicit without changing grid cache warmup or key
    semantics.
  - Validation: focused `cargo test -p fret-node --features compat-retained-canvas` coverage for
    `background_style_updates_do_not_rebuild_canvas_derived_geometry`,
    `background_style_override_survives_color_mode_theme_sync`,
    `dots_pattern_emits_rounded_quads`, and
    `cross_pattern_emits_axis_aligned_segments`.
  - Progress: `pointer_wheel_zoom.rs` now also routes wheel-zoom factor resolution, pinch-zoom
    factor resolution, and shared viewport zoom application through the private
    `ui/canvas/widget/pointer_wheel_zoom/wheel.rs`,
    `ui/canvas/widget/pointer_wheel_zoom/pinch.rs`, and
    `ui/canvas/widget/pointer_wheel_zoom/apply.rs` seams, so the root wheel-zoom reducer now
    mainly keeps the retained facade entry points explicit without changing wheel/pinch gating,
    viewport move debounce, or zoom-about-pointer semantics.
  - Validation: focused `cargo test -p fret-node --features compat-retained-canvas` coverage for
    `wheel_zoom_zooms_about_pointer`,
    `wheel_zoom_emits_move_start_and_debounced_move_end`,
    `pinch_gesture_zooms_in_about_pointer`,
    `pinch_gesture_respects_toggle`,
    `pinch_zoom_emits_move_start_and_debounced_move_end`, and
    `wheel_pan_then_wheel_zoom_ends_pan_and_starts_zoom`.
  - Progress: `pointer_wheel_pan.rs` now also routes scroll-pan gating, mode/platform delta
    resolution, and shared viewport pan application through the private
    `ui/canvas/widget/pointer_wheel_pan/gate.rs`,
    `ui/canvas/widget/pointer_wheel_pan/resolve.rs`, and
    `ui/canvas/widget/pointer_wheel_pan/apply.rs` seams, so the root wheel-pan reducer now mainly
    keeps the retained facade entry point explicit without changing pan-on-scroll enablement,
    shift remapping, viewport move debounce, or state update semantics.
  - Validation: focused `cargo test -p fret-node --features compat-retained-canvas` coverage for
    `pan_on_scroll_mode_horizontal_ignores_vertical_wheel_delta`,
    `pan_on_scroll_shift_maps_vertical_wheel_to_horizontal_on_windows`,
    `space_enables_pan_on_scroll_even_when_pan_on_scroll_is_disabled`,
    `wheel_pan_emits_move_start_and_debounced_move_end`, and
    `wheel_pan_then_wheel_zoom_ends_pan_and_starts_zoom`.
  - Progress: `pointer_up_node_drag_parent.rs` now also routes drag-release parent-change
    collection, node-rect construction, and best-parent-group selection through the private
    `ui/canvas/widget/pointer_up_node_drag_parent/collect.rs`,
    `ui/canvas/widget/pointer_up_node_drag_parent/rect.rs`, and
    `ui/canvas/widget/pointer_up_node_drag_parent/target.rs` seams, so the root node-drag-parent
    helper now mainly keeps the retained facade entry point explicit without changing release-time
    reparenting, group-override precedence, or smallest-containing-group selection semantics.
  - Validation: focused `cargo test -p fret-node --features compat-retained-canvas` coverage for
    `best_parent_group_prefers_smallest_containing_group`,
    `best_parent_group_uses_group_override_bounds`,
    `best_parent_group_returns_none_when_rect_is_not_fully_contained`,
    `group_drag_drives_canvas_derived_preview_and_edge_index`, and
    `group_drag_preview_cache_reuses_geometry_across_preview_rev_updates`.
  - Progress: `focus_nav_ports_hints.rs` now also routes focused-port hint refresh preflight,
    connection-hint evaluation, and interaction-state writeback through the private
    `ui/canvas/widget/focus_nav_ports_hints/preflight.rs`,
    `ui/canvas/widget/focus_nav_ports_hints/evaluate.rs`, and
    `ui/canvas/widget/focus_nav_ports_hints/apply.rs` seams, so the root focus-nav hint helper now
    mainly keeps the retained facade entry point explicit without changing focused-port clearing,
    connection-mode-aware validity checks, conversion fallback checks, or late state-apply guards.
  - Validation: focused `cargo test -p fret-node --features compat-retained-canvas` coverage for
    `focus_next_port_filters_by_wire_direction` and
    `activate_starts_and_commits_wire_drag`.
  - Progress: `focus_nav_ports_activation.rs` now also routes focused-port activation preflight,
    click-connect start arming, and click-connect commit/position sync through the private
    `ui/canvas/widget/focus_nav_ports_activation/preflight.rs`,
    `ui/canvas/widget/focus_nav_ports_activation/start.rs`, and
    `ui/canvas/widget/focus_nav_ports_activation/commit.rs` seams, so the root focus-nav activation
    helper now mainly keeps the retained facade entry point explicit without changing activation
    gating, activation-point lookup, click-connect state reset, or forced-target commit semantics.
  - Validation: focused `cargo test -p fret-node --features compat-retained-canvas` coverage for
    `activate_starts_and_commits_wire_drag`,
    `click_connect_emits_connect_start_and_committed_end`, and
    `click_connect_target_port_click_commits_wire_and_clears_click_connect_state`.
  - Progress: `focus_nav_traversal_port.rs` now also routes traversal preflight, candidate-port
    collection, next-port selection, and final focus/apply through the private
    `ui/canvas/widget/focus_nav_traversal_port/preflight.rs`,
    `ui/canvas/widget/focus_nav_traversal_port/collect.rs`,
    `ui/canvas/widget/focus_nav_traversal_port/select.rs`, and
    `ui/canvas/widget/focus_nav_traversal_port/apply.rs` seams, so the root focus-nav traversal
    helper now mainly keeps the retained facade entry point explicit without changing focused-node
    fallback, wire-direction filtering, cycling order, or focused-port apply semantics.
  - Validation: focused `cargo test -p fret-node --features compat-retained-canvas` coverage for
    `focus_next_port_cycles_ports_within_focused_node` and
    `focus_next_port_filters_by_wire_direction`.
  - Progress: `focus_nav_traversal_node.rs` now also routes traversal snapshot preflight,
    selectable-node ordering, next-node selection, and final focus/apply + optional auto-pan
    through the private `ui/canvas/widget/focus_nav_traversal_node/preflight.rs`,
    `ui/canvas/widget/focus_nav_traversal_node/collect.rs`,
    `ui/canvas/widget/focus_nav_traversal_node/select.rs`, and
    `ui/canvas/widget/focus_nav_traversal_node/apply.rs` seams, so the root node-traversal helper
    now mainly keeps the retained facade entry point explicit without changing draw-order-first
    traversal, unselectable-node skipping, current-node fallback, or focus-driven auto-pan
    semantics.
  - Validation: focused `cargo test -p fret-node --features compat-retained-canvas` coverage for
    `focus_next_cycles_nodes_and_updates_selection`,
    `focus_next_skips_unselectable_nodes`, and
    `focus_next_can_pan_viewport_when_auto_pan_on_node_focus_is_enabled`.
  - Progress: `focus_port_direction_candidate.rs` now also routes directional candidate center
    projection and best-candidate search through the private
    `ui/canvas/widget/focus_port_direction_candidate/center.rs` and
    `ui/canvas/widget/focus_port_direction_candidate/search.rs` seams, so the root directional-port
    candidate helper now mainly keeps the retained facade entry points explicit without changing
    required-direction filtering, rank-based comparison, or neighbor-port selection semantics.
  - Validation: focused `cargo test -p fret-node --features compat-retained-canvas` coverage for
    `focus_port_right_moves_to_neighbor_node`,
    `focus_port_left_moves_back`, and
    `better_directional_rank_prefers_smaller_angle_then_parallel_then_distance`.
  - Progress: `focus_nav_ports_center.rs` now also routes focused-port center lookup and
    activation-point fallback resolution through the private
    `ui/canvas/widget/focus_nav_ports_center/port.rs` and
    `ui/canvas/widget/focus_nav_ports_center/activation.rs` seams, so the root focused-port center
    helper now mainly keeps the retained facade entry points explicit without changing port-center
    preference, last-pointer fallback, or bounds-center fallback semantics.
  - Validation: focused `cargo test -p fret-node --features compat-retained-canvas` coverage for
    `resolve_activation_point_prefers_port_center`,
    `resolve_activation_point_falls_back_to_last_pos`,
    `resolve_activation_point_falls_back_to_bounds_center`,
    `activate_starts_and_commits_wire_drag`, and
    `focus_port_right_moves_to_neighbor_node`.
  - Progress: `move_ops/align_distribute/support.rs` now also routes support types, element
    collection, per-mode delta planning, extent-shift computation, and group/node op application
    through the private `ui/canvas/widget/move_ops/align_distribute/support/types.rs`,
    `ui/canvas/widget/move_ops/align_distribute/support/collect.rs`,
    `ui/canvas/widget/move_ops/align_distribute/support/delta.rs`,
    `ui/canvas/widget/move_ops/align_distribute/support/shift.rs`, and
    `ui/canvas/widget/move_ops/align_distribute/support/append.rs` seams, so the root support file
    now mainly keeps the retained re-export surface explicit without changing align/distribute
    planning, extent clamping, or graph-op emission semantics.
  - Validation: focused `cargo test -p fret-node --features compat-retained-canvas` coverage for
    `align_left_moves_selected_nodes_and_records_history_entry`,
    `align_center_x_preserves_alignment_under_node_extent_bounds`,
    `distribute_x_clamps_selected_group_children_to_node_extent_rect_like_xyflow`, and
    `distribute_x_clamps_selected_group_children_to_node_extent_rect_from_node_extents`.
  - Progress: `view_state/viewport.rs` now also routes viewport request application, view-state
    normalization + callback emission, and focused-point visibility adjustment through the private
    `ui/canvas/widget/view_state/viewport/set.rs`,
    `ui/canvas/widget/view_state/viewport/update.rs`, and
    `ui/canvas/widget/view_state/viewport/visible.rs` seams, so the root viewport file now mainly
    keeps the retained private module split explicit without changing viewport clamping,
    view-callback emission, or focus-driven auto-pan semantics.
  - Validation: focused `cargo test -p fret-node --features compat-retained-canvas` coverage for
    `set_viewport_via_view_queue_updates_pan_and_zoom`,
    `set_viewport_via_view_queue_clamps_zoom_to_style_limits`,
    `set_viewport_clamps_pan_to_translate_extent`,
    `translate_extent_centers_when_viewport_is_larger_than_extent`, and
    `focus_next_can_pan_viewport_when_auto_pan_on_node_focus_is_enabled`.
  - Progress: `press_session/prepare/surface.rs` now also routes surface-session clear sequences,
    focus-session clearing, and local regression fixtures through the private
    `ui/canvas/widget/press_session/prepare/surface/clear.rs`,
    `ui/canvas/widget/press_session/prepare/surface/focus.rs`, and
    `ui/canvas/widget/press_session/prepare/surface/tests.rs` seams, while
    `press_session/prepare/target.rs` now also routes target-hit clear sequences,
    focus-session clearing, and local regression fixtures through the private
    `ui/canvas/widget/press_session/prepare/target/clear.rs`,
    `ui/canvas/widget/press_session/prepare/target/focus.rs`, and
    `ui/canvas/widget/press_session/prepare/target/tests.rs` seams, with shared fixture state in
    `ui/canvas/widget/press_session/prepare/test_support.rs`, so both root prepare files now mainly
    keep the retained re-export surfaces explicit without changing press-session clearing or
    hover/focus-reset semantics.
  - Validation: focused `cargo test -p fret-node --features compat-retained-canvas` coverage for
    `prepare_for_pan_begin_preserves_edge_insert_sessions`,
    `prepare_for_background_interaction_clears_all_surface_pointer_sessions`,
    `prepare_for_port_hit_preserves_node_resize_but_clears_competing_pointer_sessions`, and
    `prepare_for_edge_anchor_hit_clears_hover_edge_and_node_resize`.
  - Progress: `wire_drag/move_update/hover.rs` now also routes source-port extraction, hover
    hit-picking, hover validity/diagnostic evaluation, and conversion probing through the private
    `ui/canvas/widget/wire_drag/move_update/hover/source.rs`,
    `ui/canvas/widget/wire_drag/move_update/hover/pick.rs`,
    `ui/canvas/widget/wire_drag/move_update/hover/validity.rs`, and
    `ui/canvas/widget/wire_drag/move_update/hover/convertible.rs` seams, so the root hover file
    now mainly keeps the retained façade entry points explicit without changing wire-hover hit
    resolution, invalid-connection diagnostics, or conversion probing semantics.
  - Validation: focused `cargo test -p fret-node --features compat-retained-canvas` coverage for
    `wire_drag_hover_tracks_invalid_port_in_strict_mode`,
    `wire_drag_hover_tracks_non_connectable_end_port_as_invalid`, and
    `wire_drag_hover_marks_valid_target_port_as_valid`.
  - Progress: `selection/box_edges.rs` now also routes box-select mode resolution, graph fallback
    edge collection, store-backed connection harvesting, and local regression fixtures through the
    private `ui/canvas/widget/selection/box_edges/mode.rs`,
    `ui/canvas/widget/selection/box_edges/graph.rs`,
    `ui/canvas/widget/selection/box_edges/store.rs`,
    `ui/canvas/widget/selection/box_edges/test_support.rs`, and
    `ui/canvas/widget/selection/box_edges/tests.rs` seams, so the root box-edges file now mainly
    keeps the retained facade module surface explicit without changing box-select edge eligibility
    or store-fallback semantics.
  - Validation: `cargo check -p fret-node --features compat-retained-canvas --tests`,
    `cargo fmt -p fret-node --check`, and focused `cargo test -p fret-node --features
    compat-retained-canvas collect_box_select_edges_` coverage for
    `collect_box_select_edges_connected_selects_any_connected_endpoint`,
    `collect_box_select_edges_both_endpoints_requires_both_nodes_selected`, and
    `collect_box_select_edges_respects_global_selection_gates`.
  - Progress: `interaction_policy/port.rs` now also routes port connectability predicates,
    bundle-candidate filtering, and local regression fixtures through the private
    `ui/canvas/widget/interaction_policy/port/connectable.rs`,
    `ui/canvas/widget/interaction_policy/port/bundle.rs`,
    `ui/canvas/widget/interaction_policy/port/test_support.rs`, and
    `ui/canvas/widget/interaction_policy/port/tests.rs` seams, so the root port-policy file now
    mainly keeps the retained facade module surface explicit without changing port-connectable or
    bundle-dedup semantics.
  - Validation: `cargo check -p fret-node --features compat-retained-canvas --tests`,
    `cargo fmt -p fret-node --check`, focused `cargo test -p fret-node --features
    compat-retained-canvas port_connectable_helpers_respect_node_and_port_overrides`, and focused
    `cargo test -p fret-node --features compat-retained-canvas
    should_add_bundle_port_requires_unique_same_direction_candidate`.
  - Progress: `wire_math.rs` now also routes route-kind distance/closest-point helpers and local
    regression tests through the private `ui/canvas/widget/wire_math/route.rs` and
    `ui/canvas/widget/wire_math/tests.rs` seams, while keeping the root wire-math file as the
    retained facade over existing `path/segment/step` helpers without changing bezier/step/path
    geometry semantics.
  - Validation: `cargo check -p fret-node --features compat-retained-canvas --tests`,
    `cargo fmt -p fret-node --check`, focused `cargo test -p fret-node --features
    compat-retained-canvas path_distance2_on_line_is_zeroish`, focused
    `cargo test -p fret-node --features compat-retained-canvas path_midpoint_and_normal_is_finite`,
    and focused `cargo test -p fret-node --features compat-retained-canvas
    path_start_end_tangents_follow_control_points`.
  - Progress: `delete_ops_builder.rs` now also routes delete-op orchestration and shared regression
    fixtures through the private `ui/canvas/widget/delete_ops_builder/assemble.rs`,
    `ui/canvas/widget/delete_ops_builder/test_support.rs`, and
    `ui/canvas/widget/delete_ops_builder/tests.rs` seams, while existing edge/group/node helpers
    remain in place, so the root delete-ops builder file now mainly keeps the retained facade
    surface explicit without changing delete ordering or edge-dedup semantics.
  - Validation: `cargo check -p fret-node --features compat-retained-canvas --tests`,
    `cargo fmt -p fret-node --check`, focused `cargo test -p fret-node --features
    compat-retained-canvas
    delete_selection_ops_does_not_double_remove_edges_already_owned_by_removed_nodes`, and focused
    `cargo test -p fret-node --features compat-retained-canvas
    collect_node_edges_deduplicates_edges_already_marked_removed`.
  - Progress: `paint_grid_tiles.rs` now also routes tile-op assembly and local regression coverage
    through the private `ui/canvas/widget/paint_grid_tiles/ops.rs` and
    `ui/canvas/widget/paint_grid_tiles/tests.rs` seams, while keeping `GridTileSpec` rooted in the
    existing support module, so the root paint-grid-tiles file now mainly keeps the retained facade
    surface explicit without changing line/dot/cross tile-op semantics.
  - Validation: `cargo check -p fret-node --features compat-retained-canvas --tests`,
    `cargo fmt -p fret-node --check`, focused `cargo test -p fret-node --features
    compat-retained-canvas dots_pattern_emits_rounded_quads`, and focused
    `cargo test -p fret-node --features compat-retained-canvas
    cross_pattern_emits_axis_aligned_segments`.
  - Progress: `context_menu/key_navigation.rs` now also routes local regression fixtures and root
    test coverage through the private
    `ui/canvas/widget/context_menu/key_navigation/test_support.rs` and
    `ui/canvas/widget/context_menu/key_navigation/tests.rs` seams, so the root key-navigation file
    now mainly keeps the retained facade entry points explicit without changing context-menu active
    item, hover, or typeahead semantics.
  - Validation: `cargo check -p fret-node --features compat-retained-canvas --tests`,
    `cargo fmt -p fret-node --check`, focused `cargo test -p fret-node --features
    compat-retained-canvas advance_active_item_skips_disabled_entries`, and focused
    `cargo test -p fret-node --features compat-retained-canvas
    sync_hovered_item_promotes_enabled_item_and_clears_typeahead`.
  - Progress: `group_draw_order.rs` now also routes shared regression fixtures and root test
    coverage through the private `ui/canvas/widget/group_draw_order/test_support.rs` and
    `ui/canvas/widget/group_draw_order/tests.rs` seams, so the root group-draw-order file now
    mainly keeps the retained facade entry points explicit without changing selected-group ordering
    semantics.
  - Validation: `cargo check -p fret-node --features compat-retained-canvas --tests`,
    `cargo fmt -p fret-node --check`, focused `cargo test -p fret-node --features
    compat-retained-canvas bring_to_front_preserves_existing_draw_order_for_selected_groups`,
    focused `cargo test -p fret-node --features compat-retained-canvas
    send_to_back_preserves_existing_draw_order_for_selected_groups`.
  - Progress: `pending_resize_session.rs` now also routes shared pending-resize regression fixtures
    and root test coverage through the private
    `ui/canvas/widget/pending_resize_session/test_support.rs` and
    `ui/canvas/widget/pending_resize_session/tests.rs` seams, so the root pending-resize-session
    file now mainly keeps the retained facade entry points explicit without changing pending→active
    resize activation semantics.
  - Validation: `cargo check -p fret-node --features compat-retained-canvas --tests`,
    `cargo fmt -p fret-node --check`, focused `cargo test -p fret-node --features
    compat-retained-canvas activate_pending_group_resize_moves_pending_into_active`, and focused
    `cargo test -p fret-node --features compat-retained-canvas
    activate_pending_node_resize_moves_pending_into_active`.
  - Progress: `context_menu/target_selection.rs` now also routes shared regression fixtures and root
    test coverage through the private
    `ui/canvas/widget/context_menu/target_selection/test_support.rs` and
    `ui/canvas/widget/context_menu/target_selection/tests.rs` seams, so the root target-selection
    file now mainly keeps the retained facade entry points explicit without changing edge/group
    context-target selection semantics.
  - Validation: `cargo check -p fret-node --features compat-retained-canvas --tests`,
    `cargo fmt -p fret-node --check`, focused `cargo test -p fret-node --features
    compat-retained-canvas select_group_context_target_clears_node_and_edge_selection`, and focused
    `cargo test -p fret-node --features compat-retained-canvas
    select_edge_context_target_clears_node_and_group_selection`.
  - Progress: `context_menu/background_execution.rs` now also routes shared background-insert
    regression fixtures and root test coverage through the private
    `ui/canvas/widget/context_menu/background_execution/test_support.rs` and
    `ui/canvas/widget/context_menu/background_execution/tests.rs` seams, so the root
    background-execution file now mainly keeps the insert plan enum plus retained execution facade
    entry points explicit without changing background insert planning or rejection semantics.
  - Validation: `cargo check -p fret-node --features compat-retained-canvas --tests`,
    `cargo fmt -p fret-node --check`, focused `cargo test -p fret-node --features
    compat-retained-canvas background_insert_menu_plan_surfaces_create_node_errors`.
  - Progress: `context_menu/connection_execution.rs` now also routes shared connection-insert /
    conversion regression fixtures and root test coverage through the private
    `ui/canvas/widget/context_menu/connection_execution/test_support.rs` and
    `ui/canvas/widget/context_menu/connection_execution/tests.rs` seams, so the root
    connection-execution file now mainly keeps the insert/conversion plan enums explicit without
    changing connection insert or conversion rejection semantics.
  - Validation: `cargo check -p fret-node --features compat-retained-canvas --tests`,
    `cargo fmt -p fret-node --check`, focused `cargo test -p fret-node --features
    compat-retained-canvas connection_insert_menu_plan_surfaces_create_node_errors`, and focused
    `cargo test -p fret-node --features compat-retained-canvas
    connection_conversion_menu_plan_rejects_missing_template`.
  - Progress: `context_menu/item_builders.rs` now also routes root test coverage through the
    private `ui/canvas/widget/context_menu/item_builders/tests.rs` seam, so the root
    item-builders file now mainly keeps the retained context-menu builder facade explicit without
    changing background/group/edge menu item ordering or command wiring semantics.
  - Validation: `cargo check -p fret-node --features compat-retained-canvas --tests`,
    `cargo fmt -p fret-node --check`, focused `cargo test -p fret-node --features
    compat-retained-canvas context_menu::item_builders::tests`.
  - Progress: `context_menu/selection_activation/payload.rs` now also routes shared activation
    payload regression fixtures and local test coverage through the private
    `ui/canvas/widget/context_menu/selection_activation/payload/test_support.rs` and
    `ui/canvas/widget/context_menu/selection_activation/payload/tests.rs` seams, so the payload
    module now mainly keeps activation payload extraction explicit without changing enabled-item or
    out-of-range index handling semantics.
  - Validation: `cargo check -p fret-node --features compat-retained-canvas --tests`,
    `cargo fmt -p fret-node --check`, focused `cargo test -p fret-node --features
    compat-retained-canvas selection_activation::payload::tests`.
  - Progress: `command_router_align.rs` and `command_router_nudge.rs` now also route local command
    mapping test coverage through the private
    `ui/canvas/widget/command_router_align/tests.rs` and
    `ui/canvas/widget/command_router_nudge/tests.rs` seams, so the root command-router helper files
    now mainly keep align/distribute and nudge request mapping explicit without changing command
    dispatch semantics.
  - Validation: `cargo check -p fret-node --features compat-retained-canvas --tests`,
    `cargo fmt -p fret-node --check`, focused `cargo test -p fret-node --features
    compat-retained-canvas command_router_align::tests`, and focused `cargo test -p fret-node
    --features compat-retained-canvas command_router_nudge::tests`.
  - Progress: `command_view/frame.rs` and `command_view/zoom.rs` now also route local helper test
    coverage through the private `ui/canvas/widget/command_view/frame/tests.rs` and
    `ui/canvas/widget/command_view/zoom/tests.rs` seams, so the command-view helper files now
    mainly keep framing node-id collection plus reset/zoom viewport helper logic explicit without
    changing view command semantics.
  - Validation: `cargo check -p fret-node --features compat-retained-canvas --tests`,
    `cargo fmt -p fret-node --check`, focused `cargo test -p fret-node --features
    compat-retained-canvas command_view::frame::tests`, and focused `cargo test -p fret-node
    --features compat-retained-canvas command_view::zoom::tests`.
  - Progress: `command_open_insert/fallback.rs` and `command_router/dispatch.rs` now also route
    local fallback / direct-command route test coverage through the private
    `ui/canvas/widget/command_open_insert/fallback/tests.rs` and
    `ui/canvas/widget/command_router/dispatch/tests.rs` seams, so the helper files now mainly keep
    insert-picker fallback projection plus direct command route lookup explicit without changing
    command-open-insert or command-router behavior.
  - Validation: `cargo check -p fret-node --features compat-retained-canvas --tests`,
    `cargo fmt -p fret-node --check`, focused `cargo test -p fret-node --features
    compat-retained-canvas command_open_insert::fallback::tests`, and focused `cargo test -p
    fret-node --features compat-retained-canvas command_router::dispatch::tests`.
  - Progress: `cursor_gate.rs`, `cursor_resolve/edge.rs`, and `cursor_resolve/resize.rs` now also
    route local cursor helper test coverage through the private
    `ui/canvas/widget/cursor_gate/tests.rs`,
    `ui/canvas/widget/cursor_resolve/edge/tests.rs`, and
    `ui/canvas/widget/cursor_resolve/resize/tests.rs` seams, so the helper files now mainly keep
    cursor gating plus anchor/resize cursor resolution rules explicit without changing runtime
    cursor semantics.
  - Validation: `cargo check -p fret-node --features compat-retained-canvas --tests`,
    `cargo fmt -p fret-node --check`, focused `cargo test -p fret-node --features
    compat-retained-canvas cursor_gate::tests`, focused `cargo test -p fret-node --features
    compat-retained-canvas cursor_resolve::edge::tests`, and focused `cargo test -p fret-node
    --features compat-retained-canvas cursor_resolve::resize::tests`.
  - Progress: `hover/state.rs` and `hover/target.rs` now also route local hover helper test
    coverage through the private `ui/canvas/widget/hover/state/tests.rs` and
    `ui/canvas/widget/hover/target/tests.rs` seams, so the hover helper files now mainly keep hover
    state synchronization plus anchor target resolution explicit without changing hover behavior.
  - Validation: `cargo check -p fret-node --features compat-retained-canvas --tests`,
    `cargo fmt -p fret-node --check`, focused `cargo test -p fret-node --features
    compat-retained-canvas hover::state::tests`, and focused `cargo test -p fret-node --features
    compat-retained-canvas hover::target::tests`.
  - Progress: `insert_candidates/menu.rs` and `insert_candidates/reroute.rs` now also route local
    insert-candidate helper test coverage through the private
    `ui/canvas/widget/insert_candidates/menu/tests.rs` and
    `ui/canvas/widget/insert_candidates/reroute/tests.rs` seams, so the helper files now mainly
    keep insert-candidate menu projection plus reroute prepending explicit without changing
    candidate-list semantics.
  - Validation: `cargo check -p fret-node --features compat-retained-canvas --tests`,
    `cargo fmt -p fret-node --check`, focused `cargo test -p fret-node --features
    compat-retained-canvas insert_candidates::menu::tests`, and focused `cargo test -p fret-node
    --features compat-retained-canvas insert_candidates::reroute::tests`.
  - Progress: `overlay_hit/context_menu.rs` and `overlay_hit/searcher.rs` now also route local
    overlay-hit helper test coverage through the private
    `ui/canvas/widget/overlay_hit/context_menu/tests.rs` and
    `ui/canvas/widget/overlay_hit/searcher/tests.rs` seams, so the helper files now mainly keep
    overlay rect sizing plus pointer-hit row/item mapping explicit without changing overlay-hit
    semantics.
  - Validation: `cargo check -p fret-node --features compat-retained-canvas --tests`,
    `cargo fmt -p fret-node --check`, focused `cargo test -p fret-node --features
    compat-retained-canvas overlay_hit::context_menu::tests`, and focused `cargo test -p
    fret-node --features compat-retained-canvas overlay_hit::searcher::tests`.
  - Progress: `interaction_gate/cache.rs`, `interaction_gate/detail.rs`,
    `interaction_gate/hover.rs`, and `interaction_gate/motion.rs` now also route local interaction
    gate test coverage through the private
    `ui/canvas/widget/interaction_gate/cache/tests.rs`,
    `ui/canvas/widget/interaction_gate/detail/tests.rs`,
    `ui/canvas/widget/interaction_gate/hover/tests.rs`, and
    `ui/canvas/widget/interaction_gate/motion/tests.rs` seams, so the helper files now mainly keep
    cache/detail/hover/motion gating rules explicit without changing runtime interaction behavior.
  - Validation: `cargo check -p fret-node --features compat-retained-canvas --tests`,
    `cargo fmt -p fret-node --check`, focused `cargo test -p fret-node --features
    compat-retained-canvas interaction_gate::cache::tests`, focused `cargo test -p fret-node
    --features compat-retained-canvas interaction_gate::detail::tests`, focused `cargo test -p
    fret-node --features compat-retained-canvas interaction_gate::hover::tests`, and focused
    `cargo test -p fret-node --features compat-retained-canvas interaction_gate::motion::tests`.
  - Progress: `keyboard_shortcuts_gate/editing.rs`,
    `keyboard_shortcuts_gate/modifier.rs`, `keyboard_shortcuts_gate/navigation.rs`,
    `keyboard_shortcuts_map/modifier.rs`, and `keyboard_shortcuts_map/navigation.rs` now also route
    local shortcut gate/map test coverage through the private
    `ui/canvas/widget/keyboard_shortcuts_gate/editing/tests.rs`,
    `ui/canvas/widget/keyboard_shortcuts_gate/modifier/tests.rs`,
    `ui/canvas/widget/keyboard_shortcuts_gate/navigation/tests.rs`,
    `ui/canvas/widget/keyboard_shortcuts_map/modifier/tests.rs`, and
    `ui/canvas/widget/keyboard_shortcuts_map/navigation/tests.rs` seams, so the helper files now
    mainly keep shortcut gating and command mapping explicit without changing keyboard shortcut
    semantics.
  - Validation: `cargo check -p fret-node --features compat-retained-canvas --tests`,
    `cargo fmt -p fret-node --check`, focused `cargo test -p fret-node --features
    compat-retained-canvas keyboard_shortcuts_gate::editing::tests`, focused `cargo test -p
    fret-node --features compat-retained-canvas keyboard_shortcuts_gate::modifier::tests`, focused
    `cargo test -p fret-node --features compat-retained-canvas
    keyboard_shortcuts_gate::navigation::tests`, focused `cargo test -p fret-node --features
    compat-retained-canvas keyboard_shortcuts_map::modifier::tests`, and focused `cargo test -p
    fret-node --features compat-retained-canvas keyboard_shortcuts_map::navigation::tests`.
  - Progress: `focus_session/focus.rs`, `focus_session/selection.rs`,
    `focus_session/hints.rs`, `view_math_viewport/snap.rs`, and
    `view_math_viewport/viewport.rs` now also route local focus/view helper test coverage through
    the private `ui/canvas/widget/focus_session/focus/tests.rs`,
    `ui/canvas/widget/focus_session/selection/tests.rs`,
    `ui/canvas/widget/focus_session/hints/tests.rs`,
    `ui/canvas/widget/view_math_viewport/snap/tests.rs`, and
    `ui/canvas/widget/view_math_viewport/viewport/tests.rs` seams, so the helper files now mainly
    keep focus-state mutation and viewport math explicit without changing runtime semantics.
  - Validation: `cargo check -p fret-node --features compat-retained-canvas --tests`,
    `cargo fmt -p fret-node --check`, focused `cargo test -p fret-node --features
    compat-retained-canvas focus_session::focus::tests`, focused `cargo test -p fret-node
    --features compat-retained-canvas focus_session::selection::tests`, focused `cargo test -p
    fret-node --features compat-retained-canvas focus_session::hints::tests`, focused `cargo test
    -p fret-node --features compat-retained-canvas view_math_viewport::snap::tests`, and focused
    `cargo test -p fret-node --features compat-retained-canvas
    view_math_viewport::viewport::tests`.
  - Progress: `left_click/edge_selection.rs`, `left_click/node_selection.rs`,
    `left_click/connection_hits/port/kind.rs`, and
    `left_click/connection_hits/port/click_connect.rs` now also route local left-click helper test
    coverage through the private `ui/canvas/widget/left_click/edge_selection/tests.rs`,
    `ui/canvas/widget/left_click/node_selection/tests.rs`,
    `ui/canvas/widget/left_click/connection_hits/port/kind/tests.rs`, and
    `ui/canvas/widget/left_click/connection_hits/port/click_connect/tests.rs` seams, so these
    helper files now mainly keep edge/node selection plus port-hit wire-drag/click-connect logic
    explicit without changing left-click runtime semantics.
  - Validation: `cargo check -p fret-node --features compat-retained-canvas --tests`,
    `cargo fmt -p fret-node --check`, focused `cargo test -p fret-node --features
    compat-retained-canvas left_click::edge_selection::tests`, focused `cargo test -p fret-node
    --features compat-retained-canvas left_click::node_selection::tests`, focused `cargo test -p
    fret-node --features compat-retained-canvas left_click::connection_hits::port::kind::tests`,
    and focused `cargo test -p fret-node --features compat-retained-canvas
    left_click::connection_hits::port::click_connect::tests`.
  - Progress: `left_click/element_hits/edge/drag.rs`,
    `left_click/element_hits/node/drag.rs`, and `left_click/element_hits/resize/state.rs` now also
    route local left-click element-hit helper test coverage through the private
    `ui/canvas/widget/left_click/element_hits/edge/drag/tests.rs`,
    `ui/canvas/widget/left_click/element_hits/node/drag/tests.rs`, and
    `ui/canvas/widget/left_click/element_hits/resize/state/tests.rs` seams, so these helper files
    now mainly keep edge/node-drag arming and resize-start sizing explicit without changing
    element-hit runtime semantics.
  - Validation: `cargo check -p fret-node --features compat-retained-canvas --tests`,
    `cargo fmt -p fret-node --check`, focused `cargo test -p fret-node --features
    compat-retained-canvas left_click::element_hits::edge::drag::tests`, focused `cargo test -p
    fret-node --features compat-retained-canvas left_click::element_hits::node::drag::tests`, and
    focused `cargo test -p fret-node --features compat-retained-canvas
    left_click::element_hits::resize::state::tests`.
  - Progress: `graph_construction/node.rs` and `graph_construction/group.rs` now also route local
    graph-construction helper test coverage through the private
    `ui/canvas/widget/graph_construction/node/tests.rs` and
    `ui/canvas/widget/graph_construction/group/tests.rs` seams, so these helper files now mainly
    keep reroute-create op assembly plus centered group creation/selection reducers explicit
    without changing retained graph-construction runtime semantics.
  - Validation: `cargo check -p fret-node --features compat-retained-canvas --tests`,
    `cargo fmt -p fret-node --check`, focused `cargo test -p fret-node --features
    compat-retained-canvas graph_construction::node::tests`, and focused `cargo test -p fret-node
    --features compat-retained-canvas graph_construction::group::tests`.
  - Progress: `pending_connection_session/edge.rs` and `pending_connection_session/wire.rs` now
    also route local pending-connection helper test coverage through the private
    `ui/canvas/widget/pending_connection_session/edge/tests.rs` and
    `ui/canvas/widget/pending_connection_session/wire/tests.rs` seams, so these helper files now
    mainly keep pending edge-insert activation plus pending wire-drag promotion explicit without
    changing retained connection-session runtime semantics.
  - Validation: `cargo check -p fret-node --features compat-retained-canvas --tests`,
    `cargo fmt -p fret-node --check`, focused `cargo test -p fret-node --features
    compat-retained-canvas pending_connection_session::edge::tests`, and focused `cargo test -p
    fret-node --features compat-retained-canvas pending_connection_session::wire::tests`.
  - Progress: `paint_overlay_toast/layout.rs` and `paint_overlay_toast/style.rs` now also route
    local toast-overlay helper test coverage through the private
    `ui/canvas/widget/paint_overlay_toast/layout/tests.rs` and
    `ui/canvas/widget/paint_overlay_toast/style/tests.rs` seams, so these helper files now mainly
    keep toast rect layout math plus severity border-color mapping explicit without changing
    retained toast-overlay runtime semantics.
  - Validation: `cargo check -p fret-node --features compat-retained-canvas --tests`,
    `cargo fmt -p fret-node --check`, focused `cargo test -p fret-node --features
    compat-retained-canvas paint_overlay_toast::layout::tests`, and focused `cargo test -p
    fret-node --features compat-retained-canvas paint_overlay_toast::style::tests`.
  - Progress: `paint_overlay_wire_hint/message.rs` and `paint_overlay_wire_hint/style.rs` now also
    route local wire-hint helper test coverage through the private
    `ui/canvas/widget/paint_overlay_wire_hint/message/tests.rs` and
    `ui/canvas/widget/paint_overlay_wire_hint/style/tests.rs` seams, so these helper files now
    mainly keep invalid-hover/bundle/yank hint text plus diagnostic border-color resolution
    explicit without changing retained wire-hint runtime semantics.
  - Validation: `cargo check -p fret-node --features compat-retained-canvas --tests`,
    `cargo fmt -p fret-node --check`, focused `cargo test -p fret-node --features
    compat-retained-canvas paint_overlay_wire_hint::message::tests`, and focused `cargo test -p
    fret-node --features compat-retained-canvas paint_overlay_wire_hint::style::tests`.
  - Progress: `pointer_up_pending/click_select.rs`, `pointer_up_pending/wire_drag.rs`,
    `pointer_up_session/cleanup.rs`, and `pointer_up_session/release.rs` now also route local
    pointer-up helper test coverage through the private
    `ui/canvas/widget/pointer_up_pending/click_select/tests.rs`,
    `ui/canvas/widget/pointer_up_pending/wire_drag/tests.rs`,
    `ui/canvas/widget/pointer_up_session/cleanup/tests.rs`, and
    `ui/canvas/widget/pointer_up_session/release/tests.rs` seams, so these helper files now mainly
    keep click-release thresholding, pending wire promotion gating, snap-guide cleanup, and active
    release clearing explicit without changing retained pointer-up runtime semantics.
  - Validation: `cargo check -p fret-node --features compat-retained-canvas --tests`,
    `cargo fmt -p fret-node --check`, focused `cargo test -p fret-node --features
    compat-retained-canvas pointer_up_pending::click_select::tests`, focused `cargo test -p
    fret-node --features compat-retained-canvas pointer_up_pending::wire_drag::tests`, focused
    `cargo test -p fret-node --features compat-retained-canvas pointer_up_session::cleanup::tests`,
    and focused `cargo test -p fret-node --features compat-retained-canvas
    pointer_up_session::release::tests`.
  - Progress: `pointer_up_commit_resize/group.rs`, `pointer_up_commit_resize/node.rs`,
    `reconnect/edges.rs`, and `reconnect/flags.rs` now also route local resize/reconnect helper
    test coverage through the private `ui/canvas/widget/pointer_up_commit_resize/group/tests.rs`,
    `ui/canvas/widget/pointer_up_commit_resize/node/tests.rs`,
    `ui/canvas/widget/reconnect/edges/tests.rs`, and
    `ui/canvas/widget/reconnect/flags/tests.rs` seams, so these helper files now mainly keep group
    and node resize op assembly plus reconnect yank/reconnectable-flag resolution explicit without
    changing retained pointer-up or reconnect runtime semantics.
  - Validation: `cargo check -p fret-node --features compat-retained-canvas --tests`,
    `cargo fmt -p fret-node --check`, focused `cargo test -p fret-node --features
    compat-retained-canvas pointer_up_commit_resize::group::tests`, focused `cargo test -p
    fret-node --features compat-retained-canvas pointer_up_commit_resize::node::tests`, focused
    `cargo test -p fret-node --features compat-retained-canvas reconnect::edges::tests`, and
    focused `cargo test -p fret-node --features compat-retained-canvas reconnect::flags::tests`.
  - Progress: `rect_math_core.rs`, `rect_math_path.rs`, `threshold.rs`, and
    `right_click/threshold.rs` now also route local math/threshold helper test coverage through the
    private `ui/canvas/widget/rect_math_core/tests.rs`,
    `ui/canvas/widget/rect_math_path/tests.rs`, `ui/canvas/widget/threshold/tests.rs`, and
    `ui/canvas/widget/right_click/threshold/tests.rs` seams, so these helper files now mainly keep
    rect extents/intersection math, path bounds derivation, generic drag-threshold checks, and
    pending right-click threshold checks explicit without changing retained input/runtime semantics.
  - Validation: `cargo check -p fret-node --features compat-retained-canvas --tests`,
    `cargo fmt -p fret-node --check`, focused `cargo test -p fret-node --features
    compat-retained-canvas rect_math_core::tests`, focused `cargo test -p fret-node --features
    compat-retained-canvas rect_math_path::tests`, focused `cargo test -p fret-node --features
    compat-retained-canvas threshold::tests`, and focused `cargo test -p fret-node --features
    compat-retained-canvas right_click::threshold::tests`.
  - Progress: `pointer_up_commit_group_drag.rs`, `pointer_up_left_route/double_click.rs`,
    `pointer_up_node_drag_parent/target.rs`, and `pointer_up_state/sync.rs` now also route local
    pointer-up/drag helper test coverage through the private
    `ui/canvas/widget/pointer_up_commit_group_drag/tests.rs`,
    `ui/canvas/widget/pointer_up_left_route/double_click/tests.rs`,
    `ui/canvas/widget/pointer_up_node_drag_parent/target/tests.rs`, and
    `ui/canvas/widget/pointer_up_state/sync/tests.rs` seams, so these helper files now mainly keep
    group-drag op assembly, edge-insert double-click gating, parent-group targeting, and pointer-up
    state sync math explicit without changing retained pointer-up runtime semantics.
  - Validation: `cargo check -p fret-node --features compat-retained-canvas --tests`,
    `cargo fmt -p fret-node --check`, focused `cargo test -p fret-node --features
    compat-retained-canvas pointer_up_commit_group_drag::tests`, focused `cargo test -p fret-node
    --features compat-retained-canvas pointer_up_left_route::double_click::tests`, focused
    `cargo test -p fret-node --features compat-retained-canvas
    pointer_up_node_drag_parent::target::tests`, and focused `cargo test -p fret-node --features
    compat-retained-canvas pointer_up_state::sync::tests`.
  - Progress: `searcher_activation_hit/candidate.rs`, `searcher_activation_state/clear.rs`,
    `searcher_input_nav/step.rs`, and `searcher_input_query/query.rs` now also route local
    searcher helper test coverage through the private
    `ui/canvas/widget/searcher_activation_hit/candidate/tests.rs`,
    `ui/canvas/widget/searcher_activation_state/clear/tests.rs`,
    `ui/canvas/widget/searcher_input_nav/step/tests.rs`, and
    `ui/canvas/widget/searcher_input_query/query/tests.rs` seams, so these helper files now mainly
    keep searcher row-to-candidate resolution, overlay clearing, active-row stepping, and query key
    filtering explicit without changing retained searcher runtime semantics.
  - Validation: `cargo check -p fret-node --features compat-retained-canvas --tests`,
    `cargo fmt -p fret-node --check`, focused `cargo test -p fret-node --features
    compat-retained-canvas searcher_activation_hit::candidate::tests`, focused `cargo test -p
    fret-node --features compat-retained-canvas searcher_activation_state::clear::tests`, focused
    `cargo test -p fret-node --features compat-retained-canvas searcher_input_nav::step::tests`,
    and focused `cargo test -p fret-node --features compat-retained-canvas
    searcher_input_query::query::tests`.
  - Progress: `searcher_pointer_hover/state.rs`, `searcher_pointer_wheel/delta.rs`,
    `searcher_row_activation/item.rs`, and `selection/selectable.rs` now also route local
    searcher/selection helper test coverage through the private
    `ui/canvas/widget/searcher_pointer_hover/state/tests.rs`,
    `ui/canvas/widget/searcher_pointer_wheel/delta/tests.rs`,
    `ui/canvas/widget/searcher_row_activation/item/tests.rs`, and
    `ui/canvas/widget/selection/selectable/tests.rs` seams, so these helper files now mainly keep
    hovered-row promotion, wheel-scroll clamping, row activation item synthesis, and selectable
    guard checks explicit without changing retained searcher or selection runtime semantics.
  - Validation: `cargo check -p fret-node --features compat-retained-canvas --tests`,
    `cargo fmt -p fret-node --check`, focused `cargo test -p fret-node --features
    compat-retained-canvas searcher_pointer_hover::state::tests`, focused `cargo test -p
    fret-node --features compat-retained-canvas searcher_pointer_wheel::delta::tests`, focused
    `cargo test -p fret-node --features compat-retained-canvas searcher_row_activation::item::tests`,
    and focused `cargo test -p fret-node --features compat-retained-canvas
    selection::selectable::tests`.
  - Progress: `cancel_gesture_state/sessions.rs`, `cancel_session/pan.rs`,
    `cancel_session/residuals.rs`, and `event_keyboard_state.rs` now also route local
    cancel/keyboard helper test coverage through the private
    `ui/canvas/widget/cancel_gesture_state/sessions/tests.rs`,
    `ui/canvas/widget/cancel_session/pan/tests.rs`,
    `ui/canvas/widget/cancel_session/residuals/tests.rs`, and
    `ui/canvas/widget/event_keyboard_state/tests.rs` seams, so these helper files now mainly keep
    gesture-session clearing, pan reset, cancel residual cleanup, and keyboard modifier sync
    explicit without changing retained cancel/input runtime semantics.
  - Validation: `cargo check -p fret-node --features compat-retained-canvas --tests`,
    `cargo fmt -p fret-node --check`, focused `cargo test -p fret-node --features
    compat-retained-canvas cancel_gesture_state::sessions::tests`, focused `cargo test -p
    fret-node --features compat-retained-canvas cancel_session::pan::tests`, focused `cargo test
    -p fret-node --features compat-retained-canvas cancel_session::residuals::tests`, and focused
    `cargo test -p fret-node --features compat-retained-canvas event_keyboard_state::tests`.
  - Progress: `pending_resize/checks.rs`, `pending_wire_drag/checks.rs`, `sticky_wire/checks.rs`,
    `edge_insert_drag/pending/checks.rs`, and `edge_insert_drag/drag/state.rs` now also route
    local pending/sticky edge-drag helper test coverage through the private
    `ui/canvas/widget/pending_resize/checks/tests.rs`,
    `ui/canvas/widget/pending_wire_drag/checks/tests.rs`,
    `ui/canvas/widget/sticky_wire/checks/tests.rs`,
    `ui/canvas/widget/edge_insert_drag/pending/checks/tests.rs`, and
    `ui/canvas/widget/edge_insert_drag/drag/state/tests.rs` seams, so these helper files now
    mainly keep pending activation thresholds, sticky-wire gating, and edge-insert drag state
    updates explicit without changing retained drag/runtime semantics.
  - Validation: `cargo check -p fret-node --features compat-retained-canvas --tests`,
    `cargo fmt -p fret-node --check`, focused `cargo test -p fret-node --features
    compat-retained-canvas pending_resize::checks::tests`, focused `cargo test -p fret-node
    --features compat-retained-canvas pending_wire_drag::checks::tests`, focused `cargo test -p
    fret-node --features compat-retained-canvas sticky_wire::checks::tests`, focused `cargo test
    -p fret-node --features compat-retained-canvas edge_insert_drag::pending::checks::tests`, and
    focused `cargo test -p fret-node --features compat-retained-canvas
    edge_insert_drag::drag::state::tests`.
  - Progress: `focus_nav_ports_center/activation.rs`, `focus_port_direction_rank.rs`,
    `focus_port_direction_wire.rs`, and `view_math_rect.rs` now also route local focus/view helper
    test coverage through the private
    `ui/canvas/widget/focus_nav_ports_center/activation/tests.rs`,
    `ui/canvas/widget/focus_port_direction_rank/tests.rs`,
    `ui/canvas/widget/focus_port_direction_wire/tests.rs`, and
    `ui/canvas/widget/view_math_rect/tests.rs` seams, so these helper files now mainly keep
    activation-point fallback, directional port ranking, wire direction resolution, and view rect
    containment math explicit without changing retained focus/navigation semantics.
  - Validation: `cargo check -p fret-node --features compat-retained-canvas --tests`,
    `cargo fmt -p fret-node --check`, focused `cargo test -p fret-node --features
    compat-retained-canvas focus_nav_ports_center::activation::tests`, focused `cargo test -p
    fret-node --features compat-retained-canvas focus_port_direction_rank::tests`, focused `cargo
    test -p fret-node --features compat-retained-canvas focus_port_direction_wire::tests`, and
    focused `cargo test -p fret-node --features compat-retained-canvas view_math_rect::tests`.
  - Progress: `event_pointer_down_route/dispatch.rs`, `event_pointer_down_state.rs`,
    `event_pointer_move_state.rs`, `event_pointer_wheel_state.rs`, `delete_ops_builder/node.rs`,
    `node_resize/math.rs`, and `paint_grid_tiles/support.rs` now also route local event/math helper
    test coverage through the private
    `ui/canvas/widget/event_pointer_down_route/dispatch/tests.rs`,
    `ui/canvas/widget/event_pointer_down_state/tests.rs`,
    `ui/canvas/widget/event_pointer_move_state/tests.rs`,
    `ui/canvas/widget/event_pointer_wheel_state/tests.rs`,
    `ui/canvas/widget/delete_ops_builder/node/tests.rs`,
    `ui/canvas/widget/node_resize/math/tests.rs`, and
    `ui/canvas/widget/paint_grid_tiles/support/tests.rs` seams, so these helper files now mainly
    keep pointer-event routing/state sync, delete-op edge deduping, node-resize math, and grid-tile
    support math explicit without changing retained runtime semantics.
  - Validation: `cargo check -p fret-node --features compat-retained-canvas --tests`,
    `cargo fmt -p fret-node --check`, focused `cargo test -p fret-node --features
    compat-retained-canvas event_pointer_down_route::dispatch::tests`, focused `cargo test -p
    fret-node --features compat-retained-canvas event_pointer_down_state::tests`, focused `cargo
    test -p fret-node --features compat-retained-canvas event_pointer_move_state::tests`, focused
    `cargo test -p fret-node --features compat-retained-canvas event_pointer_wheel_state::tests`,
    focused `cargo test -p fret-node --features compat-retained-canvas delete_ops_builder::node::tests`,
    focused `cargo test -p fret-node --features compat-retained-canvas node_resize::math::tests`,
    and focused `cargo test -p fret-node --features compat-retained-canvas
    paint_grid_tiles::support::tests`.
  - Progress: `interaction_policy/node.rs`, `pending_drag_session/group.rs`,
    `pending_drag_session/node.rs`, `split_edge_execution.rs`, and `stores/internals.rs` now also
    route local interaction/pending/store helper test coverage through the private
    `ui/canvas/widget/interaction_policy/node/tests.rs`,
    `ui/canvas/widget/pending_drag_session/group/tests.rs`,
    `ui/canvas/widget/pending_drag_session/node/tests.rs`,
    `ui/canvas/widget/split_edge_execution/tests.rs`, and
    `ui/canvas/widget/stores/internals/tests.rs` seams, so these helper files now mainly keep node
    draggable/connectable policy, pending drag activation, split-edge rejection toast fallback, and
    internals edge-center math explicit without changing retained runtime semantics.
  - Validation: `cargo check -p fret-node --features compat-retained-canvas --tests`,
    `cargo fmt -p fret-node --check`, focused `cargo test -p fret-node --features
    compat-retained-canvas interaction_policy::node::tests`, focused `cargo test -p fret-node
    --features compat-retained-canvas pending_drag_session::group::tests`, focused `cargo test -p
    fret-node --features compat-retained-canvas pending_drag_session::node::tests`, focused
    `cargo test -p fret-node --features compat-retained-canvas split_edge_execution::tests`, and
    focused `cargo test -p fret-node --features compat-retained-canvas stores::internals::tests`.
  - Progress: `paint_edge_anchors/{resolve,state,style}.rs`, `paint_groups/{chrome,overlay}.rs`,
    `paint_root_helpers/{geometry,paint}.rs`, and `paint_root/cache_plan/{hover,tiles}.rs` now
    also route local paint/cache helper test coverage through the private
    `ui/canvas/widget/paint_edge_anchors/resolve/tests.rs`,
    `ui/canvas/widget/paint_edge_anchors/state/tests.rs`,
    `ui/canvas/widget/paint_edge_anchors/style/tests.rs`,
    `ui/canvas/widget/paint_groups/chrome/tests.rs`,
    `ui/canvas/widget/paint_groups/overlay/tests.rs`,
    `ui/canvas/widget/paint_root_helpers/geometry/tests.rs`,
    `ui/canvas/widget/paint_root_helpers/paint/tests.rs`,
    `ui/canvas/widget/paint_root/cache_plan/hover/tests.rs`, and
    `ui/canvas/widget/paint_root/cache_plan/tiles/tests.rs` seams, so these helper files now
    mainly keep edge-anchor paint gating, group chrome/overlay selection, static-scene cache key
    derivation, and paint-root hover/tile planning math explicit without changing retained paint
    runtime semantics.
  - Validation: `cargo check -p fret-node --features compat-retained-canvas --tests`,
    `cargo fmt -p fret-node --check`, focused `cargo test -p fret-node --features
    compat-retained-canvas paint_edge_anchors::resolve::tests`, focused `cargo test -p fret-node
    --features compat-retained-canvas paint_edge_anchors::state::tests`, focused `cargo test -p
    fret-node --features compat-retained-canvas paint_edge_anchors::style::tests`, focused
    `cargo test -p fret-node --features compat-retained-canvas paint_groups::chrome::tests`,
    focused `cargo test -p fret-node --features compat-retained-canvas paint_groups::overlay::tests`,
    focused `cargo test -p fret-node --features compat-retained-canvas
    paint_root_helpers::geometry::tests`, focused `cargo test -p fret-node --features
    compat-retained-canvas paint_root_helpers::paint::tests`, focused `cargo test -p fret-node
    --features compat-retained-canvas paint_root::cache_plan::hover::tests`, and focused `cargo
    test -p fret-node --features compat-retained-canvas paint_root::cache_plan::tiles::tests`.
  - Progress: `paint_edges/pass.rs` now also routes edge-pass budget initialization, batch paint
    replay, and budget-driven redraw requests through the private
    `ui/canvas/widget/paint_edges/pass/budgets.rs`,
    `ui/canvas/widget/paint_edges/pass/batch.rs`, and
    `ui/canvas/widget/paint_edges/pass/redraw.rs` seams, so the root pass file now mainly keeps
    the retained pass state type plus façade entry points explicit without changing edge-pass
    ordering or budget semantics.
  - Validation: focused `cargo nextest run -p fret-node --features compat-retained-canvas`
    coverage for `skin_wire_outline_selected_draws_outline_path_before_core`,
    `skin_wire_highlight_selected_draws_highlight_after_core`,
    `paint_overrides_can_drive_edge_marker_paint_binding`, and
    `bezier_markers_align_with_bezier_start_end_tangents`.
  - Progress: `paint_edges/prepare.rs` now also routes edge paint-model construction, insert-marker
    projection, and batch partitioning through the private
    `ui/canvas/widget/paint_edges/prepare/build.rs`,
    `ui/canvas/widget/paint_edges/prepare/marker.rs`, and
    `ui/canvas/widget/paint_edges/prepare/batches.rs` seams, so the root prepare file now mainly
    keeps the retained edge-paint state types plus façade entry points explicit without changing
    insert-marker or edge-batch semantics.
  - Validation: focused `cargo nextest run -p fret-node --features compat-retained-canvas`
    coverage for `skin_wire_outline_selected_draws_outline_path_before_core`,
    `paint_overrides_can_drive_edge_marker_paint_binding`,
    `alt_drag_edge_opens_insert_node_picker_when_enabled`, and
    `custom_edge_marker_falls_back_to_from_to_tangent_when_path_has_no_tangents`.
  - Progress: `paint_edges/labels.rs` now also routes label-tail budgeted paint/repaint handling
    and cache-stat publication through the private
    `ui/canvas/widget/paint_edges/labels/tail.rs` and
    `ui/canvas/widget/paint_edges/labels/stats.rs` seams, so the root label file now mainly keeps
    the retained façade entry points explicit without changing label-tail or budget-stat semantics.
  - Validation: focused `cargo nextest run -p fret-node --features compat-retained-canvas`
    coverage for `edge_label_anchor_matches_bezier_route_math`,
    `edge_label_border_uses_edge_render_hint_color_override`,
    `cached_edge_labels_match_between_tiled_and_single_tile_cache_modes`,
    `paint_warms_edge_label_scene_cache_incrementally`, and
    `paint_warms_edge_label_scene_cache_incrementally_for_large_viewport_tiles`.
  - Progress: `paint_edges/main.rs` now also routes interaction-hint/custom-path frame
    preparation and optional drop-marker emission through the private
    `ui/canvas/widget/paint_edges/main/context.rs` and
    `ui/canvas/widget/paint_edges/main/markers.rs` seams, so the root edge main file now mainly
    keeps the top-level pass/labels/preview orchestration explicit without changing edge-main
    behavior.
  - Validation: focused `cargo nextest run -p fret-node --features compat-retained-canvas`
    coverage for `skin_wire_outline_selected_draws_outline_path_before_core`,
    `skin_wire_highlight_hovered_draws_highlight_after_core`,
    `alt_drag_edge_opens_insert_node_picker_when_enabled`,
    `paint_warms_edge_label_scene_cache_incrementally`, and
    `paint_warms_edge_label_scene_cache_incrementally_for_large_viewport_tiles`.
  - Progress: `paint_edges/preview.rs` now also routes preview target/style resolution, drop-marker
    quads, and preview wire draw/effect tails through the private
    `ui/canvas/widget/paint_edges/preview/target.rs`,
    `ui/canvas/widget/paint_edges/preview/marker.rs`, and
    `ui/canvas/widget/paint_edges/preview/draw.rs` seams, so the root edge-preview file now
    mainly keeps retained preview orchestration explicit.
  - Progress: `paint_edges/chrome.rs` now also routes wire outline paint, selected-edge glow
    effect setup, and selected/hovered highlight resolution through the private
    `ui/canvas/widget/paint_edges/chrome/outline.rs`,
    `ui/canvas/widget/paint_edges/chrome/glow.rs`, and
    `ui/canvas/widget/paint_edges/chrome/highlight.rs` seams, so the root edge-chrome file now
    mainly keeps retained chrome orchestration explicit.

## M5 - Compatibility retained convergence

- [x] Write explicit exit criteria for `compat-retained-canvas`.
- [x] Decide which retained-only behavior categories still block deprecation.
- [x] Keep the legacy demo documented as a compatibility harness, not the default teaching surface.
- [x] Prevent new retained-only surface area from growing without a documented justification.
- [x] Add a comparison checklist for declarative vs compat-retained behavior on the flows that matter
      most to editor-grade usage.

## Existing evidence and gates to keep alive

- [x] Paint-only cache and invalidation diagnostics under `tools/diag-scripts/node-graph/`.
- [x] Paint-only portal bounds and hover-anchor diagnostics.
- [x] Retained editor conformance tests in `ecosystem/fret-node/src/ui/canvas/widget/tests/`.
- [x] Store/runtime tests in `ecosystem/fret-node/src/runtime/tests.rs`.
- [x] Add a compact gate matrix to the README once the first transaction-safe declarative milestone
      lands.

## Open questions that must not get lost

- [ ] Exact naming for the split state types.
- [x] Exact naming for the controller/instance facade.
  - Landed naming: `NodeGraphSurfaceBinding` is the instance-style app-facing bundle, while
    `NodeGraphController` remains the imperative runtime facade / advanced escape hatch.
- [x] Whether `edit_queue` and `view_queue` remain public long-term or collapse behind the
      controller surface.
  - Landed decision: raw edit/view queue transport is crate-internal only; the public teaching
    surface is `NodeGraphController` plus `NodeGraphSurfaceBinding`.
- [ ] Whether diff-first controlled sync earns a public helper after the full-replace-first path
      proves insufficient.
- [ ] Which retained-only behaviors still need a deliberate temporary home while declarative parity
      is being built.
