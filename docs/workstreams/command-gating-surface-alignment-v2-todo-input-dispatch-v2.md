# Command Gating Surface Alignment v2 — TODO Tracker (input-dispatch-v2 worktree)

Status: Active (branch/worktree-local tracker to avoid doc ownership conflicts)

This tracker exists because menu/command gating docs on `main` may be updated in parallel by other
workstreams. Keep this file as the single source of truth for work done in the
`input-dispatch-v2` worktrees/branches; when backporting to `main`, reconcile into the main
workstream docs.

- Related: `docs/workstreams/os-menubar.md`
- Cross-cutting contracts: `docs/adr/1157-input-dispatch-phases-prevent-default-and-action-availability-v2.md`

## Tracking Format

Each TODO is labeled:

- ID: `CGSA2-{area}-{nnn}`
- Status: `[ ]` (open), `[~]` (in progress), `[x]` (done), `[!]` (blocked)

## P0 — Unify Native Menus With WindowCommandGatingService

- [x] CGSA2-osmenu-001 Native menus prefer `WindowCommandGatingService::snapshot`.
  - Goal: keep native menu enablement consistent with command palette / other surfaces that push a
    frozen gating snapshot (stack top wins).
  - Evidence anchors:
    - `crates/fret-launch/src/runner/desktop/windows_menu.rs` (`set_window_menu_bar`, `sync_command_gating_from_app`)
    - `crates/fret-launch/src/runner/desktop/macos_menu.rs` (`sync_command_gating_from_app`)

- [x] CGSA2-rt-002 Lock base-vs-stack semantics for gating snapshots.
  - Goal: `set_snapshot` updates the base snapshot without clobbering overlay-pushed snapshots.
  - Evidence anchors:
    - `crates/fret-runtime/src/window_command_gating.rs` (`setting_base_snapshot_does_not_override_stack_top`)
