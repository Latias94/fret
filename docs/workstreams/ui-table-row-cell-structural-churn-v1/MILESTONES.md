# Milestones: UI Table Row/Cell Structural Churn v1

Status: Active
Last updated: 2026-06-22

## M0 - Baseline Attribution

Status: Completed for the current retained/shared-transform evidence set.

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
- Current retained owner:
  the retained single-center body path still modeled each visible row as a horizontal `Scroll`
  viewport even though the header and body share one horizontal scroll handle.

## M1 - First Reversible Table-Local Slice

Status: Completed for non-retained and retained single-center table paths.

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
- The non-retained single-center body path uses per-row `ScrollContentTransform` plus one shared
  horizontal scroll owner instead of one horizontal `Scroll` per row.
- The retained single-center body path now uses the same shared-X structure: one body
  `WheelRegion`, per-row `ScrollContentTransform`, and no row-local horizontal `Scroll`.
- Retained rows no longer register duplicate per-row keyboard handlers; the retained list root owns
  the single key navigation handler, and a nested-focus regression test proves bubbling still works.
- Pinned-column layouts, grouped rows, and shadcn recipes keep the existing structural path.

## M2 - Before/After Perf Read

Status: Completed for the retained data-table comparison.

Done criteria:

- The same script is re-run after the first slice.
- `EVIDENCE_AND_GATES.md` records before/after p50/p95/max and the worst-bundle attribution.
- The interpretation separates real row membership work from avoidable structural churn.

Current retained comparison:

- Before bundle:
  `target/fret-diag/vlist-retained-filter-shrink-correct-script-v1/sessions/1781528832521-146560/1781528844457-ui-gallery-data-table-retained-filter-shrink-vlist-inputs-change/bundle.schema2.json`
- After bundle:
  `target/fret-diag/vlist-retained-shared-row-xform-v1/sessions/1781530321751-126564/1781531045060-ui-gallery-data-table-retained-filter-shrink-vlist-inputs-change/bundle.schema2.json`
- Worst frame moved from `total=24856us`, `layout=23060us`,
  `layout.engine_solve=13231us`, `layout.root apply=20407us`, `layout.nodes=810`
  to `total=11715us`, `layout=10831us`, `layout.engine_solve=6599us`,
  `layout.root apply=9541us`, `layout.nodes=646`.
- Follow-up retained key-hook prune stayed in the same band on the retained-only script:
  `total=11391us`, `layout=10522us`, `layout.engine_solve=6524us`,
  `layout.root apply=9373us`, `layout.nodes=646`.

## M3 - Closeout Or Follow-On Split

Status: Follow-on split started; current owner is outside table row/cell structure.

Done criteria:

- The lane either closes with a measured improvement / no-change verdict, or splits a narrower
  follow-on when the owner moves outside table row/cell structure.
- `WORKSTREAM.json` state and catalog entries remain valid.

Current decision:

- Do not keep optimizing row-local scroll wrappers or row-local keyboard plumbing unless a fresh
  profile makes them hot again.
- Start `retained-virtual-list-root-apply-v1` for retained/root-apply breadth because the latest
  node profile attributes the owner to retained `VirtualList` and its parent `Scroll`.
- A mechanism-level fixed-track layout primitive remains a possible later split if retained
  root-apply attribution proves row geometry is the next owner.
- The 2026-06-22 `data_table_torture` `row_click_selection(false)` slice is a harness policy prune,
  not a structural table win. Keep it to isolate the direct perf script from row-body selection
  activation, then wait for clearer fixed-row/table-primitive evidence before extending this lane.
