# Evidence And Gates: UI Table Row/Cell Structural Churn v1

Status: Active
Last updated: 2026-06-13

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
