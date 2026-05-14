# UI Prepaint Derived Surfaces v1 - Goal Handoff

Date: 2026-05-14
Status: Historical handoff; superseded by `CLOSEOUT_AUDIT_2026-05-15.md`.

Status note (2026-05-15): this handoff recorded the goal setup state before the final execution
slice. The lane is now closed. Use `WORKSTREAM.json` and
`CLOSEOUT_AUDIT_2026-05-15.md` as the current source of truth. The assumptions below remain useful
as historical context, but their M1/M2 "pending" wording has been superseded.

## Assumptions

- Area: lane status
  - Assumption: this is an active follow-on lane, not a reopening of
    `ui-frame-pipeline-v2-fearless-refactor-v1`.
  - Evidence: `WORKSTREAM.json`,
    `docs/workstreams/ui-frame-pipeline-v2-fearless-refactor-v1/FINAL_CLOSEOUT_AUDIT_2026-05-14.md`.
  - Confidence: Confident.
  - Consequence if wrong: new work could accidentally widen a closed historical lane.

- Area: ADR posture
  - Assumption: no new ADR is required before the next goal starts.
  - Evidence: ADR 0327 already accepts `ViewBoundaryState` as the retained runtime boundary owner
    for layout dependency metadata, prepaint outputs, scene fragments, paint-cache metadata, and
    boundary diagnostics.
  - Confidence: Confident.
  - Consequence if wrong: a hard contract change could land without the ADR/alignment update that
    reviewers need.

- Area: M1 state
  - Assumption: the retained virtual-list mechanism slice has focused correctness evidence, but M1
    is not closed until the selected perf/stats gate is rerun and recorded.
  - Evidence: `TODO.md`, `MILESTONES.md`, `EVIDENCE_AND_GATES.md`.
  - Confidence: Confident.
  - Consequence if wrong: the lane could overstate progress and skip the threshold evidence needed
    for a performance refactor.

- Area: M2 state
  - Assumption: the view-cache data-table torture repro passed with an earlier `gallery-dev` build,
    the retained data-table blocker has been fixed, and the remaining M2 item is to rerun the
    view-cache comparison suite against the same implementation state.
  - Evidence:
    `target/fret-diag/ui-prepaint-derived-surfaces-v1-data-table-view-cache-torture-current-dev/1778762426810-ui-gallery-data-table-view-cache-filter-shrink-vlist-inputs-change/bundle.schema2.json`,
    `target/fret-diag/ui-prepaint-derived-surfaces-v1-data-table-retained-current-dev-cargo-run/1778762568416-script-step-0022-assert-failed/bundle.schema2.json`,
    `target/fret-diag/ui-prepaint-derived-surfaces-v1-data-table-retained-after-sort-anchor-split-suite/suite.summary.json`.
  - Confidence: Confident.
  - Consequence if wrong: the next goal could either chase a stale retained-table failure or skip the
    same-state view-cache comparison needed for a clean closeout.

- Area: ownership boundary
  - Assumption: mechanism-level reusable state belongs in `crates/fret-ui`, while table policy and
    recipe behavior stay in `ecosystem/fret-ui-kit` / `ecosystem/fret-ui-shadcn`.
  - Evidence: `DESIGN.md`, `docs/architecture.md`, `docs/repo-structure.md`.
  - Confidence: Confident.
  - Consequence if wrong: the refactor could blur Fret's mechanism-vs-policy boundary.

## ADR Trigger Conditions

Create a new ADR, or update ADR 0327 plus `docs/adr/IMPLEMENTATION_ALIGNMENT.md`, only if a slice:

- changes the renderer `Scene` recording contract;
- moves `PreviousFramePaintRecording` into a per-boundary owner;
- changes public boundary hint APIs;
- changes the accepted retained ownership of `ViewCacheBuildBoundaryStore` or
  `UiTree::retained_paint_cache_entries`;
- changes diagnostics schema in a way external tooling must treat as a new contract.

## Goal-Ready Completion Target

The next user goal should be considered complete only when:

- retained virtual-list and retained data-table/view-cache proof surfaces both pass their selected
  correctness gates;
- worst-bundle attribution is recorded for both proof surfaces;
- derived prepaint/scene-fragment state is boundary-owned or explicitly retained with current
  evidence;
- diagnostics explain boundary reuse, rejection, dirty causes, and virtual-list window decisions;
- old local duplicate caches or transitional carriers touched by the lane are deleted, narrowed, or
  explicitly retained with a written reason;
- `cargo fmt --check`, focused `cargo nextest`, `cargo check`, `python3 tools/check_layering.py`,
  `python3 tools/check_workstream_catalog.py`, and selected diag/perf gates pass;
- the lane ends with a closeout audit that records final architecture, performance evidence,
  retained mechanisms, and follow-ons.

## Suggested Goal Wording

Continue `docs/workstreams/ui-prepaint-derived-surfaces-v1` until the retained virtual-list and
retained data-table/view-cache derived-surface fearless refactor is complete. Track progress in the
lane docs after each landable slice. Migrate only reusable mechanism state into `crates/fret-ui`,
keep component policy in `ecosystem/*`, delete redundant old paths after gates, update ADR/alignment
only if hard contracts change, and close the lane with correctness, perf, attribution, layering, and
deletion-audit evidence.

## First Next Slices

1. Rerun `ui-gallery-data-table-view-cache-torture` against the same implementation state used by
   the passing retained data-table suite, then record the comparison bundle and attribution result.
2. Close the M1 virtual-list perf/stats gate and record whether the current macOS evidence is
   attribution-only or backed by a seeded platform baseline.
3. Run the final focused cargo, layering, workstream catalog, and formatting gates.
4. Add a closeout audit that maps the goal's requirements to implementation files, diagnostics
   bundles, retained mechanisms, deleted/narrowed old paths, and follow-ons.
