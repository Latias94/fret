# ADR 0190: Prepaint-Windowed Virtual Surfaces (GPUI-Aligned)

Status: Accepted (v1 contract; implementation in progress)

## Context

Fret’s declarative element tree is rebuilt each frame (ADR 0028), and cross-frame state is externalized via
`ElementRuntime`. View-level caching (ADR 1152) and multi-stream recording (ADR 0055, ADR 0182) aim to make “steady
state” frames near-zero cost by reusing recorded ranges when views are clean (ADR 0180).

However, several editor-scale UI surfaces have a distinct shape:

- the visible content depends primarily on **viewport** and **scroll offset** (plus overscan),
- small scroll deltas should not require rerendering or relayout of a large view-cache root,
- the set of visible “items” (rows/lines/nodes) is naturally **ephemeral per frame** and is better derived during
  `prepaint` (GPUI practice) than during declarative render.

Today, the most visible example is `VirtualList`. In the current Fret model, the set of visible items is derived during
declarative render, which can force a view-cache rerender/contained relayout for scroll-driven changes that could be
transform-only.

We also expect the same pattern to apply broadly in the ecosystem:

- tables and trees built on virtualization,
- code/text views (visible line windows),
- long scrolling documents (markdown/log/trace views),
- canvas/node graphs (viewport culling windows),
- large charts (data window / sampling windows).

This ADR introduces a **framework-level contract** for “prepaint-windowed virtual surfaces”: a mechanism to derive a
visible window during `prepaint` and emit per-frame ephemeral items, while keeping caching correctness explainable and
bounded by dirty views + explicit cache keys.

## Decision

### 1) Define “windowed virtual surfaces” as a prepaint responsibility

Fret defines a “windowed virtual surface” as a UI primitive whose visible content is derived from:

- viewport bounds and scroll offset (or camera transform),
- overscan policy,
- stable data revision inputs (e.g. item count + `items_revision`),
- optional deferred commands (e.g. `scroll_to_item`).

The runtime MUST allow the visible window to be computed during `prepaint`, not only during declarative render.

This ADR intentionally covers both:

- 1D scroll-windowed surfaces (rows/lines/items), and
- 2D viewport-culling surfaces (nodes/edges/sprites) where “window” is a visible region in world space.

### 2) Per-frame ephemeral items are allowed, but must be identity-stable where it matters

The visible window MAY change per frame. The runtime MUST support emitting ephemeral items for that window without
requiring that the entire view-cache root rerender for small deltas.

Identity rules:

- Items SHOULD have stable keys across frames for correctness of focus, selection, and diagnostics.
- Cross-frame state for items MUST remain externalized (ADR 0028) and keyed by stable identity.

### 3) Caching and dirtiness semantics

Windowed surfaces MUST compose with view caching:

- View-cache reuse remains gated by dirty views and explicit cache keys (ADR 0180 / ADR 1152).
- A window change caused solely by scroll offset SHOULD avoid forcing a view-cache rerender when the view is otherwise
  clean; it should instead be treated as an allowed “ephemeral prepaint update” for that surface.
- Out-of-band virtual-surface commands (e.g. `scroll_to_item`) MUST schedule a redraw and invalidate the appropriate
  view boundary deterministically, without relying on unrelated input-driven `notify()` calls.
  - Examples: `scroll_to_item`, `ensure_line_visible`, `center_on_node`, `zoom_to_fit`.

Conformance note (normative):

- “Ephemeral prepaint update” MUST NOT imply silent structural mutation. It means: update prepaint/paint/interaction
  outputs (and request redraw / paint invalidation) while keeping the declarative node graph shape unchanged.

### 3A) Two conformance tracks: non-retained vs retained-host windowing (normative)

This ADR covers two distinct implementation tracks that share the same *user-visible* goal (scroll stability) but have
different structural capabilities.

**Track A (retained host; composable)** — preferred for editor-scale surfaces:

- A window shift MAY be applied during `prepaint` by reconciling retained children inside an explicit retained-host
  boundary (ADR 0192), without forcing a dirty-view rerender of the parent cache root.
- The structural churn (attach/detach) MUST be confined within that retained-host boundary, and MUST remain explainable
  and budgeted (see v2 addendum below).

**Track B (non-retained; plan-only)** — default `VirtualList` path until a retained boundary exists:

- `prepaint` MAY compute and cache a *window plan* (desired visible/required/prefetch ranges) and MAY request a redraw.
- `prepaint` MUST NOT attach/detach/reorder declarative children. Therefore, any window change that would alter the
  mounted child subtree MUST schedule a dirty-view rerender for the next frame.
- If the surface is under view-cache reuse and there is no reliable render-derived window for the current viewport (e.g.
  because a cache-hit frame skipped declarative render during initial viewport bootstrap), `prepaint` MUST conservatively
  treat the window as mismatched and schedule a one-shot rerender/reconcile so the mounted child subtree is correct.
- Scheduling responsibility (to keep behavior explainable and avoid duplicated side effects):
  - For view-cache enabled surfaces, window-boundary detection and rerender scheduling SHOULD be prepaint-driven,
    based on the post-layout bounds + current scroll offset, so a single bundle can attribute “why did we rerender?”
    to one place.
  - Layout MAY update telemetry/state, but SHOULD NOT independently mark cache roots dirty for window shifts if prepaint
    will schedule the same rerender. Double-scheduling makes diagnostics noisy and can produce confusing
    “window changed twice” stories.
- In this track, “avoid forcing rerender for small scroll deltas” is interpreted as:
  - do not rerender while the visible range stays within the currently-rendered required/prefetch window, and
  - allow a single one-shot rerender on “escape” (plus optional staged prefetch that schedules bounded rerenders).

View-cache boundary constraint:

- Nested cache roots inside barrier-driven windowed surfaces (virtual lists, scroll content, etc.) MUST remain safe under
  scroll-driven coordinate space updates. In particular, a cache root inside a windowed surface MUST NOT assume it can
  run an out-of-band “contained relayout” pass against default/stale placement bounds; otherwise semantics/hit-testing
  can be desynchronized (e.g. bounds expressed in unscrolled content space). See ADR 1152 section 5.

### 4) Diagnostics requirements

To keep the model explainable, the runtime MUST expose (debug-only) enough data to answer:

- “Why did the window change?” (scroll offset / viewport change / items revision / command such as `scroll_to_item`)
- “Why did we rerender?” vs “Why did we only prepaint/paint?”

At minimum, a perf bundle should be able to attribute:

- dirty views (with sources like `notify_call`),
- cache-root reuse reasons,
- and virtual-surface window updates (best-effort).

Virtual-surface explainability requirement (v2; normative for windowed surfaces):

- Every window shift MUST have a stable `window_shift_kind` and `window_shift_reason` that can be extracted from a single
  bundle (e.g. `prefetch|escape|scroll_to_item|viewport_resize|items_revision`).
- Every window shift MUST indicate how it was applied:
  - `apply_mode=retained_reconcile` (Track A) or
  - `apply_mode=non_retained_rerender` (Track B).
  This avoids ambiguous “window changed but nothing rerendered” failure modes.

Post-run gate (v2; normative for retained-host suites):

- If a surface is expected to be Track A (retained-host windowing), scripted diagnostics SHOULD gate on:
  - `fretboard diag stats <bundle> --check-vlist-window-shifts-non-retained-max 0`
  so regressions where a retained-host window shift falls back to `non_retained_rerender` are caught automatically.

### 5) Addendum (v2): staged prefetch and window-shift budgets

This section extends the v1 contract with a GPUI/Flutter-aligned best practice for reducing scroll-time
worst-tick spikes when a virtual surface crosses an overscan window boundary.

Definitions:

- **Visible range**: the strict range required to draw the current viewport (no overscan).
- **Required window**: the minimal range that MUST be available for correctness (often equal to the visible range; may include a small “safety” overscan).
- **Prefetch window**: an additional range derived from overscan policy intended purely for smoothness.
- **Escape**: a frame where the visible range leaves the currently-rendered prefetch window.

Contract:

- The runtime MUST treat **required-window coverage** as correctness-critical:
  - if the visible range leaves the required window, the runtime MUST update window state such that the next frame can render correctly,
    even if it implies attaching many new items (e.g. a large scroll jump).
- The runtime SHOULD treat **prefetch** as budgeted work:
  - prefetch updates may be performed incrementally across multiple frames,
  - prefetch MUST NOT require rerendering the entire view-cache root for retained hosts,
  - and the runtime SHOULD cap per-frame structural churn (attach/detach) caused purely by prefetch.

Practical guidance (non-normative, but strongly recommended for perf stability):

- Prefer **early, small window shifts** over “rare, large boundary jumps”.
  - When the visible range approaches a prefetch boundary (but is still covered), shift the window by a small step (e.g. 1–N items)
    and request a redraw so the new items attach gradually.
  - This turns a single “boundary tick” spike into a bounded stream of smaller reconciles.
- Distinguish work due to **escape** vs **prefetch** in diagnostics:
  - Escape-driven reconcile is correctness-critical; prefetch-driven reconcile is optional and should be budgeted.
  - A single `bundle.json` should explain which case occurred for each reconcile.

Diagnostics requirements (v2):

- Bundles SHOULD export, per virtual-list window update:
  - an explicit `window_shift_kind` (e.g. `none`, `prefetch`, `escape`),
  - and a stable `policy_key` / `inputs_key` so policy drift is catchable by scripted gates.
- Bundles SHOULD export, per retained-host reconcile:
  - `reconcile_kind` (`prefetch` vs `escape`),
  - and `attached_items` / `detached_items` / keep-alive reuse counters so churn spikes are attributable.

## Consequences

- VirtualList and other large surfaces can become scroll-stable (transform-only for most wheel deltas) while still
  benefiting from view-cache range reuse.
- Ecosystem crates gain a reusable mechanism to implement windowed surfaces without copying ad-hoc invalidation logic.
- The runtime must provide a narrow but explicit “ephemeral prepaint update” pathway that does not break the existing
  cache-root/dirtiness contracts.

## Relationship to existing ADRs

- ADR 0180: remains the primary gate for view-cache reuse (“dirty views block reuse”).
- ADR 0182: defines multi-stream recording and prepaint responsibilities; this ADR specifies a virtualization/windowing
  use case that benefits from `prepaint`.
- ADR 0070: defines the virtualization math/metrics contract; this ADR does not change the math, only when/how the
  visible window is derived and applied.
- ADR 1152: cache-root semantics and subtree reuse must remain correct under windowed updates.

## References

- gpui-component VirtualList derives visible range and consumes `scroll_to_item` during `prepaint`:
  - `repo-ref/gpui-component/crates/ui/src/virtual_list.rs`
- Zed/GPUI dirty views + view caching gates:
  - `repo-ref/zed/crates/gpui/src/window.rs`
  - `repo-ref/zed/crates/gpui/src/view.rs`
- Flutter element lifecycle and virtualization patterns:
  - `repo-ref/flutter/packages/flutter/lib/src/widgets/framework.dart` (`BuildOwner.finalizeTree`, `_InactiveElements`, `Element.deactivateChild`)
  - `repo-ref/flutter/packages/flutter/lib/src/rendering/sliver_multi_box_adaptor.dart` (`_keepAliveBucket`)
  - `repo-ref/flutter/packages/flutter/lib/src/widgets/scroll_view.dart` (`cacheExtent`)

## Implementation Notes (v1 Progress)

This ADR defines a contract; the implementation is tracked in `docs/workstreams/gpui-parity-refactor-todo.md` (MVP5).

Early evidence that the runtime is moving toward this model:

- VirtualList has an explicit notion of a layout-derived visible window and can keep wheel scrolling transform-only when
  the window is stable:
  - `crates/fret-ui/src/declarative/host_widget/layout/scrolling.rs`
  - `crates/fret-ui/src/declarative/host_widget/event/scroll.rs`
- Out-of-band virtual-surface commands (e.g. `scroll_to_item`) are surfaced to view-cache reuse decisions to avoid
  “stale cached output behind cache-hit frames”:
  - `crates/fret-ui/src/declarative/mount.rs` (pre-render scroll-handle invalidation gate)
  - `crates/fret-ui/src/declarative/tests/view_cache.rs` (`view_cache_rerenders_on_virtual_list_scroll_to_item`)
- Diagnostics bundles can export virtual-surface window telemetry for postmortem explanation:
  - `ecosystem/fret-bootstrap/src/ui_diagnostics.rs` (`UiTreeDebugSnapshotV1.virtual_list_windows`)
- VirtualList “rowcached” experiments (nested cache roots per row) are validated for correctness under `scroll_to_item`
  once `contained_layout` is treated as opt-in (nested row cache roots should not run contained relayout under barrier
  placement):
  - Unit regression: `crates/fret-ui/src/declarative/tests/view_cache.rs`
