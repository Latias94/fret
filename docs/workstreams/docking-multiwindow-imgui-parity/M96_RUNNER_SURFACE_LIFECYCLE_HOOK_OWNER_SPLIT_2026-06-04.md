# M96 Runner Surface Lifecycle Hook Owner Split - 2026-06-04

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split inside the desktop runner. It keeps
`DW-P1-linux-003` open because real Wayland compositor acceptance still requires the M5 runbook on a
qualifying Linux Wayland host.

## Claim

Desktop runner surface lifecycle hook handling now lives in
`crates/fret-launch/src/runner/desktop/runner/surface_lifecycle.rs` instead of the general
`ApplicationHandler` integration. The split preserves destroy-surface diagnostics, surface destroy
cleanup, Android/iOS resume redraw requests, Android/iOS resume effect draining, Android/iOS
suspend state updates, Android/iOS best-effort surface drop, and suspend control-flow wait.

Marker summary: destroy-surface diagnostics; Android/iOS resume redraw requests; suspend control-flow wait; does not close.

Marker details: surface destroy cleanup; Android/iOS suspend state updates.

## Source Shape

- `crates/fret-launch/src/runner/desktop/runner/surface_lifecycle.rs` owns
  `handle_destroy_surfaces`, `handle_resumed`, and `handle_suspended` beside the existing surface
  lifecycle helpers.
- `crates/fret-launch/src/runner/desktop/runner/app_handler.rs` still owns the
  `ApplicationHandler::destroy_surfaces`, `ApplicationHandler::resumed`, and
  `ApplicationHandler::suspended` trait hooks and delegates directly to the surface lifecycle owner.
- The original ordering is preserved: destroy diagnostics are recorded before surface cleanup;
  mobile resume clears suspension, requests redraws, and drains effects; mobile suspend marks the
  runner suspended, drops surfaces, and sets `ControlFlow::Wait`.

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
- Broader workspace gates were not run because M96 is a private `fret-launch` owner split with no
  public API or cross-crate behavior change; the package check, targeted nextest, and source gates
  cover this claim.

## Verdict

This keeps surface lifecycle trait-hook handling source-auditable without changing runtime
behavior. It does not close `DW-P1-linux-003`; the next true closure event remains a dated real
Linux Wayland compositor acceptance note from
`M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
