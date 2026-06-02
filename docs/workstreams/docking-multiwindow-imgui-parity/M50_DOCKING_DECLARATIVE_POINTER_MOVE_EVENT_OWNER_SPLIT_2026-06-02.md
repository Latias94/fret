# M50 Docking Declarative PointerMove Event Owner Split - 2026-06-02

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split of the declarative dock-space `PointerMove`
event owner. It keeps `DW-P1-linux-003` open because real Wayland compositor acceptance still
requires the M5 runbook on a qualifying Linux Wayland host.

## Claim

Declarative docking `PointerMove` event handling now lives in a private child owner under
`ecosystem/fret-docking/src/dock/declarative/events/pointer_move.rs`, while `events.rs` remains the
event-family router for InternalDrag, pointer down/move/up, and pointer-cancel dispatch. The split
does not change viewport capture forwarding, right-button viewport drag movement tracking, divider
drag fraction updates, floating title-bar drag movement, pending panel drag activation, pending
tabs-group drag activation, floating/tab hover updates, overflow menu hover updates, or cursor
projection.

## Source Shape

- `ecosystem/fret-docking/src/dock/declarative/events.rs` declares `mod pointer_move;` and routes
  `fret_core::PointerEvent::Move` through `handle_pointer_move_event(...)`.
- `ecosystem/fret-docking/src/dock/declarative/events/pointer_move.rs` owns:
  - viewport move forwarding through `ViewportInputKind::PointerMove`,
  - right-button viewport drag movement tracking through
    `viewport_context_menu_drag_threshold`,
  - divider drag fraction updates through `drag_update_adjacent_fractions(...)`,
  - floating title-bar drag movement and hover resolution through
    `declarative_resolve_floating_title_bar_drag_target(...)`,
  - pending panel drag activation through `begin_declarative_panel_drag(...)`,
  - pending tabs-group drag activation through `begin_declarative_tabs_group_drag(...)`,
  - split-handle, floating-close/title-bar, tab, and overflow hover/cursor projection.
- `events.rs` no longer owns per-event behavior; it routes to InternalDrag, PointerDown,
  PointerMove, PointerUp, and PointerCancel child owners.

## Commands Run

```powershell
cargo fmt --package fret-docking
cargo check -p fret-docking
cargo test -p fret-docking
python -m py_compile tools\gate_imui_workstream_source.py tools\gate_docking_multiwindow_workstream_source.py
python -m json.tool docs\workstreams\docking-multiwindow-imgui-parity\WORKSTREAM.json > $null
python tools\gate_docking_multiwindow_workstream_source.py
python tools\gate_imui_workstream_source.py
python tools\check_workstream_catalog.py
git diff --check
```

## Results

- `cargo fmt --package fret-docking`: pass.
- `cargo check -p fret-docking`: pass.
- `cargo test -p fret-docking`: pass, 90 tests.
- `python -m py_compile tools\gate_imui_workstream_source.py tools\gate_docking_multiwindow_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\docking-multiwindow-imgui-parity\WORKSTREAM.json > $null`: pass.
- `python tools\gate_docking_multiwindow_workstream_source.py`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Verdict

This keeps declarative pointer-move movement, activation, hover, and cursor behavior narrower and
source-auditable without changing runtime behavior. It does not close `DW-P1-linux-003`; the next
true closure event remains a dated real Linux Wayland compositor acceptance note from
`M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
