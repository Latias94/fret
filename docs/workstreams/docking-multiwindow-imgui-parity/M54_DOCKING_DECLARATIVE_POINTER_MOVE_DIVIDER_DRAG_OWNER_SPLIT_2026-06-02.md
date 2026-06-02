# M54 Docking Declarative PointerMove Divider Drag Owner Split - 2026-06-02

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split inside the declarative dock-space `PointerMove`
event owner. It keeps `DW-P1-linux-003` open because real Wayland compositor acceptance still
requires the M5 runbook on a qualifying Linux Wayland host.

## Claim

Declarative docking `PointerMove` divider-drag resize handling now lives in
`ecosystem/fret-docking/src/dock/declarative/events/pointer_move/divider_drag.rs`, while
`events/pointer_move.rs` remains the event-phase owner for viewport capture forwarding, floating
title-bar movement, pending panel drag activation, pending tabs-group drag activation, and final
hover/cursor projection. The split does not change split-handle cursor projection, left-button
release cleanup, split fraction updates, layout invalidation, redraw requests, propagation
stopping, or the Wayland acceptance boundary.

## Source Shape

- `ecosystem/fret-docking/src/dock/declarative/events/pointer_move.rs` declares
  `mod divider_drag;` and delegates the resize phase to
  `divider_drag::handle_pointer_move_divider_drag(...)`.
- `ecosystem/fret-docking/src/dock/declarative/events/pointer_move/divider_drag.rs` owns:
  - active divider drag lookup through `service.divider_drag(window, pointer_id)`,
  - left-button release cleanup through `service.take_divider_drag(window, pointer_id)`,
  - split-handle cursor projection through `declarative_split_handle_cursor(...)`,
  - split node/fraction lookup through `DockManager::default` and `DockNode::Split`,
  - adjacent fraction calculation through `drag_update_adjacent_fractions(...)`,
  - graph mutation through `update_split_fractions(divider_drag.handle.split, next)`,
  - layout invalidation, redraw, and propagation stop for handled divider moves.
- `events/pointer_move.rs` still owns the later floating/pending-drag phases and keeps
  `PointerMove` routing out of the generic `events.rs` router.

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

This keeps declarative pointer-move divider resize handling source-auditable without changing
runtime behavior. It does not close `DW-P1-linux-003`; the next true closure event remains a dated
real Linux Wayland compositor acceptance note from
`M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
