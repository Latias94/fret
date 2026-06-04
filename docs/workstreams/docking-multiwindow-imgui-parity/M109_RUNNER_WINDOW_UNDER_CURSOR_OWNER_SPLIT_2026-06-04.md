# M109 Runner Window Under-Cursor Owner Split - 2026-06-04

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split inside the desktop runner. It keeps
`DW-P1-linux-003` open because real Wayland compositor acceptance still requires the M5 runbook on a
qualifying Linux Wayland host.

## Claim

Desktop runner platform under-cursor lookup, heuristic z-order fallback, Windows root-HWND lookup,
and z-order bump bookkeeping now live in
`crates/fret-launch/src/runner/desktop/runner/window_under_cursor.rs` instead of the general
`window.rs` owner. The split preserves macOS ordered-window lookup, Windows z-order walk fallback,
heuristic rect fallback, preferred-window exclusion, and DockFloating drag target identification.

Marker summary: platform under-cursor lookup; z-order fallback; does not close.

## Source Shape

- `crates/fret-launch/src/runner/desktop/runner/window_under_cursor.rs` owns
  `WindowUnderCursorHit`, `ns_window_number_for_window`,
  `ordered_ns_window_numbers_front_to_back`, `window_under_cursor_macos`, `hwnd_for_window`,
  `window_under_cursor_win32`, `window_under_cursor_platform`,
  `window_under_cursor_best_effort`, and `bump_window_z_order`.
- `crates/fret-launch/src/runner/desktop/runner/window.rs` keeps `WindowRuntime` state plus platform
  focus/style/hit-test/background-material helpers, but no longer defines the under-cursor,
  z-order fallback, or z-order bump helper bodies.
- Existing call sites in `crates/fret-launch/src/runner/desktop/runner/event_routing.rs`,
  `crates/fret-launch/src/runner/desktop/runner/device_events.rs`,
  `crates/fret-launch/src/runner/desktop/runner/window_geometry.rs`,
  `crates/fret-launch/src/runner/desktop/runner/window_position.rs`,
  `crates/fret-launch/src/runner/desktop/runner/app_handler.rs`, and
  `crates/fret-launch/src/runner/desktop/runner/window_lifecycle.rs` continue to use the same
  `WinitRunner` helper methods, so runtime behavior and public effect surfaces remain unchanged.

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
- Broader workspace gates were not run because M109 is a private `fret-launch` owner split with no
  public API or cross-crate behavior change; the package check, targeted nextest, and source gates
  cover this claim.

## Verdict

This keeps desktop runner under-cursor and z-order fallback behavior source-auditable in a dedicated
owner without changing runtime behavior. It does not close `DW-P1-linux-003`; the next true closure
event remains a dated real Linux Wayland compositor acceptance note from
`M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
