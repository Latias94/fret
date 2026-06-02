# M52 Docking Declarative PointerMove Hover Owner Split - 2026-06-02

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split inside the declarative dock-space `PointerMove`
event owner. It keeps `DW-P1-linux-003` open because real Wayland compositor acceptance still
requires the M5 runbook on a qualifying Linux Wayland host.

## Claim

Declarative docking `PointerMove` hover and cursor projection now lives in
`ecosystem/fret-docking/src/dock/declarative/events/pointer_move/hover.rs`, while
`events/pointer_move.rs` remains the event-phase owner for viewport capture forwarding, divider
drag movement, floating title-bar movement, pending panel drag activation, and pending tabs-group
drag activation. The split does not change split-handle hover cursors, floating close/title-bar
hover, tab hover, overflow-menu hover, tab cursor projection, redraw requests, or the Wayland
acceptance boundary.

## Source Shape

- `ecosystem/fret-docking/src/dock/declarative/events/pointer_move.rs` declares `mod hover;` and
  delegates the final hover/cursor projection to
  `hover::update_pointer_move_hover(cx, window, *position)`.
- `ecosystem/fret-docking/src/dock/declarative/events/pointer_move/hover.rs` owns:
  - layout snapshot lookup for the current pointer position,
  - split-handle hover cursor projection through `declarative_split_handle_hit_for_position(...)`,
  - floating close/title-bar hover projection through `DeclarativeFloatingHover`,
  - tab hover and overflow-menu hover updates through `declarative_tab_hover_for_position(...)`,
  - pointer cursor selection and redraw request when hover state changes.
- `events/pointer_move.rs` still owns the active movement phases and keeps `PointerMove` routing
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

This keeps declarative pointer-move hover and cursor projection source-auditable without changing
runtime behavior. It does not close `DW-P1-linux-003`; the next true closure event remains a dated
real Linux Wayland compositor acceptance note from
`M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
