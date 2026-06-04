# M103 Runner About-To-Wait Mobile Surface Recreation Owner Split - 2026-06-04

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split inside the desktop runner. It keeps
`DW-P1-linux-003` open because real Wayland compositor acceptance still requires the M5 runbook on a
qualifying Linux Wayland host.

## Claim

Desktop runner about-to-wait mobile surface recreation now lives in
`crates/fret-launch/src/runner/desktop/runner/surface_lifecycle.rs` instead of the general
`ApplicationHandler` integration. The split preserves the winit `can_create_surfaces` lifecycle
gate, Android/iOS-only cfg boundary, context-present check, missing-surface scan, deferred
`try_create_missing_surfaces` call, and post-bootstrap effect drain.

Marker summary: mobile surface recreation; winit `can_create_surfaces` lifecycle gate; can-create-surfaces lifecycle gate; missing-surface scan; post-bootstrap effect drain; does not close.

## Source Shape

- `crates/fret-launch/src/runner/desktop/runner/surface_lifecycle.rs` owns
  `handle_about_to_wait_mobile_surface_recreation` beside surface create/destroy/resume/suspend
  helpers.
- `crates/fret-launch/src/runner/desktop/runner/app_handler.rs` still owns the
  `ApplicationHandler::about_to_wait` trait hook and delegates Android/iOS missing-surface
  recreation immediately after monitor topology refresh and before pre-turn internal drag polling.
- The original ordering is preserved: surface recreation still runs after diagnostic screenshot
  redraw requests, still runs before internal drag polling, and still drains effects only when a
  missing surface was recreated.

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
- Broader workspace gates were not run because M103 is a private `fret-launch` owner split with no
  public API or cross-crate behavior change; the package check, targeted nextest, and source gates
  cover this claim.

## Verdict

This keeps about-to-wait mobile surface recreation source-auditable without changing runtime
behavior. It does not close `DW-P1-linux-003`; the next true closure event remains a dated real
Linux Wayland compositor acceptance note from
`M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
