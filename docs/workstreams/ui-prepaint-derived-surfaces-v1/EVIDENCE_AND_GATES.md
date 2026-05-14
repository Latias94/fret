# UI Prepaint Derived Surfaces v1 - Evidence And Gates

Date: 2026-05-15
Status: Closed

## Goal Handoff

The goal setup note is:

- `docs/workstreams/ui-prepaint-derived-surfaces-v1/GOAL_HANDOFF_2026-05-14.md`

It records the current assumptions, ADR trigger conditions, exact completion target, and suggested
goal wording for the next execution phase.

## First Repro Surface

Primary script:

```bash
cargo run -p fretboard-dev --release -- diag perf \
  tools/diag-scripts/ui-gallery/perf/ui-gallery-virtual-list-torture-steady.json \
  --repeat 7 \
  --warmup-frames 5 \
  --reuse-launch \
  --sort time \
  --top 15 \
  --json \
  --prewarm-script tools/diag-scripts/_prelude/tooling-suite-prewarm-fonts.json \
  --prelude-script tools/diag-scripts/_prelude/tooling-suite-prelude-reset-diagnostics.json \
  --perf-baseline docs/workstreams/perf-baselines/ui-gallery-virtual-list-torture-steady.windows-rtx4090.v1.json \
  --env FRET_UI_GALLERY_VIEW_CACHE=1 \
  --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 \
  --env FRET_UI_GALLERY_VLIST_KNOWN_HEIGHTS=1 \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 \
  --env FRET_DIAG_SEMANTICS=0 \
  --env FRET_DIAG_MAX_SNAPSHOTS=240 \
  --launch -- target/release/fret-ui-gallery
```

Note: this command references the existing Windows baseline because it is the current formal
contract for this surface. On macOS, either pass a macOS-specific baseline after intentional
seeding or run the same script without `--perf-baseline` for attribution-only evidence.

Worst-bundle attribution:

```bash
cargo run -p fretboard-dev --release -- diag stats <bundle.schema2.json> --sort cpu_cycles --top 30
```

## Current Attribution Snapshot

Observed on 2026-05-14 from:

- `target/fret-diag/ui-prepaint-derived-surfaces-v1-virtual-list-attrib/1778755322233/bundle.schema2.json`
- `target/fret-diag/ui-prepaint-derived-surfaces-v1-virtual-list-attrib/1778755328023-ui-gallery-virtual-list-bottom-steady/bundle.schema2.json`

`diag stats --sort cpu_cycles --top 30` on the first bundle reported:

- time sum: total/layout/prepaint/paint = `14396/12011/252/2133us`
- time p50/p95: total `3465/5515us`, layout `2994/4698us`, prepaint `56/76us`, paint `415/745us`
- hot p50/p95: `layout.engine_solve=845/1556us`, `paint.widget=179/355us`, `paint.text_prepare=0/0us`
- top frames: `cache.reused=1`, `cache.replayed_ops=604`, `paint.cache_misses=0`

Interpretation:

- The current virtual-list proof surface is layout-dominant, not prepaint-dominant.
- The first landed slice should be judged as a boundary-ownership migration for derived window
  state. The next performance slice should look for layout dirty breadth, retained reconcile churn,
  or data-table policy churn before chasing renderer work.

## Closeout Virtual-List Perf Evidence

Observed on 2026-05-15 from:

- `target/fret-diag/1778777905741/bundle.schema2.json`

Command:

```bash
target/release/fretboard-dev diag perf \
  tools/diag-scripts/ui-gallery/perf/ui-gallery-virtual-list-torture-steady.json \
  --repeat 7 \
  --warmup-frames 5 \
  --reuse-launch \
  --sort time \
  --top 15 \
  --json \
  --prewarm-script tools/diag-scripts/_prelude/tooling-suite-prewarm-fonts.json \
  --prelude-script tools/diag-scripts/_prelude/tooling-suite-prelude-reset-diagnostics.json \
  --env FRET_UI_GALLERY_VIEW_CACHE=1 \
  --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 \
  --env FRET_UI_GALLERY_VLIST_KNOWN_HEIGHTS=1 \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 \
  --env FRET_DIAG_SEMANTICS=0 \
  --env FRET_DIAG_MAX_SNAPSHOTS=240 \
  --launch -- target/release/fret-ui-gallery
```

The run intentionally omitted the Windows RTX 4090 baseline on macOS and is attribution-only. It
passed with `repeat=7`; worst overall was `target/fret-diag/1778777905741/bundle.schema2.json`.

`diag stats --sort cpu_cycles --top 30` on that bundle reported:

- time sum: total/layout/prepaint/paint = `14833/11423/315/3095us`
- time p50/p95: total `4374/5555us`, layout `2887/4729us`, prepaint `62/139us`, paint
  `758/1348us`
- hot p50/p95: `layout.engine_solve=792/1595us`, `paint.widget=384/682us`,
  `paint.text_prepare=0/0us`
- top frame: `cache_roots=3`, `cache.reused=1`, `cache.replayed_ops=519`,
  `top_view_cache_roots_needs_rerender=2`

Interpretation:

- The proof surface is still layout-dominant.
- The 2026-05-15 script stabilization changed jump-input setup to `set_text_value`; this makes
  repeat/reuse-launch runs deterministic on macOS instead of relying on platform shortcut handling.

## Current Data-Table Evidence

Historical first failing view-cache bundle:

- `target/fret-diag/ui-prepaint-derived-surfaces-v1-data-table-view-cache-torture/1778759907796-script-step-0023-assert-failed/bundle.schema2.json`

Current passing view-cache bundle with `gallery-dev`:

- `target/fret-diag/ui-prepaint-derived-surfaces-v1-data-table-view-cache-torture-current-dev/1778762426810-ui-gallery-data-table-view-cache-filter-shrink-vlist-inputs-change/bundle.schema2.json`

Same-state passing view-cache summary after script stabilization:

- `target/fret-diag/ui-prepaint-derived-surfaces-v1-data-table-view-cache-2026-05-15-after-scroll-wait/suite.summary.json`

Same-state view-cache attribution bundle:

- `target/fret-diag/ui-prepaint-derived-surfaces-v1-data-table-view-cache-2026-05-15-after-scroll-wait/1778777531409-ui-gallery-data-table-view-cache-filter-shrink-vlist-inputs-change/bundle.schema2.json`

Historical retained-suite failing bundle:

- `target/fret-diag/ui-prepaint-derived-surfaces-v1-data-table-retained-current-dev-cargo-run/1778762568416-script-step-0022-assert-failed/bundle.schema2.json`

Current retained-suite passing summary:

- `target/fret-diag/ui-prepaint-derived-surfaces-v1-data-table-retained-after-sort-anchor-split-suite/suite.summary.json`

Same-state retained-suite passing summary after script stabilization:

- `target/fret-diag/ui-prepaint-derived-surfaces-v1-data-table-retained-2026-05-15-after-scroll-wait/suite.summary.json`

Current retained-table attribution bundles:

- filter shrink:
  `target/fret-diag/ui-prepaint-derived-surfaces-v1-data-table-retained-after-sort-anchor-split-suite/1778776884214-ui-gallery-data-table-retained-filter-shrink-vlist-inputs-change/bundle.schema2.json`
- multi-sort:
  `target/fret-diag/ui-prepaint-derived-surfaces-v1-data-table-retained-after-sort-anchor-split-suite/1778776885328-ui-gallery-data-table-retained-multi-sort-shift-click/bundle.schema2.json`

Same-state retained-table attribution bundles:

- filter shrink:
  `target/fret-diag/ui-prepaint-derived-surfaces-v1-data-table-retained-2026-05-15-after-scroll-wait/1778777573643-ui-gallery-data-table-retained-filter-shrink-vlist-inputs-change/bundle.schema2.json`
- multi-sort:
  `target/fret-diag/ui-prepaint-derived-surfaces-v1-data-table-retained-2026-05-15-after-scroll-wait/1778777574739-ui-gallery-data-table-retained-multi-sort-shift-click/bundle.schema2.json`

Commands used for attribution:

```bash
target/release/fretboard-dev diag stats \
  target/fret-diag/ui-prepaint-derived-surfaces-v1-data-table-view-cache-torture/1778759907796-script-step-0023-assert-failed/bundle.schema2.json \
  --sort cpu_cycles \
  --top 30

target/release/fretboard-dev diag stats \
  target/fret-diag/ui-prepaint-derived-surfaces-v1-data-table-view-cache-torture-current-dev/1778762426810-ui-gallery-data-table-view-cache-filter-shrink-vlist-inputs-change/bundle.schema2.json \
  --sort cpu_cycles \
  --top 30

target/release/fretboard-dev diag stats \
  target/fret-diag/ui-prepaint-derived-surfaces-v1-data-table-retained-current-dev-cargo-run/1778762568416-script-step-0022-assert-failed/bundle.schema2.json \
  --sort cpu_cycles \
  --top 30

target/release/fretboard-dev diag stats \
  target/fret-diag/ui-prepaint-derived-surfaces-v1-data-table-retained-after-sort-anchor-split-suite/1778776884214-ui-gallery-data-table-retained-filter-shrink-vlist-inputs-change/bundle.schema2.json \
  --sort cpu_cycles \
  --top 30

target/release/fretboard-dev diag stats \
  target/fret-diag/ui-prepaint-derived-surfaces-v1-data-table-retained-after-sort-anchor-split-suite/1778776885328-ui-gallery-data-table-retained-multi-sort-shift-click/bundle.schema2.json \
  --sort cpu_cycles \
  --top 30

target/release/fretboard-dev diag stats \
  target/fret-diag/ui-prepaint-derived-surfaces-v1-data-table-view-cache-2026-05-15-after-scroll-wait/1778777531409-ui-gallery-data-table-view-cache-filter-shrink-vlist-inputs-change/bundle.schema2.json \
  --sort cpu_cycles \
  --top 20

target/release/fretboard-dev diag stats \
  target/fret-diag/ui-prepaint-derived-surfaces-v1-data-table-retained-2026-05-15-after-scroll-wait/1778777573643-ui-gallery-data-table-retained-filter-shrink-vlist-inputs-change/bundle.schema2.json \
  --sort cpu_cycles \
  --top 20

target/release/fretboard-dev diag stats \
  target/fret-diag/ui-prepaint-derived-surfaces-v1-data-table-retained-2026-05-15-after-scroll-wait/1778777574739-ui-gallery-data-table-retained-multi-sort-shift-click/bundle.schema2.json \
  --sort cpu_cycles \
  --top 20
```

Observed result:

- The first view-cache suite run failed at step 23 of
  `tools/diag-scripts/ui-gallery/data-table/ui-gallery-data-table-view-cache-filter-shrink-vlist-inputs-change.json`.
- Its `diag stats` reported total/layout/prepaint/paint =
  `148369/134882/610/12877us`.
- The current rebuilt view-cache suite passes with `gallery-dev`. Its `diag stats` report
  total/layout/prepaint/paint = `185149/168294/722/16133us`, p50/p95 total =
  `14092/16867us`, and p50/p95 layout = `12793/15541us`.
- The current view-cache last snapshots show `items_len=111`, virtual-list window `22..34`,
  `window_shift_kind=none`, `prepaint_owner=view_boundary_prepaint_state`, and cache roots rejected
  by `layout_invalidated`.
- The historical retained-table suite failed at step 22 after `GlobalFilter: Process 123` became
  visible. Its `diag stats` reported total/layout/prepaint/paint = `9188/7436/922/830us`,
  p50/p95 total = `959/1018us`, and p50/p95 layout = `789/814us`; the last snapshots still showed
  `items_len=50000` and no retained-reconcile input-change record.
- After the retained row-order, idempotent sync, reset-epoch, and sort-anchor split fixes, the
  retained data-table suite passed all 12 scripts. The suite summary is
  `target/fret-diag/ui-prepaint-derived-surfaces-v1-data-table-retained-after-sort-anchor-split-suite/suite.summary.json`.
- The retained filter-shrink bundle reports total/layout/prepaint/paint =
  `20432/16832/1125/2475us`, p50/p95 total = `180/14752us`, and remains layout-dominant on the
  interaction frame.
- The retained multi-sort bundle reports total/layout/prepaint/paint =
  `40318/33192/1707/5419us`, p50/p95 total = `484/10481us`, and remains layout-dominant on sort
  interactions. Some snapshots also show high `command_availability` evaluation time; that is a
  possible follow-on once the boundary-derived-state lane closes.
- After changing scroll diagnostics checks from fixed-frame `assert` to `wait_until`, the same-state
  view-cache suite passed. Its attribution bundle reports total/layout/prepaint/paint =
  `105023/91918/949/12156us`, p50/p95 total = `12606/18445us`, and p50/p95 layout =
  `11328/17013us`.
- The same-state retained suite passed all 12 scripts. The retained filter-shrink attribution bundle
  reports total/layout/prepaint/paint = `19351/15941/1027/2383us`, p50/p95 total =
  `163/13948us`. The retained multi-sort attribution bundle reports total/layout/prepaint/paint =
  `39836/32667/1862/5307us`, p50/p95 total = `523/10077us`.

Interpretation:

- The view-cache surface is stable enough to keep as the non-retained comparison gate, but it is
  still layout-dominant.
- The retained-table correctness/model-sync blocker is fixed, and the same-state view-cache
  comparison gate is now recorded.
- The current retained-table costs point at layout invalidation breadth and component-policy churn
  rather than a missing renderer payload optimization.

## Correctness Gates

Retained virtual-list mechanism gates:

```bash
cargo nextest run -p fret-ui retained_virtual_list_host_updates_window_without_rerendering_view_cache_root --no-fail-fast
cargo nextest run -p fret-ui retained_virtual_list_keep_alive_reuses_detached_items_when_scrolling_back --no-fail-fast
cargo nextest run -p fret-ui mechanism_harness_retained_virtual_list_reconcile_matches_oracles --no-fail-fast
cargo nextest run -p fret-ui mechanism_harness_prepaint_virtual_list_window_update_matches_oracles --no-fail-fast
```

Observed focused result:

- `cargo nextest run -p fret-ui retained_virtual_list_host_updates_window_without_rerendering_view_cache_root --no-fail-fast`
  passed on 2026-05-15. This test now proves retained reconcile prefers boundary-owned
  `VirtualListPrepaintWindowOutput` even when `VirtualListState.window_range` and
  `render_window_range` are valid but intentionally stale/overbroad.
- `cargo nextest run -p fret-ui retained_virtual_list_keep_alive_reuses_detached_items_when_scrolling_back mechanism_harness_retained_virtual_list_reconcile_matches_oracles mechanism_harness_prepaint_virtual_list_window_update_matches_oracles --no-fail-fast`
  passed on 2026-05-15 (`3 passed, 943 skipped`).
- `cargo check -p fret-ui --all-targets` passed on 2026-05-15.

Virtual-list broader gate:

```bash
cargo nextest run -p fret-ui virtual_list --no-fail-fast
```

Data-table correctness repros:

```bash
target/release/fretboard-dev diag suite ui-gallery-data-table-retained \
  --reuse-launch \
  --prewarm-script tools/diag-scripts/_prelude/tooling-suite-prewarm-fonts.json \
  --prelude-script tools/diag-scripts/_prelude/tooling-suite-prelude-reset-diagnostics.json \
  --env FRET_UI_GALLERY_DATA_TABLE_RETAINED=1 \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 \
  --env FRET_DIAG_SEMANTICS=0 \
  --launch -- cargo run -p fret-ui-gallery --release --features gallery-dev

target/release/fretboard-dev diag suite ui-gallery-data-table-view-cache-torture \
  --reuse-launch \
  --prewarm-script tools/diag-scripts/_prelude/tooling-suite-prewarm-fonts.json \
  --prelude-script tools/diag-scripts/_prelude/tooling-suite-prelude-reset-diagnostics.json \
  --env FRET_UI_GALLERY_VIEW_CACHE=1 \
  --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 \
  --env FRET_DIAG_SEMANTICS=0 \
  --launch -- cargo run -p fret-ui-gallery --release --features gallery-dev
```

Focused retained-table gates:

```bash
cargo test -p fret-ui-kit --lib table_virtualized_retained_header_debug_ids_click_sort_actions -- --nocapture
cargo test -p fret-ui-shadcn --lib retained_data_table_header_debug_ids_sort_with_column_actions -- --nocapture
```

Observed focused result:

- `cargo test -p fret-ui-kit --lib table_virtualized_retained_header_debug_ids_click_sort_actions -- --nocapture`
  passed on 2026-05-15.
- `cargo test -p fret-ui-shadcn --lib retained_data_table_header_debug_ids_sort_with_column_actions -- --nocapture`
  passed on 2026-05-15.
- `cargo test -p fret-ui-gallery --test virtual_list_perf_surface -- --nocapture` passed on
  2026-05-15 and locks the deterministic virtual-list perf script setup.

## Framework Gates

```bash
cargo check -p fret-ui --all-targets
cargo check -p fret-ui --features diagnostics --all-targets
cargo check -p fret-ui-kit --all-targets
cargo fmt --check
python3 tools/check_layering.py
python3 tools/check_workstream_catalog.py
python3 -m json.tool docs/workstreams/ui-prepaint-derived-surfaces-v1/WORKSTREAM.json >/dev/null
```

Observed framework result:

- `cargo fmt --check` passed on 2026-05-15.
- `cargo check -p fret-ui --all-targets` passed on 2026-05-15.
- `cargo check -p fret-ui-kit --all-targets` passed on 2026-05-15.
- `cargo check -p fret-ui-shadcn --all-targets` passed on 2026-05-15.
- `cargo check -p fret-ui-gallery --features gallery-dev --all-targets` passed on 2026-05-15.
- `python3 tools/check_layering.py` passed on 2026-05-15.
- `python3 tools/check_workstream_catalog.py` passed on 2026-05-15.
- `python3 -m json.tool docs/workstreams/ui-prepaint-derived-surfaces-v1/WORKSTREAM.json >/dev/null`
  passed on 2026-05-15.
- `git diff --check` passed on 2026-05-15.

## Boundary Evidence To Preserve

Architecture evidence:

- `docs/adr/0327-frame-pipeline-v2-and-view-boundaries.md`
- `docs/workstreams/ui-frame-pipeline-v2-fearless-refactor-v1/FINAL_CLOSEOUT_AUDIT_2026-05-14.md`
- `docs/workstreams/ui-prepaint-derived-surfaces-v1/GOAL_HANDOFF_2026-05-14.md`
- `crates/fret-ui/src/tree/view_boundary.rs`
- `crates/fret-ui/src/tree/prepaint/virtual_list.rs`
- `crates/fret-ui/src/tree/debug/virtual_list.rs`
- `crates/fret-ui/src/tree/ui_tree_debug/record.rs`

Proof-surface evidence:

- `tools/diag-scripts/ui-gallery/perf/ui-gallery-virtual-list-torture-steady.json`
- `tools/diag-scripts/suites/ui-gallery-data-table-retained/suite.json`
- `tools/diag-scripts/suites/ui-gallery-data-table-view-cache-torture/suite.json`
- `tools/diag-scripts/ui-gallery/data-table/ui-gallery-data-table-view-cache-filter-shrink-vlist-inputs-change.json`
- `apps/fret-ui-gallery/src/ui/previews/pages/harness/virtual_list_torture.rs`
- `ecosystem/fret-ui-kit/src/declarative/table.rs`

## ADR Gate

No new ADR is required for M0/M1 if the implementation only applies ADR 0327's accepted boundary
ownership model to additional proof surfaces.

Before landing any slice, update or create ADR material if the slice changes:

- renderer `Scene` recording ownership;
- `PreviousFramePaintRecording` ownership;
- public boundary hint APIs;
- retained ownership decisions from the Frame Pipeline v2 closeout;
- diagnostics schema consumed by external tooling.

Closeout decision (2026-05-15): no new ADR or ADR alignment update is required. The lane applies
ADR 0327's accepted `ViewBoundaryState` ownership model to additional proof surfaces and changes
script stability/component policy; it does not change renderer recording ownership, public boundary
APIs, retained cache ownership, or external diagnostics schema.
