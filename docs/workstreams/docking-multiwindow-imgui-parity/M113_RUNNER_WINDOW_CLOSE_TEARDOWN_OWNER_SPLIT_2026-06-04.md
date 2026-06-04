# M113 Runner Window Close Teardown Owner Split - 2026-06-04

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split inside the desktop runner. It keeps
`DW-P1-linux-003` open because real Wayland compositor acceptance still requires the M5 runbook on a
qualifying Linux Wayland host.

## Claim

Desktop runner close-window teardown now lives in
`crates/fret-launch/src/runner/desktop/runner/window_close.rs` instead of the
`window_lifecycle.rs` create/insert owner. The split preserves `close_window`,
`force_close_window`, `close_window_impl`, `before_close_window` checks, dev-state flushes,
DockFloating `stop_dock_tearoff_follow` cancellation, drag cleanup, `webviews.close_window`,
window registry removal,
`record_window_close` diagnostics cleanup, per-window service cleanup, `WindowMetricsService`
cleanup, and main-window clearing.

Marker summary: close-window teardown; checked close; drag cleanup; diagnostics cleanup; does not
close.

## Source Shape

- `crates/fret-launch/src/runner/desktop/runner/window_close.rs` owns `handle_window_close_request`,
  `close_window`, `force_close_window`, `close_window_impl`, close gating, force-close-all,
  shutdown, and event-loop exit.
- `crates/fret-launch/src/runner/desktop/runner/window_lifecycle.rs` keeps window creation and
  insertion helpers without owning checked-close or close-window teardown.
- Existing call sites in `crates/fret-launch/src/runner/desktop/runner/window_requests.rs`,
  `crates/fret-launch/src/runner/desktop/runner/quit_effects.rs`, and
  `crates/fret-launch/src/runner/desktop/runner/window_close.rs` continue to call the same private
  runner methods.

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
- Broader workspace gates were not run because M113 is a private `fret-launch` owner split with no
  public API or cross-crate behavior change; the package check, targeted nextest, and source gates
  cover this claim.

## Verdict

This keeps desktop runner close-window teardown source-auditable in the window close owner and
leaves `window_lifecycle.rs` focused on window creation and insertion. It does not close
`DW-P1-linux-003`; the next true closure event remains a dated real Linux Wayland compositor
acceptance note from `M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
