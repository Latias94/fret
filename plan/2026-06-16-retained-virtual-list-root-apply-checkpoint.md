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
- Use the compact `plan/2026-06-16-retained-virtual-list-root-apply-checkpoint.md`
  note as the daily status sink; keep the longer history in
  `plan/retained-virtual-list-root-apply-perf.md`.

## Evidence

- `target/fret-diag/retained-vlist-root-apply-scroll-profile-v1/1781539565855/bundle.schema2.json`
- `target/fret-diag/retained-vlist-root-apply-m1-root-local-skip-v1/1781549017090/bundle.json`
- `target/fret-diag/vlist-retained-shared-row-xform-v1/sessions/1781530321751-126564/1781531045060-ui-gallery-data-table-retained-filter-shrink-vlist-inputs-change/bundle.schema2.json`
- `target/fret-diag/retained-vlist-root-apply-m2-cell-semantic-dewrapper-v2/1781578517352/bundle.schema2.json`

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
- `cargo nextest run --cargo-profile dev-fast -p fret-ui-kit table_virtualized_retained_unpinned_body_uses_shared_horizontal_transform table_virtualized_retained_nested_focus_bubbles_keyboard_to_list table_virtualized_retained_header_debug_ids_click_sort_actions --no-fail-fast --no-capture`
- `cargo nextest run --cargo-profile dev-fast -p fret-ui-shadcn --lib retained_data_table_header_debug_ids_sort_with_column_actions --no-fail-fast --no-capture`
- `python -m json.tool docs/workstreams/retained-virtual-list-root-apply-v1/WORKSTREAM.json`
- `python tools/check_workstream_catalog.py`
- `git diff --check`

## Next Check

The next slice should only land if it moves the retained `VirtualList` owner or narrows the
barrier/root-apply side in a measurable way. If the next bundle still points at the same child
traversal shape, split a narrower follow-on instead of broadening this lane.
