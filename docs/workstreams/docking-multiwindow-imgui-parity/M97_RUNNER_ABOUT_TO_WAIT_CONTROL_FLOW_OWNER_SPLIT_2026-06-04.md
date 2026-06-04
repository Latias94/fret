# M97 Runner About-To-Wait Control Flow Owner Split - 2026-06-04

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split inside the desktop runner. It keeps
`DW-P1-linux-003` open because real Wayland compositor acceptance still requires the M5 runbook on a
qualifying Linux Wayland host.

## Claim

Desktop runner about-to-wait control-flow scheduling now lives in
`crates/fret-launch/src/runner/desktop/runner/event_loop.rs` instead of the general
`ApplicationHandler` integration. The split preserves pending-front request processing, timer
deadline merging, dispatcher deadline merging, pending-front deadline merging, hotpatch deadline
merging, dock drag/follow polling pressure, RAF deadline scheduling, RAF flush behavior, and the
final `ControlFlow::Poll` / `ControlFlow::WaitUntil` / `ControlFlow::Wait` selection.

Marker summary: pending-front request processing; timer deadline merging; RAF flush behavior; final ControlFlow selection; does not close.

Marker details: hotpatch deadline merging.

## Source Shape

- `crates/fret-launch/src/runner/desktop/runner/event_loop.rs` owns
  `handle_about_to_wait_control_flow` beside the other event-loop wake helpers.
- `crates/fret-launch/src/runner/desktop/runner/app_handler.rs` still owns the
  `ApplicationHandler::about_to_wait` trait hook and delegates the final scheduling tail to
  `handle_about_to_wait_control_flow`.
- The original ordering is preserved: platform released-outside cleanup still runs before
  pending-front work; deadline sources are merged in the same order; RAF flush still clears the
  deadline before flushing; pending-front retry still waits for `Duration::from_millis(16)`.

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
- Broader workspace gates were not run because M97 is a private `fret-launch` owner split with no
  public API or cross-crate behavior change; the package check, targeted nextest, and source gates
  cover this claim.

## Verdict

This keeps about-to-wait event-loop scheduling source-auditable without changing runtime behavior.
It does not close `DW-P1-linux-003`; the next true closure event remains a dated real Linux Wayland
compositor acceptance note from `M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
