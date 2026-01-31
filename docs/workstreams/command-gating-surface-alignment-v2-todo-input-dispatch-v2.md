# Command Gating Surface Alignment v2 — TODO Tracker (archived worktree snapshot)

Status: Archived (historical record; canonical tracker: `docs/workstreams/input-dispatch-v2-todo.md`)

This file originated as a worktree-local tracker to avoid doc ownership conflicts while
developing Input Dispatch v2 in parallel. It is kept on `main` as a historical record of the
workstream breakdown and evidence anchors.

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
  - Notes:
    - Prefer `set_base_snapshot` / `clear_base_snapshot` for call-site clarity; `set_snapshot` remains as an alias.

- [x] CGSA2-rt-003 Clearing the base snapshot does not pop pushed overrides.
  - Goal: allow overlays to safely `push_snapshot` without being affected by `clear_snapshot` calls.
  - Evidence anchors:
    - `crates/fret-runtime/src/window_command_gating.rs` (`clear_snapshot_only_clears_base_not_pushed_overrides`)
