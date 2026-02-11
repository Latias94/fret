# ADR 0165: Dirty Views + `notify` (GPUI-Aligned)

Status: Accepted (v1 cache-root-first; MVP2 implementation in progress)

## Context

Fret is converging on a GPUI-style authoring model:

- build a fresh declarative element tree every frame (ADR 0028),
- externalize cross-frame state and identity (ADR 0028 / ADR 0212),
- reuse recorded output ranges when a subtree is clean (ADR 0055 / ADR 0213).

The current runtime still exposes primarily **node-level invalidation** (`Layout`/`Paint`/`HitTest`) and asks
components to be explicit about invalidation strength (ADR 0005 / ADR 0051).

This is powerful, but it has two recurring issues in an editor-scale UI:

1) **Authoring ergonomics drift**: component authors end up "managing invalidation" instead of thinking in terms of
   "I changed view state; re-render the view".
2) **Cache correctness/perf tension**: view-level caching (ADR 0213) is most effective when the runtime can answer
   "is this view dirty?" deterministically, without requiring that every widget author always picks the perfect
   invalidation mask.

GPUI (Zed) uses a clear and effective primitive:

- mutate view state,
- call `cx.notify(view)` (or `cx.notify()` in a view context),
- the window marks the view dirty and schedules the next frame.

The runtime then decides which parts of the pipeline must run (layout/prepaint/paint) and which cached ranges can be
reused.

This ADR defines the **framework-level contract** for "dirty views" and `notify` in Fret, as a step toward full
GPUI-style ergonomics while keeping Fret's layering rules intact (ADR 0066).

## Decision

### 1) Introduce a view-level invalidation primitive

Fret defines a stable concept: **View**.

- A **View** is a long-lived, stateful UI unit that renders into an element tree (ADR 0039 `Render`).
- Each view has a stable, window-scoped identifier: `ViewId`.

v1 (recommended default for MVP2):

- A **View** is defined at cache boundary granularity: a `ViewCache` root (ADR 0213).
- `ViewId` is therefore a cache-root identity (stable within a window).
- `cx.notify()` (no explicit target) marks the current/nearest cache root dirty. If no cache root is active, it falls
  back to the window root.

The runtime provides:

- `cx.notify()` for the "current view" (v1: current/nearest cache root).
- `cx.notify(view_id)` (optional extension; can be introduced later without breaking the v1 default).

Semantics:

- Calling `notify` indicates the view's rendered output may have changed.
- `notify` MUST be coalesced (multiple calls per tick collapse to one dirty mark per view).
- `notify` MUST schedule a redraw of the owning window at the next safe driver boundary (end of tick / event loop
  iteration), consistent with ADR 0034 scheduling rules.

Frame-driven updates (animation frames):

- `request_animation_frame()` requested from within a view context MUST behave as if `notify()` was called for that
  view on the next tick.
- Rationale: view-cache reuse is allowed only for clean views. If an animation can advance without explicit model
  changes (or without always triggering node-level invalidation), a paint-only frame request would allow stale cached
  output to be replayed indefinitely.

Paint-only frame loops (chrome-only):

- The runtime MAY also provide a **paint-only** animation-frame request for cases where the view is clean and only the
  paint output changes (hover fades, selection/caret blink, drag/drop indicators, scrollbar fades).
- Such a request MUST NOT mark the owning view dirty. It only forces a paint pass and schedules a frame.
- Components MUST NOT rely on paint-only frames to rebuild the declarative element tree. If the output depends on
  rerendering (structural changes, item window changes, state-driven layout), use `request_animation_frame()` (notify
  semantics) or call `notify()` explicitly.

### 2) Dirty views are the primary cache key for view caching

When a view is used behind a caching boundary (ADR 0213), cache reuse is allowed only if:

- the view is not marked dirty since it last rendered, and
- the cache key inputs for that boundary still match (bounds/scale/theme, etc; ADR 0055 / ADR 0213).

This aligns with GPUI's `AnyView::cached` behavior: "reuse previous prepaint/paint if the view wasn't notified".

Nested cache roots must compose correctly (ADR 0213):

- Marking a cache root dirty MUST also mark ancestor cache roots dirty for the relevant streams, so an ancestor never
  replays a range that includes stale descendant output.

### 3) Relationship to node-level invalidation

Node-level invalidation remains a valid internal mechanism, but the contract surface shifts:

- Component/ecosystem layers SHOULD prefer `notify` to express "my view changed".
- Runtime mechanisms MAY still use node-level invalidation internally to minimize work (e.g. mark a cache root as
  `Paint`-dirty without forcing layout).

If both occur:

- `notify(view)` is always strong enough to prevent stale cached output for that view.
- Node-level invalidation MAY further narrow the required work (e.g. paint-only).

### 4) Observations (models/globals) feed into dirty views

When a model/global change is propagated (ADR 0051), the runtime MUST be able to attribute that change to one or more
owning views and mark them dirty, rather than requiring every leaf node to be invalidated independently.

This does not remove explicit observation; it refines the target:

- "data changed -> mark affected views dirty" is the default mechanism.

### 5) Diagnostics requirements

The runtime MUST expose (via ADR 0036 / ADR 0159 tooling surfaces):

- per-window dirty view count per frame,
- list of dirty views (debug-only) and last notify source (best-effort),
- cache hit/miss breakdown by view/cache root.

## Consequences

- Authoring becomes closer to GPUI: "mutate state -> notify".
- View caching becomes easier to reason about: "notified views don't reuse cached ranges".
- It becomes practical to enforce "hover/focus/pressed are paint-only" as a runtime rule, because view dirtiness is
  explicit and centrally tracked (see ADR 0166).

## Alternatives Considered

### A) Only node-level invalidation (status quo)

Pros:
- precise control.

Cons:
- higher author burden,
- harder to enforce cache/interaction invariants consistently across the component ecosystem.

### B) Implicit dependency tracking ("track all reads")

Pros:
- minimal author effort.

Cons:
- harder to debug performance,
- larger runtime surface area and more invasive plumbing (explicitly deferred in ADR 0051).

## Rollout Plan (Incremental)

1) v1: implement `notify()` as "mark nearest cache root dirty" and gate view-cache reuse on that dirtiness.
2) Track `dirty_views` per window (cache-root IDs in v1) and coalesce redraw scheduling at the driver boundary.
3) Add diagnostics counters and inspector exposure (ADR 0159): list dirty views + last notify source (debug-only).
4) Optional: introduce a more explicit `ViewId` model (entity-first, GPUI-like) in a breaking-change window; keep v1
   behavior as a compatibility layer where possible.

## References

- Zed/GPUI `notify` + dirty views:
  - `repo-ref/zed/crates/gpui/src/window.rs` (`WindowInvalidator`, `dirty_views`, `invalidate_view`)
  - `repo-ref/zed/crates/gpui/src/elements/div.rs` (hover state changes call `cx.notify(current_view)`)
- Fret authoring/runtime model:
  - ADR 0028: `docs/adr/0028-declarative-elements-and-element-state.md`
  - ADR 0039: `docs/adr/0039-component-authoring-model-render-renderonce-and-intoelement.md`
  - ADR 0213: `docs/adr/0213-cache-roots-and-cached-subtree-semantics-v1.md`
