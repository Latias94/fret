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
