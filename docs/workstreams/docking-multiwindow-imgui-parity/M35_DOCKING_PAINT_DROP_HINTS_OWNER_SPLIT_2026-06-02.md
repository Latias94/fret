# M35 Docking Paint Drop-Hints Owner Split - 2026-06-02

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split of the docking drop-hint paint owner. It keeps
`DW-P1-linux-003` open because real Wayland compositor acceptance still requires the M5 runbook on
a qualifying Linux Wayland host.

## Claim

Drop-hint pad and icon painting is now owned by private `dock/paint/drop_hints.rs` instead of the
large `dock/paint.rs` shell, without changing drop target resolution, tab insert previews, drop
overlays, viewport surface painting, floating chrome painting, split handle painting, or the
Wayland acceptance boundary.

## Source Shape

- `ecosystem/fret-docking/src/dock/paint.rs` keeps the public-in-dock paint shell and existing
  tab, viewport, floating chrome, split handle, drag ghost, drop overlay, and tab insert preview
  paint owners. It declares `mod drop_hints;` and re-exports
  `paint_drop_hints` through `pub(super) use drop_hints::paint_drop_hints;`.
- `ecosystem/fret-docking/src/dock/paint/drop_hints.rs` owns the direction pad paint path:
  `paint_drop_hints`, private `paint_drop_hint_icon`, `dock_hint_rects_with_font` layout usage,
  active `DockDropTarget::Dock` highlighting, `DropZone::Center` handling, and the high-order
  `fret_core::DrawOrder(10_100)` hint layer.
- `tools/gate_docking_multiwindow_workstream_source.py` and
  `tools/gate_imui_workstream_source.py` reject drop-hint pad/icon painting drifting back into
  `dock/paint.rs` and source-check the new owner directly.

## Commands Run

```powershell
cargo fmt -p fret-docking
cargo check -p fret-docking
python -m py_compile tools\gate_docking_multiwindow_workstream_source.py tools\gate_imui_workstream_source.py
python tools\gate_docking_multiwindow_workstream_source.py
python tools\gate_imui_workstream_source.py
cargo nextest run -p fret-docking --no-fail-fast
python -m json.tool docs\workstreams\docking-multiwindow-imgui-parity\WORKSTREAM.json
python tools\check_workstream_catalog.py
git diff --check
```

## Results

- `cargo fmt -p fret-docking`: pass.
- `cargo check -p fret-docking`: pass.
- `gate_docking_multiwindow_workstream_source.py`: pass.
- `gate_imui_workstream_source.py`: pass.
- `cargo nextest run -p fret-docking --no-fail-fast`: pass, 90 tests.
- `WORKSTREAM.json` shape, workstream catalog, and `git diff --check`: pass.

## Verdict

This keeps docking drop-hint pad/icon painting source-auditable in a private paint owner while
preserving the existing dock-space behavior. It does not close `DW-P1-linux-003`; the next true
closure event is still a dated real Linux Wayland compositor acceptance note from
`M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
