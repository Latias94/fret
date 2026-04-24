# ImUi Collection Helper Readiness v1 - Milestones

Status: active narrow audit lane
Last updated: 2026-04-24

## M0 - Lane Opened

Status: complete

- Created `WORKSTREAM.json`, `DESIGN.md`, `TODO.md`, `MILESTONES.md`, and `EVIDENCE_AND_GATES.md`.
- Kept `imui-collection-second-proof-surface-v1` closed.
- Named the first source-policy gate for helper-readiness posture.

## M1 - Candidate Seam Audit

Status: planned

Goal: compare the two existing collection proof surfaces and name candidate helper seams without
implementing a public helper.

Exit criteria:

- each candidate names both proof surfaces,
- each candidate states what policy stays app-owned,
- and rejected candidates explain why app-owned code is still better.

## M2 - Verdict Or Split

Status: planned

Goal: either close this lane with a no-helper-widening verdict or split a separate implementation
follow-on for one exact helper shape.
