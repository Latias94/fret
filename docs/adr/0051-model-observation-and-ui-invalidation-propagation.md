# ADR 0051: Model Observation and UI Invalidation Propagation (P0)


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- Godot: https://github.com/godotengine/godot
- gpui-component: https://github.com/longbridge/gpui-component
- Zed: https://github.com/zed-industries/zed

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
Status: Accepted
Scope: UI runtime contract (`fret-ui`) + app model store integration (`fret-app`)

## Context

Fret’s current architecture intentionally separates:

- **App-owned state** via `fret-app::Model<T>` (ADR 0031).
- **UI rendering** via a retained UI tree with explicit invalidation flags (ADR 0005).
- **Cross-panel composition** via docking and panel content hosting (ADR 0013 / `DockPanelContentService`).

In practice, editor workflows are inherently cross-panel:

- click in Hierarchy → Inspector, viewport overlays, toolbar state all change,
- project selection → Inspector “Asset” section changes,
- undo/redo → many panels change at once.

If model updates do not automatically invalidate all dependent UI nodes, the user observes “stale UI until another redraw happens”.
We have already encountered this in `fret-demo` (Hierarchy selection updates did not refresh Inspector until another event caused invalidation).

We want a **GPUI-style mental model**:

- views declare what they depend on,
- state changes trigger a targeted re-render/invalidation,
- apps do not manually broadcast “please redraw everyone”.

We also want to preserve Godot’s hard-learned editor safety lessons:

- selection/property change notifications are often **deferred** to avoid dangling references during object deletion,
- updates should be coalesced to avoid UI thrash during drags.

## Decision

Fret adopts a first-class, framework-level mechanism to propagate model changes into UI invalidations:

- UI nodes can **observe** app models (and later global services) with an explicit invalidation mask.
- When an observed model changes, all observing UI nodes are invalidated automatically.
- Implementations must support **multi-window** (multiple UI trees observing the same model id).

This contract is a prerequisite for scaling editor surfaces without a proliferation of bespoke “invalidate all panels” glue.

### P0 Decisions (Lock-In)

To avoid ambiguity and partial implementations, P0 locks the following choices:

- Observation registration is allowed in **both** `layout` and `paint`:
  - rationale: many widgets read models during paint-only code paths (e.g. display text, hover chrome); requiring all reads to be mirrored in `layout` is error-prone.
- Observation registration is **idempotent per node per frame**:
  - duplicates coalesce (union of invalidation masks).
- Invalidation mask escalation uses a strict ordering:
  - `HitTest` > `Layout` > `Paint` (the strongest requested invalidation wins).
- Propagation happens at a **safe driver boundary** (end of event handling / tick), not inside model update closures:
  - coalesce “changed model ids” within the tick.
- P0 supports observation of **`ModelId` only**:
  - observing app globals (TypeId services) is deferred to P1.
- “Deferred notify” is achieved by the above tick-boundary propagation:
  - no separate “signal system” is introduced in P0.

Note: the P1 extension (“observe globals by `TypeId`”) is now implemented in the workspace as an additive feature.

### 1) What can be observed (P0)

P0 supports observation of:

- `fret-app::Model<T>` (by `ModelId`)

P1 may extend to:

- global services in the `App` typemap (by `TypeId` + monotonic revision)
- theme revision (already tracked via `ThemeSnapshot.revision`, see ADR 0032 / ADR 0050)

P1 (implemented) supports observation of:

- app globals (by `TypeId`) with a per-tick changed set drained by the driver/runner.
  - This is **best-effort**: globals mutated outside `set_global` / `with_global_mut` cannot be detected.

### 2) Observation API surface (P0)

The UI runtime exposes an observation registration API that is usable from widgets:

- `cx.observe_model(model, Invalidation::Paint)` (most reads)
- `cx.observe_model(model, Invalidation::Layout)` (reads that can change size/structure)
- `cx.observe_model(model, Invalidation::HitTest)` (reads that affect hit testing)

P1 (implemented) adds:

- `cx.observe_global::<T>(Invalidation::Layout | Paint | HitTest)`

Ergonomics note (non-contractual):

- To reduce bugs where a widget reads a model but forgets to register observation, higher layers may
  provide “observe + read” helpers/wrappers (e.g. `ElementContext::read_model_ref(...)` in `fret-ui`, or
  component-layer sugar in `fret-ui-kit`). These helpers must remain semantically explicit
  about the invalidation strength and must not introduce implicit “track all reads” behavior in P0.

Semantics:

- Observations are registered during `layout` and/or `paint`.
- Observations are **per node** and must be cleared/rebuilt when the node is re-laid out or removed.

This keeps the programming model explicit and avoids hidden “global watchers” owned by widgets.

### 3) Propagation algorithm (P0)

The UI runtime maintains a dependency graph:

- `node -> observed models` (for cleanup)
- `model -> observing nodes` (for fast invalidation)

When `Model<T>` is updated (dirty):

1. the `ModelStore` increments its revision (existing behavior),
2. the `App` exposes a per-tick queue/set of “changed model ids”,
3. after event processing, each active `UiTree` consumes the changed set and invalidates:
   - `Layout` nodes: schedule layout pass,
   - `Paint` nodes: schedule paint pass,
   - `HitTest` nodes: rebuild hit testing state,
4. affected windows request redraw (ADR 0034).

**Coalescing rule**: multiple changes to the same model id in a single tick coalesce into one propagation.

P1 (implemented): globals (`TypeId`) follow the same pattern:

1. the host enqueues changed `TypeId`s (best-effort),
2. the driver drains the per-tick changed set,
3. each active `UiTree` invalidates observing nodes for those globals,
4. affected windows request redraw (ADR 0034).

### 4) Multi-window semantics (P0)

Model changes must invalidate nodes in **all** windows that observe the model id.

This requires the driver (app/runner integration) to:

- drain the “changed model ids” once per tick (or per event loop iteration),
- broadcast the set to all active UI trees.

Implementations must not assume that the window that initiated the update is the only observer.

P1 (implemented) applies the same rule to globals: the driver broadcasts changed `TypeId`s to all windows’ UI trees.

### 5) Ordering and safety (Godot alignment)

To avoid invalidating UI while an app is mid-update:

- propagation happens after the model update scope completes (after the lease drops),
- drivers should apply propagation at a safe boundary (end of event handling / tick),
- long-running interactive drags should still request redraw continuously (ADR 0034), but invalidation propagation is coalesced.

If an editor model can delete entities referenced by UI, apps should prefer **stable ids** (already aligned with ADR 0026 / GUID and with the demo selection model).

### 6) Explicit non-goals (P0)

- A full reactive “virtual DOM” diff model (this is invalidation propagation, not a new authoring model).
- Automatic dependency discovery via “tracking all model reads” (may be considered later, but P0 uses explicit `observe_model` for clarity).
- Cross-thread model updates (still constrained by the app runtime thread model; see ADR 0008).

## Consequences

- Cross-panel editor workflows become reliable: “update model → dependent panels refresh (same frame)”.
- Widget code becomes simpler and less error-prone: fewer ad-hoc invalidation broadcasts.
- UI runtime gets a clear, scalable path to GPUI-like `observe`/`notify` ergonomics without forcing a full rewrite today.

## Alternatives Considered

### A) Manual invalidation broadcasts (status quo in demo glue)

Pros:

- trivial to implement per feature.

Cons:

- easy to miss a dependent panel → stale UI bugs,
- scales poorly as widget count grows,
- obscures true data dependencies.

### B) Implicit dependency discovery (“track all model reads”)

Pros:

- minimal widget author burden (no explicit `observe_model` calls),
- closest to a fully reactive runtime.

Cons:

- requires plumbing a read-tracker through all model/global reads,
- can hide invalidation costs and make performance debugging harder,
- more invasive to retrofit into the current retained widget APIs.

### C) Explicit subscription objects (`Subscription` handles stored in widget state)

Pros:

- aligns with GPUI’s explicit `Subscription` mental model.

Cons:

- complicates widget lifecycle and cleanup in the current `Widget` trait model,
- requires stable “mount/unmount” hooks for all widgets.

P0 selects an explicit `observe_model` registration API that is lifecycle-safe and incremental.

## Implementation Notes (Guidance)

Suggested incremental plan:

1. Add `observe_model` plumbing to `LayoutCx` / `PaintCx` and store dependencies in the UI tree.
2. Extend `ModelStore` with a “changed model ids” queue drained by the driver.
3. Wire the driver to broadcast changed ids to all windows’ UI trees.
4. Migrate demo glue invalidations (e.g. selection → invalidate all panels) to targeted observation.

## Conformance Notes

This contract is considered P0 and must remain deterministic across windows:

- model changes are drained once per tick by the driver/runner and broadcast to all active windows,
- each window's UI tree invalidates observing nodes based on the current frame's observation graph.

P1 (implemented):

- global changes are drained once per tick by the driver/runner and broadcast to all active windows,
- observing nodes are invalidated based on the current frame's global observation graph.

See also: `crates/fret-ui/src/tree/tests/` for multi-window invalidation coverage.

## Open Questions

- P1: How should global services expose revisions in a uniform way (TypeId-based, or explicit service traits)?
- P1: Do we want a first-class “deferred notify” API (Godot-style) for potentially destructive changes, beyond tick-boundary coalescing?
- P2: Do we want optional implicit dependency discovery (“track all reads”) behind a debug/feature flag to reduce boilerplate?

## References

- ADR 0005: `docs/adr/0005-retained-ui-tree.md`
- ADR 0031: `docs/adr/0031-app-owned-models-and-leasing-updates.md`
- ADR 0034: `docs/adr/0034-timers-animation-and-redraw-scheduling.md`
- GPUI / Zed observation + notify patterns:
  - `repo-ref/zed/crates/acp_thread/src/diff.rs` (`cx.observe`, `cx.spawn`, `cx.notify`)
  - `repo-ref/gpui-component/crates/story/src/main.rs` (`cx.notify`)
- Godot editor selection notification (deferred signal emission):
  - `repo-ref/godot/editor/editor_data.cpp` (`EditorSelection::update` / `call_deferred` / `selection_changed`)
