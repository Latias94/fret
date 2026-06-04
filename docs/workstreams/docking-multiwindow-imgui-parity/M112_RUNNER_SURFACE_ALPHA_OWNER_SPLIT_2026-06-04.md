# M112 Runner Surface Alpha Owner Split - 2026-06-04

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split inside the desktop runner. It keeps
`DW-P1-linux-003` open because real Wayland compositor acceptance still requires the M5 runbook on a
qualifying Linux Wayland host.

## Claim

Desktop runner composited-alpha surface configuration now lives in
`crates/fret-launch/src/runner/desktop/runner/surface_lifecycle.rs` instead of the
`window_lifecycle.rs` create/insert/destroy owner. The split preserves
`want_surface_composited_alpha_for_style`, `configure_surface_alpha_mode_for_composited_window`,
background material implied transparency, alpha-mode selection order, surface reconfigure behavior,
window creation surface setup, style updates, and diagnostics snapshot publication.

Marker summary: composited-alpha surface configuration; background material implied transparency;
surface reconfigure; does not close.

## Source Shape

- `crates/fret-launch/src/runner/desktop/runner/surface_lifecycle.rs` owns
  `want_surface_composited_alpha_for_style` and
  `configure_surface_alpha_mode_for_composited_window`.
- `crates/fret-launch/src/runner/desktop/runner/window_lifecycle.rs` keeps window creation,
  insertion, close, and force-close lifecycle helpers while delegating initial surface alpha setup to
  the surface owner.
- `crates/fret-launch/src/runner/desktop/runner/window_style.rs` continues to update effective
  style diagnostics, background material, surface alpha mode, and surface config diagnostics through
  the same runtime path, but now calls the surface owner directly.

## Commands Run

```powershell
cargo fmt --package fret-launch
cargo check -p fret-launch --lib
cargo fmt --package fret-launch -- --check
cargo nextest run -p fret-launch --lib linux_windowing_capability_posture --no-fail-fast
python -m py_compile tools\gate_docking_multiwindow_workstream_source.py tools\gate_imui_workstream_source.py
python -m json.tool docs\workstreams\docking-multiwindow-imgui-parity\WORKSTREAM.json
python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json
python tools\gate_docking_multiwindow_workstream_source.py
python tools\gate_imui_workstream_source.py
python tools\check_workstream_catalog.py
git diff --check
```

## Results

- `cargo fmt --package fret-launch`: pass.
- `cargo check -p fret-launch --lib`: pass.
- `cargo fmt --package fret-launch -- --check`: pass.
- `cargo nextest run -p fret-launch --lib linux_windowing_capability_posture --no-fail-fast`:
  pass.
- `python -m py_compile tools\gate_docking_multiwindow_workstream_source.py tools\gate_imui_workstream_source.py`:
  pass.
- `python -m json.tool docs\workstreams\docking-multiwindow-imgui-parity\WORKSTREAM.json`:
  pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_docking_multiwindow_workstream_source.py`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass, with the existing `WORKSTREAM.json` CRLF normalization warning.
- Broader workspace gates were not run because M112 is a private `fret-launch` owner split with no
  public API or cross-crate behavior change; the package check, targeted nextest, and source gates
  cover this claim.

## Verdict

This keeps desktop runner surface alpha configuration source-auditable in the surface lifecycle
owner and leaves `window_lifecycle.rs` focused on window creation/insert/destroy. It does not close
`DW-P1-linux-003`; the next true closure event remains a dated real Linux Wayland compositor
acceptance note from `M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
