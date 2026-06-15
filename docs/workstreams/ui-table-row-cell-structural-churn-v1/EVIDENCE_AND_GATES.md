# Evidence And Gates: UI Table Row/Cell Structural Churn v1

Status: Active
Last updated: 2026-06-15

## Baseline Sources

Prior closeout boundary:

- `docs/workstreams/ui-layout-dirty-breadth-data-table-v1/CLOSEOUT_AUDIT_2026-05-15.md`
- Key verdict: avoidable page-cache breadth, Input chrome motion, and a redundant runtime
  structural walk were handled there; remaining work should target row/cell structural churn inside
  the contained table subtree.

Current general-app probe:

- Suite: `tools/diag-scripts/suites/perf-ui-gallery-general-app-components/suite.json`
- Summary: `target/fret-general-app-perf/data-table-view-cache-filter-shrink-r3/regression.summary.json`
- Worst bundle:
  `target/fret-general-app-perf/data-table-view-cache-filter-shrink-r3/1781333281689/bundle.schema2.json`
- Layout sidecar:
  `target/fret-general-app-perf/data-table-view-cache-filter-shrink-r3/layout.perf.summary.v1.json`

## Current Attribution

Data-table view-cache/filter/vlist:

- top total p50/p95/max: `16262/17068/17068us`
- layout p95: `15336us`
- layout engine solve p95: `5103us`
- paint p95: `1543us`
- renderer encode/text p95: `178/121us`
- worst frame: `total=17068us`, `layout=15336us`, `paint=1543us`,
  `layout.nodes=1074`, `paint.nodes=1145`, `cache_roots=2`, `cache.reused=0`,
  `contained_relayouts=1`
- top layout solve: `Pressable` at `ecosystem/fret-ui-kit/src/declarative/table.rs:8058`,
  `reason=first_solve`, `batch_roots=33`, `subtree_nodes=297`, `solve_time_us=829`
- top layout hotspots:
  - gallery content `Scroll`, inclusive `11839us`;
  - data-table `VirtualList`, inclusive `9149us`;
  - inner table horizontal `Scroll`, layout `673us`.

Virtual-list comparison:

- Summary: `target/fret-general-app-perf/virtual-list-torture-steady-r3/regression.summary.json`
- Worst bundle:
  `target/fret-general-app-perf/virtual-list-torture-steady-r3/1781332244223/bundle.schema2.json`
- top total p50/p95/max: `8216/9311/9311us`
- layout p95: `8232us`
- layout engine solve p95: `3359us`
- paint p95: `778us`
- renderer encode/text p95: `395/125us`
- top layout solve: row `Container` at
  `apps/fret-ui-gallery/src/ui/previews/pages/harness/virtual_list_torture.rs:477`,
  `reason=first_solve`, `batch_roots=35`, `subtree_nodes=455`,
  `measure_calls=210`, `solve_time_us=2571`

Interpretation:

- The common signature is row/list structural work and many first-solved batch roots. Renderer work
  is not the dominant owner.
- The data-table script is fixed-height by default (`measure_rows(false)` unless
  `FRET_UI_GALLERY_DATA_TABLE_VARIABLE_HEIGHT` is set), so the remaining cost is not simply
  "variable row measurement is expensive".
- The first implementation should reduce or prove row/cell wrapper churn in
  `ecosystem/fret-ui-kit/src/declarative/table.rs`.

## 2026-06-13 First Slice - Single Center Group Fast Path

Owner:

- Default non-retained data tables start with no pinned left/right columns and a non-empty center
  column group.
- The old header/body path still rendered empty left/right groups plus an outer horizontal grouping
  row before reaching the center group.

Change:

- `ecosystem/fret-ui-kit/src/declarative/table.rs` adds a table-local fast path for
  `left_len == 0 && center_len > 0 && right_len == 0`.
- In that case, the non-retained header and body render the center group directly with the existing
  horizontal scroll handle.
- Pinned columns, empty tables, retained table rendering, grouped rows, row selection, and shadcn
  recipe code stay on their existing paths.

Expected effect:

- Reduce one wrapper/grouping row level around the header and each visible body row in the default
  shadcn-style data-table path.
- Reduce avoidable row/cell structural churn before considering broader retained/windowing or
  runtime layout-cache changes.

Focused correctness gates passed:

```powershell
cargo fmt --package fret-ui-kit
cargo nextest run -p fret-ui-kit table_single_center_group_fast_path_requires_nonempty_center_only --no-fail-fast
cargo nextest run -p fret-ui-kit table_virtualized_alignment_gate_header_matches_rows_under_overflow_and_variable_height --no-fail-fast
cargo nextest run -p fret-ui-kit table_virtualized_pointer_row_selection_policy_list_like --no-fail-fast
cargo nextest run -p fret-ui-kit table_virtualized_nested_pressable_remains_hittable_when_pointer_row_selection_disabled --no-fail-fast
git diff --check
python -m json.tool docs\workstreams\ui-table-row-cell-structural-churn-v1\WORKSTREAM.json
python tools\check_workstream_catalog.py
```

Additional shadcn gate retry:

```powershell
cargo nextest run -p fret-ui-shadcn retained_data_table_header_debug_ids_sort_with_column_actions --no-fail-fast
cargo nextest run -p fret-ui-shadcn --lib retained_data_table_header_debug_ids_sort_with_column_actions --no-fail-fast
```

- The unscoped package run failed while listing an unrelated integration-test executable:
  `extras_relative_time_auto_update` returned Windows `os error 740`.
- The target itself lives in the `fret-ui-shadcn` library test binary.
- Retried with `--lib`; the gate passed:
  `data_table::tests::retained_data_table_header_debug_ids_sort_with_column_actions`.

Perf read:

- Not yet re-run after this first slice.
- Next evidence step is to run the fresh data-table layout-node repro below and compare the same
  counters against the baseline bundle.

## 2026-06-15 ScrollContentTransform Flow Subtree Contract

Owner:

- The shared horizontal transform table path moved default unpinned body rows away from one
  horizontal `Scroll` per row, but its row descendants still rely on the layout engine flow builder
  seeing `ScrollContentTransform` as a normal wrapper.
- Without that flow recursion, table pixels can recover through widget-local fallback solves, but
  dense `Flex -> cell container` rows pay extra per-row layout work and the fallback-free contract
  is false.

Change:

- `crates/fret-ui/src/layout/engine/flow.rs` now includes
  `ElementInstance::ScrollContentTransform(_)` in the wrapper/pass-through flow lists.
- `crates/fret-ui/src/declarative/tests/layout/scroll.rs` adds
  `scroll_content_transform_solves_flow_descendants_without_widget_fallback`, which builds the
  table-like shape `ScrollContentTransform -> Flex -> fixed cell containers` and requires zero
  `layout_engine_widget_fallback_solves`.

Focused gates passed:

```powershell
cargo nextest run --cargo-profile dev-fast -p fret-ui scroll_content_transform_moves_children_without_owning_scroll_extent scroll_content_transform_solves_flow_descendants_without_widget_fallback --no-fail-fast --no-capture
$env:FRET_LAYOUT_FORBID_WIDGET_FALLBACK_SOLVES='1'; cargo nextest run --cargo-profile dev-fast -p fret-ui-kit table_virtualized_alignment_gate_header_matches_rows_under_overflow_and_variable_height table_virtualized_unpinned_body_uses_shared_horizontal_transform table_virtualized_pointer_select_does_not_shift_row_bounds --no-fail-fast --no-capture
```

Interpretation:

- This is a mechanism-layer performance correctness fix, not a table-local geometry workaround.
- It does not replace a future fixed-track/row geometry primitive if node profiles still show row
  `Flex` as the next material owner, but it removes the accidental widget-local fallback solves from
  the current shared-row-transform table path.

## 2026-06-15 Retained Table Shared Horizontal Transform

Owner:

- The non-retained single-center table path already used a shared horizontal scroll owner plus row
  `ScrollContentTransform` wrappers.
- The retained table body still created one horizontal `Scroll` per visible row in the same
  single-center column shape, which made retained data-table frames layout/root-apply dominated.

Change:

- `ecosystem/fret-ui-kit/src/declarative/table.rs` now uses the same shared-X structure in
  `table_virtualized_retained_v0` for the unpinned single-center body path:
  retained rows use `ScrollContentTransform`, and the retained body list is wrapped in one shared
  X-axis `WheelRegion`.
- Pinned and mixed column groups stay on the previous per-group scroll structure until a separate
  alignment/perf gate proves a safer representation.
- `table_virtualized_retained_unpinned_body_uses_shared_horizontal_transform` proves retained body
  rows contain no row-local horizontal `Scroll`, contain exactly one `ScrollContentTransform`, and
  keep header/body visual bounds aligned after horizontal wheel input.

Focused gates passed:

```powershell
cargo nextest run --cargo-profile dev-fast -p fret-ui-kit table_virtualized_retained_unpinned_body_uses_shared_horizontal_transform --no-fail-fast --no-capture
cargo nextest run --cargo-profile dev-fast -p fret-ui-kit table_virtualized_retained_unpinned_body_uses_shared_horizontal_transform table_virtualized_retained_colpin_alignment_gate_across_pin_resize_and_overflow table_virtualized_retained_colpin_alignment_gate_measured_rows_do_not_shrink_width table_virtualized_retained_pointer_row_selection_policy_list_like table_virtualized_retained_nested_pressable_remains_hittable_when_pointer_row_selection_disabled table_virtualized_retained_selected_semantics_follow_windowed_row_selection table_virtualized_retained_header_debug_ids_click_sort_actions --no-fail-fast --no-capture
cargo nextest run --cargo-profile dev-fast -p fret-ui-shadcn --lib retained_data_table_header_debug_ids_sort_with_column_actions --no-fail-fast --no-capture
cargo fmt -p fret-ui-kit
```

Retained repro:

```powershell
target\release\fretboard-dev.exe diag stats target\fret-diag\vlist-retained-filter-shrink-correct-script-v1\sessions\1781528832521-146560\1781528844457-ui-gallery-data-table-retained-filter-shrink-vlist-inputs-change\bundle.schema2.json --sort cpu_cycles --top 30
target\release\fretboard-dev.exe diag stats target\fret-diag\vlist-retained-shared-row-xform-v1\sessions\1781530321751-126564\1781531045060-ui-gallery-data-table-retained-filter-shrink-vlist-inputs-change\bundle.schema2.json --sort cpu_cycles --top 30
```

Before/after stats:

- Before:
  `target/fret-diag/vlist-retained-filter-shrink-correct-script-v1/sessions/1781528832521-146560/1781528844457-ui-gallery-data-table-retained-filter-shrink-vlist-inputs-change/bundle.schema2.json`
- Before worst retained frame: `total=24856us`, `layout=23060us`,
  `layout.engine_solve=13231us`, `layout.root apply=20407us`, `layout.nodes=810`.
- After:
  `target/fret-diag/vlist-retained-shared-row-xform-v1/sessions/1781530321751-126564/1781531045060-ui-gallery-data-table-retained-filter-shrink-vlist-inputs-change/bundle.schema2.json`
- After worst retained frame: `total=11715us`, `layout=10831us`,
  `layout.engine_solve=6599us`, `layout.root apply=9541us`, `layout.nodes=646`.

Interpretation:

- This is a material retained-path win on the correct retained script: the worst frame moved from
  well over budget to close to a 120Hz frame budget, with layout/root-apply/solve all roughly halved.
- Row-level horizontal `Scroll` nodes are no longer the retained body hotspot. Remaining work should
  target retained root apply, VirtualList reconciliation, or a first-class fixed-track layout
  primitive rather than adding more row-local scroll special cases.

## First Repro Commands

Existing bundle attribution:

```powershell
target\release\fretboard-dev.exe diag stats target\fret-general-app-perf\data-table-view-cache-filter-shrink-r3\1781333281689\bundle.schema2.json --sort cpu_cycles --top 30
target\release\fretboard-dev.exe diag stats target\fret-general-app-perf\virtual-list-torture-steady-r3\1781332244223\bundle.schema2.json --sort cpu_cycles --top 30
```

Fresh data-table layout-node repro:

```powershell
target\release\fretboard-dev.exe diag perf tools\diag-scripts\ui-gallery\data-table\ui-gallery-data-table-view-cache-filter-shrink-vlist-inputs-change.json `
  --repeat 3 `
  --warmup-frames 5 `
  --reuse-launch `
  --prewarm-script tools\diag-scripts\_prelude\tooling-suite-prewarm-fonts.json `
  --prelude-script tools\diag-scripts\_prelude\tooling-suite-prelude-reset-diagnostics.json `
  --env FRET_UI_GALLERY_VIEW_CACHE=1 `
  --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 `
  --env FRET_LAYOUT_NODE_PROFILE=1 `
  --env FRET_LAYOUT_NODE_PROFILE_TOP=20 `
  --env FRET_LAYOUT_NODE_PROFILE_MIN_US=300 `
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 `
  --env FRET_DIAG_SEMANTICS=0 `
  --sort time `
  --top 15 `
  --json `
  --dir target\fret-diag\ui-table-row-cell-structural-churn-v1-data-table-r3 `
  --launch -- target\release\fret-ui-gallery.exe
```

## Correctness Gates

```powershell
cargo nextest run -p fret-ui-kit table_virtualized_retained_header_debug_ids_click_sort_actions --no-fail-fast
cargo nextest run -p fret-ui-shadcn retained_data_table_header_debug_ids_sort_with_column_actions --no-fail-fast
cargo nextest run --cargo-profile dev-fast -p fret-ui-kit table_virtualized_retained_unpinned_body_uses_shared_horizontal_transform --no-fail-fast --no-capture
```

## Mechanism And Boundary Gates

Run these when code changes cross the related crates:

```powershell
cargo check -p fret-ui-kit --all-targets
cargo check -p fret-ui-shadcn --all-targets
cargo check -p fret-ui-gallery --features gallery-dev --all-targets
python tools/check_layering.py
```

## Documentation Gates

```powershell
python -m json.tool docs\workstreams\ui-table-row-cell-structural-churn-v1\WORKSTREAM.json
python tools\check_workstream_catalog.py
git diff --check
```
