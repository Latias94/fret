# UI Prepaint Derived Surfaces v1 - Milestones

Date: 2026-05-15
Status: Closed (M0/M1/M2/M3 complete; closeout audit records retained mechanisms and follow-ons)

## M0 - Lane Ready

Exit criteria:

- Workstream doc set exists.
- `WORKSTREAM.json` points to the canonical first-open docs.
- ADR posture is explicit.
- First repro/gate surface is selected.

Initial first surface:

- `ui-gallery-virtual-list-torture-steady`

M0 evidence:

- first attribution bundle:
  `target/fret-diag/ui-prepaint-derived-surfaces-v1-virtual-list-attrib/1778755322233/bundle.schema2.json`
- dominant phase: layout, with time sum total/layout/prepaint/paint =
  `14396/12011/252/2133us`

## M1 - Retained Virtual List Boundary-Derived Proof

Exit criteria:

- Baseline perf bundle and worst-frame attribution are recorded.
- The current retained virtual-list derived-state ownership is audited.
- At least one measured runtime path is either moved into boundary-owned derived state or retained
  with a written reason.
- Focused virtual-list correctness tests pass.
- The selected virtual-list perf gate passes.
- `diag stats` or bundle inspection proves boundary reuse/rejection and virtual-list window-shift
  diagnostics remain explainable.

Representative evidence:

- `crates/fret-ui/src/declarative/tests/virtual_list/retained.rs`
- `crates/fret-ui/src/tree/prepaint/virtual_list.rs`
- `crates/fret-ui/src/tree/prepaint/tests/prepaint_virtual_list_window_update_harness.rs`
- `tools/diag-scripts/ui-gallery/perf/ui-gallery-virtual-list-torture-steady.json`
- `docs/workstreams/perf-baselines/ui-gallery-virtual-list-torture-steady.windows-rtx4090.v1.json`
- `target/fret-diag/ui-prepaint-derived-surfaces-v1-virtual-list-attrib/1778755322233/bundle.schema2.json`
- `target/fret-diag/ui-prepaint-derived-surfaces-v1-virtual-list-attrib/1778755328023-ui-gallery-virtual-list-bottom-steady/bundle.schema2.json`

Current M1 slice evidence:

- Boundary-owned derived output: `VirtualListPrepaintWindowOutput`.
- Consumer: retained virtual-list reconcile in `crates/fret-ui/src/declarative/mount.rs`.
- Guard test: `retained_virtual_list_host_updates_window_without_rerendering_view_cache_root`
  intentionally makes the element-local virtual-list window stale/overbroad and requires the final
  retained window to equal the boundary-owned output.
- Retained with reason: `VirtualListState` for declarative render/layout bridge state and
  `RetainedVirtualListKeepAliveState` for detached live-node reuse.
- Closeout perf/stats evidence:
  `target/release/fretboard-dev diag perf tools/diag-scripts/ui-gallery/perf/ui-gallery-virtual-list-torture-steady.json --repeat 7 --warmup-frames 5 --reuse-launch --sort time --top 15 --json --prewarm-script tools/diag-scripts/_prelude/tooling-suite-prewarm-fonts.json --prelude-script tools/diag-scripts/_prelude/tooling-suite-prelude-reset-diagnostics.json --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --env FRET_UI_GALLERY_VLIST_KNOWN_HEIGHTS=1 --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --env FRET_DIAG_MAX_SNAPSHOTS=240 --launch -- target/release/fret-ui-gallery`
  passed on 2026-05-15 as a macOS attribution-only run.
- Worst bundle:
  `target/fret-diag/1778777905741/bundle.schema2.json`, with total/layout/prepaint/paint =
  `14833/11423/315/3095us` and p50/p95 total = `4374/5555us`.

## M2 - Retained Data Table / View-Cache Proof

Exit criteria:

- Retained data-table correctness suite remains green.
- View-cache filter-shrink script has a bundle and attribution record.
- Data-table reuse/rejection is explained through boundary diagnostics or an explicitly documented
  component-policy reason.
- Any mechanism change stays in `crates/fret-ui`; policy changes stay in `ecosystem/fret-ui-kit`.
- A stable data-table perf or stats gate is selected if the surface becomes a durable contract.

Representative evidence:

- `ecosystem/fret-ui-kit/src/declarative/table.rs`
- `tools/diag-scripts/suites/ui-gallery-data-table-retained/suite.json`
- `tools/diag-scripts/suites/ui-gallery-data-table-view-cache-torture/suite.json`
- `tools/diag-scripts/ui-gallery/data-table/ui-gallery-data-table-view-cache-filter-shrink-vlist-inputs-change.json`

Current view-cache evidence:

- The initial `ui-gallery-data-table-view-cache-torture` run failed at step 23:
  `target/fret-diag/ui-prepaint-derived-surfaces-v1-data-table-view-cache-torture/1778759907796-script-step-0023-assert-failed/bundle.schema2.json`
- A rebuilt `gallery-dev` run passed:
  `target/fret-diag/ui-prepaint-derived-surfaces-v1-data-table-view-cache-torture-current-dev/1778762426810-ui-gallery-data-table-view-cache-filter-shrink-vlist-inputs-change/bundle.schema2.json`
- A same-state run after script stabilization passed:
  `target/fret-diag/ui-prepaint-derived-surfaces-v1-data-table-view-cache-2026-05-15-after-scroll-wait/suite.summary.json`
- Current `diag stats` reports total/layout/prepaint/paint = `185149/168294/722/16133us`,
  p50/p95 total `14092/16867us`, and p50/p95 layout `12793/15541us`.
- Same-state `diag stats` on
  `target/fret-diag/ui-prepaint-derived-surfaces-v1-data-table-view-cache-2026-05-15-after-scroll-wait/1778777531409-ui-gallery-data-table-view-cache-filter-shrink-vlist-inputs-change/bundle.schema2.json`
  reports total/layout/prepaint/paint = `105023/91918/949/12156us`, p50/p95 total =
  `12606/18445us`, and p50/p95 layout = `11328/17013us`.
- The last snapshots show filtered `items_len=111`, virtual-list window `22..34`,
  `window_shift_kind=none`, `prepaint_owner=view_boundary_prepaint_state`, and cache roots rejected
  by `layout_invalidated`.

Retained-table correctness closeout so far:

- Historical failure bundle:
  `target/fret-diag/ui-prepaint-derived-surfaces-v1-data-table-retained-current-dev-cargo-run/1778762568416-script-step-0022-assert-failed/bundle.schema2.json`
- Passing retained suite summary after the fixes:
  `target/fret-diag/ui-prepaint-derived-surfaces-v1-data-table-retained-after-sort-anchor-split-suite/suite.summary.json`
- Same-state passing retained suite summary after script stabilization:
  `target/fret-diag/ui-prepaint-derived-surfaces-v1-data-table-retained-2026-05-15-after-scroll-wait/suite.summary.json`
- Focused gates added for the row-order and sort-anchor fixes:
  `table_virtualized_retained_header_debug_ids_click_sort_actions` and
  `retained_data_table_header_debug_ids_sort_with_column_actions`.
- Retained filter-shrink attribution bundle:
  `target/fret-diag/ui-prepaint-derived-surfaces-v1-data-table-retained-after-sort-anchor-split-suite/1778776884214-ui-gallery-data-table-retained-filter-shrink-vlist-inputs-change/bundle.schema2.json`
  with total/layout/prepaint/paint = `20432/16832/1125/2475us`, p50/p95 total =
  `180/14752us`.
- Same-state retained filter-shrink attribution bundle:
  `target/fret-diag/ui-prepaint-derived-surfaces-v1-data-table-retained-2026-05-15-after-scroll-wait/1778777573643-ui-gallery-data-table-retained-filter-shrink-vlist-inputs-change/bundle.schema2.json`
  with total/layout/prepaint/paint = `19351/15941/1027/2383us`, p50/p95 total =
  `163/13948us`.
- Retained multi-sort attribution bundle:
  `target/fret-diag/ui-prepaint-derived-surfaces-v1-data-table-retained-after-sort-anchor-split-suite/1778776885328-ui-gallery-data-table-retained-multi-sort-shift-click/bundle.schema2.json`
  with total/layout/prepaint/paint = `40318/33192/1707/5419us`, p50/p95 total =
  `484/10481us`.
- Same-state retained multi-sort attribution bundle:
  `target/fret-diag/ui-prepaint-derived-surfaces-v1-data-table-retained-2026-05-15-after-scroll-wait/1778777574739-ui-gallery-data-table-retained-multi-sort-shift-click/bundle.schema2.json`
  with total/layout/prepaint/paint = `39836/32667/1862/5307us`, p50/p95 total =
  `523/10077us`.
- Root causes fixed: retained row-order used constrained sizing columns and lost filter/sort
  closures; recipe/toolbar sync paths were not fully idempotent; reset needed a keyed epoch to clear
  local toolbar state between scripts; header debug IDs overlapped the column-actions trigger until
  sort and accessory nodes were split.

M2 closeout result:

- Retained and view-cache data-table proof surfaces both pass on the same implementation state.
- The cost profile remains layout-dominant. That points future optimization work toward layout dirty
  breadth, command availability, and component-policy churn rather than renderer payload.

## M3 - Cleanup And Closeout

Exit criteria:

- Old duplicated local caches/carriers touched by M1/M2 are deleted, narrowed, or explicitly
  retained.
- ADR alignment is refreshed if the implementation changed a hard contract.
- `cargo fmt --check`, focused `cargo nextest`, workstream catalog, layering, and selected diag/perf
  gates pass.
- A dated closeout audit records final architecture, performance evidence, retained mechanisms, and
  follow-ons:
  `docs/workstreams/ui-prepaint-derived-surfaces-v1/CLOSEOUT_AUDIT_2026-05-15.md`.

## Deferred Follow-Ons

These are intentionally not milestones for this lane:

- renderer `Scene` display-list contract evolution;
- per-boundary previous-frame scene recording;
- Linux-specific performance closure;
- broad UI Gallery suite baseline redefinition;
- component-policy parity work unrelated to boundary-derived state.
