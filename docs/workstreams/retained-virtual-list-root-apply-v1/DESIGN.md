# Retained VirtualList Root Apply v1

Status: Active
Last updated: 2026-06-16

## Scope

This lane owns the next narrow performance follow-on after retained data-table row/cell structural
churn stopped being the measured owner. The current hotspot is retained `VirtualList` layout/root
application under the retained data-table filter-shrink repro, with the first-pass child layout
path of fixed/known-height retained windows now clearly owning the worst frame.

In scope:

- `crates/fret-ui` retained `VirtualList` reconciliation and layout-root application.
- Barrier root application and relayout scheduling when retained window membership changes.
- Diagnostics that explain retained `VirtualList` layout phase costs, including first-pass child
  layout and corrected-content relayout behavior.
- Focused `crates/fret-ui` retained VirtualList harness tests and retained data-table perf gates.

Out of scope:

- shadcn recipe visual or interaction policy changes.
- Table-local row/cell wrapper cleanup unless a fresh node profile moves ownership back there.
- Broad `ViewCache`, renderer, or frame-pipeline rewrites.
- A general fixed-track layout primitive until profiling proves root apply is not the first owner.

## Why This Lane Exists

The retained data-table shared-row-transform slice moved the worst retained frame from clearly over
budget to near the 120Hz frame budget. After that, pruning duplicate row key handlers was a safe
cleanup, but the latest profiling pass shows the remaining layout-heavy frame is owned by retained
`VirtualList` child layout under the content viewport scroll shell. `Scroll` still pays for the
root-apply/barrier side, but the first-pass child subtree is the real hotspot.

This is a mechanism-layer concern. ADR 0175 defines windowed virtual surfaces as a runtime contract,
and ADR 0177 gives retained hosts responsibility for keyed child-subtree lifecycle, attach/detach
churn, keep-alive reuse, and diagnostics. Asking every table/tree/list recipe to hand-own that work
would make the recipe interface shallow and spread retained-host complexity across callers.

## Assumptions-First State

- Area: follow-on split
  - Assumption: the active table lane should split this follow-on rather than continue expanding
    table-local scope.
  - Evidence:
    `docs/workstreams/ui-table-row-cell-structural-churn-v1/WORKSTREAM.json` says to split if the
    owner moves to retained VirtualList reconciliation, and the latest node profile points there.
  - Confidence: Confident.
  - Consequence if wrong: the first diagnostic pass will point back to `fret-ui-kit::table`.

- Area: interface depth
  - Assumption: retained `VirtualList` should hide more root-apply and barrier scheduling detail
    behind its interface.
  - Evidence: ADR 0177 assigns lifecycle and churn budgeting to the host; current evidence still
    reports `layout.root apply` around 9-10ms after table-local duplication was removed.
  - Confidence: Likely.
  - Consequence if wrong: the performance answer may be a new layout primitive rather than a
    retained host change.

- Area: first gates
- Assumption: existing retained VirtualList tests plus the retained data-table script are enough
  for the first slice.
- Evidence: `crates/fret-ui/src/declarative/tests/virtual_list/retained.rs`,
  `crates/fret-ui/src/declarative/tests/retained_virtual_list_reconcile_harness.rs`, and the
  retained data-table diag script already cover cache-hit reconcile and data-table integration.
  - Confidence: Likely.
  - Consequence if wrong: add a new harness fixture before changing behavior.

- Area: row height mode
  - Assumption: the next meaningful optimization target is the fixed/known-height retained
    `VirtualList` path, not measured-row variability.
  - Evidence: the current repro uses the gallery's default retained data-table mode with
    `measure_rows=false` unless `FRET_UI_GALLERY_DATA_TABLE_VARIABLE_HEIGHT` is set, and the hot
    child profile shows `VirtualList self_us=6675 total_us=8259`.
  - Confidence: Confident.
  - Consequence if wrong: we should split a measured-row-specific lane or widen this lane to cover
    measured row variability explicitly.

## Candidate Implementation Areas

1. Avoid unnecessary `set_children_barrier` relayout scheduling when retained reconcile preserves
   child membership/order and no child subtree is dirty.
2. Reduce repeated barrier-root solves for fixed/known-height retained rows when item starts and
   extents are already stable.
3. Add missing diagnostics around `layout_virtual_list_impl` phases before changing semantics if
   the current perf bundle cannot separate reconcile, barrier solve, and per-child layout cost.
4. Re-evaluate a first-class fixed-track strip/grid primitive only after retained root-apply
   attribution is precise enough to show table row geometry is the remaining owner.

## Reference Contracts

- `docs/adr/0175-prepaint-windowed-virtual-surfaces.md`
- `docs/adr/0177-retained-windowed-surface-hosts.md`
- `docs/adr/0066-fret-ui-runtime-contract-surface.md`
- `docs/workstreams/ui-table-row-cell-structural-churn-v1/EVIDENCE_AND_GATES.md`
- `docs/workstreams/retained-layout-orchestration-v1/CLOSEOUT_AUDIT_2026-05-18.md`

## Source Anchors

- `crates/fret-ui/src/declarative/mount.rs`
- `crates/fret-ui/src/declarative/host_widget/layout/scrolling.rs`
- `crates/fret-ui/src/tree/ui_tree_mutation/barrier.rs`
- `crates/fret-ui/src/widget.rs`
- `crates/fret-ui/src/declarative/tests/virtual_list/retained.rs`
- `crates/fret-ui/src/declarative/tests/retained_virtual_list_reconcile_harness.rs`
