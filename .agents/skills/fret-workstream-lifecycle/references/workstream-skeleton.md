# Workstream skeleton

Use the smallest doc set that still makes the lane executable.

## Minimum doc set

- `WORKSTREAM.json`
  - machine-readable lane status
  - authoritative docs
  - primary repro/gate/evidence
- `DESIGN.md`
  - scope
  - owning layer/crates
  - why this lane exists
- `TODO.md`
  - current checklist
  - next executable slices
- `MILESTONES.md`
  - sequence and exit criteria
- `EVIDENCE_AND_GATES.md`
  - exact gate commands
  - smallest repro target
  - evidence anchors

## Optional docs

Add only when the lane needs them:

- `TARGET_INTERFACE_STATE.md`
  - when the lane is about converging on a specific shipped surface
- `OPEN_QUESTIONS.md`
  - when unresolved decisions would otherwise be buried in chat or TODO comments
- dated audit/status notes
  - when a meaningful decision, audit, or state transition deserves a bounded record
- `CLOSEOUT_AUDIT_YYYY-MM-DD.md` or `FINAL_STATUS.md`
  - when the lane stops being an active execution surface

## Recording rules

- Refresh `WORKSTREAM.json` when lane status or first-open doc/gate state changes.
- Update `TODO.md` and `MILESTONES.md` after real progress, not every tiny command.
- Keep decision notes short and dated.
- Record commands in `EVIDENCE_AND_GATES.md`, not in ad-hoc chat summaries.
- Do not create many companion docs until the lane has a real first slice.
