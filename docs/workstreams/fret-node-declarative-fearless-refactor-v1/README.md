# Workstream: `fret-node` Fearless Refactor (v1)

Status: Reframed and active (last updated 2026-03-07)
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

## Intent

Make `ecosystem/fret-node` the canonical example of how Fret should ship a complex editor surface:

- **headless asset model first** (`Graph`, `GraphTransaction`, rules, profiles, diagnostics),
- **declarative-first public authoring** for ecosystem and app code,
- **retained semantics in the runtime** without leaking retained authoring into long-term APIs,
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
- `NodeGraphViewState` currently mixes pure view state with interaction policy and runtime tuning,
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

2. **Retained remains a compatibility strategy, not the public design center.**
   - `compat-retained-canvas` is allowed as an escape hatch.
   - It is not the default feature posture and should stay delete-planned.

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

- **Retained interaction engine**
  - `NodeGraphCanvas` remains the most complete editor-grade interaction path today.
  - Store integration, edit/view queues, overlays, and portal host all exist around this path.

- **Declarative-first direction is already visible**
  - `node_graph_surface` is the default lightweight declarative demo path.
  - `node_graph_surface_compat_retained` already proves that retained can be hidden behind a
    declarative entry surface.
  - The retained bridge is already opt-in only.

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
  - The first M2 slice is now landed: `NodeGraphViewState` persists `interaction`
    (`NodeGraphInteractionConfig`) separately from `runtime_tuning`
    (`NodeGraphRuntimeTuning`), while runtime/widget code still resolves a combined
    `NodeGraphInteractionState` for compatibility.
  - Store selector subscriptions now observe non-viewport view-state changes (draw order,
    interaction config, runtime tuning) without emitting misleading empty `ViewChanged` events.
  - Persistence ownership is now explicit: `NodeGraphViewStateFileV1` writes pure view-state in
    `state`, with `interaction` and `runtime_tuning` promoted to wrapper-owned fields.
  - Full closure is still unresolved: in-memory `NodeGraphViewState` still carries persisted
    interaction policy alongside pure view state, and the long-term runtime ownership story is not
    final.

- **Ergonomic API fragmentation**
  - Viewport helpers, lookups, commands, store subscriptions, and controlled updates do not yet read
    like one coherent instance/controller surface.

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

### H2. `NodeGraphViewState` still persists more than true view state

- The first extraction slice is landed: runtime-heavy knobs now live in
  `NodeGraphRuntimeTuning`, separate from `NodeGraphInteractionConfig`.
- The remaining hazard is that persisted interaction policy still lives inside `NodeGraphViewState`,
  so the final ?pure view state only? boundary is not closed yet.
- Evidence:
  - `ecosystem/fret-node/src/io/mod.rs` (`NodeGraphViewState`, `NodeGraphInteractionConfig`,
    `NodeGraphRuntimeTuning`, `NodeGraphInteractionState`)
  - `ecosystem/fret-node/src/ui/canvas/widget/view_state/sync.rs`
  - `docs/workstreams/fret-node-declarative-fearless-refactor-v1/milestones.md` (`M2`)

### H3. `NodeGraphController` is landed, but not yet fully closed as the teaching surface

- The controller now covers the first query / transaction / viewport / selection helpers, including
  XyFlow-style node/handle connection lookups; retained canvas / minimap glue can also bind through
  the controller now, while raw queue ownership, richer edit commands, and callback layering are
  still open.
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

- it wraps `NodeGraphStore` plus an optional private viewport-transport seam,
- it provides common query helpers and transaction-safe commit helpers,
- it now includes the first bounds-aware viewport helpers (`set_center_in_bounds*`,
  `fit_view_nodes_in_bounds*`) so paint-only / fallback hosts can drive viewport state without
  requiring a retained widget queue,
- declarative keyboard/wheel/pinch/pan viewport updates have started converging on the same
  controller/store-backed view-state path when a controller/store is present,
- the deferred `fit-to-portals` viewport application path now also uses that same view-state
  replacement route,
- declarative selection and marquee preview/cancel flows have also started converging on
  controller/store-backed selection helpers when a controller/store is present,
- diagnostics-only paint-only graph hotkeys now also build/commit transactions instead of teaching
  direct `Graph` mutation,
- it can sync external graph/view models from store after commits,
- `NodeGraphSurfaceBinding` now also exposes object-safe viewport entry points for declarative
  action hooks (`set_viewport_action_host`, `fit_view_nodes_in_bounds_action_host`) so first-party
  controls do not need to teach raw view queues,
- retained rename / portal / blackboard / compatibility glue now also prefers controller-owned
  transaction submission when a controller/store exists,
- the retained legacy demo now routes its canvas / rename overlay / blackboard / portal / minimap
  glue through the same controller-first surface,
- the default declarative demo now uses it.

This is intentionally not the final shape yet. Richer viewport commands, callback layering, and the
long-term public naming/ownership story are still open; `edit_queue` is trending toward a
transport/compatibility seam rather than the preferred app-facing teaching surface.

For retained composition, the preferred teaching posture is now controller-first:
`compat_retained` takes a controller binding at the declarative boundary, while the public retained
widget posture is `new(...)` plus optional `with_controller(...)`. Raw queue binding on retained
widgets now stay crate-internal for compatibility harnesses, focused retained tests, and temporary
migration glue.
Queue-first APIs such as `NodeGraphEditQueue` should now be treated as advanced transport seams
rather than the default app-facing integration surface. Raw view-queue transport is now crate-
internal, and the temporary `NodeGraphViewportHelper` facade is deleted, so app-facing composition
should call
`NodeGraphController::{set_viewport*, set_center_in_bounds*, fit_view_nodes*,`
`fit_view_nodes_in_bounds*}` directly, while declarative action hooks should prefer
`NodeGraphSurfaceBinding::{set_viewport_action_host, fit_view_nodes_in_bounds_action_host}` over
owning raw transport queues.

`fret_node::ui::advanced::*` is now the explicit namespace for retained edit-transport seams, and
root `fret_node::ui::*` no longer re-exports the raw queue/helper surfaces. Retained-backed
samples and crate-internal retained/test callers now use `advanced::*` or explicit module paths
directly, while viewport option types stay on the root `ui::*` surface without exposing the raw
view queue itself.
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
  - current Fret surface: `NodeGraphController::fit_view_nodes*`, `fit_view_nodes_in_bounds*`
- node / handle connections:
  - XyFlow mental model: `getNodeConnections`, `getHandleConnections`
  - current Fret surface: `NodeGraphController::node_connections`, `port_connections`
- graph replacement / transaction-safe updates:
  - XyFlow mental model: imperative instance/store writes
  - current Fret surface: `replace_graph`, `submit_transaction*`,
    `submit_transaction_and_sync_*`, `dispatch_transaction*`
- still open:
  - `updateNode` / `updateEdge`-style ergonomic helpers,
  - final `Controller` vs `Instance` naming,
  - whether raw queues survive as public transport or become mostly internal wiring.

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

Use a **declarative root surface**, but allow the internal engine to remain retained-backed where
full interaction parity is still only available there.

Concretely:

- prefer declarative composition at the app boundary,
- prefer binding-first declarative integration (`NodeGraphSurfaceBinding` + `node_graph_surface(...)`),
- prefer controller-driven commands and treat raw edit/view queues as transport or compatibility
  seams,
- do not author directly against retained `NodeGraphCanvas` unless you are working inside
  `fret-node` internals, tests, or temporary compatibility harnesses.

### Golden path for new app code

For new editor surfaces, teach and copy this shape first:

1. create one `NodeGraphSurfaceBinding::new(models, graph, view_state)`,
2. render `node_graph_surface(cx, binding.surface_props())` for the default surface props,
3. use the binding itself for common app-facing helpers (`viewport`, `graph_snapshot`,
   `view_state_snapshot`, `set_viewport`, `fit_view_nodes`, `replace_document`,
   `replace_graph`, `replace_view_state`, `set_selection`, `outgoers`, `incomers`,
   `connected_edges`, `port_connections`, `node_connections`, `undo`, `redo`),
4. drop to `binding.controller()` only for advanced helpers or retained/compat composition,
5. treat raw graph/view models as advanced seams rather than the default teaching surface.

This is the public teaching surface now used by `apps/fret-examples/src/node_graph_demo.rs`.

For controlled sync, the current canonical posture is **full replace first**: use
`binding.replace_document(...)` (or the controller's sync helper) when external authority swaps the
whole graph document and wants a fresh history boundary; keep `replace_graph(...)` as the graph-only
helper when view/history policy should stay caller-controlled, and treat diff-first replace helpers
as a later optimization rather than the starting contract.

### Recommended target posture

Ship a declarative editor-grade surface whose committed edits flow through transactions/store and
whose node content progressively moves toward portal-based declarative composition.

### Temporary compatibility posture

`node_graph_surface_compat_retained` is acceptable as the transition path when the fully
declarative editor-grade surface is not yet ready.

Until then, the compatibility path should stay bounded to:

- the legacy demo as a compatibility harness,
- focused retained conformance tests,
- temporary parity investigations where declarative evidence is still missing.

### Exit criteria for `compat-retained-canvas`

The compatibility path can be deleted or permanently demoted only when all of the following are
true:

- `NodeGraphSurfaceBinding` + `node_graph_surface(...)` is the default documented app-facing path,
- committed declarative edits and viewport changes stay transaction-safe and controller-driven,
- declarative gates cover the editor-grade flows that still justify retained today,
- no new public app-facing APIs depend on retained widget types or raw queue ownership,
- the legacy demo remains only a harness and is no longer needed to teach the recommended posture.

### Current deprecation blockers

The remaining blockers should be tracked explicitly rather than hand-waved as "retained for now":

- declarative parity evidence for retained-backed editor chrome flows,
- declarative parity evidence for portal/overlay lifecycle flows that still rely on retained
  conformance coverage,
- a stable review checklist comparing the flows that matter most for editor-grade usage.

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
  integration first, retained hidden behind compatibility only when needed?
- Is the **target** posture clear: declarative editor-grade surface with transaction-safe commits and
  progressively more declarative node content?
- Does the change avoid treating direct retained `NodeGraphCanvas` authoring as the default
  downstream story?
- If a declarative gesture commits graph or view-state data, does it route through
  controller/store/transaction entry points instead of mutating `Graph` directly?
- Does the change avoid pushing more interaction policy or runtime tuning into
  `NodeGraphViewState`?
- Do examples/docs keep retained surfaces explicitly labeled as internal, IMUI-specific, test-only,
  or temporary compatibility?

## Wording audit snapshot

- `docs/workstreams/xyflow-gap-analysis.md` is aligned with this workstream's public recommendation.
- `apps/fret-examples/src/imui_node_graph_demo.rs` remains intentionally retained-bridge specific
  and should stay scoped as a compatibility/example surface, not the default downstream recipe.
- No other in-tree workstream docs currently recommend direct retained `NodeGraphCanvas` authoring as
  the normal downstream entrypoint.

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
- `ecosystem/fret-node/src/ui/declarative/paint_only.rs`
- `ecosystem/fret-node/src/ui/declarative/compat_retained.rs`
- `ecosystem/fret-node/src/ui/canvas/widget.rs`
- `ecosystem/fret-node/src/ui/portal.rs`
- `ecosystem/fret-node/src/ui/overlays/blackboard.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/tests/overlay_blackboard_conformance.rs`
- `apps/fret-examples/src/node_graph_demo.rs`
- `apps/fret-examples/src/node_graph_legacy_demo.rs`
- `tools/diag-scripts/node-graph/`

## Minimal runnable targets and gates

Canonical runnable targets:

- default declarative demo: `cargo run -p fretboard -- dev native --bin node_graph_demo`
- compatibility harness: `cargo run -p fret-demo --features node-graph-demos-legacy --bin node_graph_legacy_demo`

### Compact gate matrix

| Gate | Command | Why it stays |
| --- | --- | --- |
| declarative + compat conformance | `cargo nextest run -p fret-node --features compat-retained-canvas` | keeps declarative reducers and retained compatibility closure from drifting apart while deprecation is still blocked |
| example wiring smoke | `cargo check -p fret-examples` | keeps `node_graph_demo` and the legacy compatibility harness compiling against the current public teaching surface |
| paint-only diagnostics | `cargo run -p fretboard -- diag suite fret-examples-node-graph-paint-only --dir target/fret-diag-node-graph --launch -- cargo run -p fret-demo --bin node_graph_demo --features node-graph-demos` | protects cache, portal-bounds, hover-anchor, and paint-only scripted regressions |
| layering | `python tools/check_layering.py` | catches accidental boundary drift while the surface is still moving |

The TODO tracker defines the next gate additions still required for full transaction-safe declarative parity.


