# M36 Docking Paint Viewport Surface Owner Split - 2026-06-02

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split of the docking viewport surface paint owner. It
keeps `DW-P1-linux-003` open because real Wayland compositor acceptance still requires the M5
runbook on a qualifying Linux Wayland host.

## Claim

Viewport surface input projection and painting is now owned by private
`dock/paint/viewport_surface.rs` instead of the large `dock/paint.rs` shell, without changing
viewport layout reuse, fallback viewport mapping, overlay hook invocation, tab painting, floating
chrome painting, drop overlays, drop hints, or the Wayland acceptance boundary.

## Source Shape

- `ecosystem/fret-docking/src/dock/paint.rs` keeps the public-in-dock paint shell and declares
  `mod viewport_surface;`. It re-exports `ViewportSurfacePaintInput`,
  `viewport_surface_paint_inputs`, and `paint_viewport_surface_inputs` through
  `pub(super) use viewport_surface::{...};`.
- `ecosystem/fret-docking/src/dock/paint/viewport_surface.rs` owns `ViewportSurfacePaintInput`,
  private `viewport_surface_paint_input`, viewport input collection, `SceneOp::ViewportSurface`,
  fallback `ViewportMapping`, `DockViewportLayout`, and `DockViewportOverlayHooks::paint_with_layout`
  invocation through `hooks.paint_with_layout(...)`.
- `tools/gate_docking_multiwindow_workstream_source.py` and
  `tools/gate_imui_workstream_source.py` reject viewport surface paint ownership drifting back into
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

This keeps docking viewport surface projection and painting source-auditable in a private paint
owner while preserving the existing dock-space behavior. It does not close `DW-P1-linux-003`; the
next true closure event is still a dated real Linux Wayland compositor acceptance note from
`M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
