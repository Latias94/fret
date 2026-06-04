# M101 Runner About-To-Wait Turn Bookkeeping Owner Split - 2026-06-04

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split inside the desktop runner. It keeps
`DW-P1-linux-003` open because real Wayland compositor acceptance still requires the M5 runbook on a
qualifying Linux Wayland host.

## Claim

Desktop runner about-to-wait turn bookkeeping now lives in
`crates/fret-launch/src/runner/desktop/runner/event_loop.rs` instead of the general
`ApplicationHandler` integration. The split preserves tick-id advancement, app tick publication,
per-turn left-button release reset, `Instant::now()` turn timestamp sampling, window environment
polling, dev-state window observation timestamp reuse, and final control-flow scheduling.

Marker summary: tick-id advancement; app tick publication; left-release reset; environment poll; dev-state timestamp reuse; does not close.

## Source Shape

- `crates/fret-launch/src/runner/desktop/runner/event_loop.rs` owns
  `handle_about_to_wait_turn_bookkeeping` beside the other event-loop about-to-wait helpers.
- `crates/fret-launch/src/runner/desktop/runner/app_handler.rs` still owns the
  `ApplicationHandler::about_to_wait` trait hook and delegates turn bookkeeping to the event-loop
  owner immediately after pre-turn internal drag polling.
- The original ordering is preserved: pre-turn internal drag polling still runs before tick
  advancement, dev-state window observation still reuses the same turn timestamp, and final
  control-flow scheduling still receives a fresh later timestamp after effect/fallback drains.

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
- Broader workspace gates were not run because M101 is a private `fret-launch` owner split with no
  public API or cross-crate behavior change; the package check, targeted nextest, and source gates
  cover this claim.

## Verdict

This keeps about-to-wait turn bookkeeping source-auditable without changing runtime behavior. It
does not close `DW-P1-linux-003`; the next true closure event remains a dated real Linux Wayland
compositor acceptance note from `M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
