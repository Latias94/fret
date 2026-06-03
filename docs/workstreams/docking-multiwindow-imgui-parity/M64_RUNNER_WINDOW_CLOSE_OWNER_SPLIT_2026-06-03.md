# M64 Runner Window Close Owner Split - 2026-06-03

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split inside the desktop runner. It keeps
`DW-P1-linux-003` open because real Wayland compositor acceptance still requires the M5 runbook on a
qualifying Linux Wayland host.

## Claim

Desktop runner `WindowRequest::Close` shutdown policy now lives in
`crates/fret-launch/src/runner/desktop/runner/window_close.rs` instead of the general effect
dispatcher. The split preserves checked close, main-window exit policy, force-close of remaining
windows, and app/event-loop shutdown.

## Source Shape

- `crates/fret-launch/src/runner/desktop/runner/mod.rs` declares `mod window_close;`.
- `crates/fret-launch/src/runner/desktop/runner/window_close.rs` owns
  `handle_window_close_request`.
- `handle_window_close_request` owns:
  - checked close through `close_window`,
  - `exit_on_main_window_close` policy,
  - force-closing remaining windows,
  - shutdown when no windows remain,
  - dispatcher shutdown and event-loop exit.
- `crates/fret-launch/src/runner/desktop/runner/effects.rs` still owns the generic
  `WindowRequest::Close` effect branch, but now delegates directly to
  `handle_window_close_request`.

## Commands Run

```powershell
cargo fmt --package fret-launch -- --check
cargo check -p fret-launch --lib
cargo nextest run -p fret-launch --lib linux_windowing_capability_posture --no-fail-fast
python -m py_compile tools\gate_docking_multiwindow_workstream_source.py tools\gate_imui_workstream_source.py
python -m json.tool docs\workstreams\docking-multiwindow-imgui-parity\WORKSTREAM.json
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
- `python -m json.tool docs\workstreams\docking-multiwindow-imgui-parity\WORKSTREAM.json`: pass.
- `python tools\gate_docking_multiwindow_workstream_source.py`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Verdict

This keeps window-close and app-exit policy source-auditable without changing runtime behavior. It does not close
`DW-P1-linux-003`; the next true closure event remains a dated real Linux Wayland compositor acceptance note from
`M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
