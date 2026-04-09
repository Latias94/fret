# Workstream Catalog Integrity Gate v1 — TODO

Status: Closed
Last updated: 2026-04-10

## Lane opening

- [x] WCIG-001 Open a narrow follow-on instead of widening the previous skill-owner drift lane.
- [x] WCIG-002 Record the specific README drift modes before adding a new gate.

## M0 — Baseline audit

- [x] WCIG-010 Audit the current dedicated-directory catalog, standalone-file catalog, and tracked
  count lines.
- [x] WCIG-011 Freeze the non-goal that this lane will validate curated indexes, not generate them.

## M1 — Gate contract freeze

- [x] WCIG-020 Decide which README sections and count lines the checker owns.
- [x] WCIG-021 Decide whether non-index prose links should count toward the checker.
- [x] WCIG-022 Decide where the checker should run by default.
- [x] WCIG-023 Keep full alphabetical normalization of the historical directory index out of scope
  for this lane.

## M2 — Proof surface

- [x] WCIG-030 Land the new catalog-integrity checker.
- [x] WCIG-031 Wire it into common gate entrypoints.
- [x] WCIG-032 Fix the currently detected catalog drift and rerun the gates.

## M3 — Closeout

- [x] WCIG-040 Leave one narrow workstream record with repro/gates/evidence.
- [x] WCIG-041 Update the top-level workstream index so the new lane is discoverable.
- [x] WCIG-042 Close this lane and keep broader docs automation as separate follow-ons.

## Boundaries to protect

- Do not turn this lane into a README generator.
- Do not parse every markdown link in the README files as if it were an index row.
- Do not reopen roadmap or workstream-state schema work from this lane.
- Do not leave the current README drift unfixed while landing the checker.

Completed M0 evidence:

- `docs/workstreams/workstream-catalog-integrity-gate-v1/BASELINE_AUDIT_2026-04-10.md`

Completed M1 decision:

- `docs/workstreams/workstream-catalog-integrity-gate-v1/M1_CONTRACT_FREEZE_2026-04-10.md`

Completed M2 proof:

- `docs/workstreams/workstream-catalog-integrity-gate-v1/M2_PROOF_SURFACE_2026-04-10.md`

Closeout:

- `docs/workstreams/workstream-catalog-integrity-gate-v1/CLOSEOUT_AUDIT_2026-04-10.md`
