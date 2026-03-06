# `fret-node` Fearless Refactor (v1) - Design Map

Status: execution-oriented companion (last updated 2026-03-06)
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
    `fit_view_nodes*`, `fit_view_nodes_in_bounds*`),
  - owns transaction-safe graph commits and query helpers.
- `node_graph_surface_paint_only(...)`
  - remains the recommended lightweight declarative authoring surface,
  - should keep converging toward the editor-grade path rather than staying a "lite demo only"
    surface forever.
- `node_graph_surface_compat_retained(...)`
  - is still allowed as a compatibility path,
  - but should be taught as compatibility-only, not as the long-term public design center.

### Advanced transport surface

Only use these when an integration intentionally owns raw queue transport and accepts the retained /
compatibility mental model.

- `fret_node::ui::advanced::NodeGraphEditQueue`
- `fret_node::ui::advanced::NodeGraphViewQueue`
- `fret_node::ui::advanced::NodeGraphViewRequest`
- `fret_node::ui::advanced::NodeGraphViewportHelper`
  - queue-model helper only,
  - not a second controller-like app-facing facade.
- `fret_node::ui::advanced::NodeGraphControllerTransportExt`
  - explicit advanced trait for binding a controller to raw edit/view transport queues,
  - not part of the default app-facing controller story.

### Crate-internal retained compatibility seams

These still exist because retained conformance coverage and compatibility glue still need them, but
they should not read like public downstream authoring APIs.

- `NodeGraphCanvas::with_view_queue`
- `NodeGraphPortalHost::with_edit_queue`
- `NodeGraphOverlayHost::with_edit_queue`
- `NodeGraphBlackboardOverlay::with_edit_queue`
- `NodeGraphMiniMapOverlay::with_view_queue`

These methods are crate-private on purpose.

### Deleted or demoted surfaces already landed

These are no longer part of the recommended public story.

- Root `fret_node::ui::*` queue/helper aliases are removed.
- `NodeGraphViewportHelper::from_controller(...)` is removed.
- `NodeGraphController::with_edit_queue(...)` / `with_view_queue(...)` are replaced by the explicit
  advanced trait.
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

### Recipe B - compatibility retained shell

Use this when a retained-only harness still needs the legacy canvas / overlay stack.

1. Create `NodeGraphStore`.
2. Create `NodeGraphController`.
3. Pass the controller to retained widgets via `with_controller(...)`.
4. Let retained internals fall back to crate-private queue seams only where the compatibility path
   still needs them.

### Recipe C - advanced transport-owned integration

Use this only when the integration intentionally owns queue transport.

1. Import `fret_node::ui::advanced::*` explicitly.
2. Bind raw queues via `NodeGraphControllerTransportExt` if controller ergonomics are still useful.
3. Use `NodeGraphViewportHelper::new(view_state, view_queue)` only when queue-model viewport
   ownership is the point of the integration.
4. Do not teach this recipe as the default downstream path in examples or cookbook docs.

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
  oversized event closure.

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
