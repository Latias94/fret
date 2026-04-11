# Device-Shell Recipe Wrapper Surface v1 — TODO

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

- [x] DSRW-001 Record why this is a new narrow follow-on instead of reopening a closed lane.
- [x] DSRW-002 Audit current wrapper candidates and non-wrapper proof surfaces.
- [x] DSRW-003 State the smallest landable slice before considering new public APIs.

## M1 — Landing

- [x] DSRW-010 Keep `Combobox` as the current recipe-owned wrapper exemplar.
- [x] DSRW-011 Align the existing `Combobox` wrapper internals with the shared `device_shell_*`
  helper owner.
- [x] DSRW-012 Add a focused source gate that keeps wrapper, app-local, and app-shell boundaries
  explicit.
- [x] DSRW-013 Update repo entrypoint docs to point at this closeout record.

## M2 — Closeout

- [x] DSRW-020 Close the lane with explicit "no new wrapper growth yet" follow-on policy.
