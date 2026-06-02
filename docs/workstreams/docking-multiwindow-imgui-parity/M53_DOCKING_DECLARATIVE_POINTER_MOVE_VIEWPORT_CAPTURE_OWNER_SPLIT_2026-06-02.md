# M53 Docking Declarative PointerMove Viewport Capture Owner Split - 2026-06-02

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split inside the declarative dock-space `PointerMove`
event owner. It keeps `DW-P1-linux-003` open because real Wayland compositor acceptance still
requires the M5 runbook on a qualifying Linux Wayland host.

## Claim

Declarative docking `PointerMove` viewport-capture forwarding now lives in
`ecosystem/fret-docking/src/dock/declarative/events/pointer_move/viewport_capture.rs`, while
`events/pointer_move.rs` remains the event-phase owner for divider drag movement, floating
title-bar movement, pending panel drag activation, pending tabs-group drag activation, and final
hover/cursor projection. The split does not change viewport input forwarding, right-button
context-menu drag movement tracking, redraw requests, propagation stopping, same-window capture
suppression, or the Wayland acceptance boundary.

## Source Shape

- `ecosystem/fret-docking/src/dock/declarative/events/pointer_move.rs` declares
  `mod viewport_capture;` and delegates the first move phase to
  `viewport_capture::handle_pointer_move_viewport_capture(...)`.
- `ecosystem/fret-docking/src/dock/declarative/events/pointer_move/viewport_capture.rs` owns:
  - active viewport capture lookup through `service.viewport_capture(window, pointer_id)`,
  - right-button context-menu drag movement tracking through `viewport_context_menu_drag_threshold`,
  - clamped viewport input construction through `viewport_input_from_hit_clamped(...)`,
  - `ViewportInputKind::PointerMove` effect emission,
  - capture-state persistence through `service.begin_viewport_capture(window, capture)`,
  - redraw plus propagation stop when the active capture is forwarded,
  - same-window capture suppression through `service.has_viewport_capture_for_window(window)`.
- `events/pointer_move.rs` still owns the later active movement phases and keeps `PointerMove`
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

This keeps declarative pointer-move viewport capture forwarding source-auditable without changing
runtime behavior. It does not close `DW-P1-linux-003`; the next true closure event remains a dated
real Linux Wayland compositor acceptance note from
`M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
