# M151 Runner Surface Bootstrap Owner Split - 2026-06-05

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split inside the desktop runner. It keeps
`DW-P1-linux-003` open because real Wayland compositor acceptance still requires the M5 runbook on a
qualifying Linux Wayland host.

## Claim

Desktop runner `ApplicationHandler::can_create_surfaces` bootstrap orchestration now lives in
`crates/fret-launch/src/runner/desktop/runner/surface_bootstrap.rs`. The split moves
can-create-surface lifecycle diagnostics, WGPU init blocked gating, already-initialized missing
surface recovery, Android/iOS main-window deferred creation, RenderDoc pre-WGPU initialization, WGPU
context construction for default/provided/factory paths, Android SwiftShader guard, adapter
diagnostics publication, renderer bootstrap installation, factory-provided main surface attachment,
driver initialization, startup incoming-open delivery, initial redraw/font-rescan scheduling, and
post-bootstrap effect draining out of `app_handler.rs` while preserving runtime behavior and public
effect surfaces.

Marker summary: surface bootstrap owner; ApplicationHandler can_create_surfaces dispatch;
can-create-surface lifecycle diagnostics; WGPU init blocked gate; missing-surface bootstrap;
mobile main-window creation; RenderDoc pre-WGPU initialization; WgpuInit default/provided/factory;
Android SwiftShader guard; adapter diagnostics; renderer bootstrap; factory surface attach; driver
initialization; startup incoming-open delivery; initial redraw scheduling; startup font rescan;
post-bootstrap effect drain.

Projection marker: winit surface creation lifecycle bootstrap from can_create_surfaces dispatch
through driver initialization and post-bootstrap effect drain.

## Source Shape

- `crates/fret-launch/src/runner/desktop/runner/surface_bootstrap.rs` owns
  `handle_can_create_surfaces`.
- The owner sequences existing child owners for OS window creation, window insertion, WGPU adapter
  diagnostics, renderer bootstrap, factory surface attachment, missing-surface creation, startup
  incoming-open delivery, system-font rescan requests, and effect draining.
- `crates/fret-launch/src/runner/desktop/runner/app_handler.rs` keeps only the winit
  `ApplicationHandler::can_create_surfaces` trait hook and dispatches to
  `handle_can_create_surfaces`.
- Existing focused owners remain intact; this split adds a lifecycle/bootstrap workflow owner above
  them rather than moving child policy back into `app_handler.rs`.

## Commands Run

```powershell
cargo fmt --package fret-launch
cargo check -p fret-launch --lib
cargo check -p fret-launch --features diag-screenshots --lib
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
- `cargo check -p fret-launch --features diag-screenshots --lib`: pass.
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
- Broader workspace gates were not run because M151 is a private `fret-launch` owner split with no
  public API or cross-crate behavior change; the package checks, targeted nextest, and source gates
  cover this claim.

## Verdict

This keeps surface-creation lifecycle bootstrap source-auditable in a named owner while leaving
`app_handler.rs` responsible for winit event dispatch. It does not close `DW-P1-linux-003`; the next
true closure event remains a dated real Linux Wayland compositor acceptance note from
`M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
