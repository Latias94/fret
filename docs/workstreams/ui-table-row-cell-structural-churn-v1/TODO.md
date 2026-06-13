# TODO: UI Table Row/Cell Structural Churn v1

Status: Active
Last updated: 2026-06-13

## M0 - Baseline And Attribution

- [x] Open a narrow follow-on instead of reopening `ui-layout-dirty-breadth-data-table-v1`.
- [x] Record the current general-app data-table and virtual-list evidence.
- [ ] Re-run the data-table view-cache/filter shrink script with layout node profiling enabled.
- [ ] Run `diag stats --sort cpu_cycles --top 30` and save the worst-frame summary.
- [x] Identify the first implementation owner as default unpinned table column group structure:
      the non-retained data-table path rendered empty left/right groups and an outer row around the
      only non-empty center group.

## M1 - First Row/Cell Slice

- [x] Pick one measured structural-churn owner and make a small reversible change.
- [x] Prefer table-local cleanup in `ecosystem/fret-ui-kit/src/declarative/table.rs` unless the
      bundle proves a generic runtime mechanism owner.
- [x] Add or adjust a focused unit test if row identity, row selection, focus, or semantics could
      regress.
- [x] Keep shadcn recipe behavior unchanged unless attribution moves to recipe-owned controls.

## M2 - Perf Evidence

- [ ] Re-run the same data-table script after the slice.
- [ ] Compare:
  - `top_total_time_us`;
  - `layout_time_us`;
  - `layout_engine_solve_time_us`;
  - `layout.nodes`;
  - `top_layout_engine_solves`;
  - `cache.reused`;
  - `contained_relayouts`.
- [ ] Record bundle paths and interpretation in `EVIDENCE_AND_GATES.md`.
- [ ] Decide whether a checked-in formal baseline is justified. Default: no baseline changes from
      local noisy evidence.

## M3 - Closeout Or Split

- [ ] If the owner remains row/cell churn, complete a second small slice or record why not.
- [ ] If the owner moves to VirtualList retained reconciliation, command availability, renderer
      payload, or public runtime semantics, split a narrower follow-on.
- [ ] Update `WORKSTREAM.json`, `MILESTONES.md`, and `EVIDENCE_AND_GATES.md`.
- [ ] Add `CLOSEOUT_AUDIT_YYYY-MM-DD.md` when the lane stops being active.
