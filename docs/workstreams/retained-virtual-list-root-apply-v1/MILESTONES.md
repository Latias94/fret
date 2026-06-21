# Milestones: Retained VirtualList Root Apply v1

Status: Active
Last updated: 2026-06-21

## M0 - Baseline And Lane Split

Status: In progress.

Done criteria:

- The lane exists as a narrow follow-on.
- Baseline retained data-table evidence is recorded.
- The first diagnostic command includes both layout node profiling and scroll layout profiling.

Current evidence:

- Retained shared-row-transform bundle:
  `target/fret-diag/vlist-retained-shared-row-xform-v1/sessions/1781530321751-126564/1781531045060-ui-gallery-data-table-retained-filter-shrink-vlist-inputs-change/bundle.schema2.json`
- Retained row key-hook prune bundle:
  `target/fret-diag/vlist-retained-row-key-hook-prune-v3-retained-only/1781536422863/bundle.json`
- Post-key-hook node-profile bundle:
  `target/fret-diag/vlist-retained-post-key-hook-node-profile-v1/1781537495673/bundle.json`
- Scroll layout profile bundle:
  `target/fret-diag/retained-vlist-root-apply-scroll-profile-v1/1781539565855/bundle.schema2.json`

## M1 - Mechanism Attribution

Status: Completed.

Done criteria:

- `layout_virtual_list_impl` phase costs are separated enough to name one owner.
- The owner is classified as reconcile, barrier child list mutation, barrier-root solve, per-child
  layout, or diagnostics gap.
- One focused correctness test is chosen or added before implementation.
- The current evidence already points to fixed/known-height retained `VirtualList` first-pass child
  layout as the main owner.
- A root-local clean-layout characterization test now proves clean roots can skip layout-engine
  entry even when unrelated nodes are dirty.

## M2 - First Runtime Slice

Status: In progress.

Done criteria:

- One reversible `crates/fret-ui` mechanism change lands.
- Retained VirtualList correctness gates pass.
- The retained data-table script is re-run with the same measurement surface.
- Evidence distinguishes real performance improvement from measurement noise or structural cleanup.
- If the next slice targets the fixed-height path directly, capture before/after evidence against
  `retained-vlist-root-apply-scroll-profile-v1`.

Current evidence:

- Root-local layout fast path slice landed in `crates/fret-ui/src/tree/layout/entrypoints.rs`.
- Focused characterization test landed in `crates/fret-ui/src/tree/tests/view_cache.rs`.
- Retained VirtualList gates passed after the slice.
- Fresh perf bundle:
  `target/fret-diag/retained-vlist-root-apply-m1-root-local-skip-v1/1781549017090/bundle.json`
  with `top_total_time_us=11278`.
- Follow-up bundle reading confirms the remaining owner is still the retained `VirtualList`
  first-pass child layout path, but the hot frame is not a retained-reconcile burst:
  `layout_child_max_subtree_dirty_count=625`, `layout_child_max_nodes_performed=625`,
  `retained_virtual_list_reconciles=0`, and `set_children_barrier_writes=1`.
- This shifts the next slice from root-local cleanup toward a deeper retained-host/layout dirty
  propagation question or a narrower barrier short-circuit.
- The latest code inspection also shows the fixed-path `VirtualList` still walks every visible child
  and only skips measurement, not first-pass layout. The remaining 625-node breadth may therefore
  reflect the retained table row/cell tree itself, not only framework overhead.
- A follow-up slice removed pure test-id `Semantics` wrappers from retained table cell rendering in
  `ecosystem/fret-ui-kit/src/declarative/table.rs`. Focused `fret-ui-kit` gates still pass.
- The retained data-table repro was rerun with the mouse-wheel path after the wrapper deletion:
  `target/fret-diag/retained-vlist-root-apply-m2-cell-semantic-dewrapper-v2/1781578517352/bundle.schema2.json`.
  It reported `top_total_time_us=10607`, `layout=9882`, `layout.engine_solve=6516`,
  `layout.root apply=8912`, and `layout.nodes=514`.
- This is a measurable improvement from the prior retained bundles, but the remaining owner is
  still retained `VirtualList` plus the parent `Scroll`. Continue this lane; do not move ownership
  back to broad table-local wrapper shaving without a new node profile that proves it.
- Fresh scroll telemetry in
  `target/fret-diag/retained-vlist-root-apply-m4-scroll-roots-v2/1781584457222/bundle.schema2.json`
  shows the hot retained `VirtualList` child path is still one dirty subtree with deep performed
  breadth, not a partially skippable mix of clean and dirty roots. That keeps the next slice in
  mechanism depth / barrier propagation territory.

## M3 - Follow-on Decision

Status: Pending.

Done criteria:

- The perf repro after the wrapper deletion is captured.
- We can say whether the hot path moved enough to justify continuing row/cell flattening.
- If the owner does not move, the lane should split a narrower follow-on instead of widening back
  into broad `VirtualList` cleanup.

## M3 - Closeout Or Follow-On

Status: Pending.

Done criteria:

- The lane either closes with a retained `VirtualList` root-apply improvement, or records why no
  runtime change should land.
- Any next owner is split into a narrower lane instead of widening this one.
- If the next evidence pass shows the table/row tree is the main cost, split a table-tree-depth
  follow-on instead of forcing more retained `VirtualList` cleanup.

## M4 - Retained Body Hoist

Status: Complete.

Done criteria:

- The retained single-center body owns the shared horizontal transform once.
- The focused body-hoist gate passes.
- A fresh perf bundle is captured against rebuilt release binaries.
- The remaining hotspot is still retained `VirtualList` plus the parent `Scroll`, so the lane
  keeps pointing at deeper mechanism work instead of row-wrapper shaving.

Evidence:

- `target/fret-diag/1781592842180/bundle.json`
- `diag stats --sort cpu_cycles --top 30`: `top_total_time_us=10130`, `layout=9468`,
  `layout.engine_solve=6435`, `layout.root apply=8595`, `layout.nodes=417`

## M5 - Cell Anchor Toggle

Status: Complete.

Done criteria:

- The heavy torture preview stops paying per-cell debug anchor formatting cost.
- The retained data-table perf repro is rerun against rebuilt release binaries.
- The hotspot remains retained `VirtualList` plus the parent `Scroll`, so the lane still points
  at mechanism work rather than table-local wrapper cleanup.

Evidence:

- `target/fret-diag/1781594910783/bundle.schema2.json`
- `diag stats --sort cpu_cycles --top 30`: `top_total_time_us=9965`, `layout=9328`,
  `layout.engine_solve=6595`, `layout.root apply=8546`, `layout.nodes=417`

## M6 - Root Apply Owner Attribution

Status: Complete.

Done criteria:

- `layout_roots_apply_time_us` has owner attribution instead of only an aggregate phase total.
- Debug snapshots expose top root apply records with mode, dirty-state, node deltas, and
  clean-geometry deltas.
- `fret-diag` carries the records through stats JSON, human detail rows, triage JSON, and
  `layout_perf_summary`.
- The slice is diagnostic-only; normal layout behavior is unchanged.

Evidence:

- `crates/fret-ui/src/tree/layout/entrypoints.rs` records `debug.layout_root_applies[]` around the
  window-root apply loop only when debug is enabled.
- `ecosystem/fret-bootstrap/src/ui_diagnostics/layout_paint_hotspot_diagnostics.rs` exports
  `UiLayoutRootApplyV1`.
- `crates/fret-diag/src/stats/bundle_stats_snapshot.rs`,
  `crates/fret-diag/src/triage_json.rs`, and
  `crates/fret-diag/src/layout_perf_summary.rs` expose the new attribution in diag tooling.
- Focused gates passed:
  `clean_geometry_window_root_resize_consumes_apply_plan_without_root_layout`,
  `triage_includes_hints_and_unit_costs_for_worst_frame`, `layout_perf_summary`, and
  `cargo check -p fret-bootstrap --lib`.

## M7 - Retained Fixed Row Inline Cell Padding

Status: Complete.

Done criteria:

- Fresh `layout_root_applies` evidence confirms the aggregate apply owner still expands through
  the content `Scroll` into the retained data-table `VirtualList`.
- Upstream table/list shape is checked before changing the row tree: shadcn keeps table cell
  chrome on the `td` boundary, while Base UI keeps scroll-area content as its own viewport/content
  part rather than per-row shell policy.
- The retained fixed-row hot path drops per-cell `Container` wrappers when grid lines and per-cell
  debug anchors are disabled, keeping the existing slower path for cell anchors, grid lines, and
  measured rows.
- Focused retained table and shadcn retained data-table gates pass.

Evidence:

- `ecosystem/fret-ui-kit/src/declarative/table.rs` now lets the fixed row `ManagedSurface` lay out
  cell children directly into padded rects on the hot no-cell-anchor path.
- `table_virtualized_retained_plain_fixed_rows_can_inline_cell_padding` asserts the row subtree has
  no per-cell `Container` wrappers while the cell renderer still mounts.
- Pre-slice retained repro:
  `target/fret-diag/retained-vlist-root-apply-nextowner-codex-20260621/1782065143290/bundle.schema2.json`
  reported `p95.us(total/layout)=4248/3408`, `layout.root apply=2678`, and `layout.nodes=382`.
- Post-slice retained repro:
  `target/fret-diag/retained-vlist-inline-cell-padding-codex-20260621/1782066104208/bundle.json`
  reported `p95.us(total/layout)=1983/1642`, `layout.root apply=1366`, and `layout.nodes=250`.
- Scroll profile deltas on the hot retained `VirtualList` child path:
  nodes performed `330 -> 198`, `Container` nodes `132 -> 0`, and child-layout time
  `1770us -> 667us`.

## M8 - Retained View-Cache Settle Contract

Status: Complete.

Done criteria:

- The retained-host reconcile test keeps every frame on one stable `view_cache` callsite so the
  cache root `GlobalElementId` does not change between warmup, scroll, and settle frames.
- The settle frame after retained membership refresh keeps the cache-root render count stable.
- The same settle frame performs zero clean child `layout_in` calls.
- Any runtime reuse-root marking experiment is removed unless a stable-identity repro still fails.

Evidence:

- `crates/fret-ui/src/declarative/tests/virtual_list/retained.rs` now uses a shared
  `build_cached_list` helper for the cache-root callsite in
  `retained_virtual_list_host_updates_window_without_rerendering_view_cache_root`.
- Focused gate passed:
  `cargo nextest run -p fret-ui retained_virtual_list_host_updates_window_without_rerendering_view_cache_root --no-fail-fast --no-capture`.
- Retained VirtualList group gate passed:
  `cargo nextest run -p fret-ui retained_virtual_list --no-fail-fast --no-capture`.
- Interpretation: the initial third-frame miss was a duplicated-callsite test artifact, not a
  proven runtime `ViewCache` or retained membership bug.
