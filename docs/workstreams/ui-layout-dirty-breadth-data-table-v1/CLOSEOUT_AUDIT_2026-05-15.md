# Closeout Audit: UI Layout Dirty Breadth Data Table v1

Date: 2026-05-15
Status: Closed

## Objective Restated

Complete `ui-layout-dirty-breadth-data-table-v1` for data-table retained and view-cache proof
surfaces:

- attribute layout invalidation breadth for filter, sort, pinning, and reset interactions;
- reduce avoidable breadth without leaking table policy into `crates/fret-ui`;
- add boundary dirty-cause diagnostics only if existing diagnostics are insufficient;
- validate with correctness gates, diag stats, perf bundles, and workstream closeout docs.

## Prompt-To-Artifact Checklist

| Requirement | Evidence | Verdict |
| --- | --- | --- |
| Target retained/view-cache data-table proof surfaces | Retained suite `target/fret-diag/ui-layout-dirty-breadth-data-table-v1-baseline-retained-2026-05-15/suite.summary.json`; view-cache suite `target/fret-diag/ui-layout-dirty-breadth-data-table-v1-baseline-view-cache-2026-05-15/suite.summary.json` | Covered |
| Attribute layout invalidation breadth | `EVIDENCE_AND_GATES.md` sections `Current Attribution`, `After Slice B`, and `After Slice C` | Covered |
| Reduce avoidable breadth | `Input::chrome_motion(false)` in `ecosystem/fret-ui-shadcn/src/input.rs`; data-table opt-out in `ecosystem/fret-ui-shadcn/src/data_table_recipes.rs`; page cache containment in `apps/fret-ui-gallery/src/spec.rs`; mount fastpath in `crates/fret-ui/src/tree/ui_tree_mutation/mount.rs` | Covered |
| Preserve mechanism/policy layering | Policy changes stay in `ecosystem/fret-ui-shadcn`; proof-surface metadata stays in `apps/fret-ui-gallery`; runtime fastpath is mechanism-owned and table-agnostic | Covered |
| Dirty-cause diagnostics if needed | Existing `cache_roots`, `layout_request_build_roots`, `invalidation_walks`, and `element_runtime` fields were sufficient; no new schema added | Covered |
| Correctness gates | Focused unit gates recorded in `EVIDENCE_AND_GATES.md`; retained and view-cache suites passed | Covered |
| Perf bundles and diag stats | Baseline, after-containment, and after-mount-fastpath bundle paths recorded in `EVIDENCE_AND_GATES.md` | Covered |
| Workstream closeout | This document; `WORKSTREAM.json`, `TODO.md`, `MILESTONES.md`, and `EVIDENCE_AND_GATES.md` updated | Covered |

## Before / After Summary

View-cache filter shrink:

- Baseline:
  `target/fret-diag/ui-layout-dirty-breadth-data-table-v1-baseline-view-cache-2026-05-15/1778802051417-ui-gallery-data-table-view-cache-filter-shrink-vlist-inputs-change/bundle.schema2.json`
- Baseline total/layout/prepaint/paint: `107617/94075/990/12552us`
- After containment:
  `target/fret-diag/ui-layout-dirty-breadth-data-table-v1-after-containment-2026-05-15/1778805078239-ui-gallery-data-table-view-cache-filter-shrink-vlist-inputs-change/bundle.schema2.json`
- After containment total/layout/prepaint/paint: `65056/57725/692/6639us`
- Final mount-fastpath:
  `target/fret-diag/ui-layout-dirty-breadth-data-table-v1-after-mount-fastpath-2026-05-15/1778807288450-ui-gallery-data-table-view-cache-filter-shrink-vlist-inputs-change/bundle.schema2.json`
- Final total/layout/prepaint/paint: `65614/58210/693/6711us`

Retained filter shrink:

- Baseline:
  `target/fret-diag/ui-layout-dirty-breadth-data-table-v1-baseline-retained-2026-05-15/1778802016962-ui-gallery-data-table-retained-filter-shrink-vlist-inputs-change/bundle.schema2.json`
- Baseline total/layout/prepaint/paint: `20565/16861/1102/2602us`
- Final:
  `target/fret-diag/ui-layout-dirty-breadth-data-table-v1-after-mount-fastpath-retained-2026-05-15-cargo/1778807646043-ui-gallery-data-table-retained-filter-shrink-vlist-inputs-change/bundle.schema2.json`
- Final total/layout/prepaint/paint: `20272/16768/1227/2277us`

Retained multi-sort:

- Baseline:
  `target/fret-diag/ui-layout-dirty-breadth-data-table-v1-baseline-retained-2026-05-15/1778802018159-ui-gallery-data-table-retained-multi-sort-shift-click/bundle.schema2.json`
- Baseline total/layout/prepaint/paint: `41576/33729/1897/5950us`
- Final:
  `target/fret-diag/ui-layout-dirty-breadth-data-table-v1-after-mount-fastpath-retained-2026-05-15-cargo/1778807647231-ui-gallery-data-table-retained-multi-sort-shift-click/bundle.schema2.json`
- Final total/layout/prepaint/paint: `42520/35016/1747/5757us`

## Final Interpretation

The primary wall-time win came from proof-surface boundary policy: data-table torture pages have
fixed known pane bounds, so their page cache should be contained when bounds are known. That removed
the parent-dependent cache breadth and reduced the view-cache filter-shrink layout sum by about
`38.6%`.

The filter input chrome slice removed a real policy churn source by preventing high-frequency data
table filters from scheduling decorative RAF-backed border/ring tweens. It intentionally leaves
ordinary shadcn `Input` transition parity intact.

The runtime fastpath removed a table-agnostic redundant structural invalidation walk during initial
mount. The final view-cache bundle shows the same contained-layout work, but without the broad
`698` structural walk in the slow contained relayout frame.

The remaining retained/table cost is legitimate contained-subtree work for this lane: filtering and
sorting change row/window membership and row/cell children. A future optimization should be scoped
as a narrower `fret-ui-kit` table-subtree structural-churn lane rather than reopening this one.

## ADR And Boundary Decision

No new ADR was required:

- no public runtime API changed;
- no diagnostics schema consumed outside first-party tooling changed;
- no `ViewBoundaryState` ownership changed;
- no table policy entered `crates/fret-ui`.

## Final Gates

Recorded gates:

- `cargo fmt --check`
- `python3 tools/check_layering.py`
- `python3 tools/check_workstream_catalog.py`
- `git diff --check`
- `cargo test -p fret-ui-shadcn --lib input_chrome_motion_can_be_disabled_for_high_frequency_controls -- --nocapture`
- `cargo test -p fret-ui-kit --lib table_virtualized_retained_header_debug_ids_click_sort_actions -- --nocapture`
- `cargo test -p fret-ui-shadcn --lib retained_data_table_header_debug_ids_sort_with_column_actions -- --nocapture`
- retained diag suite:
  `target/fret-diag/ui-layout-dirty-breadth-data-table-v1-after-mount-fastpath-retained-2026-05-15-cargo/suite.summary.json`
- view-cache diag suite:
  `target/fret-diag/ui-layout-dirty-breadth-data-table-v1-after-mount-fastpath-2026-05-15/suite.summary.json`

## Follow-On

Open a new, narrower lane only if the next target is row/cell structural churn inside
`ecosystem/fret-ui-kit/src/declarative/table.rs`. That lane should start from retained/view-cache
row membership proof bundles and avoid widening back into page-cache or input-chrome policy.
