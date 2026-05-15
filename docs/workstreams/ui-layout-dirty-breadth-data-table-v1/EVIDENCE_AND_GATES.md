# Evidence And Gates: UI Layout Dirty Breadth Data Table v1

Status: Closed
Last updated: 2026-05-15

## Baseline Sources

Prior closeout evidence:

- `docs/workstreams/ui-prepaint-derived-surfaces-v1/CLOSEOUT_AUDIT_2026-05-15.md`
- view-cache filter shrink:
  `target/fret-diag/ui-prepaint-derived-surfaces-v1-data-table-view-cache-2026-05-15-after-scroll-wait/1778777531409-ui-gallery-data-table-view-cache-filter-shrink-vlist-inputs-change/bundle.schema2.json`,
  total/layout/prepaint/paint = `105023/91918/949/12156us`
- retained filter shrink:
  `target/fret-diag/ui-prepaint-derived-surfaces-v1-data-table-retained-2026-05-15-after-scroll-wait/1778777573643-ui-gallery-data-table-retained-filter-shrink-vlist-inputs-change/bundle.schema2.json`,
  total/layout/prepaint/paint = `19351/15941/1027/2383us`
- retained multi-sort:
  `target/fret-diag/ui-prepaint-derived-surfaces-v1-data-table-retained-2026-05-15-after-scroll-wait/1778777574739-ui-gallery-data-table-retained-multi-sort-shift-click/bundle.schema2.json`,
  total/layout/prepaint/paint = `39836/32667/1862/5307us`

Current-lane baseline evidence is recorded below.

## First Repro Commands

Build the gallery and diag runner:

```bash
cargo build -p fretboard-dev -p fret-ui-gallery --release --features gallery-dev
```

Retained suite:

```bash
cargo run -p fretboard-dev --release -- diag suite ui-gallery-data-table-retained \
  --reuse-launch \
  --prewarm-script tools/diag-scripts/_prelude/tooling-suite-prewarm-fonts.json \
  --prelude-script tools/diag-scripts/_prelude/tooling-suite-prelude-reset-diagnostics.json \
  --env FRET_UI_GALLERY_DATA_TABLE_RETAINED=1 \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 \
  --env FRET_DIAG_SEMANTICS=0 \
  --env FRET_DIAG_MAX_SNAPSHOTS=240 \
  --launch -- target/release/fret-ui-gallery
```

View-cache suite:

```bash
cargo run -p fretboard-dev --release -- diag suite ui-gallery-data-table-view-cache-torture \
  --reuse-launch \
  --prewarm-script tools/diag-scripts/_prelude/tooling-suite-prewarm-fonts.json \
  --prelude-script tools/diag-scripts/_prelude/tooling-suite-prelude-reset-diagnostics.json \
  --env FRET_UI_GALLERY_VIEW_CACHE=1 \
  --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 \
  --env FRET_DIAG_SEMANTICS=0 \
  --env FRET_DIAG_MAX_SNAPSHOTS=240 \
  --launch -- target/release/fret-ui-gallery
```

Bundle attribution:

```bash
cargo run -p fretboard-dev --release -- diag stats <bundle.schema2.json> --sort invalidation --top 30
```

Node-level layout attribution when needed:

```bash
cargo run -p fretboard-dev --release -- diag script \
  tools/diag-scripts/ui-gallery/data-table/ui-gallery-data-table-retained-filter-shrink-vlist-inputs-change.json \
  --env FRET_LAYOUT_NODE_PROFILE=1 \
  --env FRET_LAYOUT_NODE_PROFILE_TOP=20 \
  --env FRET_LAYOUT_NODE_PROFILE_MIN_US=300 \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 \
  --env FRET_DIAG_SEMANTICS=0 \
  --launch -- target/release/fret-ui-gallery
```

## Correctness Gates

```bash
cargo test -p fret-ui-kit --lib table_virtualized_retained_header_debug_ids_click_sort_actions -- --nocapture
cargo test -p fret-ui-shadcn --lib retained_data_table_header_debug_ids_sort_with_column_actions -- --nocapture
```

## Mechanism And Boundary Gates

```bash
cargo check -p fret-ui --all-targets
cargo check -p fret-ui-kit --all-targets
cargo check -p fret-ui-shadcn --all-targets
cargo check -p fret-ui-gallery --features gallery-dev --all-targets
python3 tools/check_layering.py
```

## Documentation Gates

```bash
cargo fmt --check
python3 tools/check_workstream_catalog.py
python3 -m json.tool docs/workstreams/ui-layout-dirty-breadth-data-table-v1/WORKSTREAM.json >/dev/null
git diff --check
```

## Current Attribution

Baseline runs from the current lane:

- retained suite: `target/fret-diag/ui-layout-dirty-breadth-data-table-v1-baseline-retained-2026-05-15/suite.summary.json`
  - Result: 12/12 scripts passed.
  - Required env: `FRET_UI_GALLERY_DATA_TABLE_RETAINED=1`. Without it, the retained virtual-list
    proof surface falls back to the non-retained path and reports `retained_virtual_list_reconciles=0`.
- view-cache suite: `target/fret-diag/ui-layout-dirty-breadth-data-table-v1-baseline-view-cache-2026-05-15/suite.summary.json`
  - Result: 1/1 scripts passed.

Worst retained filter-shrink bundle:

- `target/fret-diag/ui-layout-dirty-breadth-data-table-v1-baseline-retained-2026-05-15/1778802016962-ui-gallery-data-table-retained-filter-shrink-vlist-inputs-change/bundle.schema2.json`
- total/layout/prepaint/paint = `20565/16861/1102/2602us`
- worst frame: layout `13755us`, invalidation calls/nodes `568/694`, layout nodes `803/825`,
  layout engine solve `3513us`
- retained virtual-list window shift: `escape`, reason `inputs_change`, apply mode
  `retained_reconcile`, count changed `50000 -> 111`

Worst retained multi-sort bundle:

- `target/fret-diag/ui-layout-dirty-breadth-data-table-v1-baseline-retained-2026-05-15/1778802018159-ui-gallery-data-table-retained-multi-sort-shift-click/bundle.schema2.json`
- total/layout/prepaint/paint = `41576/33729/1897/5950us`
- worst layout frames are roughly `9.5-9.9ms`, with invalidation calls/nodes in the
  `313-401/580-809` range.
- Some frames also record large command availability snapshots. This is tracked as a possible
  follow-on because it is not the layout dirty-breadth mechanism owned by this lane.

Worst view-cache filter-shrink bundle:

- `target/fret-diag/ui-layout-dirty-breadth-data-table-v1-baseline-view-cache-2026-05-15/1778802051417-ui-gallery-data-table-view-cache-filter-shrink-vlist-inputs-change/bundle.schema2.json`
- total/layout/prepaint/paint = `107617/94075/990/12552us`
- worst frame: layout `17120us`, invalidation calls/nodes `698/810`, layout nodes `1081/1086`
- repeated expensive frames continue even with low invalidation counts, commonly with
  `cache.reused=0` and `layout.nodes ~= 1030-1081`
- Boundary dirty samples point at declarative animation-frame requests from shadcn `Input`
  border/ring tween chrome (`ecosystem/fret-ui-shadcn/src/input.rs`).

Interpretation:

- Retained filter/sort frames still include legitimate table/virtual-list work, so the first
  reduction target is not renderer or prepaint.
- View-cache filter-shrink has an avoidable policy component: high-frequency data-table filter
  controls inherit decorative shadcn input chrome tweens, which keep scheduling animation frames
  and dirtying the cached proof surface.
- The first landable slice is therefore policy-owned: keep default `Input` transition parity, but
  let data-table toolbar filter inputs opt out of decorative chrome motion.

## After Slice A - Data-Table Filter Input Chrome Motion

Change:

- `ecosystem/fret-ui-shadcn/src/input.rs`: `Input::chrome_motion(bool)` defaults to `true`; when
  disabled, border/ring focus chrome snaps to the target value and does not drive RAF-backed
  tweens.
- `ecosystem/fret-ui-shadcn/src/data_table_recipes.rs`: global and column filter inputs call
  `.chrome_motion(false)`.

Validation:

- `cargo test -p fret-ui-shadcn --lib input_chrome_motion_can_be_disabled_for_high_frequency_controls -- --nocapture`
- `cargo test -p fret-ui-shadcn --lib input_focus_ring_tweens_in_and_out_like_a_transition -- --nocapture`
- `cargo test -p fret-ui-kit --lib table_virtualized_retained_header_debug_ids_click_sort_actions -- --nocapture`
- `cargo test -p fret-ui-shadcn --lib retained_data_table_header_debug_ids_sort_with_column_actions -- --nocapture`
- retained suite: `target/fret-diag/ui-layout-dirty-breadth-data-table-v1-after-retained-2026-05-15-cargo/suite.summary.json`
- view-cache suite:
  `target/fret-diag/ui-layout-dirty-breadth-data-table-v1-after-view-cache-2026-05-15-cargo/suite.summary.json`

After-slice bundles:

- retained filter-shrink:
  `target/fret-diag/ui-layout-dirty-breadth-data-table-v1-after-retained-2026-05-15-cargo/1778803928181-ui-gallery-data-table-retained-filter-shrink-vlist-inputs-change/bundle.schema2.json`,
  total/layout/prepaint/paint = `20389/16963/1162/2264us`
- retained multi-sort:
  `target/fret-diag/ui-layout-dirty-breadth-data-table-v1-after-retained-2026-05-15-cargo/1778803929410-ui-gallery-data-table-retained-multi-sort-shift-click/bundle.schema2.json`,
  total/layout/prepaint/paint = `76596/68956/1907/5733us`
- view-cache filter-shrink:
  `target/fret-diag/ui-layout-dirty-breadth-data-table-v1-after-view-cache-2026-05-15-cargo/1778803966510-ui-gallery-data-table-view-cache-filter-shrink-vlist-inputs-change/bundle.schema2.json`,
  total/layout/prepaint/paint = `108829/94957/1019/12853us`

Interpretation:

- The policy slice removed data-table filter `Input` RAF churn:
  `debug.element_runtime.continuous_frame_leases = 0` and
  `debug.element_runtime.animation_frame_request_roots = 0` in the after view-cache bundle.
- The dominant view-cache layout cost remained. Slow frames still had
  `layout.nodes ~= 1030-1081` and repeated `ViewCache` layout invalidation, so Input chrome motion
  was a real churn source but not the main dirty-breadth source.
- The retained multi-sort after run had a noisy single large layout frame and command availability
  work; treat it as follow-up attribution, not proof that the Input policy slice regressed sort.

## After Slice B - Data-Table Page Content Cache Containment

Change:

- `apps/fret-ui-gallery/src/spec.rs`: data-table torture and retained-table torture pages now opt
  their whole-page content cache into `contain_layout_when_bounds_known(true)`, matching the
  editor-grade page policy already used by code-editor torture pages.
- This is proof-surface metadata, not a `crates/fret-ui` contract change: the runtime default
  remains explicit/author-controlled.

Validation:

- `cargo fmt --check`
- `cargo test -p fret-ui-gallery --features gallery-dev editor_grade_pages_use_boundary_layout_containment_hint -- --nocapture`
- `cargo build -p fretboard-dev -p fret-ui-gallery --release --features gallery-dev`
- view-cache suite:
  `target/fret-diag/ui-layout-dirty-breadth-data-table-v1-after-containment-2026-05-15/suite.summary.json`

After-slice view-cache bundle:

- `target/fret-diag/ui-layout-dirty-breadth-data-table-v1-after-containment-2026-05-15/1778805078239-ui-gallery-data-table-view-cache-filter-shrink-vlist-inputs-change/bundle.schema2.json`
- total/layout/prepaint/paint = `65056/57725/692/6639us`
- vs current-lane baseline `107617/94075/990/12552us`: total decreased by about `39.6%`, layout
  decreased by about `38.6%`.
- vs Slice A `108829/94957/1019/12853us`: total decreased by about `40.2%`, layout decreased by
  about `39.2%`.
- slow frames now show outer page content cache root as `layout_dependency:
  contained_when_bounds_known` with `contained_relayout_in_frame: true`; `layout_invalidated`
  root expansion is no longer attributed to a parent-dependent page cache.
- `debug.element_runtime.continuous_frame_leases = 0` and
  `debug.element_runtime.animation_frame_request_roots = 0`.

Interpretation:

- The large win came from correcting the proof-surface boundary policy: data-table torture pages
  have fixed pane bounds and local interaction state, so their whole-page cache should not be
  parent-dependent.
- The remaining dominant cost is inside the contained data-table subtree: slow frames still layout
  roughly `1066-1071` nodes, and the inner table cache can still be rejected by `cache_key_mismatch`
  or `layout_invalidated` on filter changes.
- Further reduction should target the table subtree itself: retained/view-cache row structure,
  virtual-list window updates, and table model observation granularity.

## Remaining Contained-Subtree Attribution

After Slice B, the slow frames are no longer dominated by parent-dependent page-cache breadth:

- `debug.element_runtime.view_cache_key_mismatch_roots_count = 0` in the after-containment slow
  frames inspected.
- The main remaining invalidation group is
  `structural_children_changed|hit_test|other`, with `698` invalidation walks in the
  after-containment bundle.
- Most structural samples point into `ecosystem/fret-ui-kit/src/declarative/table.rs` row/cell
  construction paths (`7281`, `7481`, `7714`, `7770`, `7787`, `7866`, `7888`) and the UI builder
  wrappers they emit.

Interpretation:

- This is likely the legitimate cost of changing the filtered row/window membership plus rebuilding
  row/cell children in the contained table subtree.
- The next worthwhile fearless refactor is narrower than the first two slices: reduce row/cell
  structural churn inside `fret-ui-kit` table rendering, or prove the current retained/view-cache
  row rebuild is the correct cost for this interaction.

## After Slice C - Mount-Time Redundant Structural Walk Fastpath

Change:

- `crates/fret-ui/src/tree/ui_tree_mutation/mount.rs`: `set_children_in_mount` now detects the
  initial mount shape where the parent is detached, has no existing children, and is already dirty
  for layout, paint, and hit-test. That path keeps command availability and semantics dirty, updates
  subtree layout dirty counts, and skips the redundant structural invalidation walk.
- `crates/fret-ui/src/tree/tests/children.rs`: added
  `set_children_in_mount_new_dirty_detached_parent_skips_redundant_structural_walk`.

Validation:

- `cargo test -p fret-ui --lib set_children_in_mount_new_dirty_detached_parent_skips_redundant_structural_walk -- --nocapture`
- view-cache suite:
  `target/fret-diag/ui-layout-dirty-breadth-data-table-v1-after-mount-fastpath-2026-05-15/suite.summary.json`
- retained suite:
  `target/fret-diag/ui-layout-dirty-breadth-data-table-v1-after-mount-fastpath-retained-2026-05-15-cargo/suite.summary.json`

Final view-cache filter-shrink bundle:

- `target/fret-diag/ui-layout-dirty-breadth-data-table-v1-after-mount-fastpath-2026-05-15/1778807288450-ui-gallery-data-table-view-cache-filter-shrink-vlist-inputs-change/bundle.schema2.json`
- total/layout/prepaint/paint = `65614/58210/693/6711us`
- compared with after-containment `65056/57725/692/6639us`, total time is effectively flat; the
  useful change is dirty-breadth attribution, not wall-time improvement.
- the slow contained relayout frames no longer include the redundant `698` structural walk seen in
  the after-containment bundle; final slow frames show invalidation calls/nodes such as `4/108`,
  `3/63`, `2/32`, and `0/0`.

Final retained filter-shrink bundle:

- `target/fret-diag/ui-layout-dirty-breadth-data-table-v1-after-mount-fastpath-retained-2026-05-15-cargo/1778807646043-ui-gallery-data-table-retained-filter-shrink-vlist-inputs-change/bundle.schema2.json`
- total/layout/prepaint/paint = `20272/16768/1227/2277us`
- worst retained layout frame remains legitimate row/window membership work:
  `layout.nodes=803`, `layout.solve_us=3505`, `inv.calls/inv.nodes=3/93`.

Final retained multi-sort bundle:

- `target/fret-diag/ui-layout-dirty-breadth-data-table-v1-after-mount-fastpath-retained-2026-05-15-cargo/1778807647231-ui-gallery-data-table-retained-multi-sort-shift-click/bundle.schema2.json`
- total/layout/prepaint/paint = `42520/35016/1747/5757us`
- remaining expensive frames are sort/row-membership and command availability work, not a broad
  cache-boundary invalidation.

Interpretation:

- Slice C is a mechanism-owned bookkeeping fix. It does not change public boundary APIs or table
  policy, so no ADR was required.
- The final retained/view-cache proof surfaces are still layout-dominant when the row/window
  membership truly changes, but the avoidable page-cache breadth and redundant initial mount walk
  have been removed.

## Final Gates

Current local gates recorded for closeout:

- `cargo fmt --check`: passed.
- `python3 tools/check_layering.py`: passed.
- `python3 tools/check_workstream_catalog.py`: passed.
- `git diff --check`: passed.
- `cargo test -p fret-ui-shadcn --lib input_chrome_motion_can_be_disabled_for_high_frequency_controls -- --nocapture`: passed.
- `cargo test -p fret-ui-kit --lib table_virtualized_retained_header_debug_ids_click_sort_actions -- --nocapture`: passed.
- `cargo test -p fret-ui-shadcn --lib retained_data_table_header_debug_ids_sort_with_column_actions -- --nocapture`: passed.

Closeout audit:

- `docs/workstreams/ui-layout-dirty-breadth-data-table-v1/CLOSEOUT_AUDIT_2026-05-15.md`
