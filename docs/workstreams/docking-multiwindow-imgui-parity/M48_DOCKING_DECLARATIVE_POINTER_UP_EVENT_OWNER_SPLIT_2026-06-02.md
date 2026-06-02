# M48 Docking Declarative PointerUp Event Owner Split - 2026-06-02

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split of the declarative dock-space `PointerUp` event
owner. It keeps `DW-P1-linux-003` open because real Wayland compositor acceptance still requires the
M5 runbook on a qualifying Linux Wayland host.

## Claim

Declarative docking `PointerUp` event handling now lives in a private child owner under
`ecosystem/fret-docking/src/dock/declarative/events/pointer_up.rs`, while `events.rs` remains the
event-family router for InternalDrag, pointer down/up, and pointer-cancel orchestration. The split
does not change viewport release forwarding, context-menu suppression after right-button viewport
drag, floating close commit, floating title-bar merge commit, divider fraction commit, pending drag
cleanup, pending tabs-group drag cleanup, or tab close commit behavior.

## Source Shape

- `ecosystem/fret-docking/src/dock/declarative/events.rs` declares `mod pointer_up;` and routes
  `fret_core::PointerEvent::Up` through `handle_pointer_up_event(...)`.
- `ecosystem/fret-docking/src/dock/declarative/events/pointer_up.rs` owns:
  - viewport release dispatch through `ViewportInputKind::PointerUp`,
  - right-button viewport drag context-menu suppression through
    `suppress_context_menu_during_viewport_capture`,
  - floating close commit through `declarative_hit_test_floating_close(...)` and
    `DockOp::MergeFloatingInto`,
  - floating title-bar drag merge commit through
    `declarative_resolve_floating_title_bar_drag_target(...)`,
  - divider fraction commit through `DockOp::SetSplitFractionsMany`,
  - pending single-panel and tabs-group drag cleanup,
  - tab close commit through `declarative_hit_test_tab_close(...)`,
    `pointer_move_within_slop(...)`, and `DockOp::ClosePanel`.
- `events.rs` keeps InternalDrag routing, pointer-down activation, and pointer-cancel cleanup.

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

This keeps declarative pointer-up release/commit behavior narrower and source-auditable without
changing runtime behavior. It does not close `DW-P1-linux-003`; the next true closure event remains
a dated real Linux Wayland compositor acceptance note from
`M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
