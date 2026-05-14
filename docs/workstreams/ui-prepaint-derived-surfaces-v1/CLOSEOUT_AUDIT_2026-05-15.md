# UI Prepaint Derived Surfaces v1 - Closeout Audit

Date: 2026-05-15
Status: Closed

## Objective

Complete the `ui-prepaint-derived-surfaces-v1` fearless refactor by using retained virtual-list and
retained data-table/view-cache torture surfaces as the first proof surfaces. Reusable derived
prepaint, scene-fragment, and cache metadata should converge on the `ViewBoundaryState` boundary
model where appropriate; old local cache paths should be deleted or explicitly retained; correctness
gates, `diag stats`, perf bundles, and workstream docs must close the lane.

## Prompt-To-Artifact Checklist

- Requirement: retained virtual-list proof surface.
  - Evidence:
    `crates/fret-ui/src/tree/prepaint/virtual_list.rs`,
    `crates/fret-ui/src/declarative/mount.rs`,
    `crates/fret-ui/src/declarative/tests/virtual_list/retained.rs`,
    `crates/fret-ui/src/tree/prepaint/tests/prepaint_virtual_list_window_update_harness.rs`.
  - Gates:
    `cargo nextest run -p fret-ui retained_virtual_list_host_updates_window_without_rerendering_view_cache_root --no-fail-fast`;
    `cargo nextest run -p fret-ui retained_virtual_list_keep_alive_reuses_detached_items_when_scrolling_back mechanism_harness_retained_virtual_list_reconcile_matches_oracles mechanism_harness_prepaint_virtual_list_window_update_matches_oracles --no-fail-fast`.
  - Status: complete.

- Requirement: retained data-table/view-cache torture proof surfaces.
  - Evidence:
    `ecosystem/fret-ui-kit/src/declarative/table.rs`,
    `ecosystem/fret-ui-shadcn/src/data_table.rs`,
    `ecosystem/fret-ui-shadcn/src/data_table_controls.rs`,
    `ecosystem/fret-ui-shadcn/src/data_table_recipes.rs`,
    `apps/fret-ui-gallery/src/ui/previews/gallery/data/table_torture.rs`,
    `tools/diag-scripts/suites/ui-gallery-data-table-retained/suite.json`,
    `tools/diag-scripts/suites/ui-gallery-data-table-view-cache-torture/suite.json`.
  - Gates:
    `target/release/fretboard-dev diag suite ui-gallery-data-table-retained ...` passed all 12
    scripts;
    `target/release/fretboard-dev diag suite ui-gallery-data-table-view-cache-torture ...` passed.
  - Status: complete.

- Requirement: migrate reusable derived prepaint state into `ViewBoundaryState`.
  - Evidence:
    `VirtualListPrepaintWindowOutput` in `crates/fret-ui/src/tree/prepaint/mod.rs`;
    `prepaint_virtual_list_window_from_interaction_record` stores the output from
    `crates/fret-ui/src/tree/prepaint/virtual_list.rs`;
    retained reconcile reads the boundary-owned output in `crates/fret-ui/src/declarative/mount.rs`.
  - Status: complete for the measured reusable M1 derived window output.

- Requirement: keep reusable scene-fragment/cache metadata aligned with the boundary model.
  - Evidence:
    `ViewBoundaryState` owns `prepaint`, `scene_fragment`, and `paint_cache` state in
    `crates/fret-ui/src/tree/view_boundary.rs`;
    `canvas_scene_fragment_is_boundary_owned_and_keyed_by_prepaint_key` verifies boundary-owned
    scene fragments in `crates/fret-ui/src/declarative/tests/canvas.rs`;
    `view_cache_allows_paint_cache_for_boundary_nodes` and the paint-cache tests verify
    view-cache boundary nodes store replay entries in `ViewBoundaryState`.
  - Status: complete. This lane did not need to change renderer `Scene` recording ownership because
    Frame Pipeline v2 had already moved reusable boundary scene/cache metadata into
    `ViewBoundaryState`; this lane extended that model with the virtual-list prepaint output and
    kept retained non-boundary stores explicit.

- Requirement: delete, narrow, or explicitly retain old local cache/state paths.
  - Narrowed:
    `VirtualListState.window_range` / `render_window_range` remain fallback and layout/render bridge
    state; retained reconcile validates and prefers the boundary-owned prepaint output first.
  - Explicitly retained:
    `VirtualListState` remains element-local because it owns declarative render/layout bridge state,
    row metrics, key cache, scroll bookkeeping, and layout scratch.
    `RetainedVirtualListKeepAliveState` remains in `WindowElementState` because it owns detached
    live `NodeId`s and reuse order, not geometry-derived prepaint output.
    `ViewCacheBuildBoundaryStore`, `UiTree::retained_paint_cache_entries`, and
    `PreviousFramePaintRecording` remain as accepted Frame Pipeline v2 retentions.
  - Status: complete.

- Requirement: correctness gates.
  - Evidence: all focused cargo/nextest gates and the retained/view-cache diag suites passed on
    2026-05-15.
  - Status: complete.

- Requirement: `diag stats` and perf bundles.
  - Virtual-list worst bundle:
    `target/fret-diag/1778777905741/bundle.schema2.json`, with total/layout/prepaint/paint =
    `14833/11423/315/3095us`.
  - Same-state view-cache bundle:
    `target/fret-diag/ui-prepaint-derived-surfaces-v1-data-table-view-cache-2026-05-15-after-scroll-wait/1778777531409-ui-gallery-data-table-view-cache-filter-shrink-vlist-inputs-change/bundle.schema2.json`,
    with total/layout/prepaint/paint = `105023/91918/949/12156us`.
  - Same-state retained filter-shrink bundle:
    `target/fret-diag/ui-prepaint-derived-surfaces-v1-data-table-retained-2026-05-15-after-scroll-wait/1778777573643-ui-gallery-data-table-retained-filter-shrink-vlist-inputs-change/bundle.schema2.json`,
    with total/layout/prepaint/paint = `19351/15941/1027/2383us`.
  - Same-state retained multi-sort bundle:
    `target/fret-diag/ui-prepaint-derived-surfaces-v1-data-table-retained-2026-05-15-after-scroll-wait/1778777574739-ui-gallery-data-table-retained-multi-sort-shift-click/bundle.schema2.json`,
    with total/layout/prepaint/paint = `39836/32667/1862/5307us`.
  - Status: complete.

- Requirement: workstream docs track progress and closeout.
  - Evidence:
    `WORKSTREAM.json`, `DESIGN.md`, `TODO.md`, `MILESTONES.md`, `EVIDENCE_AND_GATES.md`, and this
    closeout audit.
  - Status: complete.

## Final Architecture

The lane keeps the Frame Pipeline v2 direction intact:

```text
component/recipe policy
  -> stable boundary identity and hints
  -> boundary-owned reusable prepaint output
  -> retained reconcile / paint consumes validated current boundary state
  -> diagnostics explain reuse, rejection, dirty cause, and window decisions
```

The concrete M1 migration is `VirtualListPrepaintWindowOutput`. It is written during prepaint and
owned by the relevant view boundary. Retained virtual-list reconcile now treats that output as the
authoritative prepaint-derived geometry when it validates against current props, viewport, scroll
offset, and visible range. Element-local `VirtualListState` remains necessary, but no longer has to
be the only carrier for the derived prepaint window.

The M2 data-table fixes deliberately stay in policy/component layers. The retained table row-order
path now uses original column definitions for filter/sort closures instead of constrained sizing
columns. Recipe and toolbar sync paths update only when values actually change. The gallery reset
harness increments a keyed epoch so suite scripts start from clean toolbar-local state. Header sort
anchors and column-action accessories are separated so scripted header clicks do not open menus.

## Performance Interpretation

The measured proof surfaces are layout-dominant:

- virtual-list worst frame: layout `11423us` of `14833us` total;
- view-cache data-table filter shrink: layout `91918us` of `105023us` total;
- retained data-table filter shrink: layout `15941us` of `19351us` total;
- retained multi-sort: layout `32667us` of `39836us` total.

That matches the refactor expectation: moving derived prepaint output into `ViewBoundaryState` is a
correctness and ownership migration first. It gives later layout/dirty-breadth optimizations a
stable owner and diagnostics surface, but it does not by itself make prepaint the dominant cost.

## ADR Decision

No new ADR is required.

This lane applies ADR 0327's accepted `ViewBoundaryState` ownership model to additional proof
surfaces. It does not change:

- renderer `Scene` recording ownership;
- `PreviousFramePaintRecording` ownership;
- public boundary hint APIs;
- `ViewCacheBuildBoundaryStore` or `UiTree::retained_paint_cache_entries` ownership;
- externally consumed diagnostics schema.

## Final Gates

Passed on 2026-05-15:

- `cargo fmt --check`
- `cargo test -p fret-ui-gallery --test virtual_list_perf_surface -- --nocapture`
- `cargo test -p fret-ui-kit --lib table_virtualized_retained_header_debug_ids_click_sort_actions -- --nocapture`
- `cargo test -p fret-ui-shadcn --lib retained_data_table_header_debug_ids_sort_with_column_actions -- --nocapture`
- `cargo nextest run -p fret-ui retained_virtual_list_host_updates_window_without_rerendering_view_cache_root --no-fail-fast`
- `cargo nextest run -p fret-ui retained_virtual_list_keep_alive_reuses_detached_items_when_scrolling_back mechanism_harness_retained_virtual_list_reconcile_matches_oracles mechanism_harness_prepaint_virtual_list_window_update_matches_oracles --no-fail-fast`
- `cargo check -p fret-ui --all-targets`
- `cargo check -p fret-ui-kit --all-targets`
- `cargo check -p fret-ui-shadcn --all-targets`
- `cargo check -p fret-ui-gallery --features gallery-dev --all-targets`
- `python3 tools/check_layering.py`
- `python3 tools/check_workstream_catalog.py`
- `python3 -m json.tool docs/workstreams/ui-prepaint-derived-surfaces-v1/WORKSTREAM.json >/dev/null`
- `git diff --check`

## Follow-Ons

- Layout dirty-breadth reduction for data-table/view-cache interaction frames.
- Command availability evaluation attribution if the large eval-time snapshots persist under a
  dedicated command-routing workstream.
- Renderer `Scene` display-list evolution only in a separate ADR-backed lane.
- Per-boundary previous-frame recording only in a separate ADR-backed lane.
- Platform-specific perf baselines, especially a macOS baseline if this proof surface should become
  threshold-gated on Apple Silicon.
