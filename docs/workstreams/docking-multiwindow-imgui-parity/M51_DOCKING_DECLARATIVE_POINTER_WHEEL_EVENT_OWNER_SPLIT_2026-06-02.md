# M51 Docking Declarative PointerWheel Event Owner Split - 2026-06-02

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split of the declarative dock-space `PointerEvent::Wheel`
event owner. It keeps `DW-P1-linux-003` open because real Wayland compositor acceptance still
requires the M5 runbook on a qualifying Linux Wayland host.

## Claim

Declarative docking pointer-wheel event handling now lives in a private child owner under
`ecosystem/fret-docking/src/dock/declarative/events/pointer_wheel.rs`, while `events.rs` remains the
event-family router for InternalDrag, pointer down/move/up/wheel, and pointer-cancel dispatch. The
split does not change overflow menu wheel scrolling, tab strip wheel scrolling, scroll state sync,
redraw requests, or propagation stopping.

## Source Shape

- `ecosystem/fret-docking/src/dock/declarative/events.rs` declares `mod pointer_wheel;` and routes
  `fret_core::PointerEvent::Wheel` through `handle_pointer_wheel_event(...)`.
- `ecosystem/fret-docking/src/dock/declarative/events/pointer_wheel.rs` owns:
  - layout snapshot lookup through `declarative_layout_snapshot_for_bounds(...)`,
  - overflow menu wheel handling through `declarative_handle_tab_overflow_menu_wheel(...)`,
  - tab strip wheel handling through `declarative_handle_tab_strip_wheel(...)`,
  - scroll state sync through `declarative_sync_tab_scroll_for_window(...)`,
  - redraw and propagation control for handled wheel input.
- `events.rs` no longer owns per-event behavior; it routes to InternalDrag, PointerDown,
  PointerMove, PointerUp, PointerWheel, and PointerCancel child owners.

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

This keeps declarative pointer-wheel scroll handling narrower and source-auditable without changing
runtime behavior. It does not close `DW-P1-linux-003`; the next true closure event remains a dated
real Linux Wayland compositor acceptance note from
`M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
