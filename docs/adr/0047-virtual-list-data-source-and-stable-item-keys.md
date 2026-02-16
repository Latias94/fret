# ADR 0047: Virtual List Data Source and Stable Item Keys


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- gpui-component: https://github.com/longbridge/gpui-component
- virtualizer (Rust): https://github.com/Latias94/virtualizer
- Zed: https://github.com/zed-industries/zed

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
Status: Accepted

## Context

ADR 0042 locks a hard constraint for editor-grade UIs: **no unbounded children in layout engines** and
virtualization must be first-class for large lists/tables/editors.

We now have a working prototype (`VirtualList`, `TreeView`) that validates the rendering approach,
but its current API shape is intentionally “demo-grade” (e.g. a list owns a `Vec` of items).

Before we scale widget count and start building Unity-style panels (Hierarchy/Project/Console/Inspector),
we must lock a *data source contract* that:

- keeps work bounded to the visible range (+ overscan),
- preserves selection/focus across reorders/filters (stable identity),
- works with both the retained `UiTree` widgets and the declarative elements model (ADR 0028),
- stays compatible with app-owned state and borrow-friendly updates (ADR 0031),
- can evolve to variable-height rows (future work in ADR 0042) without breaking API.

References:

- ADR 0042: `docs/adr/0042-virtualization-and-large-lists.md`
- GPUI component library patterns (pinned in repo):
  - `repo-ref/gpui-component/crates/ui/src/virtual_list.rs`
  - `repo-ref/gpui-component/crates/ui/src/list/list.rs`
  - `repo-ref/gpui-component/crates/ui/src/table/state.rs`
- Zed/GPUI runtime list primitives (non-normative):
  - `repo-ref/zed/crates/gpui/src/elements/list.rs`
  - `repo-ref/zed/crates/gpui/src/elements/uniform_list.rs`

## Decision

### 1) Stable item identity is mandatory: `ItemKey`

Virtualized surfaces must identify items by a **stable key**, not by index.

Requirements:

- The key must be **unique within the list**.
- The key must be **stable across frames** and across reorder/filter operations.
- Keys must be cheap to compare and hash (editor lists can be huge).

Runtime contract:

- `crates/fret-ui` uses **`u64`** as its stable `ItemKey` type for virtualization caches and APIs.
- Component-layer code maps its own stable identity types (entity IDs, asset IDs, etc.) into `u64`.

Rationale:

- keeps the runtime substrate minimal (no generic-key plumbing through public runtime APIs),
- matches the dominant “editor ID” pattern (entity/asset IDs are typically u64-like already),
- stays cheap to hash/compare for large datasets.

### 2) Virtualization is driven by a data source / delegate contract

Virtualization containers compute:

- viewport constraints,
- scroll offset,
- visible range (+ overscan),

and then ask a data source to provide the item views for that range.

The contract must support two authoring backends:

- **Retained widgets** (`UiTree`): build rows as widgets or draw ops.
- **Declarative elements** (ADR 0028): build rows as elements.

The *conceptual* API is:

```text
DataSource {
  type Key

  len() -> usize
  key(index) -> Key

  // Called only for visible items (+ overscan).
  render_range(range) -> Vec<Row>

  // Optional: selection/focus helpers.
  index_of_key(key) -> Option<usize>
}
```

Notes:

- `render_range` must be side-effect-free w.r.t. identity. It may read app state, but it must not
  implicitly “shift identity” when indices change; identity is defined by `key(index)`.
- `index_of_key` may be O(n) for small lists, but large lists should provide an indexed lookup.

### 2.1) Key mapping changes must be signaled: `items_revision`

To avoid O(n) key scanning on every frame while still supporting stable-keyed measurement caches,
virtualized surfaces must provide an **`items_revision: u64`** that changes whenever the
`key(index) -> ItemKey` mapping changes for any index.

Typical sources for `items_revision`:

- the underlying list model revision (`Model<Vec<...>>::revision(...)`),
- a stable “data version” counter in the view model,
- for small ephemeral lists (e.g. command palette), a cheap hash of keys.

### 3) Selection and focus are key-based (index is a view detail)

Editor selection and focus must survive:

- reorder and sorting,
- filtering,
- partial loading,
- collapsing/expanding in trees (the visible set changes).

Therefore:

- selection state stores keys (and an anchor key for shift-range selection),
- the virtualized view resolves keys → indices when needed (scrolling, focus movement),
- if a selected key becomes non-visible (e.g. filtered out), the view may:
  - keep it selected but not rendered, and/or
  - fall back to a visible ancestor (tree collapse) at the component layer.

This ADR does not lock the full selection model (single/multi/range), but it locks that **keys are
the stable identity**.

### 4) Variable-size rows are reserved behind the same contract

To align with ADR 0042 (future variable-height items), the data source contract must be compatible with:

- an optional size cache keyed by `(ItemKey, style inputs, DPI scale, text metrics revisions)`,
- a two-phase “estimate → measure visible → update cache → relayout if needed” loop.

For MVP 13, we can ship **fixed row height** only, but we must not paint ourselves into a corner.

### 5) virtualizer vocabulary alignment

Fret’s runtime virtualization vocabulary aligns with `virtualizer` (see `repo-ref/virtualizer`):

- `VirtualItem { key, index, start, end, size }` (vertical axis in P0)
- `gap` between items
- `scrollMargin` (origin shift for headers/multiple virtualizers in one scroll surface)
- `rangeExtractor` remains a **component-layer policy hook**, but the runtime must expose a
  visible-range output that can feed it.

## Consequences

- We can build Hierarchy/Project/Console panels without allocating all rows as nodes.
- Selection and focus behavior stays stable across list mutations.
- The contract can be implemented for both retained widgets and declarative elements without rewriting.
- Future table/grid/code-editor virtualization can reuse the same “key + range rendering” foundation.

## MVP 13 Implementation Plan (Non-Normative)

1. Introduce a `VirtualListDataSource`/delegate API (or equivalent) that provides:
   - `len`, `key`, `render_range`, optional `index_of_key`.
2. Update `VirtualList`/`TreeView` to use the data source contract (no owned `Vec` in the core widget).
3. Add a demo that renders 100k+ rows from a lazy data source and validates:
   - scroll performance,
   - selection stability across reorder/filter.

## Prototype Notes (Non-Normative)

These notes track real implementation findings and follow-ups for reaching “editor-grade” smoothness.

### `index_of_key` must be efficient for large datasets

If the data source implements `index_of_key` as a linear scan, key-based selection can become O(n)
in hot paths (e.g. repeated keyboard navigation or syncing selection after updates).

For editor-scale surfaces, the data source should usually provide O(1) or O(log n) lookup (e.g. a
hash map, btree, or a stable direct-mapping key like `u64 -> usize` when valid).

### Prefer borrowed row text to avoid allocations

If `row_at` allocates a new `String` per visible row (e.g. via `format!`) during scroll, it can
produce stutter even if virtualization is correct.

Preferred patterns:

- `Cow::Borrowed(&str)` for stable labels (most common for editor hierarchies),
- interned strings or shared buffers for computed labels,
- avoid per-frame formatting in the scroll hot path.

### Reduce work on scroll: paint-only + incremental resource updates

Scrolling should typically request **Paint only**, not a full layout pass, unless scrollbars or
content geometry changes.

Also, prepared per-row resources (notably text blobs) should be updated incrementally as the visible
window changes, rather than rebuilt from scratch every tick.

Note:

- “Progressive fill” (limiting per-frame text preparation during fast scroll) is a valid UX lever,
  but it is not considered part of the baseline contract. Prefer solving stutter via caching and
  avoiding hot-path allocations first, then consider progressive fill as a last-mile polish option.
