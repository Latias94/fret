# imui stack fearless refactor v2 - milestones

Tracking doc: `docs/workstreams/imui-stack-fearless-refactor-v2/DESIGN.md`

TODO board: `docs/workstreams/imui-stack-fearless-refactor-v2/TODO.md`

Baseline audit: `docs/workstreams/imui-stack-fearless-refactor-v2/BASELINE_AUDIT_2026-03-31.md`

Teaching-surface audit:
`docs/workstreams/imui-stack-fearless-refactor-v2/TEACHING_SURFACE_AUDIT_2026-03-31.md`

Closeout audit:
`docs/workstreams/imui-stack-fearless-refactor-v2/CLOSEOUT_AUDIT_2026-03-31.md`

This file is forward-looking only.
Earlier `imui` closure lanes remain valuable as audit history, but the milestones below describe
the recommended execution order for the next fearless pass.

## Phase A - Documentation reset and execution-surface handoff

Status: Completed

Goal:

- establish one current `imui` source of truth,
- stop older active notes from teaching stale gaps,
- and freeze the current baseline before code changes begin.

Deliverables:

- a new v2 directory with `DESIGN.md`, `TODO.md`, `MILESTONES.md`, and a baseline audit,
- top-level docs entrypoints that point to the v2 lane,
- older active `imui` notes clearly marked as historical or partially superseded.

Exit gates:

- contributors can find the current `imui` plan without reading multiple older lanes first,
- the repo has one clear answer to "what is the current immediate-mode source of truth?",
- and stale gap statements no longer read like active backlog.

## Phase B - Editor adapter closure freeze

Status: Completed

Goal:

- finish the thin adapter coverage for editor-owned immediate nouns,
- and make the missing-vs-intentionally-declarative boundary explicit.

Planned closure set:

- `FieldStatusBadge`
- `GradientEditor`
- `PropertyRow` decision

Deliverables:

- updated `ecosystem/fret-ui-editor/src/imui.rs`,
- focused adapter-policy and compile-smoke coverage,
- explicit rationale for any editor noun that remains declarative-only.

Exit gates:

- the official editor immediate surface is not missing obvious editor-owned nouns,
- new adapters remain one-hop forwarders,
- and no adapter-local state machine or duplicate widget logic is introduced.

## Phase C - Proof/demo migration

Status: Completed

Goal:

- make first-party immediate proof surfaces follow the intended adapter boundary,
- and remove direct lower-layer editor calls from the immediate side where they are no longer
  justified.

Deliverables:

- immediate-side `imui_editor_proof_demo` call sites migrated to promoted adapters,
- declarative comparison surfaces kept explicit and separate,
- stable `test_id` and proof anchors retained after migration.

Exit gates:

- first-party immediate examples no longer teach direct declarative bypasses where an official
  adapter exists,
- the proof surface remains runnable and reviewable,
- and the adapter boundary is visible in real code, not only in docs.

## Phase D - Delete-ready cleanup and closeout

Status: Completed

Goal:

- remove stale overlap,
- verify the owner split after the migration,
- and close the lane with a delete-ready summary.

Deliverables:

- updated docs/evidence after the code-moving phase,
- no stale missing-gap statements for already-shipped helpers,
- source-policy gates that keep active first-party `imui` teaching surfaces on the current facade
  and adapter story,
- one final audit capturing survived/promoted/declarative-only/deleted outcomes.

Exit gates:

- the codebase can delete overlap confidently without reopening ownership questions,
- the remaining `imui` surfaces have one clear owner each,
- active first-party teaching surfaces do not resurrect deleted names or contract-only seam
  modules,
- and this lane can become the historical closeout record for the next follow-on only if fresh
  evidence appears.

Closeout result (2026-03-31):

- The lane is now closed with a final survive/promote/declarative-only/delete audit.
- No active first-party `imui` teaching surface currently bypasses the official editor adapter
  layer where an adapter exists, and no active example teaches deleted historical helper names.
- The next follow-on should open a new lane only if new evidence appears, not by reopening this one
  as a vague backlog bucket.

## Recommended execution order

1. Finish Phase A fully before landing code changes that depend on the new lane.
2. Use Phase B to close the missing editor adapter set and make the `PropertyRow` decision once.
3. Use Phase C to migrate proof/demo call sites immediately after the adapter closure lands.
4. Use Phase D to delete stale overlap, lock the gates, and capture the final audit.
