# M56 Docking Declarative PointerMove Pending Panel Drag Owner Split - 2026-06-02

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split inside the declarative dock-space `PointerMove`
event owner. It keeps `DW-P1-linux-003` open because real Wayland compositor acceptance still
requires the M5 runbook on a qualifying Linux Wayland host.

## Claim

Declarative docking `PointerMove` pending panel drag activation now lives in
`ecosystem/fret-docking/src/dock/declarative/events/pointer_move/pending_panel_drag.rs`, while
`events/pointer_move.rs` remains the event-phase owner for viewport capture forwarding, divider
drag resize handling, floating title-bar movement, pending tabs-group drag activation, and final
hover/cursor projection. The split does not change pending panel drag lookup, left-button release
cleanup, activation threshold handling, panel drag startup, hover clearing, capture release, redraw
requests, propagation stopping, or the Wayland acceptance boundary.

## Source Shape

- `ecosystem/fret-docking/src/dock/declarative/events/pointer_move.rs` declares
  `mod pending_panel_drag;` and delegates the panel activation phase to
  `pending_panel_drag::handle_pointer_move_pending_panel_drag(...)`.
- `ecosystem/fret-docking/src/dock/declarative/events/pointer_move/pending_panel_drag.rs` owns:
  - pending panel drag lookup through `service.pending_dock_drag(window, pointer_id)`,
  - left-button release cleanup through `service.take_pending_dock_drag(window, pointer_id)`,
  - distance activation through `fret_dnd::ActivationConstraint::Distance`,
  - panel drag startup through `begin_declarative_panel_drag(...)`,
  - hover clearing through `DockManager::default` and `dock.hover = None`,
  - capture release, redraw, and propagation stop for handled pending panel moves.
- `events/pointer_move.rs` still owns the later pending tabs-group phase and keeps `PointerMove`
  routing out of the generic `events.rs` router.

## Commands Run

```powershell
cargo fmt --package fret-docking
cargo check -p fret-docking
cargo test -p fret-docking --no-fail-fast
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
- `cargo test -p fret-docking --no-fail-fast`: pass, 87 library tests plus 3 policy tests.
- `python -m py_compile tools\gate_imui_workstream_source.py tools\gate_docking_multiwindow_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\docking-multiwindow-imgui-parity\WORKSTREAM.json > $null`: pass.
- `python tools\gate_docking_multiwindow_workstream_source.py`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Verdict

This keeps declarative pointer-move pending panel drag activation source-auditable without
changing runtime behavior. It does not close `DW-P1-linux-003`; the next true closure event remains
a dated real Linux Wayland compositor acceptance note from
`M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
