# M123 Runner Window Moved Events Owner Split - 2026-06-04

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split inside the desktop runner. It keeps
`DW-P1-linux-003` open because real Wayland compositor acceptance still requires the M5 runbook on a
qualifying Linux Wayland host.

## Claim

Desktop runner macOS window-moved event handling now lives in
`crates/fret-launch/src/runner/desktop/runner/window_moved_events.rs`. The split moves the
`WindowEvent::Moved(..)` macOS hit-test region refresh path out of `app_handler.rs` while preserving
the `macos-hit-test-regions` feature/target guard, active-region check, and latest mouse-location refresh.

Marker summary: moved window event owner; macOS hit-test refresh; app-handler dispatch only.

## Source Shape

- `crates/fret-launch/src/runner/desktop/runner/window_moved_events.rs` owns
  `handle_window_moved`.
- `handle_window_moved` owns the `macos_hit_test::has_active_regions` guard and
  `macos_hit_test::apply_latest_mouse_location` refresh.
- `crates/fret-launch/src/runner/desktop/runner/mod.rs` only declares the owner under
  `#[cfg(all(target_os = "macos", feature = "macos-hit-test-regions"))]`.
- `crates/fret-launch/src/runner/desktop/runner/app_handler.rs` keeps only the guarded
  `WindowEvent::Moved(..)` dispatch call.

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
- Broader workspace gates were not run because M123 is a private, cfg-gated `fret-launch` owner
  split with no public API or cross-crate behavior change; the package check, targeted nextest, and
  source gates cover this claim.

## Verdict

This keeps macOS moved-event hit-test refresh source-auditable in a named owner while leaving
`app_handler.rs` as dispatch plus redraw orchestration. It does not close `DW-P1-linux-003`; the
next true closure event remains a dated real Linux Wayland compositor acceptance note from
`M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
