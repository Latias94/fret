# ADR 0180: Dirty Views + `notify` (GPUI-Aligned)

Status: Proposed

## Context

Fret is converging on a GPUI-style authoring model:

- build a fresh declarative element tree every frame (ADR 0028),
- externalize cross-frame state and identity (ADR 0028 / ADR 1151),
- reuse recorded output ranges when a subtree is clean (ADR 0055 / ADR 1152).

The current runtime still exposes primarily **node-level invalidation** (`Layout`/`Paint`/`HitTest`) and asks
components to be explicit about invalidation strength (ADR 0005 / ADR 0051).

This is powerful, but it has two recurring issues in an editor-scale UI:

1) **Authoring ergonomics drift**: component authors end up “managing invalidation” instead of thinking in terms of
   “I changed view state; re-render the view”.
2) **Cache correctness/perf tension**: view-level caching (ADR 1152) is most effective when the runtime can answer
   “is this view dirty?” deterministically, without requiring that every widget author always picks the perfect
   invalidation mask.

GPUI (Zed) uses a clear and effective primitive:

- mutate view state,
- call `cx.notify(view)` (or `cx.notify()` in a view context),
- the window marks the view dirty and schedules the next frame.

The runtime then decides which parts of the pipeline must run (layout/prepaint/paint) and which cached ranges can be
reused.

This ADR defines the **framework-level contract** for “dirty views” and “notify” in Fret, as a step toward full
GPUI-style ergonomics while keeping Fret’s layering rules intact (ADR 0066).

## Decision

### 1) Introduce a view-level invalidation primitive

Fret defines a stable concept: **View**.

- A **View** is a long-lived, stateful UI unit that renders into an element tree (ADR 0039 `Render`).
- Each view has a stable, window-scoped identifier: `ViewId`.

The runtime provides:

- `cx.notify(view_id)` (or `cx.notify()` for “current view”) to mark that view dirty.

Semantics:

- Calling `notify` indicates the view’s rendered output may have changed.
- `notify` MUST be coalesced (multiple calls per tick collapse to one dirty mark per view).
- `notify` MUST schedule a redraw of the owning window at the next safe driver boundary (end of tick / event loop
  iteration), consistent with ADR 0034 scheduling rules.

### 2) Dirty views are the primary cache key for view caching

When a view is used behind a caching boundary (ADR 1152), cache reuse is allowed only if:

- the view is not marked dirty since it last rendered, and
- the cache key inputs for that boundary still match (bounds/scale/theme, etc; ADR 0055 / ADR 1152).

This aligns with GPUI’s `AnyView::cached` behavior: “reuse previous prepaint/paint if the view wasn’t notified”.

### 3) Relationship to node-level invalidation

Node-level invalidation remains a valid internal mechanism, but the contract surface shifts:

- Component/ecosystem layers SHOULD prefer `notify` to express “my view changed”.
- Runtime mechanisms MAY still use node-level invalidation internally to minimize work (e.g. mark a cache root as
  `Paint`-dirty without forcing layout).

If both occur:

- `notify(view)` is always strong enough to prevent stale cached output for that view.
- Node-level invalidation MAY further narrow the required work (e.g. paint-only).

### 4) Observations (models/globals) feed into dirty views

When a model/global change is propagated (ADR 0051), the runtime MUST be able to attribute that change to one or more
owning views and mark them dirty, rather than requiring every leaf node to be invalidated independently.

This does not remove explicit observation; it refines the target:

- “data changed → mark affected views dirty” is the default mechanism.

### 5) Diagnostics requirements

The runtime MUST expose (via ADR 0036 / ADR 0174 tooling surfaces):

- per-window dirty view count per frame,
- list of dirty views (debug-only) and last notify source (best-effort),
- cache hit/miss breakdown by view/cache root.

## Consequences

- Authoring becomes closer to GPUI: “mutate state → notify”.
- View caching becomes easier to reason about: “notified views don’t reuse cached ranges”.
- It becomes practical to enforce “hover/focus/pressed are paint-only” as a runtime rule, because view dirtiness is
  explicit and centrally tracked (see ADR 0181).

## Alternatives Considered

### A) Only node-level invalidation (status quo)

Pros:
- precise control.

Cons:
- higher author burden,
- harder to enforce cache/interaction invariants consistently across the component ecosystem.

### B) Implicit dependency tracking (“track all reads”)

Pros:
- minimal author effort.

Cons:
- harder to debug performance,
- larger runtime surface area and more invasive plumbing (explicitly deferred in ADR 0051).

## Rollout Plan (Incremental)

1) Define `ViewId` and `notify` API shape at the authoring/runtime boundary (`fret-app` / `fret-ui`).
2) Track `dirty_views` per window and schedule redraws at the driver boundary.
3) Integrate with caching (ADR 1152): cache reuse checks dirty views.
4) Add diagnostics counters and inspector exposure (ADR 0174).

## References

- Zed/GPUI `notify` + dirty views:
  - `repo-ref/zed/crates/gpui/src/window.rs` (`WindowInvalidator`, `dirty_views`, `invalidate_view`)
  - `repo-ref/zed/crates/gpui/src/elements/div.rs` (hover state changes call `cx.notify(current_view)`)
- Fret authoring/runtime model:
  - ADR 0028: `docs/adr/0028-declarative-elements-and-element-state.md`
  - ADR 0039: `docs/adr/0039-component-authoring-model-render-renderonce-and-intoelement.md`
  - ADR 1152: `docs/adr/1152-view-cache-subtree-reuse-and-state-retention.md`

