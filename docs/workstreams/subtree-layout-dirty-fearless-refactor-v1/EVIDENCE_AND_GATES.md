# Evidence and Gates

## Evidence anchors (starting points)

- Scroll extent correctness / caching:
  - `crates/fret-ui/src/declarative/host_widget/layout/scrolling.rs`
  - `docs/workstreams/scroll-extents-dom-parity.md`
- Invalidation marking/truncation:
  - `crates/fret-ui/src/tree/ui_tree_invalidation_walk/mark.rs`
- Layout clearing:
  - `crates/fret-ui/src/tree/layout/node.rs`

## Gates (required to mark milestones “Done”)

- Unit tests:
  - Add/extend tests in `crates/fret-ui/src/declarative/tests/layout/scroll.rs` for:
    - extent grows at scroll end when content grows
    - descendant invalidation does not “pin” max offset
- Diagnostics (preferred):
  - A `tools/diag-scripts/ui-gallery/...` repro that:
    - navigates to a docs page with Preview/Code
    - scrolls to a bottom section
    - switches to Code
    - verifies the target content can be scrolled fully into view

## “Done” definition

We consider the refactor “done” when:

1. Scroll extent correctness no longer depends on deep subtree scans or forced invalidation hacks.
2. The aggregation mechanism is validated (no drift) via tests + debug checks.
3. Performance overhead is bounded and measured (at least in debug builds on UI Gallery).

## Minimal perf telemetry (recommended)

To make the “propagation strategy” decision data-driven (eager-to-root vs deferred), record at least:

- aggregation updates per frame (count of `layout false -> true` and `true -> false` transitions)
- max parent-walk length (worst-case height traversed in a single update)
- deferred repair work (number of cache roots with pending deltas; total deltas applied)

These can be debug-only counters at first. The goal is to avoid arguing about cost without numbers.
