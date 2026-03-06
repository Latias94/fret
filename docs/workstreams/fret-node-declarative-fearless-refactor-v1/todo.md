# `fret-node` Fearless Refactor (v1) - TODO

This tracker is intentionally biased toward small, reviewable slices. Keep items concrete enough to
land in code review; move design discussion back to `README.md` if a TODO turns into prose.

Execution companion: `design.md` (surface map + next worktree order).

## Cross-cutting guardrails

- [x] Keep `compat-retained-canvas` out of default features.
- [x] Keep the compatibility retained path feature-gated and explicitly named.
- [x] Keep the default lightweight declarative demo path (`node_graph_demo`).
- [x] Keep the workstream docs aligned with the actual public recommendation after each milestone.
- [ ] Update ADR alignment or ADR text when a hard contract changes rather than hiding the change in
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
- [ ] Decide whether to introduce a future canonical wrapper name such as `node_graph_surface(...)`
      over the existing paint-only / compat-retained milestones.

## M2 - State boundary split

- [ ] Shrink `NodeGraphViewState` to true view state only:
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
  - `NodeGraphViewState.interaction` now stores `NodeGraphInteractionConfig`
  - `NodeGraphViewState.runtime_tuning` now stores `NodeGraphRuntimeTuning`
  - runtime/widget code still resolves `NodeGraphInteractionState` for compatibility
  - legacy serialized `interaction` payloads migrate at load time
- [x] Decide where these new types live and who owns persistence for them.
- [x] Design the migration/compat strategy for existing serialized `NodeGraphViewState` payloads.
- [x] Update store code and tests so the new boundary is explicit in subscriptions and controlled
      synchronization.
- [x] Add focused tests for serialization migration and store behavior after the split.

## M3 - Controller / instance facade

- [x] Introduce a first thin controller facade in `ecosystem/fret-node/src/ui/controller.rs`.
- [x] Make the first landing slice responsible for an ergonomic app-facing surface for:
  - viewport read / set-viewport glue
  - controlled graph replacement / synchronization
  - common graph queries (`outgoers`, `incomers`, `connected_edges`)
  - canonical transaction-safe update entry points
- [x] Wire the default declarative demo to use `NodeGraphController` instead of teaching raw store
      plumbing directly.
- [x] Extend the controller surface with the first bounds-aware viewport helpers:
  - `set_center_in_bounds` / `set_center_in_bounds_with_options`
  - `fit_view_nodes_in_bounds` / `fit_view_nodes_in_bounds_with_options`
  - store fallback when no `view_queue` exists, while still routing through queued `SetViewport`
    requests when a queue is present
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
- [ ] Extend the controller surface further for broader imperative viewport choreography beyond the
      first bounds-aware helper set.
- [ ] Decide whether `view_queue` stays as the transport for imperative viewport requests or becomes
      an internal detail of the controller.
  - Progress: retained canvas / minimap composition can now bind through `NodeGraphController`, so
    new app/UI glue no longer needs to teach raw queue mutation first.
- [ ] Decide whether `edit_queue` stays public, becomes controller-owned, or is limited to internal
      composition seams.
- [x] Collapse the remaining legacy-demo-only raw `edit_queue` command hotkey (`Bump Float Value`)
      into controller-owned submission helpers so example code stops teaching ad-hoc queue mutation.
- [x] Normalize retained controller binding APIs toward `new + with_*` composition where it improves
      teaching clarity, instead of growing more parallel constructor names.
  - Progress: retained edit glue now prefers controller-owned submission helpers when a controller is
    available:
    - `NodeGraphCanvas::with_controller` now carries both store + optional edit/view queues.
    - `NodeGraphPortalHost::with_controller` submits transactions through the controller before
      falling back to raw queue transport.
    - `NodeGraphOverlayHost::new(...).with_controller(...)` and `compat_retained` now teach
      controller-first rename / portal composition instead of requiring raw queue mutation at the
      app boundary, and `compat_retained` now takes a controller binding instead of exposing public
      `edit_queue` / `view_queue` transport props.
    - `NodeGraphBlackboardOverlay` now supports controller-first retained symbol actions, while raw
      queue fallback remains crate-internal for compatibility harnesses and focused retained tests.
    - `apps/fret-examples/src/node_graph_legacy_demo.rs` now uses the same controller-first canvas /
      overlay / blackboard / portal / minimap wiring and no longer keeps a demo-owned
      `NodeGraphEditQueue`; remaining queue ownership is limited to generic compatibility transport
      seams that still need explicit queue binding.
- [x] Land the first XyFlow-style connection-query mapping on the controller surface:
  - `NodeGraphController::node_connections`
  - `NodeGraphController::port_connections` (XyFlow `getHandleConnections` analogue)
- [x] Document the current XyFlow-style viewport/controller mapping in the workstream README.
- [ ] Add a clear mapping from the remaining XyFlow-style expectations to the controller API:
  - update node/edge style helpers where appropriate
- [ ] Decide the long-term public naming/ownership story (`Controller` vs `Instance` vs split
      facades) before widening the teaching surface further.

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
  - `NodeGraphMiniMapOverlay::with_view_queue` / `NodeGraphMiniMapNavigationBinding::ViewQueue`
    (`ecosystem/fret-node/src/ui/overlays/minimap.rs`): still needed because the minimap keeps an
    explicit queue-driven navigation mode internally, but the direct retained queue-binding method is now
    crate-private.

- **Most likely next shrink targets**
  - Raw queue / viewport transport still exported from `ecosystem/fret-node/src/ui/advanced.rs`
    (`NodeGraphEditQueue`, `NodeGraphViewQueue`, `NodeGraphViewRequest`, viewport request option types):
    now explicit and bounded, but still a likely future shrink target once controller-first coverage is complete.
  - `fret_node::ui::advanced::NodeGraphControllerTransportExt::{bind_edit_queue_transport, bind_view_queue_transport}`
    (`ecosystem/fret-node/src/ui/advanced.rs`): probably keep, but document as advanced transport
    binding rather than the default integration recipe.

- **Landable follow-ups from this audit**
  - [x] Demote `NodeGraphViewportHelper` to `fret_node::ui::advanced::*` only and delete the
        controller-first constructor.
  - [x] Move raw queue / viewport transport into the explicit `fret_node::ui::advanced::*`
        namespace.
  - [x] Demote retained widget / overlay queue binding methods to crate-private compatibility seams.
  - [x] Remove root `fret_node::ui::*` queue/helper re-exports from the public surface.
  - [x] Add one short README/workstream note that queue APIs are advanced retained transport seams, not
        the default app-facing integration surface.
  - [x] Migrate retained-only examples / docs that still import queue types from root `fret_node::ui::*`
        to `fret_node::ui::advanced::*` (`node_graph_domain_demo`, workflow gallery snippet).
  - [x] Clear in-tree uses of root queue/helper aliases (apps, gallery snippet, crate-internal retained/tests).
  - [x] Skip the external compatibility/deprecation phase and remove the old root aliases directly.





## M3 - Transaction-safe declarative commits

- [x] Land the first declarative transaction-safe commit slice:
  - `ecosystem/fret-node/src/ui/declarative/paint_only.rs` node-drag commit now builds a
    `GraphTransaction` instead of mutating `Graph` in place.
  - When `NodeGraphSurfacePaintOnlyProps.controller` is present, the commit now prefers the
    controller facade, which in turn dispatches through `NodeGraphStore` and syncs graph/view
    models back from store.
  - `NodeGraphSurfacePaintOnlyProps.store` remains as a compatibility fallback for callers that have
    not adopted the controller surface yet.
  - When no store/controller is present, the declarative path still applies the transaction as a
    transaction, rather than doing ad-hoc position mutation.
- [x] Wire `apps/fret-examples/src/node_graph_demo.rs` to provide a `NodeGraphStore` so the default
      declarative demo path exercises the transaction-safe architecture.
- [x] Add a focused regression test for the drag transaction builder used by the declarative path.
- [ ] Expand the same transaction-safe pattern to the rest of committed declarative edit paths,
      rather than stopping at node drag.
- [ ] Keep ephemeral drag/hover session state local where that improves ergonomics, but route final
      commits through transactions.
- [ ] Add undo/redo coverage for the declarative path once commits stop mutating `Graph` directly.
- [x] Add at least one gate proving that a declarative drag or marquee commit produces a
      transaction-safe update path.
  - Landed via `paint_only.rs` callback gates: controller-backed node-drag commit proves it
    dispatches through store commit callbacks, and controller-backed pending-selection / marquee
    commits prove they dispatch through store selection callbacks rather than only syncing local
    view models.
- [ ] Define the policy for full replace vs diff-based replace in controlled mode.
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

- [ ] Migrate selection/marquee state machines toward declarative reducers with explicit commit and
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
- [x] Keep pointer-capture and cancel behavior as a first-class regression target while doing this.
  - Landed initial declarative gates for selection-only release, escape cancel, and pointer-cancel
    cleanup in `paint_only.rs`.
- [ ] Decide which interaction pieces remain local surface state vs store-backed editor state.
- [ ] Ensure new declarative interaction work does not regress cache discipline.
- [ ] Add at least one parity gate meaningful to real editor usage, not just synthetic paint-only
      counters.

## M4 - Portal and overlay closure

- [ ] Move from portal/bounds experimentation toward a declared editor-grade portal hosting path for
      the visible subset.
- [ ] Clarify how node content subtrees publish measured geometry into derived stores.
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
- [ ] Add at least one gate that exercises portal + overlay anchoring under motion.
  - Progress: the feature-gated retained conformance files now include controller-first rename and
    portal commit scenarios (`overlay_group_rename_conformance.rs`,
    `portal_lifecycle_conformance.rs`), and those retained gates now run again after the harness was
    updated for the latest `EventCx` / `LayoutCx` contract.

## M5 - Compatibility retained convergence

- [ ] Write explicit exit criteria for `compat-retained-canvas`.
- [ ] Decide which retained-only behaviors still block deprecation.
- [ ] Keep the legacy demo as a compatibility harness, not the default teaching surface.
- [ ] Prevent new retained-only surface area from growing without a documented justification.
- [ ] Add a comparison checklist for declarative vs compat-retained behavior on the flows that matter
      most to editor-grade usage.

## Existing evidence and gates to keep alive

- [x] Paint-only cache and invalidation diagnostics under `tools/diag-scripts/node-graph/`.
- [x] Paint-only portal bounds and hover-anchor diagnostics.
- [x] Retained editor conformance tests in `ecosystem/fret-node/src/ui/canvas/widget/tests/`.
- [x] Store/runtime tests in `ecosystem/fret-node/src/runtime/tests.rs`.
- [ ] Add a compact gate matrix to the README once the first transaction-safe declarative milestone
      lands.

## Open questions that must not get lost

- [ ] Exact naming for the split state types.
- [ ] Exact naming for the controller/instance facade.
- [ ] Whether `edit_queue` and `view_queue` remain public long-term or collapse behind the
      controller surface.
- [ ] Whether controlled sync should expose diff-first helpers by default.
- [ ] Which retained-only behaviors still need a deliberate temporary home while declarative parity
      is being built.












