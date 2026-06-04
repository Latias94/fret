# M92 Runner Renderer Bootstrap Owner Split - 2026-06-04

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split inside the desktop runner. It keeps
`DW-P1-linux-003` open because real Wayland compositor acceptance still requires the M5 runbook on a
qualifying Linux Wayland host.

## Claim

Desktop runner renderer bootstrap now lives in
`crates/fret-launch/src/runner/desktop/runner/renderer_bootstrap.rs` instead of the general
`ApplicationHandler` integration. The split preserves renderer capability publication, renderer
creation, renderer budget configuration, startup font environment initialization, context and
renderer installation, `gpu_ready` callback ordering, and startup async font rescan gating.

Marker summary: renderer creation; renderer capability publication; budget configuration; startup
font environment initialization; startup async font rescan gating.

## Source Shape

- `crates/fret-launch/src/runner/desktop/runner/mod.rs` declares `mod renderer_bootstrap;`.
- `crates/fret-launch/src/runner/desktop/runner/renderer_bootstrap.rs` owns
  `install_renderer_bootstrap`.
- `crates/fret-launch/src/runner/desktop/runner/app_handler.rs` still owns winit
  `can_create_surfaces`, wgpu context construction, main-window insertion, factory surface
  attachment, driver initialization, and `ApplicationHandler` wiring.
- The original ordering is preserved: both mobile and desktop context construction paths publish
  adapter diagnostics, install renderer bootstrap, then continue their existing window startup
  flow.

## Commands Run

```powershell
cargo fmt --package fret-launch -- --check
cargo check -p fret-launch --lib
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

- `cargo fmt --package fret-launch -- --check`: pass.
- `cargo check -p fret-launch --lib`: pass.
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
- Broader workspace gates were not run because M92 is a private `fret-launch` owner split with no
  public API or cross-crate behavior change; the package check, targeted nextest, and source gates
  cover this claim.

## Verdict

This keeps desktop runner renderer bootstrap source-auditable without changing runtime behavior. It
does not close `DW-P1-linux-003`; the next true closure event remains a dated real Linux Wayland
compositor acceptance note from
`M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
