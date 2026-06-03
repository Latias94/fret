# M86 Runner Effect Queue Owner Split - 2026-06-04

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split inside the desktop runner. It keeps
`DW-P1-linux-003` open because real Wayland compositor acceptance still requires the M5 runbook on a
qualifying Linux Wayland host.

## Claim

Desktop runner effect queue dispatch now lives in
`crates/fret-launch/src/runner/desktop/runner/effect_queue.rs` instead of the general effect drain
loop. The split preserves effect ordering, handler routing, streaming upload stats mutation, dirty
window frame preparation, dirty-window tracking, and the early-exit behavior for `QuitApp` and
exiting window requests.

## Source Shape

- `crates/fret-launch/src/runner/desktop/runner/mod.rs` declares `mod effect_queue;`.
- `crates/fret-launch/src/runner/desktop/runner/effect_queue.rs` owns
  `dispatch_effect_queue`.
- `crates/fret-launch/src/runner/desktop/runner/effects.rs` still owns the fixed-point drain loop,
  inbox/system-font/streaming preprocessing, turn tracing, lifecycle polling, streaming diagnostics,
  dirty-window preparation, timers, drag-hover cleanup, model/global propagation, and pending
  streaming redraw wakeups.
- The original ordering is preserved: `did_work` records non-empty effect queues before dispatch,
  effect handlers run in queue order, image update handlers still mutate the same streaming stats
  before diagnostics publish, and queue dispatch returns the same early-exit signal previously set
  inline by `QuitApp` and exiting `Window` requests.

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
- Broader workspace gates were not run because M86 is a private `fret-launch` owner split with no
  public API or cross-crate behavior change; the package check, targeted nextest, and source gates
  cover this claim.

## Verdict

This keeps desktop runner effect queue dispatch source-auditable without changing runtime behavior.
It does not close `DW-P1-linux-003`; the next true closure event remains a dated real Linux Wayland
compositor acceptance note from `M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
