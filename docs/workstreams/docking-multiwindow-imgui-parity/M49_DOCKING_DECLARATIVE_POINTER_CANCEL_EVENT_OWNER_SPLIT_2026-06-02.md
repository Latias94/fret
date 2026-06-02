# M49 Docking Declarative PointerCancel Event Owner Split - 2026-06-02

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split of the declarative dock-space `PointerCancel`
event owner. It keeps `DW-P1-linux-003` open because real Wayland compositor acceptance still
requires the M5 runbook on a qualifying Linux Wayland host.

## Claim

Declarative docking `PointerCancel` event handling now lives in a private child owner under
`ecosystem/fret-docking/src/dock/declarative/events/pointer_cancel.rs`, while `events.rs` is reduced
to the event-family router for InternalDrag, pointer down/up, and pointer-cancel dispatch. The split
does not change viewport cancel forwarding, dock hover clearing, pointer capture release, tab close
cleanup, pending dock drag cleanup, pending tabs-group drag cleanup, divider drag cleanup, floating
close cleanup, or floating drag cleanup.

## Source Shape

- `ecosystem/fret-docking/src/dock/declarative/events.rs` declares `mod pointer_cancel;` and routes
  `fret_core::Event::PointerCancel` through `handle_pointer_cancel_event(...)`.
- `ecosystem/fret-docking/src/dock/declarative/events/pointer_cancel.rs` owns:
  - viewport cancel forwarding through `ViewportInputKind::PointerCancel`,
  - viewport cancel clamping through `viewport_input_from_hit_clamped(...)`,
  - dock hover clearing after viewport cancel,
  - tab close cleanup through `take_tab_close(...)`,
  - pending dock drag cleanup through `take_pending_dock_drag(...)`,
  - pending tabs-group drag cleanup through `take_pending_dock_tabs_drag(...)`,
  - divider drag cleanup through `take_divider_drag(...)`,
  - floating close/drag cleanup through `take_floating_close(...)` and
    `take_floating_drag(...)`.
- `events.rs` no longer owns per-event behavior; it routes to InternalDrag, PointerDown, PointerUp,
  and PointerCancel child owners.

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

This makes declarative event handling source-auditable as a router plus four narrow child owners
without changing runtime behavior. It does not close `DW-P1-linux-003`; the next true closure event
remains a dated real Linux Wayland compositor acceptance note from
`M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
