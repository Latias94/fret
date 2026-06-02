# M46 Docking Declarative InternalDrag Event Owner Split - 2026-06-02

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split of the declarative dock-space `InternalDrag`
event owner. It keeps `DW-P1-linux-003` open because real Wayland compositor acceptance still
requires the M5 runbook on a qualifying Linux Wayland host.

## Claim

Declarative docking `InternalDrag` event handling now lives in a private child owner under
`ecosystem/fret-docking/src/dock/declarative/events/internal_drag.rs`, while
`events.rs` remains the event-family router for pointer down/up/cancel orchestration. The split does
not change dock-drag hover resolution, drop resolution, layout/paint invalidation, dock drag cancel,
or hover clearing behavior.

## Source Shape

- `ecosystem/fret-docking/src/dock/declarative/events.rs` declares `mod internal_drag;` and routes
  `fret_core::Event::InternalDrag(...)` through `handle_internal_drag_event(...)`.
- `ecosystem/fret-docking/src/dock/declarative/events/internal_drag.rs` owns:
  - `InternalDragKind::Enter | Over` hover routing through
    `declarative_resolve_internal_drag_hover(...)`,
  - `InternalDragKind::Drop` routing through `declarative_resolve_internal_drag_drop(...)`,
  - drag-session cancellation when a resolved dock drop ends the drag,
  - `InternalDragKind::Leave | Cancel` hover clearing.
- `events.rs` keeps pointer-down, pointer-up, and pointer-cancel orchestration for overflow menus,
  floating chrome, split handles, viewport forwarding, tab close press/commit, and pending dock
  drag activation.

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

This keeps the declarative event owner narrower and makes cross-window dock drag hover/drop routing
source-auditable without changing runtime behavior. It does not close `DW-P1-linux-003`; the next
true closure event remains a dated real Linux Wayland compositor acceptance note from
`M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
