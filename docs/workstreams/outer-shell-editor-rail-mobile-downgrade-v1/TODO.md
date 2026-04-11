# Outer-Shell Editor Rail Mobile Downgrade v1 — TODO

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

- [x] OSEMD-001 Audit the current device-shell strategy proof against the editor-rail owner split.
- [x] OSEMD-002 Confirm that the repo already has a correct explicit desktop/mobile proof pattern.

## M1 — Verdict

- [x] OSEMD-010 Decide which layer owns editor-rail mobile downgrade composition.
- [x] OSEMD-011 Decide whether a shared helper is justified yet.
- [x] OSEMD-012 Close the lane if the answer is still explicit outer-shell ownership.

## Reopen Criteria

- [x] OSEMD-020 Record what evidence would justify reopening:
  - two real editor shells sharing the same downgrade shape,
  - a repeated need for more than generic `device_shell_switch(...)`,
  - and proof that the shared part is not just app-local drawer/sheet/route policy.
