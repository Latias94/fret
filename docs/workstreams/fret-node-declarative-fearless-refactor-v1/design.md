# `fret-node` Fearless Refactor (v1) - Design Map

Status: execution-oriented companion (last updated 2026-03-31)
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
    `fit_view_nodes_in_bounds*`),
  - owns transaction-safe graph commits and query helpers.
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
- `NodeGraphEditQueue`
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
3. Pass the controller to retained widgets via `with_controller(...)`.
4. Let retained internals fall back to crate-private queue seams only where the compatibility path
   still needs them.

### Recipe C - retained transport compatibility internals

This is not a downstream recipe. It exists only for in-tree retained compatibility harnesses and
focused tests that still need queue transport while the retained stack is being collapsed.

1. Keep raw edit/view queues inside `fret-node` internals.
2. Bind them only through crate-private retained/controller glue.
3. Do not re-export or teach this path in examples, cookbook docs, or app-facing guides.

## Next worktree order

### Slice 1 - callback surface split

Why first:

- it is still one of the largest remaining "hard to explain" API surfaces,
- it blocks a cleaner app-facing controller story,
- it can land without reopening the already-finished transport cleanup.

What should be true after landing:

- reviewers can distinguish commit callbacks vs view-state callbacks vs gesture-lifecycle callbacks,
- controller/store commit callbacks stop getting mixed with retained gesture glue,
- app code has one obvious callback layer to adopt.

First landing in this worktree:

- `runtime::callbacks` is now split into `NodeGraphCommitCallbacks`,
  `NodeGraphViewCallbacks`, and `NodeGraphGestureCallbacks`.
- `NodeGraphCallbacks` remains only as the composite seam consumed by
  `install_callbacks(...)` and `NodeGraphCanvas::with_callbacks(...)`.
- App-facing docs/examples now teach commit/view first, while retained glue owns gesture hooks.

### Slice 2 - declarative transaction closure

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
