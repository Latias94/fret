# ADR 0217: Scroll Offset Children Transform and ScrollHandle Invalidation (v2)

Status: Accepted

## Context

Fret uses a retained `UiTree` substrate with a declarative per-frame element tree and an
invalidation model that controls when layout/paint/hit-testing re-run.

`ScrollHandle` is an imperative, clone-cheap handle used by component-layer code to drive scrolling
state. ADR 0215 introduced a revision-based binding registry so imperative handle changes can
invalidate the right subtrees.

In profiling runs (e.g. `tools/diag-scripts/ui-gallery-virtual-list-torture.json`), we observed a
major performance cliff where **scroll offset changes** trigger `Invalidation::Layout`, which
causes layout-engine solves and expensive `Scroll` measurement (`MaxContent` probing) even though
the offset change does not affect intrinsic layout size.

This harms "editor-grade" UX: scroll should feel like a paint/input translation, not a full layout
solve.

## Decision

### 1) Introduce a children-only transform hook

Add `Widget::children_render_transform(bounds) -> Option<Transform2D>`.

This transform:

- Applies to **children only** (not to this node's own bounds/clip/hit-test).
- Is used consistently by the runtime for:
  - hit-testing recursion (map pointer position into child space),
  - pointer-bearing event coordinate mapping down the dispatch chain,
  - semantics snapshot bounds export,
  - debug visual-bounds queries,
  - paint-time visual bounds recording (by keeping the transform stack bookkeeping consistent).

Non-invertible transforms are ignored for input mapping (same policy as `render_transform`).

### 2) Model scroll offset as a children-only transform

`Scroll` layout no longer shifts child layout bounds by the current scroll offset. Instead, the
scroll offset is represented as a children-only transform:

- Content is painted translated by `(-offset_x, -offset_y)` under the scroll viewport clip.
- Hit-testing and event coordinates are mapped through the inverse translation.
- The layout geometry of the content subtree remains stable across scroll offset changes.

### 3) Add `Invalidation::HitTestOnly`

Add a new invalidation tier:

- `Invalidation::HitTestOnly`: recompute hit-testing + repaint, **without** forcing a layout pass.

This is used for changes that affect coordinate mapping but not layout geometry (e.g. scrolling).

### 4) Make scroll-handle revision invalidation fine-grained

Extend the scroll-handle binding registry to classify revision changes:

- If **only** the handle `offset` changed: invalidate bound nodes with `Invalidation::HitTestOnly`.
- If `viewport_size` or `content_size` changed: invalidate bound nodes with `Invalidation::Layout`.
- If the revision changed but none of `(offset, viewport_size, content_size)` changed (revision-only
  touch): treat as `Invalidation::Layout` (covers deferred scroll requests consumed during layout).

This preserves determinism for imperative scroll requests while avoiding layout solves for
translation-only scrolling.

## Consequences

- Scroll offset changes no longer force layout engine solves for unaffected subtrees.
- The scroll contract becomes closer to GPUI/Zed's “scroll is a translation” behavior.
- Diagnostics can attribute `scroll_handle` invalidations as `hit_test_only` vs `layout`.
- Call sites that truly need layout changes must keep using `Invalidation::Layout` or `HitTest`
  (which still escalates to layout by design).

## References

- Supersedes ADR 0215: `docs/adr/0215-scroll-handle-revision-invalidation-contract.md`
- Render transform input contract (related): `docs/adr/0082-render-transform-hit-testing.md`
- Profiling report: `docs/perf/ui-gallery-profile-report.md`
- Non-normative GPUI reference: `repo-ref/zed/crates/gpui/src/window.rs`

