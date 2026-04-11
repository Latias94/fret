# M0 Baseline Audit — 2026-04-11

Status: closed baseline note

Related:

- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `WORKSTREAM.json`
- `docs/workstreams/device-shell-strategy-surface-v1/TARGET_INTERFACE_STATE.md`
- `docs/workstreams/device-shell-strategy-surface-v1/CLOSEOUT_AUDIT_2026-04-11.md`

## Findings

The promotion trigger is already satisfied before this lane starts:

1. `device-shell-strategy-surface-v1` closed with two real helper consumers
2. `TARGET_INTERFACE_STATE.md` already says promotion can happen after that threshold
3. `fret::adaptive` is already the explicit app-facing adaptive lane
4. default app/component preludes already intentionally exclude adaptive helper nouns

## Immediate implication

This follow-on does not need another owner-split audit.

It only needs to answer one narrow question:

- should the already-shipped `device_shell_*` helper now be promoted from the owner crate
  (`fret-ui-kit`) onto the explicit app-facing facade lane (`fret::adaptive`)?

## Baseline boundary

Even if the answer is yes, these constraints stay fixed:

- `fret-ui-kit` remains the real owner
- `fret::adaptive` only re-exports
- `fret::app::prelude::*` stays unchanged
- `fret::component::prelude::*` stays unchanged
