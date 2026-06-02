# M45 Docking Declarative Event Owner Split - 2026-06-02

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split of the declarative dock-space event owner. It
keeps `DW-P1-linux-003` open because real Wayland compositor acceptance still requires the M5
runbook on a qualifying Linux Wayland host.

## Claim

Declarative docking event orchestration now lives in a private child owner instead of remaining
inline inside `ecosystem/fret-docking/src/dock/declarative.rs`, without changing tab drag
activation, floating chrome interactions, split-handle dragging, viewport input forwarding,
overflow-menu routing, panel close behavior, or the Wayland acceptance boundary.

## Source Shape

- `ecosystem/fret-docking/src/dock/declarative.rs` keeps the managed-surface assembly shell,
  paint/layout orchestration, and delegates event routing through `handle_declarative_event(...)`.
- `ecosystem/fret-docking/src/dock/declarative/events.rs` owns:
  - internal dock-drag hover/drop/cancel event routing,
  - pointer-down orchestration for overflow menus, floating chrome, split handles, viewport
    capture, tab close press, and pending tab/group drag activation,
  - pointer-up orchestration for viewport release, floating merge-back, divider finalization,
    pending drag cleanup, and tab-close commit,
  - pointer-cancel cleanup for viewport capture, tab/floating press state, and pending drags.
- Existing private declarative child owners remain responsible for geometry, drag resolution,
  overflow policy, frame state, paint-state projection, and tear-off clamping.

## Commands Run

```powershell
cargo fmt --package fret-docking
cargo check -p fret-docking
git diff --check
```

## Results

- `cargo fmt --package fret-docking`: pass.
- `cargo check -p fret-docking`: pass.
- `git diff --check`: not run in this slice.

## Verdict

This keeps the declarative dock-space host narrower and more source-auditable while preserving the
existing docking interaction behavior. It does not close `DW-P1-linux-003`; the next true closure
event is still a dated real Linux Wayland compositor acceptance note from
`M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
