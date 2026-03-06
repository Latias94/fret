# Workstream: `fret-node` Fearless Refactor (v1)

Status: Reframed and active (last updated 2026-03-06)
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
  obvious controller/instance facade,
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
  - `node_graph_surface_paint_only` is the default lightweight declarative demo path.
  - `node_graph_surface_compat_retained` already proves that retained can be hidden behind a
    declarative entry surface.
  - The retained bridge is already opt-in only.

### Still unresolved

- **Public posture ambiguity**
  - There is not yet one crisp answer to "what should ecosystem authors use for a shipping,
    editor-grade node graph today?"

- **Transaction boundary ambiguity in the declarative path**
  - The first M3 slice is now landed: the paint-only node-drag commit builds a
    `GraphTransaction` and prefers `NodeGraphStore::dispatch_transaction(...)` when a store is
    present.
  - Broader declarative commit coverage is still unresolved: this pattern must extend to the rest of
    committed edit flows and converge behind a clearer controller/instance surface.

- **Overgrown view-state boundary**
  - `NodeGraphViewState` currently bundles pure view state together with interaction configuration
    and runtime tuning concerns.

- **Ergonomic API fragmentation**
  - Viewport helpers, lookups, commands, store subscriptions, and controlled updates do not yet read
    like one coherent instance/controller surface.

- **Mixed callback responsibilities**
  - The current callback surface mixes store/headless commit signals with UI gesture lifecycle.

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

### P4. The API surface needs a coherent controller facade

The final reference architecture should let app authors ask for one clear surface for:

- viewport manipulation,
- controlled graph replacement/synchronization,
- common graph queries,
- canonical edit/update entry points,
- subscription and callback wiring.

A first minimal slice is now landed in `ecosystem/fret-node/src/ui/controller.rs` as
`NodeGraphController`:

- it wraps `NodeGraphStore` and optional `NodeGraphViewQueue`,
- it provides common query helpers and transaction-safe commit helpers,
- it now includes the first bounds-aware viewport helpers (`set_center_in_bounds*`,
  `fit_view_nodes_in_bounds*`) so paint-only / fallback hosts can drive viewport state without
  requiring a retained widget queue,
- declarative keyboard/wheel/pinch/pan viewport updates have started converging on the same
  controller/store-backed view-state path when a controller/store is present,
- declarative selection and marquee preview/cancel flows have also started converging on
  controller/store-backed selection helpers when a controller/store is present,
- it can sync external graph/view models from store after commits,
- the default declarative demo now uses it.

This is intentionally not the final shape yet. `edit_queue`, richer viewport commands, callback
layering, and the long-term public naming/ownership story are still open.

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
- a new thin **controller/instance facade** that exposes the ergonomic app-facing surface,
- explicit controlled-mode helpers for full replace vs diff-driven replace.

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
- prefer store-driven integration,
- prefer edit/view queue or controller-driven commands,
- do not author directly against retained `NodeGraphCanvas` unless you are working inside
  `fret-node` internals, tests, or temporary compatibility harnesses.

### Recommended target posture

Ship a declarative editor-grade surface whose committed edits flow through transactions/store and
whose node content progressively moves toward portal-based declarative composition.

### Temporary compatibility posture

`node_graph_surface_compat_retained` is acceptable as the transition path when the fully
declarative editor-grade surface is not yet ready.

### API red lines

Do not add or normalize any of the following as long-term best practice:

- public constructors that require retained types,
- editor-grade interactions that commit by mutating `Graph` directly,
- new UI-policy defaults hidden in mechanism code,
- tutorial/demo guidance that implies retained authoring is the normal downstream path.

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
- XYFlow gap analysis: `docs/workstreams/xyflow-gap-analysis.md`

## Evidence anchors to preserve while refactoring

- `ecosystem/fret-node/src/runtime/store.rs`
- `ecosystem/fret-node/src/runtime/changes.rs`
- `ecosystem/fret-node/src/runtime/lookups.rs`
- `ecosystem/fret-node/src/ui/declarative/paint_only.rs`
- `ecosystem/fret-node/src/ui/declarative/compat_retained.rs`
- `ecosystem/fret-node/src/ui/canvas/widget.rs`
- `ecosystem/fret-node/src/ui/portal.rs`
- `apps/fret-examples/src/node_graph_demo.rs`
- `apps/fret-examples/src/node_graph_legacy_demo.rs`
- `tools/diag-scripts/node-graph/`

## Minimal runnable targets and gates

Canonical runnable targets:

- default declarative demo: `cargo run -p fretboard -- dev native --bin node_graph_demo`
- compatibility harness: `cargo run -p fret-demo --features node-graph-demos-legacy --bin node_graph_legacy_demo`

Canonical gate families to keep alive:

- `cargo nextest run -p fret-node`
- `cargo run -p fretboard -- diag suite fret-examples-node-graph-paint-only --dir target/fret-diag-node-graph --launch -- cargo run -p fret-demo --bin node_graph_demo --features node-graph-demos`

The TODO tracker defines the new gate additions required for transaction-safe declarative parity.
