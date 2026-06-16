# Evidence And Gates: Retained VirtualList Root Apply v1

Status: Active
Last updated: 2026-06-16

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

## Documentation Gates

```powershell
python -m json.tool docs\workstreams\retained-virtual-list-root-apply-v1\WORKSTREAM.json
python tools\check_workstream_catalog.py
git diff --check
```
