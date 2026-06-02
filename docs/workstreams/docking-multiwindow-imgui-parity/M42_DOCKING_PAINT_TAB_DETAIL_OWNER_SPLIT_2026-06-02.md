# M42 Docking Paint Tab Detail Owner Split - 2026-06-02

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split of docking tab detail paint ownership. It keeps
`DW-P1-linux-003` open because real Wayland compositor acceptance still requires the M5 runbook on
a qualifying Linux Wayland host.

## Claim

Tab title clipping, tab title paint, close affordance paint, overflow button paint, and overflow
menu row/detail paint are now owned by private `dock/paint/tab_detail.rs` instead of the large
`dock/paint.rs` shell, without changing tab chrome shell painting, drag ghost painting, complex
drop overlays, split handle painting, viewport surface painting, floating chrome painting, or the
Wayland acceptance boundary.

## Source Shape

- `ecosystem/fret-docking/src/dock/paint.rs` keeps the public-in-dock paint shell and declares
  `mod tab_detail;`. It re-exports `TabDetailPaintInput`, `tab_detail_paint_inputs`, and
  `paint_tab_detail_inputs` through `pub(super) use tab_detail::{...};`.
- `ecosystem/fret-docking/src/dock/paint/tab_detail.rs` owns `TabDetailPaintInput`,
  `tab_detail_paint_inputs`, private `paint_tab_detail_input`, `paint_tab_detail_inputs`,
  `tab_title_clip_rect`, `tab_close_rect`, `tab_overflow_menu_rect`, and `overflow_menu_row_rect`
  orchestration for tab detail and overflow menu paint.
- `tools/gate_docking_multiwindow_workstream_source.py` and
  `tools/gate_imui_workstream_source.py` reject tab detail paint ownership drifting back into
  `dock/paint.rs` and source-check the new owner directly.

## Commands Run

```powershell
cargo test -p fret-docking
python tools\gate_docking_multiwindow_workstream_source.py
python tools\gate_imui_workstream_source.py
python -m json.tool docs\workstreams\docking-multiwindow-imgui-parity\WORKSTREAM.json
git diff --check
```

## Results

- `cargo test -p fret-docking`: pass.
- `gate_docking_multiwindow_workstream_source.py`: pass after adding this note and `WORKSTREAM.json`
  markers.
- `gate_imui_workstream_source.py`: expected pass because the IMUI workstream source gate is part of
  the recorded owner-split guard surface.
- `WORKSTREAM.json` shape and `git diff --check`: pass.

## Verdict

This keeps tab detail paint source-auditable in a private paint owner while preserving the
existing dock-space behavior. It does not close `DW-P1-linux-003`; the next true closure event is
still a dated real Linux Wayland compositor acceptance note from
`M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
