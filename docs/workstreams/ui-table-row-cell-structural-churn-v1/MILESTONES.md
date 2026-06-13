# Milestones: UI Table Row/Cell Structural Churn v1

Status: Active
Last updated: 2026-06-13

## M0 - Baseline Attribution

Status: In progress; first implementation owner classified from the existing worst-bundle evidence.

Done criteria:

- The current data-table and virtual-list general-app bundles are recorded.
- A fresh data-table run with layout node profiling identifies the hottest row/cell roots.
- The first owner is classified before code changes.

Current evidence:

- data-table view-cache/filter/vlist worst bundle:
  `target/fret-general-app-perf/data-table-view-cache-filter-shrink-r3/1781333281689/bundle.schema2.json`
- virtual-list torture worst bundle:
  `target/fret-general-app-perf/virtual-list-torture-steady-r3/1781332244223/bundle.schema2.json`
- First implementation owner:
  default non-retained data-table column group structure. The unpinned path had no left/right
  columns but still built empty left/right groups and an outer grouping row around the center
  group.

## M1 - First Reversible Table-Local Slice

Status: First slice landed; before/after perf read pending.

Done criteria:

- One measured row/cell structural owner is changed or explicitly rejected.
- The change is table-local unless the evidence proves a mechanism-level owner.
- Focused table correctness gates pass.

Potential owners:

- row `Pressable` roots in `table.rs`;
- per-row pinned/center/right group wrappers;
- cell content wrapper construction;
- row and cache key stability under filter/sort;
- fixed-height row contracts not being exploited by layout/cache reuse.

Landed slice:

- `ecosystem/fret-ui-kit/src/declarative/table.rs` now renders the center group directly when
  `left_len == 0 && center_len > 0 && right_len == 0`.
- Pinned-column layouts, grouped rows, retained table rendering, and shadcn recipes keep the
  existing structural path.

## M2 - Before/After Perf Read

Status: Pending.

Done criteria:

- The same script is re-run after the first slice.
- `EVIDENCE_AND_GATES.md` records before/after p50/p95/max and the worst-bundle attribution.
- The interpretation separates real row membership work from avoidable structural churn.

## M3 - Closeout Or Follow-On Split

Status: Pending.

Done criteria:

- The lane either closes with a measured improvement / no-change verdict, or splits a narrower
  follow-on when the owner moves outside table row/cell structure.
- `WORKSTREAM.json` state and catalog entries remain valid.
