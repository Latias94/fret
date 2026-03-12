# Evidence and Gates

## Evidence anchors (starting points)

- Scroll extent correctness / caching:
  - `crates/fret-ui/src/declarative/host_widget/layout/scrolling.rs`
  - `docs/workstreams/scroll-extents-dom-parity/scroll-extents-dom-parity.md`
- Aggregation mechanism + validation:
  - `crates/fret-ui/src/tree/ui_tree_subtree_layout_dirty.rs`
  - `crates/fret-ui/src/runtime_config.rs`
- Invalidation marking/truncation:
  - `crates/fret-ui/src/tree/ui_tree_invalidation_walk/mark.rs`
  - `crates/fret-ui/src/tree/ui_tree_invalidation.rs`
- Layout clearing:
  - `crates/fret-ui/src/tree/layout/node.rs`
- Structural mutations:
  - `crates/fret-ui/src/tree/ui_tree_mutation/core.rs`
  - `crates/fret-ui/src/tree/ui_tree_mutation/mount.rs`
  - `crates/fret-ui/src/tree/ui_tree_mutation/barrier.rs`
  - `crates/fret-ui/src/tree/ui_tree_mutation/remove.rs`

## Gates (required to mark milestones “Done”)

- Unit tests:
  - Add/extend tests in `crates/fret-ui/src/declarative/tests/layout/scroll.rs` for:
    - extent grows at scroll end when content grows
    - descendant invalidation does not “pin” max offset
  - Current coverage:
    - `scroll_extent_updates_when_descendant_invalidated_but_child_root_cleared`
- Diagnostics (preferred):
  - A `tools/diag-scripts/ui-gallery/...` repro that:
    - navigates to a docs page with Preview/Code
    - scrolls to a bottom section
    - switches to Code
    - verifies the target content can be scrolled fully into view
  - Example:
    - `tools/diag-scripts/ui-gallery/typography/ui-gallery-typography-inline-code-tab-scroll-range.json`
  - Additional (bottom-of-page notes regression):
    - `tools/diag-scripts/ui-gallery/collapsible/ui-gallery-collapsible-rtl-code-tab-scroll-range.json`

## “Done” definition

We consider the refactor “done” when:

1. Scroll extent correctness no longer depends on deep subtree scans.
2. The aggregation mechanism is validated (no drift) via tests + debug checks.
3. Performance overhead is bounded and measured (at least in debug builds on UI Gallery).

## Minimal perf telemetry (recommended)

To make the “propagation strategy” decision data-driven (eager-to-root vs deferred), record at least:

- aggregation updates per frame (count of `layout false -> true` and `true -> false` transitions)
- max parent-walk length (worst-case height traversed in a single update)
- deferred repair work (number of cache roots with pending deltas; total deltas applied)

These can be debug-only counters at first. The goal is to avoid arguing about cost without numbers.

## Validation knobs (debug)

- `FRET_UI_LAYOUT_SUBTREE_DIRTY_AGGREGATION_VALIDATE=1`
- `FRET_UI_LAYOUT_SUBTREE_DIRTY_AGGREGATION_VALIDATE_PANIC=1`
