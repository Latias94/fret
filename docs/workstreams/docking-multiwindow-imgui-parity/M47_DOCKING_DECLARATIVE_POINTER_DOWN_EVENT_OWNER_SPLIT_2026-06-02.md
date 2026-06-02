# M47 Docking Declarative PointerDown Event Owner Split - 2026-06-02

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split of the declarative dock-space `PointerDown`
event owner. It keeps `DW-P1-linux-003` open because real Wayland compositor acceptance still
requires the M5 runbook on a qualifying Linux Wayland host.

## Claim

Declarative docking `PointerDown` event handling now lives in a private child owner under
`ecosystem/fret-docking/src/dock/declarative/events/pointer_down.rs`, while `events.rs` remains the
event-family router for InternalDrag, pointer down/up, and pointer-cancel orchestration. The split
does not change tab-overflow menu clicks, overflow menu opening, floating close press, floating title
drag activation, split-handle drag activation, active viewport pointer-down forwarding/capture, tab
close press, pending dock drag activation, or tabs-group drag activation.

## Source Shape

- `ecosystem/fret-docking/src/dock/declarative/events.rs` declares `mod pointer_down;` and routes
  `fret_core::PointerEvent::Down` through `handle_pointer_down_event(...)`.
- `ecosystem/fret-docking/src/dock/declarative/events/pointer_down.rs` owns:
  - tab overflow menu left-click dispatch through
    `declarative_handle_tab_overflow_menu_left_click(...)`,
  - tab overflow menu opening through `declarative_open_tab_overflow_menu(...)`,
  - floating close press through `declarative_hit_test_floating_close(...)`,
  - floating title-bar drag activation through `declarative_hit_test_floating_title_bar(...)`,
  - split-handle divider drag activation through `declarative_split_handle_hit_for_position(...)`,
  - active viewport `ViewportInputKind::PointerDown` forwarding and capture through
    `declarative_hit_test_active_viewport_panel(...)`,
  - tab close press state,
  - pending single-panel dock drag activation,
  - pending tabs-group dock drag activation.
- `events.rs` keeps InternalDrag routing, pointer-up release/commit behavior, and pointer-cancel
  cleanup.

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

This keeps declarative pointer-down activation narrower and makes the entry points for overflow,
floating, split-handle, viewport, and tab-drag activation source-auditable without changing runtime
behavior. It does not close `DW-P1-linux-003`; the next true closure event remains a dated real
Linux Wayland compositor acceptance note from
`M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
