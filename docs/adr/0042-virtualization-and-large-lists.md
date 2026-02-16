# ADR 0042: Virtualization for Large Lists and Editors (No Unbounded Children in Layout Engines)


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- gpui-component: https://github.com/longbridge/gpui-component
- Zed: https://github.com/zed-industries/zed

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
Status: Accepted

## Context

Editor UIs contain extreme-scale surfaces:

- project trees with tens of thousands of nodes,
- tables/grids with thousands of rows/columns,
- log views with unbounded append,
- code editors with 100k+ lines.

If the UI runtime naïvely builds all children and/or feeds all nodes into a constraint solver (e.g. `taffy`),
performance will collapse:

- building the element/widget subtree dominates frame time,
- the layout engine becomes O(n) or worse per tick,
- memory usage explodes.

This interacts directly with:

- declarative rebuilds (ADR 0028): “build every frame” must still be bounded,
- optional `taffy` integration (ADR 0035): `taffy` cannot be the substrate for unbounded children,
- text system scale-up (ADR 0029): code editor widgets must remain feasible.

References:

- GPUI component library virtualization patterns:
  - `repo-ref/gpui-component/crates/ui/src/virtual_list.rs`
  - `repo-ref/gpui-component/crates/ui/src/table/state.rs`
- Zed/GPUI runtime list primitives (non-normative):
  - list elements and scroll/clip integration patterns:
    `repo-ref/zed/crates/gpui/src/elements/list.rs`,
    `repo-ref/zed/crates/gpui/src/elements/uniform_list.rs`

## Decision

### 1) Virtualization is a first-class UI capability (P0 contract)

The framework must provide (or reserve) a virtualization container contract:

- `VirtualList` / `ListView` / `VirtualGrid` style components,
- a scroll model,
- a way to render only the visible items (+ overscan),
- stable identity for items via explicit keys (align with ADR 0028).

Virtualization is required for editor-grade components and must be considered a “decide early” contract,
even if implementation ships later in `fret-components-*`.

### 2) Layout engines must not require unbounded children

Hard rule:

- No layout engine (including `taffy`) is allowed to require constructing or maintaining nodes for “all items”
  in an unbounded list/table/editor.

Instead:

- virtualization containers compute visible ranges from scroll offset and viewport constraints,
- only the visible items are instantiated and laid out each frame.

### 3) Virtualization owns the primary axis; item internals are free to use Flex/Grid

To avoid “taffy vs virtualization” conflicts:

- the virtualization container owns the primary axis positioning (stacking) and scroll offset,
- each visible item is laid out independently inside its slot:
  - items may use `Flex/Grid` internally (and thus `taffy`) without requiring `taffy` to see the entire dataset.

This matches how editor UIs typically work (rows/cells are composable, but the list is windowed).

### 4) Variable-size items are supported via two-phase measurement (P0 baseline)

Many editor lists have variable row heights (wrapped text, nested rows).

Baseline approach:

- keep an `ItemSizes` cache (estimated or measured),
- compute a visible window using cached sizes,
- measure/layout only visible items,
- update the size cache for items that changed, and request another layout pass if needed.

To avoid “jumping” when estimated sizes are replaced by measured sizes, the scroll model should preserve a
stable **scroll anchor** (typically the first visible item + an in-item offset). When the size cache changes,
adjust the scroll offset by the delta of the anchor item’s prefix sum so that the same content stays under the
cursor/viewport.

The caching key must include:

- item stable key,
- style inputs (tokens),
- DPI scale,
- text metrics revisions (ADR 0006 / ADR 0029).

### 5) Semantics and hit-testing follow the visible set

- Hit-testing and event routing operate only on instantiated (visible) items.
- Semantics/A11y output may expose only visible nodes by default, but must remain correct for focus/selection:
  - focused/selected items must be materialized or represented by a minimal semantics stub when offscreen (future work).

## Consequences

- Code-editor-grade widgets remain feasible without rewriting the layout/runtime model.
- `taffy` remains useful for local layouts without becoming a global bottleneck.
- The declarative element model (ADR 0028) remains compatible with large datasets by construction.

## Future Work

- Decide the exact virtualization API shape (delegate pattern vs item builder closures).
- Add a virtualized table/grid contract (column sizing, pinned headers, selection).
- Define offscreen focus/semantics policies for assistive technologies.

## Implementation Notes (Non-Normative)

These notes record performance lessons from prototype implementations. They are not part of the
normative contract.

### Avoid scroll-driven full layout invalidation

In a retained UI tree, invalidating `Layout` on every scroll tick tends to be the primary source of
stutter, especially when lists are nested (a small inner list can become smooth while the outer list
still forces large parts of the tree to relayout).

A recommended direction (mirrors GPUI patterns) is:

- keep layout stable for the list container,
- treat scroll offset as paint/prepaint-time translation + clip,
- request **Paint only** while scrolling unless the scrollbars/layout geometry actually changes.

### Incremental visible-window updates

Rebuilding all visible row resources (e.g. releasing + re-shaping all visible text) on every visible
range change can dominate frame time on large datasets.

Prefer incremental updates:

- retain prepared resources for rows that stay within the new visible window,
- only create resources for newly-visible rows,
- release resources for rows that leave the window.

### Text is often the bottleneck (not the list math)

For uniform-height lists, visible range computation is O(1) and usually not the bottleneck.
Stutter commonly comes from text shaping/rasterization and per-row allocations during scroll.

Practical follow-ups (still compatible with this ADR):

- cache prepared text blobs by `(ItemKey, width, style, DPI, text metrics revision)`,
- avoid allocating formatted strings in hot paths (prefer borrowed labels or interned strings),
- treat “per-frame prepare budgets / progressive fill” as an optional UX optimization; prefer robust
  caching and prepaint-time virtualization first.
