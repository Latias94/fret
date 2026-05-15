# Design: UI Layout Dirty Breadth Data Table v1

Status: Closed
Last updated: 2026-05-15

## Problem

`ui-prepaint-derived-surfaces-v1` closed the retained virtual-list and data-table correctness
blockers, but the retained and view-cache data-table proof surfaces remain layout-dominant:

- view-cache filter shrink: layout `91918us` of `105023us`;
- retained filter shrink: layout `15941us` of `19351us`;
- retained multi-sort: layout `32667us` of `39836us`.

This lane owns the next step: attribute and reduce the breadth of layout invalidation caused by
data-table interactions such as global filter, column filter, sorting, column pinning, visibility,
and reset flows.

## Assumptions-first pass

- Area: lane status
  - Assumption: this is a new narrow follow-on, not a reopen of
    `ui-prepaint-derived-surfaces-v1`.
  - Evidence: `docs/workstreams/ui-prepaint-derived-surfaces-v1/CLOSEOUT_AUDIT_2026-05-15.md`
    lists layout dirty-breadth as a follow-on and marks the prior lane closed.
  - Confidence: Confident.
  - Consequence if wrong: new work could blur the closed derived-state ownership record.

- Area: primary bottleneck
  - Assumption: the first optimization target is layout invalidation breadth, not prepaint or
    renderer scene ownership.
  - Evidence: same closeout audit records layout as the dominant phase for retained and view-cache
    data-table bundles.
  - Confidence: Confident.
  - Consequence if wrong: a renderer or prepaint slice would optimize the wrong phase.

- Area: ownership
  - Assumption: generic dirty-cause telemetry and invalidation narrowing belong in
    `crates/fret-ui`; table interaction policy and idempotent recipe sync belong in
    `ecosystem/fret-ui-kit` / `ecosystem/fret-ui-shadcn`.
  - Evidence: `docs/adr/0066-fret-ui-runtime-contract-surface.md` keeps `fret-ui` as mechanism
    surface, while the prior closeout kept data-table policy fixes in ecosystem crates.
  - Confidence: Confident.
  - Consequence if wrong: policy could leak into the runtime or runtime mechanics could be hidden
    behind recipe-only fixes.

- Area: diagnostics
  - Assumption: existing boundary reuse/rejection diagnostics may be insufficient to explain dirty
    breadth per interaction; adding a narrow dirty-cause diagnostic is allowed if it is mechanism
    owned and documented.
  - Evidence: the objective explicitly permits boundary dirty-cause diagnostics when needed, and
    the prior closeout already relies on boundary dirty/reuse reasons.
  - Confidence: Likely.
  - Consequence if wrong: the lane should instead use existing bundle stats and node-level layout
    profiling only.

## Goals

1. Attribute layout invalidation breadth for retained and view-cache data-table interaction frames.
2. Reduce unnecessary layout invalidation or policy churn for at least one measured interaction
   class, unless attribution proves the current breadth is required and records that verdict.
3. Preserve mechanism/policy boundaries:
   - `crates/fret-ui`: invalidation bookkeeping, boundary diagnostics, layout mechanisms.
   - `ecosystem/fret-ui-kit`: headless table model and declarative table policy.
   - `ecosystem/fret-ui-shadcn`: shadcn recipes, toolbar controls, and demo-facing composition.
4. Keep retained and view-cache proof surfaces green.
5. Close the lane with correctness gates, `diag stats`, perf bundle paths, and a prompt-to-artifact
   audit.

## Non-goals

- No renderer `Scene` ownership changes.
- No public runtime API widening unless an ADR or ADR alignment update explicitly justifies it.
- No Linux-specific baseline work in this lane.
- No compatibility layer for old data-table paths; this repo accepts fearless cleanup when evidence
  supports it.

## Target architecture

The intended end state is:

```text
data-table interaction policy
  -> idempotent model/recipe state updates
  -> narrow mechanism invalidation causes
  -> boundary diagnostics explain dirty breadth
  -> layout solves only the dirty surface required for the interaction
  -> retained/view-cache suites prove behavior and perf attribution
```

The critical architectural distinction is that a table interaction may legitimately change row
order, visible rows, column geometry, or toolbar state, but these should not all imply the same broad
layout invalidation. The runtime should expose enough mechanism-level cause information to separate
"row membership changed" from "column/header geometry changed" from "toolbar-local value changed".

## Final Refactor Result

The lane landed three bounded slices:

- Component policy: high-frequency data-table filter controls opt out of decorative shadcn input
  chrome motion while ordinary `Input` keeps transition parity.
- Proof-surface cache policy: data-table torture pages use contained whole-page content cache
  boundaries when bounds are known, matching their fixed-pane editor-grade usage.
- Runtime bookkeeping: mount-time child attachment skips a redundant structural invalidation walk
  when the detached parent is already dirty on layout/paint/hit-test.

This has reduced the view-cache filter-shrink bundle from `107617/94075/990/12552us` to
`65056/57725/692/6639us`, and removed the redundant structural-walk breadth in final
mount-fastpath bundles. The remaining row/cell rebuild cost is inside the contained table subtree;
it is not reopened here because it needs a narrower `fret-ui-kit` table-subtree lane.

## ADR trigger conditions

No new ADR is required for:

- private diagnostics that refine existing boundary dirty-cause reporting;
- private invalidation bookkeeping;
- ecosystem-layer idempotence or recipe cleanup;
- additional tests and diag scripts.

Add or update an ADR if the lane changes:

- public runtime boundary APIs;
- the diagnostics schema consumed outside first-party tooling;
- `ViewBoundaryState` ownership;
- layout invalidation semantics across cache boundaries.

## Completion criteria

- `WORKSTREAM.json`, `DESIGN.md`, `TODO.md`, `MILESTONES.md`, and `EVIDENCE_AND_GATES.md` are
  current.
- Baseline and after-change bundles are recorded for retained and view-cache data-table surfaces.
- `diag stats` summaries explain the dominant phase and dirty/reuse cause.
- Correctness gates pass for retained table sort/header behavior.
- Retained and view-cache diag suites pass.
- `cargo fmt --check`, layering checks, workstream catalog checks, and relevant crate checks pass.
- A closeout audit maps every prompt requirement to concrete evidence and either closes or splits
  remaining work.
