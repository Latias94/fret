# M55 Docking Declarative PointerMove Floating Drag Owner Split - 2026-06-02

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split inside the declarative dock-space `PointerMove`
event owner. It keeps `DW-P1-linux-003` open because real Wayland compositor acceptance still
requires the M5 runbook on a qualifying Linux Wayland host.

## Claim

Declarative docking `PointerMove` floating title-bar movement now lives in
`ecosystem/fret-docking/src/dock/declarative/events/pointer_move/floating_drag.rs`, while
`events/pointer_move.rs` remains the event-phase owner for viewport capture forwarding, divider
drag resize handling, pending panel drag activation, pending tabs-group drag activation, and final
hover/cursor projection. The split does not change floating drag activation, drag inversion preview
policy, in-window floating rect updates, dock hover preview resolution, drag-state persistence,
cursor projection, redraw requests, propagation stopping, or the Wayland acceptance boundary.

## Source Shape

- `ecosystem/fret-docking/src/dock/declarative/events/pointer_move.rs` declares
  `mod floating_drag;` and delegates the floating movement phase to
  `floating_drag::handle_pointer_move_floating_drag(...)`.
- `ecosystem/fret-docking/src/dock/declarative/events/pointer_move/floating_drag.rs` owns:
  - active floating drag lookup through `service.take_floating_drag(window, pointer_id)`,
  - left-button release cleanup and propagation stop,
  - distance activation through `fret_dnd::ActivationConstraint::Distance`,
  - drag inversion preview selection through `settings.drag_inversion.wants_dock_previews(...)`,
  - clamped in-window floating rect updates through `DockOp::SetFloatingRect`,
  - dock hover preview resolution through `declarative_resolve_floating_title_bar_drag_target(...)`,
  - drag-state persistence through `service.begin_floating_drag(window, drag)`,
  - default cursor projection, redraw, and propagation stop for handled floating moves.
- `events/pointer_move.rs` still owns the later pending-drag phases and keeps `PointerMove` routing
  out of the generic `events.rs` router.

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

This keeps declarative pointer-move floating title-bar movement source-auditable without changing
runtime behavior. It does not close `DW-P1-linux-003`; the next true closure event remains a dated
real Linux Wayland compositor acceptance note from
`M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
