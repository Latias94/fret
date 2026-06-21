# TODO: Retained VirtualList Root Apply v1

Status: Active
Last updated: 2026-06-21

## M0 - Lane Setup And Baseline

- [x] Split a narrow follow-on from `ui-table-row-cell-structural-churn-v1`.
- [x] Record the post-key-hook retained data-table evidence and node-profile attribution.
- [x] Re-run the retained data-table repro with scroll layout profiling enabled.
- [x] Extract the smallest retained `VirtualList` phase owner from the new bundle.

## M1 - Mechanism Attribution

- [x] Inspect retained `VirtualList` reconcile and layout phases against ADR 0175/0177.
- [x] Decide whether the first owner is reconcile, `set_children_barrier`, barrier-root solve, or
      child layout.
- [x] Confirm that the hot owner in the current bundle is the fixed/known-height retained
      `VirtualList` first-pass child layout path, with `Scroll` still paying a secondary
      root-apply/barrier cost.
- [x] Add diagnostics if the current bundle cannot answer that split.
- [x] Add a focused retained VirtualList test before changing mechanism behavior.
- [x] Narrow the single-root layout fast path so clean roots no longer depend on global dirty
      counters before reusing cached size.

## M2 - First Reversible Optimization

- [x] Land one narrow `crates/fret-ui` slice only after the owner is measured.
- [x] Keep shadcn/table recipe behavior unchanged unless profiling moves back to recipe code.
- [x] Remove pure test-id `Semantics` wrappers from retained table cell rendering when the same
      anchor can be attached to an existing node.
- [x] Run retained VirtualList harness gates.
- [x] Re-run the retained data-table diag script and record the before/after stats.
- [x] Re-run the retained data-table perf repro after the cell wrapper deletion with the mouse-wheel
      path, avoiding the missing `diag.pointer_kind_touch` capability.
- [x] Compare the retained row subtree shape against upstream table/list references before the next
      mechanism slice.
      `repo-ref/ui/apps/v4/registry/new-york-v4/ui/table.tsx` and
      `repo-ref/ui/apps/v4/registry/new-york-v4/examples/data-table-demo.tsx` keep the body row
      shape at `TableRow -> TableCell -> content`, while Base UI keeps scroll-area content as a
      separate viewport/content concern. Fresh stats moved the hot retained path back to per-cell
      wrapper breadth, not a broader scroll-area mismatch.
- [x] Decide whether the next slice should deepen `VirtualList` or flatten the row/cell tree.
- [x] Disable per-cell debug anchors in the torture preview and verify the perf rerun stays
      owned by retained `VirtualList` plus the parent `Scroll`.

## M3 - Closeout Or Next Split

- [ ] If root apply/barrier work is reduced and the next owner moves to fixed row geometry, split a
      fixed-track layout primitive lane.
- [ ] If the owner moves to a narrower fixed-height retained `VirtualList` fast path, split that
      as a follow-on rather than widening the current lane.
- [ ] If the owner moves to `ViewCache`, renderer, or frame pipeline code, split a narrower owner
      lane instead of widening this one.
- [ ] If the current retained `VirtualList` frame still reports broad dirty-subtree fanout with no
      retained-reconcile burst, inspect scroll-handle invalidation and subtree dirty propagation
      before adding more root-local layout cleanup.
- [x] Characterize the retained ViewCache settle frame after retained-host reconcile; stable
      `view_cache` callsite identity proves the initial third-frame miss was a test artifact, so no
      runtime reuse-root marking change should land from that evidence alone.
- [x] Re-run the retained data-table repro with `debug.layout_root_applies[]` available, then use
      `layout_root_applies` to decide whether the next owner is root apply, retained `VirtualList`,
      `Scroll`, or a narrower follow-on.
- [x] Re-run the retained data-table repro after the fixed-row inline cell-padding slice and record
      the before/after stats.
- [ ] If the current evidence says the row/cell tree itself is the dominant cost, close this lane
      with a follow-on that owns table/recipe tree depth instead of more `VirtualList` cleanup.
- [ ] Update `WORKSTREAM.json`, `MILESTONES.md`, and `EVIDENCE_AND_GATES.md` after each landed
      slice.
- [ ] Add a closeout note when this lane stops owning active implementation.
