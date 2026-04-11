# M1 Contract Freeze — 2026-04-11

Status: active decision note

Related:

- `TARGET_INTERFACE_STATE.md`
- `M0_BRANCH_SITE_AUDIT_2026-04-11.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `WORKSTREAM.json`

## Verdict

M1 can now be treated as closed.

The contract decision for this lane is:

1. shared device-shell classification continues to live on `fret::adaptive::{...}`
2. shared binary desktop/mobile branch helpers should land in `fret-ui-kit` first
3. the first helper should target repeated `Popover` / `DropdownMenu` / `Drawer` branch shapes
4. `Sidebar` remains provider/app-shell-owned and is not the template for this helper
5. no new shared helper should use generic `responsive(...)` naming

## What this unblocks

- one bounded first extraction in M2 without reopening adaptive taxonomy work
- a crate-local helper prototype in `fret-ui-kit` before any `fret` facade promotion
- focused source/gate work around repeated overlay-shell branch shapes

## What this does not reopen

- panel/container adaptive work
- editor rail ownership
- generic `children(...)` growth
- runtime mechanism changes in `crates/fret-ui`
