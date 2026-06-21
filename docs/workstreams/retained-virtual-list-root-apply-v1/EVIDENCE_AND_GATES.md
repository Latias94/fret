# Evidence And Gates: Retained VirtualList Root Apply v1

Status: Active
Last updated: 2026-06-21

## Baseline Evidence

Retained data-table shared row transform:

```powershell
target\release\fretboard-dev.exe diag stats target\fret-diag\vlist-retained-shared-row-xform-v1\sessions\1781530321751-126564\1781531045060-ui-gallery-data-table-retained-filter-shrink-vlist-inputs-change\bundle.schema2.json --sort cpu_cycles --top 30
```

Stats:

- `total=11715us`
- `layout=10831us`
- `layout.engine_solve=6599us`
- `layout.root apply=9541us`
- `layout.nodes=646`

Retained row key-hook prune:

```powershell
target\release\fretboard-dev.exe diag stats target\fret-diag\vlist-retained-row-key-hook-prune-v3-retained-only\1781536422863\bundle.json --sort cpu_cycles --top 10
```

Stats:

- `total=11391us`
- `layout=10522us`
- `layout.engine_solve=6524us`
- `layout.root apply=9373us`
- `layout.nodes=646`

Post-key-hook node profile:

```powershell
target\release\fretboard-dev.exe diag stats target\fret-diag\vlist-retained-post-key-hook-node-profile-v1\1781537495673\bundle.json --sort cpu_cycles --top 30
```

Stats:

- `total=13489us`
- `layout=12062us`
- `layout.engine_solve=7498us`
- `layout.root apply=10508us`
- `layout.nodes=646`
- Top node: retained `VirtualList`, `test_id=ui-gallery-data-table-torture-root`,
  `self_us=7421`, `total_us=9073`.
- Parent owner: content `Scroll`, `self_us=1114`, `total_us=10404`.

Scroll layout profile:

```powershell
target\release\fretboard-dev.exe diag stats target\fret-diag\retained-vlist-root-apply-scroll-profile-v1\1781539565855\bundle.schema2.json --sort cpu_cycles --top 30
```

Stats:

- `total=11560us`
- `layout=10763us`
- `layout.engine_solve=6716us`
- `layout.root apply=9535us`
- Hot scroll node: `ui-gallery-content-viewport`, `total_us=9374`
- `solve_barrier_us=862`
- `layout_children_first_pass_us=8451`
- `layout_children_corrected_content_us=13`
- `corrected_content_relayout=true`
- Hot child owner inside the first-pass layout profile: retained `VirtualList`,
  `self_us=6675`, `total_us=8259`, `nodes_performed=625`

Interpretation:

- The retained table component shape no longer owns the main hotspot.
- `Scroll` still owns root-apply and barrier solve work, but the dominant per-child cost is the
  retained `VirtualList` subtree itself.
- The next evidence loop should split retained `VirtualList` first-pass child layout from any
  narrower root-apply or barrier follow-up before landing another optimization.

Code inspection follow-up:

- `layout_virtual_list_impl` keeps the fixed/known-height path shallow only in one respect: it
  skips measurement work. It still walks every visible child and submits first-pass layout for the
  window.
- The retained data-table surface already uses that fixed path by default, so the 625-node breadth
  in the hot frame is not explained by a missing `measure_rows` toggle.
- That shifts the next question back to tree depth: whether the row/cell subtree should be
  flattened further, or whether `VirtualList` should expose a deeper table/list-specific seam.
- A follow-up slice removed pure test-id `Semantics` wrappers from retained table cell rendering,
  and the corresponding focused `fret-ui-kit` gates still pass.

## First Repro

Use the retained data-table script and enable node-level plus scroll layout profiling:

```powershell
target\release\fretboard-dev.exe diag perf tools\diag-scripts\ui-gallery\data-table\ui-gallery-data-table-retained-filter-shrink-vlist-inputs-change.json `
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

Then inspect:

```powershell
target\release\fretboard-dev.exe diag stats <bundle.json> --sort cpu_cycles --top 30
```

Follow-up check for the current owner:

- Re-run the same repro with `FRET_UI_GALLERY_DATA_TABLE_VARIABLE_HEIGHT=1` only when you want
  to compare measured-row behavior against the fixed-height path.
- Otherwise keep the default fixed-height mode so the next slice measures the hot retained
  `VirtualList` path that the current bundle actually owns.

## Correctness Gates

Focused retained VirtualList gates:

```powershell
cargo nextest run --cargo-profile dev-fast -p fret-ui retained_virtual_list --no-fail-fast --no-capture
```

Retained table integration gates:

```powershell
cargo nextest run --cargo-profile dev-fast -p fret-ui-kit table_virtualized_retained_unpinned_body_uses_shared_horizontal_transform table_virtualized_retained_nested_focus_bubbles_keyboard_to_list table_virtualized_retained_header_debug_ids_click_sort_actions --no-fail-fast --no-capture
cargo nextest run --cargo-profile dev-fast -p fret-ui-shadcn --lib retained_data_table_header_debug_ids_sort_with_column_actions --no-fail-fast --no-capture
```

## Mechanism And Boundary Gates

Run these when code changes touch retained layout or crate boundaries:

```powershell
cargo check --cargo-profile dev-fast -p fret-ui --all-targets
python tools\check_layering.py
```

## Slice Evidence

Root-local layout fast path slice:

```powershell
cargo nextest run --cargo-profile dev-fast -p fret-ui layout_in_skips_clean_root_even_when_another_node_is_layout_dirty view_cache_contained_relayout_does_not_force_next_frame_rerender view_cache_layout_dirty_expansion_reaches_clean_nested_cache_root_descendants --no-fail-fast --no-capture
cargo nextest run --cargo-profile dev-fast -p fret-ui retained_virtual_list --no-fail-fast --no-capture
```

Perf repro after the slice:

```powershell
target\release\fretboard-dev.exe diag perf tools\diag-scripts\ui-gallery\data-table\ui-gallery-data-table-retained-filter-shrink-vlist-inputs-change.json --repeat 1 --warmup-frames 5 --dir target\fret-diag\retained-vlist-root-apply-m1-root-local-skip-v1 --env FRET_UI_GALLERY_DATA_TABLE_RETAINED=1 --env FRET_LAYOUT_NODE_PROFILE=1 --env FRET_LAYOUT_NODE_PROFILE_TOP=30 --env FRET_LAYOUT_NODE_PROFILE_MIN_US=80 --env FRET_SCROLL_LAYOUT_PROFILE=1 --env FRET_SCROLL_LAYOUT_PROFILE_MIN_US=0 --env FRET_SCROLL_LAYOUT_PROFILE_MIN_MEASURE_US=0 --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --sort cpu_cycles --top 15 --json --launch -- cargo run -p fret-ui-gallery --release --features gallery-dev,gallery-ai,gallery-chart,gallery-web-ime-harness
```

Observed bundle:

- `target/fret-diag/retained-vlist-root-apply-m1-root-local-skip-v1/1781549017090/bundle.json`
- `top_total_time_us=11278`
- `top_layout_time_us=10489`
- `top_layout_engine_solve_time_us=6647`
- `layout_children_first_pass_us=4262`

Interpretation:

- This slice fixes a root-local fast-path over-conservatism and removes a source of unnecessary
  layout-engine entry.
- It does not move the main retained `VirtualList` first-pass child-layout owner.
- The next runtime slice should still target the retained `VirtualList` child path or a narrower
  barrier solve follow-up, depending on the next bundle.
- The first perf attempt for the wrapper-deletion slice failed at diagnostics capability admission
  because the script used `diag.pointer_kind_touch`, which is not exposed by the current filesystem
  capability set. The script was then moved to the mouse-wheel path and rerun successfully.

Cell semantics de-wrapper slice:

```powershell
cargo nextest run --cargo-profile dev-fast -p fret-ui-kit table_virtualized_retained_accepts_capability_first_cell_renderer table_virtualized_retained_header_debug_ids_click_sort_actions --no-fail-fast --no-capture
```

Observed gate:

- `table_virtualized_retained_accepts_capability_first_cell_renderer` passed.
- `table_virtualized_retained_header_debug_ids_click_sort_actions` passed.

Follow-up perf repro:

```powershell
target\release\fretboard-dev.exe diag perf tools\diag-scripts\ui-gallery\data-table\ui-gallery-data-table-retained-filter-shrink-vlist-inputs-change.json --repeat 1 --warmup-frames 5 --dir target\fret-diag\retained-vlist-root-apply-m2-cell-semantic-dewrapper-v1 --env FRET_UI_GALLERY_DATA_TABLE_RETAINED=1 --env FRET_LAYOUT_NODE_PROFILE=1 --env FRET_LAYOUT_NODE_PROFILE_TOP=30 --env FRET_LAYOUT_NODE_PROFILE_MIN_US=80 --env FRET_SCROLL_LAYOUT_PROFILE=1 --env FRET_SCROLL_LAYOUT_PROFILE_MIN_US=0 --env FRET_SCROLL_LAYOUT_PROFILE_MIN_MEASURE_US=0 --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --sort cpu_cycles --top 15 --json --launch -- cargo run -p fret-ui-gallery --release --features gallery-dev,gallery-ai,gallery-chart,gallery-web-ime-harness
```

Mouse-wheel rerun:

```powershell
target\release\fretboard-dev.exe diag perf tools\diag-scripts\ui-gallery\data-table\ui-gallery-data-table-retained-filter-shrink-vlist-inputs-change.json --repeat 1 --warmup-frames 5 --dir target\fret-diag\retained-vlist-root-apply-m2-cell-semantic-dewrapper-v2 --env FRET_UI_GALLERY_DATA_TABLE_RETAINED=1 --env FRET_LAYOUT_NODE_PROFILE=1 --env FRET_LAYOUT_NODE_PROFILE_TOP=30 --env FRET_LAYOUT_NODE_PROFILE_MIN_US=80 --env FRET_SCROLL_LAYOUT_PROFILE=1 --env FRET_SCROLL_LAYOUT_PROFILE_MIN_US=0 --env FRET_SCROLL_LAYOUT_PROFILE_MIN_MEASURE_US=0 --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --sort cpu_cycles --top 15 --json --launch -- cargo run -p fret-ui-gallery --release --features gallery-dev,gallery-ai,gallery-chart,gallery-web-ime-harness
```

Observed bundle:

- `target/fret-diag/retained-vlist-root-apply-m2-cell-semantic-dewrapper-v2/1781578517352/bundle.schema2.json`
- `top_total_time_us=10607`
- `top_layout_time_us=9882`
- `top_layout_engine_solve_time_us=6516`
- `layout.root apply=8912`
- `layout.nodes=514`

Interpretation:

- The de-wrapper plus repaired script produced a valid retained data-table comparison.
- The slice reduced retained layout breadth compared with the prior `646`-node bundles, and the
  top frame improved relative to the `11278` root-local-skip and `11391` row-key-prune retained
  runs.
- Ownership did not move back to table-local recipe code. `diag stats` still shows the top frame as
  layout/root-apply dominated, with retained `VirtualList` and the parent `Scroll` as the relevant
  owners.

Row background wrapper prune slice:

```powershell
cargo nextest run --cargo-profile dev-fast -p fret-ui-kit table_virtualized_retained_plain_rows_omit_background_wrapper table_virtualized_retained_selected_rows_keep_background_wrapper table_virtualized_retained_fixed_rows_mount_as_clip_boundaries table_virtualized_retained_measured_rows_do_not_force_row_clip table_virtualized_retained_unpinned_body_uses_shared_horizontal_transform table_virtualized_retained_nested_focus_bubbles_keyboard_to_list table_virtualized_retained_header_debug_ids_click_sort_actions table_virtualized_retained_selected_semantics_follow_windowed_row_selection --no-fail-fast --no-capture
```

Observed gate:

- All 8 focused retained table tests passed.
- Plain retained rows are gated to connect `Pressable -> ScrollContentTransform` directly.
- Selected retained rows are gated to keep `Pressable -> Container -> ScrollContentTransform`, so
  active-row background geometry remains stable.

Perf repro:

```powershell
target\release\fretboard-dev.exe diag perf tools\diag-scripts\ui-gallery\data-table\ui-gallery-data-table-retained-filter-shrink-vlist-inputs-change.json --repeat 1 --warmup-frames 5 --dir target\fret-diag\retained-vlist-root-apply-m3-row-bg-wrapper-prune-v1 --env FRET_UI_GALLERY_DATA_TABLE_RETAINED=1 --env FRET_LAYOUT_NODE_PROFILE=1 --env FRET_LAYOUT_NODE_PROFILE_TOP=30 --env FRET_LAYOUT_NODE_PROFILE_MIN_US=80 --env FRET_SCROLL_LAYOUT_PROFILE=1 --env FRET_SCROLL_LAYOUT_PROFILE_MIN_US=0 --env FRET_SCROLL_LAYOUT_PROFILE_MIN_MEASURE_US=0 --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --sort cpu_cycles --top 15 --json --launch -- cargo run -p fret-ui-gallery --release --features gallery-dev,gallery-ai,gallery-chart,gallery-web-ime-harness
```

Observed bundle:

- `target/fret-diag/retained-vlist-root-apply-m3-row-bg-wrapper-prune-v1/1781580973922/bundle.schema2.json`
- `top_total_time_us=10531`
- `top_layout_time_us=9604`
- `top_layout_engine_solve_time_us=6332`
- `layout.root apply=8637`
- `layout.nodes=481`

Interpretation:

- This is a valid row-subtree breadth reduction over the m2 `10607` / `9882` / `514` shape.
- The gain is intentionally small and confirms the same architectural conclusion: retained
  data-table still needs a denser fixed-height list/table mechanism instead of relying only on
  recipe-level wrapper deletion.

Scroll telemetry expansion:

- The retained `VirtualList` scroll profile now carries child-root splits and root-state counters
  through `fret-ui`, `fret-bootstrap`, and `fret-diag`.
- Fresh evidence:
  `target/fret-diag/retained-vlist-root-apply-m4-scroll-roots-v2/1781584457222/bundle.schema2.json`
- The hottest scroll node reported:
  - `layout_child_first_pass_roots=1`
  - `layout_child_first_pass_layout_invalidated_roots=1`
  - `layout_child_first_pass_subtree_dirty_roots=1`
  - `layout_child_first_pass_performed_roots=1`
  - `layout_child_first_pass_skipped_roots=0`
  - `layout_child_first_pass_nodes_visited=473`
  - `layout_child_first_pass_nodes_performed=460`
  - `layout_child_max_subtree_dirty_count=460`
- A secondary scroll node reported a broad but less interesting subtree:
  - `layout_child_first_pass_roots=33`
  - `layout_child_first_pass_layout_invalidated_roots=33`
  - `layout_child_first_pass_subtree_dirty_roots=33`
  - `layout_child_first_pass_performed_roots=33`
  - `layout_child_first_pass_skipped_roots=0`
  - `layout_child_first_pass_nodes_visited=429`
  - `layout_child_first_pass_nodes_performed=429`
- Interpretation: the telemetry expansion was worthwhile because it removes ambiguity, but it does
  not expose a clean-root fast path worth pursuing first. The main retained `VirtualList` child
  path still looks like a deep dirty subtree, so the next mechanism slice should keep aiming at
  deeper fixed-height layout primitives or tighter barrier propagation, not more root skipping.

## 2026-06-16 Cell-Anchor Toggle Follow-up

- The heavy retained data-table torture preview now disables per-cell debug anchors via
  `TableDebugIds::row_cell_test_ids = false`, while preserving row anchors for automation.
- Rebuilt release binaries before remeasuring so the bundle reflects the latest preview change.
- Fresh perf bundle:
  `target/fret-diag/1781594910783/bundle.schema2.json`.
- `diag stats --sort cpu_cycles --top 30` reported:
  - `top_total_time_us=9965`
  - `top_layout_time_us=9328`
  - `top_layout_engine_solve_time_us=6595`
  - `layout.root apply=8546`
  - `layout.nodes=417`
- Interpretation: this is a small harness-noise reduction, not an owner change. The hotspot still
  lives in retained `VirtualList` plus the parent `Scroll`, so this evidence does not justify
  widening the lane back into table-local wrapper cleanup.

## 2026-06-16 Retained Body Hoist

- Rebuilt the release binaries before remeasuring, so this bundle reflects the latest body-hoist
  code.
- The single-center retained table body now owns the shared horizontal transform at the body
  wrapper.
- Focused gate
  `table_virtualized_retained_unpinned_body_uses_shared_horizontal_transform`
  passes and directly asserts the body shell.
- Fresh perf bundle:
  `target/fret-diag/1781592842180/bundle.json`.
- `diag stats --sort cpu_cycles --top 30` reported:
  - `top_total_time_us=10130`
  - `top_layout_time_us=9468`
  - `top_layout_engine_solve_time_us=6435`
  - `layout.root apply=8595`
  - `layout.nodes=417`
- Interpretation: this is a measurable improvement over the prior row-background wrapper prune
  bundle and the stale first-pass body-hoist run, but retained `VirtualList` plus the parent
  `Scroll` still own the hotspot. The next slice should target a deeper fixed-height list/table
  mechanism or barrier/root contract, not another small row-wrapper cleanup.

## 2026-06-21 Root Apply Owner Attribution

```bash
cargo nextest run -p fret-ui clean_geometry_window_root_resize_consumes_apply_plan_without_root_layout --no-fail-fast
cargo nextest run -p fret-diag triage_includes_hints_and_unit_costs_for_worst_frame --no-fail-fast
cargo nextest run -p fret-diag layout_perf_summary --no-fail-fast
cargo check -p fret-bootstrap --lib
cargo run -p fretboard-dev -- diag stats target/fret-diag/inspector-direct-entry-retained-noop-skip-codex-20260621/1782061219853/bundle.schema2.json --sort cpu_cycles --top 3
```

Observed gates:

- The focused `fret-ui` clean-geometry root apply test passed and asserts
  `mode=clean_geometry_plan` with `nodes_performed=0`.
- The focused `fret-diag` triage fixture passed and now carries
  `worst.layout_root_applies[]`.
- The `layout_perf_summary` focused suite passed and now clips `layout_root_applies` alongside the
  other layout attribution arrays.
- `cargo check -p fret-bootstrap --lib` passed for the debug snapshot schema wiring.
- The old retained noop-skip bundle still parses through `diag stats`; it predates
  `debug.layout_root_applies[]`, so it is compatibility evidence, not new owner evidence.

Interpretation:

- This slice does not change layout behavior. It makes the aggregate
  `layout_roots_apply_time_us` phase attributable to the top root apply records.
- New snapshots expose `debug.layout_root_applies[]`; `fret-diag` surfaces that as
  `layout_root_applies` in stats JSON, human `diag stats` detail rows, triage JSON, and
  `layout_perf_summary`.
- The next optimization slice should first rerun the retained data-table repro and inspect
  `layout_root_applies` to decide whether the root owner is still retained `VirtualList` plus
  parent `Scroll`, or whether ownership has moved to a narrower follow-on.

## 2026-06-21 Retained Fixed Row Inline Cell Padding

```bash
cargo nextest run --cargo-profile dev-fast -p fret-ui-kit table_virtualized_retained_plain_fixed_rows_can_inline_cell_padding table_virtualized_retained_plain_rows_omit_background_wrapper table_virtualized_retained_selected_rows_keep_background_wrapper table_virtualized_retained_fixed_rows_mount_as_clip_boundaries --no-fail-fast --no-capture
cargo nextest run --cargo-profile dev-fast -p fret-ui-kit table_virtualized_retained_unpinned_body_uses_shared_horizontal_transform table_virtualized_retained_nested_focus_bubbles_keyboard_to_list table_virtualized_retained_header_debug_ids_click_sort_actions table_virtualized_retained_selected_semantics_follow_windowed_row_selection table_virtualized_retained_colpin_alignment_gate_across_pin_resize_and_overflow table_virtualized_retained_colpin_alignment_gate_measured_rows_do_not_shrink_width --no-fail-fast --no-capture
cargo nextest run --cargo-profile dev-fast -p fret-ui-shadcn --lib retained_data_table_header_debug_ids_sort_with_column_actions --no-fail-fast --no-capture
target/release/fretboard-dev diag perf tools/diag-scripts/ui-gallery/data-table/ui-gallery-data-table-retained-filter-shrink-vlist-inputs-change.json --repeat 1 --warmup-frames 5 --dir target/fret-diag/retained-vlist-inline-cell-padding-codex-20260621 --env FRET_UI_GALLERY_DATA_TABLE_RETAINED=1 --env FRET_LAYOUT_NODE_PROFILE=1 --env FRET_LAYOUT_NODE_PROFILE_TOP=30 --env FRET_LAYOUT_NODE_PROFILE_MIN_US=80 --env FRET_SCROLL_LAYOUT_PROFILE=1 --env FRET_SCROLL_LAYOUT_PROFILE_MIN_US=0 --env FRET_SCROLL_LAYOUT_PROFILE_MIN_MEASURE_US=0 --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --sort cpu_cycles --top 15 --json --launch -- cargo run -p fret-ui-gallery --release --features gallery-dev,gallery-ai,gallery-chart,gallery-web-ime-harness
target/release/fretboard-dev diag stats target/fret-diag/retained-vlist-inline-cell-padding-codex-20260621/1782066104208/bundle.json --sort cpu_cycles --top 30
```

Observed gates:

- Focused row-structure tests passed, including the new direct cell-padding assertion.
- Retained table integration gates passed for horizontal transform alignment, nested focus
  bubbling, header sort actions, selected-row semantics, and pinned/measured-row alignment.
- The shadcn retained data-table header sort gate passed.
- The release retained data-table repro completed and wrote
  `target/fret-diag/retained-vlist-inline-cell-padding-codex-20260621/1782066104208/bundle.json`.

Stats:

- Before:
  `target/fret-diag/retained-vlist-root-apply-nextowner-codex-20260621/1782065143290/bundle.schema2.json`
  reported `p95.us(total/layout/prepaint/paint)=4248/3408/293/575`,
  `layout.root apply=2678`, `layout.nodes=382`, and the retained `VirtualList` child path had
  `layout_children_first_pass=1770us`, `nodes_performed=330`, `Container nodes=132`.
- After:
  `target/fret-diag/retained-vlist-inline-cell-padding-codex-20260621/1782066104208/bundle.json`
  reported `p95.us(total/layout/prepaint/paint)=1983/1642/76/306`,
  `layout.root apply=1366`, `layout.nodes=250`, and the retained `VirtualList` child path had
  `layout_children_first_pass=667us`, `nodes_performed=198`, `Container nodes=0`.

Interpretation:

- The root apply owner is still the window root in `layout_root_applies[]`, but the scroll profile
  now makes the child owner explicit: the prior cost was row/cell subtree breadth inside the
  retained data-table `VirtualList`.
- Inline cell padding is a bounded table mechanism/policy optimization, not a broad runtime
  shortcut. It only applies to fixed-height retained rows when grid lines and per-cell debug anchors
  are disabled; the previous wrapper path remains for cell anchors, grid lines, and measured rows.
- The next owner is no longer the deleted cell `Container` shell. The remaining retained child path
  is `Text` + `ManagedSurface` + `Pressable`, plus the content `Scroll` shell.

## 2026-06-21 Retained View-Cache Settle Contract

```bash
cargo nextest run -p fret-ui retained_virtual_list_host_updates_window_without_rerendering_view_cache_root --no-fail-fast --no-capture
cargo nextest run -p fret-ui retained_virtual_list --no-fail-fast --no-capture
```

Observed gates:

- The focused retained-host reconcile test passed.
- The retained VirtualList group gate passed: 8 tests run, 8 passed.

Interpretation:

- The first third-frame settle assertion failed because the test duplicated the
  `cx.view_cache(...)` callsite in a later render block. Declarative cache-root identity is
  callsite-driven, so that block created a different cache-root `GlobalElementId` and forced the
  render closure to run.
- The test now uses a shared `build_cached_list` helper so warmup, scroll, and settle frames all
  exercise the same cache root.
- With stable cache-root identity, the settle frame after retained-host membership refresh keeps the
  render count unchanged and records zero clean child `layout_in` calls.
- No runtime reuse-root marking change landed from this characterization.

## Documentation Gates

```powershell
python -m json.tool docs\workstreams\retained-virtual-list-root-apply-v1\WORKSTREAM.json
python tools\check_workstream_catalog.py
git diff --check
```
