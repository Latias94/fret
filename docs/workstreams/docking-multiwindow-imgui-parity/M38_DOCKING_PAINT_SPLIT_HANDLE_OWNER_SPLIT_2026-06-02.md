# M38 Docking Paint Split Handle Owner Split - 2026-06-02

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split of docking split handle paint ownership. It keeps
`DW-P1-linux-003` open because real Wayland compositor acceptance still requires the M5 runbook on
a qualifying Linux Wayland host.

## Claim

Split handle input projection and handle painting is now owned by private
`dock/paint/split_handle.rs` instead of the large `dock/paint.rs` shell, without changing split
layout geometry, active handle color selection, draw order, drop overlays, viewport surface
painting, floating chrome painting, drop hints, or the Wayland acceptance boundary.

## Source Shape

- `ecosystem/fret-docking/src/dock/paint.rs` keeps the public-in-dock paint shell and declares
  `mod split_handle;`. It re-exports `SplitHandlePaintInput`, `split_handle_paint_inputs`, and
  `paint_split_handle_inputs` through `pub(super) use split_handle::{...};`.
- `ecosystem/fret-docking/src/dock/paint/split_handle.rs` owns `SplitHandlePaintInput`,
  `split_handle_paint_inputs`, `paint_split_handle_inputs`, `split_geometry::compute_layout`,
  `SplitHandle::paint`, and the active `ring` versus inactive `border` color selection.
- `tools/gate_docking_multiwindow_workstream_source.py` and
  `tools/gate_imui_workstream_source.py` reject split handle paint ownership drifting back into
  `dock/paint.rs` and source-check the new owner directly.

## Commands Run

```powershell
rustup run 1.92 cargo fmt -p fret-docking
rustup run 1.92 cargo check -p fret-docking
python -m py_compile tools\gate_docking_multiwindow_workstream_source.py tools\gate_imui_workstream_source.py
python tools\gate_docking_multiwindow_workstream_source.py
python tools\gate_imui_workstream_source.py
rustup run 1.92 cargo test -p fret-docking
python -m json.tool docs\workstreams\docking-multiwindow-imgui-parity\WORKSTREAM.json
python tools\check_workstream_catalog.py
git diff --check
```

## Results

- `rustup run 1.92 cargo fmt -p fret-docking`: pass.
- `rustup run 1.92 cargo check -p fret-docking`: pass.
- `gate_docking_multiwindow_workstream_source.py`: pass.
- `gate_imui_workstream_source.py`: pass.
- `rustup run 1.92 cargo test -p fret-docking`: pass, 90 tests.
- `WORKSTREAM.json` shape, workstream catalog, and `git diff --check`: pass.
- `cargo nextest run -p fret-docking --no-fail-fast`: not run because this shell could not find
  `cargo.exe` or `cargo-nextest.exe`; `rustup run 1.92 cargo test -p fret-docking` was used as the
  package-level fallback.

## Verdict

This keeps split handle paint source-auditable in a private paint owner while preserving the
existing dock-space behavior. It does not close `DW-P1-linux-003`; the next true closure event is
still a dated real Linux Wayland compositor acceptance note from
`M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
