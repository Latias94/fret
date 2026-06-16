# Retained VirtualList Root-Apply Perf Plan

Status: Active
Last updated: 2026-06-16

## Goal

Continue the heavy-component performance goal after the retained data-table row/cell work moved the
hot owner out of table-local structure and into retained `VirtualList` / layout-root application.
The target remains general-app density: shadcn-style data tables, long lists, and editor surfaces
should stay near a 120Hz frame budget without making every recipe hand-optimize retained internals.

## Current Assumptions

- Area: lane ownership
  - Assumption: `ui-table-row-cell-structural-churn-v1` reached its table-local boundary for the
    current retained data-table repro.
  - Evidence:
    `docs/workstreams/ui-table-row-cell-structural-churn-v1/EVIDENCE_AND_GATES.md`,
    `target/fret-diag/vlist-retained-row-key-hook-prune-v3-retained-only/1781536422863/bundle.json`,
    and
    `target/fret-diag/vlist-retained-post-key-hook-node-profile-v1/1781537495673/bundle.json`.
  - Confidence: Confident.
  - Consequence if wrong: we may spend time in `crates/fret-ui` before finishing a smaller
    `fret-ui-kit` table cleanup.

- Area: mechanism layer
  - Assumption: the next owner is a runtime mechanism issue, not a shadcn recipe issue.
  - Evidence: node profiling attributes the worst retained frame to `VirtualList`
    (`test_id=ui-gallery-data-table-torture-root`) and its parent `Scroll`, while shadcn correctness
    gates already pass on the retained table shape.
  - Confidence: Likely.
  - Consequence if wrong: the first mechanism diagnostic pass should reveal a recipe-specific
    wrapper owner and the workstream can split again.

- Area: ADR alignment
  - Assumption: ADR 0175 and ADR 0177 make retained window membership, attach/detach churn,
    keep-alive reuse, and window-shift diagnostics first-class runtime concerns.
  - Evidence: `docs/adr/0175-prepaint-windowed-virtual-surfaces.md` and
    `docs/adr/0177-retained-windowed-surface-hosts.md`.
  - Confidence: Confident.
  - Consequence if wrong: optimization could incorrectly push runtime obligations into each table,
    tree, or shadcn recipe.

- Area: first optimization target
  - Assumption: the first implementation candidate should reduce unnecessary retained
    `VirtualList` root apply / barrier relayout work before introducing a new fixed-track layout
    primitive.
  - Evidence: latest node profile shows `VirtualList self_us=7421 total_us=9073`, with total layout
    still dominated by `layout.root apply`.
  - Confidence: Likely.
  - Consequence if wrong: a fixed-track strip primitive may become the right first mechanism slice,
    but it should be justified by more granular layout-phase evidence.

## Baseline Evidence

- Retained table before shared row transform:
  `target/fret-diag/vlist-retained-filter-shrink-correct-script-v1/sessions/1781528832521-146560/1781528844457-ui-gallery-data-table-retained-filter-shrink-vlist-inputs-change/bundle.schema2.json`
  - `total=24856us`
  - `layout=23060us`
  - `layout.engine_solve=13231us`
  - `layout.root apply=20407us`
  - `layout.nodes=810`
- Retained table after shared row transform:
  `target/fret-diag/vlist-retained-shared-row-xform-v1/sessions/1781530321751-126564/1781531045060-ui-gallery-data-table-retained-filter-shrink-vlist-inputs-change/bundle.schema2.json`
  - `total=11715us`
  - `layout=10831us`
  - `layout.engine_solve=6599us`
  - `layout.root apply=9541us`
  - `layout.nodes=646`
- Retained table after row key-hook prune:
  `target/fret-diag/vlist-retained-row-key-hook-prune-v3-retained-only/1781536422863/bundle.json`
  - `total=11391us`
  - `layout=10522us`
  - `layout.engine_solve=6524us`
  - `layout.root apply=9373us`
  - `layout.nodes=646`
- Post-key-hook node profile:
  `target/fret-diag/vlist-retained-post-key-hook-node-profile-v1/1781537495673/bundle.json`
  - `total=13489us`
  - `layout=12062us`
  - `layout.engine_solve=7498us`
  - `layout.root apply=10508us`
  - `layout.nodes=646`
  - Top node: `VirtualList`, `test_id=ui-gallery-data-table-torture-root`,
    `self_us=7421`, `total_us=9073`

## Execution Plan

1. Start `docs/workstreams/retained-virtual-list-root-apply-v1/` as a narrow follow-on.
2. Re-run the retained data-table filter-shrink script with both layout node profiling and scroll
   layout profiling enabled.
3. Inspect `reconcile_retained_virtual_list_hosts`, `layout_virtual_list_impl`,
   `set_children_barrier`, and `solve_barrier_child_roots_if_needed` for repeated work that the
   retained host interface could hide.
4. Add a focused `crates/fret-ui` test before any mechanism optimization.
5. Land only reversible slices with retained VirtualList correctness gates and retained data-table
   perf evidence.

## Current Decision

The next optimization should not ask every shadcn/data-table caller to flatten away all component
structure. The material win already came from deleting a bad component shape (row-local horizontal
scroll owners), but the remaining owner sits behind the retained `VirtualList` interface. That
interface should gain more depth: callers should provide stable identity, fixed/known height, and
window policy, while the implementation concentrates root application, barrier solves, and
diagnostic attribution.

The current slice chose the smallest row/cell-depth deletion with a clear proof point:
replace pure test-id `Semantics` wrappers with semantics attached to existing nodes. This keeps the
table recipe behavior unchanged while still reducing wrapper breadth in the hot path.

## Progress Log

### 2026-06-16

- Re-ran the retained data-table repro with scroll layout profiling enabled:
  `target/fret-diag/retained-vlist-root-apply-scroll-profile-v1/1781539565855/bundle.schema2.json`
- The worst frame stayed layout-bound:
  `total=11560us`, `layout=10763us`, `layout.root apply=9535us`, `layout.engine_solve=6716us`.
- The hot scroll node was `ui-gallery-content-viewport` with `total_us=9374`,
  `solve_barrier_us=862`, `layout_children_first_pass_us=8451`, and
  `corrected_content_relayout=true`.
- The first-pass child profile still points at retained `VirtualList` as the main owner:
  `self_us=6675`, `total_us=8259`, with 625 child nodes performed.
- This repro is still using the gallery's default fixed-height table mode
  (`measure_rows=false` unless `FRET_UI_GALLERY_DATA_TABLE_VARIABLE_HEIGHT` is set), so the next
  optimization should target the fixed/known-height retained `VirtualList` path rather than a
  measured-row-only special case.
- The scroll shell is no longer the main question; the open question is whether the retained
  `VirtualList` layout path can skip enough child work on stable fixed-height windows to justify a
  narrower fast path or a new sub-lane.

### 2026-06-16 Follow-up

- Narrowed the single-root layout fast path in `crates/fret-ui` so a clean root no longer depends
  on global `invalidated_layout_nodes` / `invalidated_hit_test_nodes` counters before reusing its
  cached size.
- Added `layout_in_skips_clean_root_even_when_another_node_is_layout_dirty` to prove a clean root
  can skip layout-engine entry even when another node in the tree is dirty.
- Retained VirtualList correctness gates still pass, but the perf repro still points to the same
  first-pass child-layout owner. The root-local skip is a correctness-and-noise cleanup, not the
  main hotspot owner.
- Captured a fresh perf bundle for this slice:
  `target/fret-diag/retained-vlist-root-apply-m1-root-local-skip-v1/1781549017090/bundle.json`
  with `top_total_time_us=11278`.

### 2026-06-16 Owner Confirmation

- Re-read the latest bundle with full stats output and confirmed the remaining hot frame is still
  layout-bound rather than root-local bookkeeping.
- The strongest signal is `layout_children_first_pass_us=8300`, with the top child profile owned
  by `VirtualList` itself (`self_us=6551`, `total_us=8053`, `nodes=1`).
- Secondary contributors inside the same profile are the nested component kinds you would expect
  from a dense shadcn-style row tree: `Container`, `Flex`, `Semantics`, `Pressable`, and
  `ScrollContentTransform`. Their totals are materially smaller than the `VirtualList` owner.
- The scroll shell still pays a secondary `solve_barrier_us=819` and `corrected_content_relayout`
  cost, but that is not the primary budget consumer in this bundle.
- The current repro does not show a meaningful retained-host reconcile spike; the measured work is
  still concentrated in the first-pass child layout path.
- Next optimization target: keep the lane focused on `layout_virtual_list_impl` first-pass child
  layout and the narrow barrier follow-up around it. Do not widen back to generic root-apply
  cleanup unless a future bundle moves the owner again.

### 2026-06-16 Dirty-Subtree Confirmation

- A second pass over `target/fret-diag/retained-vlist-root-apply-m1-root-local-skip-v1/1781549017090/bundle.json`
  showed the hot `VirtualList` path is not just "doing work on clean nodes". The first-pass child
  root still reports `layout_child_max_subtree_dirty_count=625` and `layout_child_max_nodes_performed=625`.
- The hot frame also records `set_children_barrier_writes=1` and `barrier(set_children/scheduled/performed)=1/1/0`,
  which means we are still paying a real barrier write on the hot path, but the bigger cost remains
  the layout traversal underneath it.
- `retained_virtual_list_reconciles=0` in the hot frame, so this is not a simple retained-reconcile
  attach/detach burst. The expensive work is being carried by the retained `VirtualList` layout
  subtree itself.
- The immediate hypothesis is that the retained list is still feeding layout with a subtree whose
  dirty aggregation stays broad enough to defeat the clean translation fast path for most visible
  rows. The next useful slice is therefore likely to be a retained-host/layout contract change or
  a narrower barrier short-circuit, not another root-local clean-layout tweak.

### 2026-06-16 Cell-Semantics De-wrapper

- Removed pure test-id `Semantics` wrappers from the retained table cell hot path in
  `ecosystem/fret-ui-kit/src/declarative/table.rs`.
- The retained capability-first cell helper now attaches semantics directly to the existing text
  element instead of creating an extra layout node.
- The retained row cell hot path now uses `cell.test_id(...)` for the debug anchor, which should
  reduce wrapper breadth without changing visible table structure.
- Focused `fret-ui-kit` gates still pass:
  `table_virtualized_retained_accepts_capability_first_cell_renderer` and
  `table_virtualized_retained_header_debug_ids_click_sort_actions`.
- Next step: rerun the retained data-table perf repro and compare `layout_children_first_pass_us`
  and `layout_child_max_nodes_performed` against the current bundle.

### 2026-06-16 Current Hypothesis Update

- If the perf repro does not move meaningfully after the wrapper deletion, the row/cell tree is
  still the main cost and the next follow-on should own tree-depth reduction more aggressively.
- If the repro does move, we have evidence that a part of the cost was wrapper breadth rather than
  only the retained `VirtualList` mechanism, and the next slice should stay on that surface.

### 2026-06-16 Perf Repro Attempt

- A fresh retained data-table perf repro attempt was started for
  `retained-vlist-root-apply-m2-cell-semantic-dewrapper-v1`.
- The run failed before bundle emission because the diagnostics capability set reported
  `diag.pointer_kind_touch` as missing.
- No new `bundle.json` or `bundle.schema2.json` was produced, so there is not yet a before/after
  perf delta for the wrapper deletion.
- The code slice itself still landed and the focused `fret-ui-kit` gates passed.

### 2026-06-16 Script Repair

- The retained data-table repro script was adjusted to use a mouse wheel step instead of touch.
- This keeps the repro within the currently available filesystem capability set and allows the same
  performance comparison to run without requiring `diag.pointer_kind_touch`.

### 2026-06-16 Mouse-Wheel Perf Rerun

- The repaired retained repro completed successfully:
  `target/fret-diag/retained-vlist-root-apply-m2-cell-semantic-dewrapper-v2/1781578517352/bundle.schema2.json`
- `diag stats --sort cpu_cycles --top 30` reported `top_total_time_us=10607`, `layout=9882`,
  `layout.engine_solve=6516`, `layout.root apply=8912`, and `layout.nodes=514`.
- Compared with the prior retained row-key-prune and root-local-skip evidence
  (`11391` / `11278` top frame, `646` nodes), the de-wrapper plus script repair moved the surface in
  the intended direction.
- The remaining top frame is still layout/root-apply dominated, with retained `VirtualList` and the
  parent content `Scroll` as the relevant owners. Continue toward retained host/layout dirty
  propagation and barrier seams; do not re-expand the table wrapper cleanup unless a new profile
  proves table-local ownership again.

### 2026-06-16 Row Background Wrapper Prune

- Compared retained table row composition with shadcn/Base UI/ImGui references:
  shadcn delegates table structure to native DOM table primitives, Base UI virtualized examples
  delegate windowing to a virtualizer with estimated fixed row sizes, and ImGui dense tables rely on
  clipper/fixed row constraints instead of a deep generic widget tree.
- Applied a narrow Fret-side cleanup: retained table body rows now omit the row background
  container when there is no hover, pressed, or selected background to paint.
- Selected/active rows still keep the background wrapper, so full-row background geometry remains
  stable.
- Added structure gates for both shapes:
  `table_virtualized_retained_plain_rows_omit_background_wrapper` and
  `table_virtualized_retained_selected_rows_keep_background_wrapper`.
- Focused retained table gates passed:
  `table_virtualized_retained_plain_rows_omit_background_wrapper`,
  `table_virtualized_retained_selected_rows_keep_background_wrapper`,
  `table_virtualized_retained_fixed_rows_mount_as_clip_boundaries`,
  `table_virtualized_retained_measured_rows_do_not_force_row_clip`,
  `table_virtualized_retained_unpinned_body_uses_shared_horizontal_transform`,
  `table_virtualized_retained_nested_focus_bubbles_keyboard_to_list`,
  `table_virtualized_retained_header_debug_ids_click_sort_actions`, and
  `table_virtualized_retained_selected_semantics_follow_windowed_row_selection`.
- Fresh bundle:
  `target/fret-diag/retained-vlist-root-apply-m3-row-bg-wrapper-prune-v1/1781580973922/bundle.schema2.json`
- `diag stats --sort cpu_cycles --top 30` reported `top_total_time_us=10531`, `layout=9604`,
  `layout.engine_solve=6332`, `layout.root apply=8637`, and `layout.nodes=481`.
- This is a small but real breadth reduction over the m2 `10607` / `9882` / `514` shape. The
  important conclusion is negative: recipe-level wrapper pruning is not enough to reach the 120Hz
  target for dense shadcn-style tables. The next meaningful work should design a dense retained
  fixed-row/list primitive or a deeper fixed-height `VirtualList` child-layout fast path.
