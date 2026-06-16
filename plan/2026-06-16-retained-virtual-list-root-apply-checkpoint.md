# Retained VirtualList Root Apply Checkpoint

Last updated: 2026-06-16

This note is a compact working checkpoint for the retained data-table performance lane.
The authoritative lane docs live in:

- `docs/workstreams/retained-virtual-list-root-apply-v1/WORKSTREAM.json`
- `docs/workstreams/retained-virtual-list-root-apply-v1/DESIGN.md`
- `docs/workstreams/retained-virtual-list-root-apply-v1/TODO.md`
- `docs/workstreams/retained-virtual-list-root-apply-v1/MILESTONES.md`
- `docs/workstreams/retained-virtual-list-root-apply-v1/EVIDENCE_AND_GATES.md`

## Current Read

- The current hotspot is still the fixed/known-height retained `VirtualList`
  first-pass child layout path.
- The root-local clean-layout fast path was useful noise reduction, but it did not
  move the main owner.
- `Scroll` and barrier work remain secondary contributors.
- The retained data-table surface already uses the fixed path by default, so the
  current owner is not explained by a missing `measure_rows` toggle.

## Latest Follow-up

- Fresh owner confirmation from `target/fret-diag/retained-vlist-root-apply-m1-root-local-skip-v1/1781549017090/bundle.json`
  still shows the hot frame as layout-bound rather than root-local bookkeeping.
- `layout_children_first_pass_us=8300`, and the top child profile is still retained
  `VirtualList` itself with `self_us=6551`, `total_us=8053`, `nodes=1`.
- The scroll shell remains secondary at `solve_barrier_us=819` with a small
  `corrected_content_relayout` cost.
- Code inspection of `crates/fret-ui/src/declarative/host_widget/layout/scrolling.rs`
  and `crates/fret-ui/src/declarative/mount.rs` shows the fixed retained path still
  walks every visible child and only skips measurement work. That makes the current
  hotspot a traversal / subtree-depth problem, not a measurement-mode problem.
- The next useful split is therefore still `layout_virtual_list_impl` first-pass child
  layout vs. the narrower barrier follow-up, not generic root-apply cleanup.

## Latest De-wrapper

- A pure test-id `Semantics` wrapper was removed from the retained table cell hot path in
  `ecosystem/fret-ui-kit/src/declarative/table.rs`.
- The retained capability-first cell helper now uses `attach_semantics` on the existing text node.
- The retained hot-path cell wrapper now uses `cell.test_id(...)` instead of wrapping the cell in a
  standalone semantics node.
- Focused `fret-ui-kit` tests still pass after the change.
- A perf repro attempt for this slice failed before bundle emission because
  `diag.pointer_kind_touch` was missing from the filesystem capability set.
- That cleanup is intentionally structural and should be treated as supporting evidence,
  not the main perf claim.

## Latest Perf Rerun

- The retained data-table repro script now uses the mouse-wheel path instead of touch, so it can run
  under the current filesystem diagnostics capability set.
- Fresh evidence:
  `target/fret-diag/retained-vlist-root-apply-m2-cell-semantic-dewrapper-v2/1781578517352/bundle.schema2.json`
- `diag stats --sort cpu_cycles --top 30` reported:
  - `top_total_time_us=10607`
  - `top_layout_time_us=9882`
  - `top_layout_engine_solve_time_us=6516`
  - `layout.root apply=8912`
  - `layout.nodes=514`
- Interpretation: the de-wrapper plus script repair is a valid small win over the prior
  `11278` / `11391` retained bundles and lowered node breadth from the previous 646-node shape.
  The owner still has not moved back to broad table-local code; retained `VirtualList` and parent
  `Scroll` remain the next optimization seam.

## Latest Row Background Wrapper Prune

- Retained table body rows now skip the row background container when the row has no hover,
  pressed, or selected background.
- Rows that need a full-row background still keep the wrapper, preserving selected/active row
  geometry and paint semantics.
- New structure gates:
  - `table_virtualized_retained_plain_rows_omit_background_wrapper`
  - `table_virtualized_retained_selected_rows_keep_background_wrapper`
- Focused retained table gates still pass after the change.
- Fresh evidence:
  `target/fret-diag/retained-vlist-root-apply-m3-row-bg-wrapper-prune-v1/1781580973922/bundle.schema2.json`
- `diag stats --sort cpu_cycles --top 30` reported:
  - `top_total_time_us=10531`
  - `top_layout_time_us=9604`
  - `top_layout_engine_solve_time_us=6332`
  - `layout.root apply=8637`
  - `layout.nodes=481`
- Interpretation: this slice is a valid narrow breadth reduction over the m2
  `10607` / `9882` / `514` shape, but the movement is small. The remaining performance problem is
  not explained by one more retained table wrapper. The next real architecture seam is still a
  dense retained list/table layout contract or a deeper `VirtualList` fixed-height child-layout
  fast path.

## Decisions

- Keep the next optimization focused on retained `VirtualList` child layout and the
  narrow barrier follow-up around it.
- Do not widen this lane back to generic root-apply cleanup unless a fresh bundle
  moves the owner again.
- Treat measured-row variability as a separate comparison unless profiling makes it
  the next real owner.
- Before the next slice, compare the retained row subtree shape against upstream
  `repo-ref/shadcn` and `repo-ref/base-ui` references so any flattening decision is
  evidence-led rather than style-led.
- Treat row/cell wrapper deletion as supporting cleanup, not the long-term answer for ImGui-grade
  dense surfaces. shadcn/Base UI rely on native DOM/table or virtualizer primitives, and ImGui
  relies on clipper/fixed row constraints; Fret needs an equivalent dense primitive rather than
  asking every component recipe to pay for a deep generic layout tree.
- Use the compact `plan/2026-06-16-retained-virtual-list-root-apply-checkpoint.md`
  note as the daily status sink; keep the longer history in
  `plan/retained-virtual-list-root-apply-perf.md`.

## Evidence

- `target/fret-diag/retained-vlist-root-apply-scroll-profile-v1/1781539565855/bundle.schema2.json`
- `target/fret-diag/retained-vlist-root-apply-m1-root-local-skip-v1/1781549017090/bundle.json`
- `target/fret-diag/vlist-retained-shared-row-xform-v1/sessions/1781530321751-126564/1781531045060-ui-gallery-data-table-retained-filter-shrink-vlist-inputs-change/bundle.schema2.json`
- `target/fret-diag/retained-vlist-root-apply-m2-cell-semantic-dewrapper-v2/1781578517352/bundle.schema2.json`
- `target/fret-diag/retained-vlist-root-apply-m3-row-bg-wrapper-prune-v1/1781580973922/bundle.schema2.json`

## Repro

Use the retained data-table filter-shrink script with retained mode plus layout and scroll
profiling enabled:

```powershell
target\release\fretboard-dev.exe diag perf tools/diag-scripts/ui-gallery/data-table/ui-gallery-data-table-retained-filter-shrink-vlist-inputs-change.json `
  --repeat 1 `
  --warmup-frames 5 `
  --dir target\fret-diag\retained-vlist-root-apply-m0-scroll-profile-v1 `
  --env FRET_UI_GALLERY_DATA_TABLE_RETAINED=1 `
  --env FRET_LAYOUT_NODE_PROFILE=1 `
  --env FRET_LAYOUT_NODE_PROFILE_TOP=30 `
  --env FRET_LAYOUT_NODE_PROFILE_MIN_US=80 `
  --env FRET_SCROLL_LAYOUT_PROFILE=1 `
  --env FRET_SCROLL_LAYOUT_PROFILE_MIN_US=0 `
  --env FRET_SCROLL_LAYOUT_PROFILE_MIN_MEASURE_US=0 `
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 `
  --sort cpu_cycles `
  --top 15 `
  --json `
  --launch -- cargo run -p fret-ui-gallery --release --features gallery-dev,gallery-ai,gallery-chart,gallery-web-ime-harness
```

## Gates

- `cargo nextest run --cargo-profile dev-fast -p fret-ui retained_virtual_list --no-fail-fast --no-capture`
- `cargo nextest run --cargo-profile dev-fast -p fret-ui-kit table_virtualized_retained_plain_rows_omit_background_wrapper table_virtualized_retained_selected_rows_keep_background_wrapper table_virtualized_retained_fixed_rows_mount_as_clip_boundaries table_virtualized_retained_measured_rows_do_not_force_row_clip table_virtualized_retained_unpinned_body_uses_shared_horizontal_transform table_virtualized_retained_nested_focus_bubbles_keyboard_to_list table_virtualized_retained_header_debug_ids_click_sort_actions table_virtualized_retained_selected_semantics_follow_windowed_row_selection --no-fail-fast --no-capture`
- `cargo nextest run --cargo-profile dev-fast -p fret-ui-shadcn --lib retained_data_table_header_debug_ids_sort_with_column_actions --no-fail-fast --no-capture`
- `python -m json.tool docs/workstreams/retained-virtual-list-root-apply-v1/WORKSTREAM.json`
- `python tools/check_workstream_catalog.py`
- `git diff --check`

## Next Check

The next slice should only land if it moves the retained `VirtualList` owner or narrows the
barrier/root-apply side in a measurable way. If the next bundle still points at the same child
traversal shape, split a narrower follow-on instead of broadening this lane.

## Latest Scroll Telemetry Read

- Fresh `m4` evidence from
  `target/fret-diag/retained-vlist-root-apply-m4-scroll-roots-v2/1781584457222/bundle.schema2.json`
  shows the hot retained `VirtualList` child path is still one dirty subtree with deep performed
  breadth, not a partially skippable mix of clean and dirty roots.
- The hottest scroll node reported
  `layout_child_first_pass_roots=1`,
  `layout_child_first_pass_layout_invalidated_roots=1`,
  `layout_child_first_pass_subtree_dirty_roots=1`,
  `layout_child_first_pass_performed_roots=1`,
  `layout_child_first_pass_skipped_roots=0`,
  `layout_child_first_pass_bounds_changed_roots=0`,
  `layout_child_first_pass_input_mismatch_roots=0`, and
  `layout_child_first_pass_input_size_mismatch_roots=0`.
- The same snapshot also reported
  `layout_child_first_pass_nodes_visited=473`,
  `layout_child_first_pass_nodes_performed=460`, and
  `layout_child_max_subtree_dirty_count=460`.
- A secondary scroll node showed a broad subtree too, but it was not the main owner:
  `layout_child_first_pass_roots=33`,
  `layout_child_first_pass_layout_invalidated_roots=33`,
  `layout_child_first_pass_subtree_dirty_roots=33`,
  `layout_child_first_pass_performed_roots=33`,
  `layout_child_first_pass_skipped_roots=0`, with
  `layout_child_first_pass_nodes_visited=429` and
  `layout_child_first_pass_nodes_performed=429`.
- Interpretation: the telemetry expansion removed ambiguity, but it does not expose an obvious
  clean-root skip win. The next meaningful optimization should stay on mechanism depth or a tighter
  barrier/root-apply contract.

## Latest Body Hoist Read

- Current release binaries were rebuilt before this measurement, so the bundle reflects the latest
  body-hoist code.
- The single-center retained table body now owns the shared horizontal transform at the body
  wrapper.
- Focused gate `table_virtualized_retained_unpinned_body_uses_shared_horizontal_transform` passes.
- Fresh perf bundle:
  `target/fret-diag/1781592842180/bundle.json`.
- `diag stats --sort cpu_cycles --top 30` reported:
  - `top_total_time_us=10130`
  - `top_layout_time_us=9468`
  - `top_layout_engine_solve_time_us=6435`
  - `layout.root apply=8595`
  - `layout.nodes=417`
- Interpretation: the body-hoist slice is a real improvement, but the remaining owner is still
  retained `VirtualList` plus the parent `Scroll`. The next slice should target a deeper
  fixed-height list/table mechanism or barrier propagation, not more row-wrapper cleanup.

## Latest Cell-Anchor Toggle Read

- The heavy retained data-table torture preview now disables per-cell debug anchors through
  `TableDebugIds::row_cell_test_ids = false`, while preserving row anchors.
- Fresh perf bundle:
  `target/fret-diag/1781594910783/bundle.schema2.json`.
- `diag stats --sort cpu_cycles --top 30` reported:
  - `top_total_time_us=9965`
  - `top_layout_time_us=9328`
  - `top_layout_engine_solve_time_us=6595`
  - `layout.root apply=8546`
  - `layout.nodes=417`

## Latest Fixed-Track Clean-Root Skip

- Added a `VirtualList` child-root filter that reuses the tree's clean-root layout skip predicate
  before solving barrier child roots or calling `layout_in`.
- The implementation keeps full `barrier_roots` for virtual-list bookkeeping and diagnostics, but
  uses a scratch `roots_needing_layout` list for actual solve/layout work.
- Scroll child-layout telemetry now records clean skipped roots even when `VirtualList` skips before
  entering `layout_in`.
- New gate:
  `retained_fixed_virtual_list_skips_clean_child_layout_in_on_steady_frame`.
- Verified:
  - `cargo fmt -p fret-ui --check`
  - `cargo check -p fret-ui --lib`
  - `cargo nextest run -p fret-ui retained_fixed_virtual_list_skips_clean_child_layout_in_on_steady_frame`
  - `cargo nextest run -p fret-ui 'declarative::tests::virtual_list::retained'`
- Interpretation: this removes avoidable child layout work on clean fixed retained windows. It does
  not yet prove that the retained data-table filter-shrink hot frame will move, because the latest
  bundle showed a genuinely dirty `VirtualList` subtree. The next step is a perf rerun; if that
  remains dirty-subtree dominated, the lane should deepen the fixed-track/dense retained table
  contract.

## Latest Clean-Root Skip Perf Rerun

- Fresh perf bundle:
  `target/fret-diag/retained-vlist-root-apply-m5-clean-root-skip-v1/1781600101441/bundle.schema2.json`
- `diag stats --sort cpu_cycles --top 10` reported:
  - `top_total_time_us=9278`
  - `top_layout_time_us=8669`
  - `top_layout_engine_solve_time_us=5977`
  - `layout.root apply=7897`
  - `layout.nodes=417`
- Interpretation: this is a real but bounded improvement over the prior cell-anchor toggle bundle
  (`9965` / `9328` / `6595` / `8546` / `417`). It confirms the clean-root filter is worth keeping,
  but ownership still stays with retained `VirtualList` plus parent `Scroll`. The next slice should
  deepen the fixed-height/fixed-track retained list/table contract instead of pruning another
  generic wrapper.
