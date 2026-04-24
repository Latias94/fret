# ImUi Collection Helper Readiness v1 - Milestones

Status: closed closeout record
Last updated: 2026-04-24

## M0 - Lane Opened

Status: complete

- Created `WORKSTREAM.json`, `DESIGN.md`, `TODO.md`, `MILESTONES.md`, and `EVIDENCE_AND_GATES.md`.
- Kept `imui-collection-second-proof-surface-v1` closed.
- Named the first source-policy gate for helper-readiness posture.

## M1 - Candidate Seam Audit

Status: complete

Goal: compare the two existing collection proof surfaces and name candidate helper seams without
implementing a public helper.

Exit criteria:

- each candidate names both proof surfaces,
- each candidate states what policy stays app-owned,
- and rejected candidates explain why app-owned code is still better.

Result: `M1_CANDIDATE_SEAM_AUDIT_2026-04-24.md` keeps shared helper widening closed for M1 and
classifies the current shared pressure as documentation/test-id convention rather than a public
helper API.

## M2 - Verdict Or Split

Status: complete

Goal: either close this lane with a no-helper-widening verdict or split a separate implementation
follow-on for one exact helper shape.

Result: `CLOSEOUT_AUDIT_2026-04-24.md` closes the lane with no helper widening. No implementation
follow-on is justified from the current two proof surfaces.
