# Milestones: UI Layout Dirty Breadth Data Table v1

Status: Closed
Last updated: 2026-05-15

## M0 - Baseline + Attribution

Status: Complete for the first optimization loop.

Done criteria:

- Retained data-table and view-cache data-table suites pass or fail with recorded bundle evidence.
- Worst retained and view-cache bundles have `diag stats --sort cpu_cycles --top 30` summaries.
- The first attribution separates:
  - required row/window membership layout;
  - column/header geometry layout;
  - toolbar or control-local layout;
  - broad boundary/cache-root invalidation.

Outcome:

- Current-lane retained and view-cache baseline suites passed with bundle paths recorded in
  `EVIDENCE_AND_GATES.md`.
- The first attribution separated:
  - legitimate retained virtual-list window membership work;
  - decorative data-table filter input chrome motion;
  - parent-dependent whole-page content cache invalidation;
  - remaining contained data-table subtree layout.

## M1 - Diagnostic Sufficiency

Status: Complete for the first optimization loop; no new diagnostics were required.

Done criteria:

- Existing diagnostics are either proven sufficient or extended with a narrow mechanism-owned
  dirty-cause signal.
- Any diagnostic addition has a focused correctness test or scripted assertion.
- No table policy enters `crates/fret-ui`.

Outcome:

- Existing `debug.cache_roots`, `debug.layout_request_build_roots`,
  `debug.invalidation_walks`, and `debug.element_runtime` fields were sufficient.
- No `crates/fret-ui` diagnostic schema or public contract was changed.

## M2 - Ecosystem Policy Churn Reduction

Status: Complete.

Done criteria:

- At least one retained/view-cache interaction avoids unnecessary state writes or model/recipe
  resyncs.
- Table behavior remains correct for sort, filter, pinning, visibility, and reset scripts.
- Focused retained table unit gates pass.

Outcome:

- Landed a data-table filter-input policy slice: shadcn `Input` keeps default chrome transition
  parity, while high-frequency data-table filters opt out of decorative border/ring motion.
- Landed a proof-surface cache policy slice: data-table torture pages use contained page content
  cache boundaries when bounds are known.
- View-cache filter-shrink improved from `107617/94075/990/12552us` baseline to
  `65056/57725/692/6639us` after containment.
- The retained table state-write audit found guarded sync paths for global filter, column filter,
  column pinning, visibility, faceted selections, pagination bounds, and output models. Reset writes
  are intentional because the control only appears when filters exist.
- Remaining row/cell rebuild cost is treated as legitimate contained-subtree work for this lane.

## M3 - Runtime Dirty-Breadth Reduction

Status: Complete.

Done criteria:

- If runtime invalidation is the measured cause, the layout invalidation path is narrowed without
  changing public contracts.
- View-cache containment and retained boundary semantics remain intact.
- Mechanism tests and layering checks pass.

Outcome:

- Landed a narrow runtime bookkeeping fastpath in `crates/fret-ui`: `set_children_in_mount` skips a
  redundant structural invalidation walk for a detached, already-dirty initial mount parent.
- The fastpath preserves command availability and semantics dirtiness while avoiding a second walk
  that did not narrow the next layout pass.
- Focused coverage: `set_children_in_mount_new_dirty_detached_parent_skips_redundant_structural_walk`.

## M4 - Final Evidence + Closeout

Status: Complete.

Done criteria:

- Baseline and final bundles are recorded for retained and view-cache proof surfaces.
- `diag stats` shows the effect of the landed change or records why no further breadth reduction is
  correct in this lane.
- Workstream docs are updated and a closeout audit maps objective requirements to evidence.
- Final gates pass:
  - formatting;
  - focused retained table tests;
  - retained/view-cache diag suites;
  - crate checks;
  - layering;
  - workstream catalog;
  - `git diff --check`.

Outcome:

- Baseline, Slice A, Slice B, and final mount-fastpath bundles are recorded in
  `EVIDENCE_AND_GATES.md`.
- Closeout audit:
  `docs/workstreams/ui-layout-dirty-breadth-data-table-v1/CLOSEOUT_AUDIT_2026-05-15.md`.
