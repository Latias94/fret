# Fearless Architecture Convergence v1 - Milestones

Status: Closed
Last updated: 2026-07-02

## M0 - Six-Cut Map Frozen

Exit criteria:

- The six architecture cuts are named in one owner map.
- Closed lanes are read as evidence, not silently reopened.
- The first executable cut is selected.

Status: Complete.

Evidence:

- `docs/workstreams/fearless-architecture-convergence-v1/DESIGN.md`
- `docs/workstreams/fearless-architecture-convergence-v1/TODO.md`

## M1 - Retained Surface Contract Landed

Exit criteria:

- ADR 0330 is accepted.
- `fret-ui` no longer exports retained widget authoring types by default.
- `fret-node/compat-retained-canvas` explicitly opts into the compatibility feature.
- Focused gates pass.

Status: Complete.

Evidence:

- `docs/adr/0330-retained-runtime-internal-and-compat-surface.md`
- `crates/fret-ui/src/lib.rs`
- `ecosystem/fret-node/Cargo.toml`
- `docs/workstreams/retained-public-surface-exit-v1/EVIDENCE_AND_GATES.md`

## M2 - Follow-On Lanes Split

Exit criteria:

- Node low-level adapter lane is selected or opened.
- Kit taxonomy lane is opened.
- Overlay/focus/dismissal oracle lane is selected or opened.
- Frame Pipeline v2 follow-on is opened for one explicit phase-contract proof.
- Launch root-surface convergence posture is assigned to maintenance or a follow-on.

Status: Complete.

Evidence:

- `docs/workstreams/fret-node-low-level-adapter-v1/WORKSTREAM.json`
- `docs/workstreams/fret-ui-kit-taxonomy-boundaries-v1/WORKSTREAM.json`
- `docs/workstreams/ui-overlay-focus-dismissal-oracle-v1/WORKSTREAM.json`
- `docs/workstreams/ui-frame-pipeline-v2-phase-contract-followon-v1/WORKSTREAM.json`
- `docs/workstreams/fret-launch-root-surface-convergence-v1/WORKSTREAM.json`

## M3 - Coordinator Closeout

Exit criteria:

- Every cut has an owner lane and a first validation command.
- The coordinator handoff names the next follow-on entry points.
- `WORKSTREAM.json` status is updated to maintenance or closed.

Status: Complete.

Evidence:

- `docs/workstreams/fearless-architecture-convergence-v1/CLOSEOUT_AUDIT_2026-07-02.md`
- `docs/workstreams/fearless-architecture-convergence-v1/WORKSTREAM.json`
- `docs/workstreams/fearless-architecture-convergence-v1/HANDOFF.md`

## M4 - 2026 UI Framework Convergence Closed

Exit criteria:

- U1-U9 from `docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md` have
  implementation evidence or an explicit deferred owner.
- Source-policy, layering, consumption-profile, perf-baseline, scaffold, text-budget, and focused
  runtime gates have current evidence.
- Retained compatibility paths are named with a reason and future gate.

Status: Complete with explicit follow-ons.

Evidence:

- `docs/workstreams/fearless-architecture-convergence-v1/CLOSEOUT_AUDIT_2026-07-02.md`
- `docs/knowledge/engineering/current-state.md`
