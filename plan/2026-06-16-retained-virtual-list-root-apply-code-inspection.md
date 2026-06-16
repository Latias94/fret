# Retained VirtualList Root Apply Code Inspection

Status: Active note
Last updated: 2026-06-16

## Purpose

Record the current evidence pass for the retained VirtualList performance lane and keep the next
decision point explicit.

## What I checked

- Latest bundle: `target/fret-diag/retained-vlist-root-apply-m1-root-local-skip-v1/1781549017090/bundle.json`
- Hot frame still reports:
  - `layout_children_first_pass_us=8300`
  - `layout_child_max_subtree_dirty_count=625`
  - `layout_child_max_nodes_performed=625`
  - `retained_virtual_list_reconciles=0`
  - `set_children_barrier_writes=1`
- Code paths reviewed:
  - `crates/fret-ui/src/declarative/host_widget/layout/scrolling.rs`
  - `crates/fret-ui/src/declarative/mount.rs`
  - `ecosystem/fret-ui-kit/src/declarative/table.rs`
  - `apps/fret-ui-gallery/src/ui/previews/gallery/data/table_torture.rs`

## What I found

1. The fixed/measured split in `layout_virtual_list_impl` is real, but the fixed path still lays out
   every visible child. It only skips per-row measurement work; it does not skip the first-pass
   child traversal.
2. The retained data-table surface already uses the fixed path by default. `table_virtualized_retained_v0`
   sets `VirtualListMeasureMode::Fixed` and `VirtualListKeyCacheMode::VisibleOnly` for retained
   fixed-height rows.
3. The 625-node dirty/performed count looks more like visible row subtree breadth than like a
   retained-reconcile burst. That is an inference from the bundle and the row/cell wrapper shape.
4. `Scroll` still pays barrier solve cost, but it is secondary in the current frame.

## Decision

- Do not jump to a broad `VirtualList` mechanism rewrite yet.
- Treat the next optimization choice as a fork:
  - narrower fixed-height retained-host fast path, or
  - component-surface simplification in the table/row tree.

## Follow-up Slice

- Removed a pure test-id `Semantics` wrapper from the retained table cell hot path.
- Replaced the capability-first helper's wrapper with `attach_semantics` on the existing text node.
- Kept the visible table shape unchanged.

## Follow-up Evidence

- The repaired mouse-wheel repro produced
  `target/fret-diag/retained-vlist-root-apply-m2-cell-semantic-dewrapper-v2/1781578517352/bundle.schema2.json`.
- The worst retained frame improved to `top_total_time_us=10607`, with `layout=9882`,
  `layout.engine_solve=6516`, `layout.root apply=8912`, and `layout.nodes=514`.
- This confirms wrapper breadth was part of the remaining cost, but the ownership did not move back
  to generic table-local code. Retained `VirtualList` plus the parent `Scroll` remains the next
  mechanism seam.

## Next Check

- Compare the retained row subtree shape against upstream shadcn/base-ui references.
- Add a focused test before any contract change.
- Re-run the same retained data-table repro after the next slice.
