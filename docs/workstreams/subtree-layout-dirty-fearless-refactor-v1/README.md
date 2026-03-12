# Subtree Layout Dirty Aggregation (Fearless Refactor v1)

Status: **in progress**

Last updated: **2026-03-02**

## Motivation

Some correctness bugs manifest as “content grew, but the scroll range didn’t” (e.g. switching a
Preview/Code tab near the bottom of a long docs page and being unable to scroll further).

At the mechanism level, these failures are usually caused by an implicit assumption:

> If a descendant needs layout, the scroll’s direct child root will also be marked layout-dirty.

When that assumption breaks (due to caching boundaries, truncation, bookkeeping bugs, or other
edge cases), a scroll container can reuse last-frame extent caches and “pin” its `max_offset` to a
stale value.

This workstream proposes a **mechanism-level** aggregation primitive that makes it cheap and
reliable to answer:

- “Does this node’s subtree contain any layout-dirty nodes?” (O(1))

…without requiring deep subtree scans or relying on invalidation bubbling to the nearest root.

## Non-goals

- Redesigning scroll extent computation end-to-end (tracked in
  `docs/workstreams/scroll-extents-dom-parity/scroll-extents-dom-parity.md`).
- Changing view-cache semantics or contained-layout contracts:
  - Cache roots + cached subtree semantics:
    `docs/adr/0213-cache-roots-and-cached-subtree-semantics-v1.md`
  - Dirty views + notify:
    `docs/adr/0165-dirty-views-and-notify-gpui-aligned.md`
- Introducing CSS-like “percent under auto-height” semantics in the layout engine.

## Scope

- Primary: `crates/fret-ui` invalidation bookkeeping + scroll layout fast paths.
- Secondary: view-cache repair passes and diagnostics (optional, as needed for correctness).

## Current status (v1)

Implemented the aggregation counter + bookkeeping and migrated the scroll “extent edge” workaround
from a deep DFS scan to an O(1) subtree query.

Evidence anchors:

- Node storage:
  - `crates/fret-ui/src/tree/node_storage.rs`
  - `crates/fret-ui/src/tree/ui_tree_subtree_layout_dirty.rs`
- Invalidation marking + truncation behavior:
  - `crates/fret-ui/src/tree/ui_tree_invalidation_walk/mark.rs`
  - `crates/fret-ui/src/tree/ui_tree_invalidation.rs`
- Layout clearing:
  - `crates/fret-ui/src/tree/layout/node.rs`
- Structural mutations (attach/detach):
  - `crates/fret-ui/src/tree/ui_tree_mutation/core.rs`
  - `crates/fret-ui/src/tree/ui_tree_mutation/mount.rs`
  - `crates/fret-ui/src/tree/ui_tree_mutation/barrier.rs`
  - `crates/fret-ui/src/tree/ui_tree_mutation/remove.rs`
- Scroll consumer:
  - `crates/fret-ui/src/declarative/host_widget/layout/scrolling.rs`

Runtime flags / validation:

- `FRET_UI_LAYOUT_SUBTREE_DIRTY_AGGREGATION=0` disables the mechanism (default-on).
- `FRET_UI_LAYOUT_SUBTREE_DIRTY_AGGREGATION_VALIDATE=1` enables drift validation.
- `FRET_UI_LAYOUT_SUBTREE_DIRTY_AGGREGATION_VALIDATE_PANIC=1` panics on drift.

## Documents

- Design: `docs/workstreams/subtree-layout-dirty-fearless-refactor-v1/DESIGN.md`
- Milestones: `docs/workstreams/subtree-layout-dirty-fearless-refactor-v1/MILESTONES.md`
- TODO tracker: `docs/workstreams/subtree-layout-dirty-fearless-refactor-v1/TODO.md`
- Evidence and gates: `docs/workstreams/subtree-layout-dirty-fearless-refactor-v1/EVIDENCE_AND_GATES.md`
- Open questions: `docs/workstreams/subtree-layout-dirty-fearless-refactor-v1/OPEN_QUESTIONS.md`

## Recommended v1 decisions

For the current recommended direction on the major open questions (propagation strategy, counter vs
epoch, invariants), see:

- `docs/workstreams/subtree-layout-dirty-fearless-refactor-v1/OPEN_QUESTIONS.md`
