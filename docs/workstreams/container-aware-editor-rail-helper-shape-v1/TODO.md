# Container-Aware Editor Rail Helper Shape v1 — TODO

Status: Closed
Last updated: 2026-04-11

Companion docs:

- `DESIGN.md`
- `M0_BASELINE_AUDIT_2026-04-11.md`
- `CLOSEOUT_AUDIT_2026-04-11.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `WORKSTREAM.json`

## M0 — Baseline

- [x] CAERH-001 Audit the two shell-mounted rail consumers after the previous lane closeout.
- [x] CAERH-002 Separate already-owned mechanism from still-divergent wrapper policy.

## M1 — Verdict

- [x] CAERH-010 Decide whether a shared helper owner is justified yet.
- [x] CAERH-011 Close the lane on a no-new-helper verdict if the repeated evidence is still only
  seam + inner-content reuse.

## Reopen Criteria

- [x] CAERH-020 Record what new evidence would justify reopening:
  - repeated container-aware rail behavior, not only repeated fixed-slot mounting,
  - explicit outer-shell downgrade ownership,
  - and a helper shape that is more than width/chrome sugar.
