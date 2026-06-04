# M99 Runner About-To-Wait Dock Follow Stop Owner Split - 2026-06-04

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split inside the desktop runner. It keeps
`DW-P1-linux-003` open because real Wayland compositor acceptance still requires the M5 runbook on a
qualifying Linux Wayland host.

## Claim

Desktop runner about-to-wait DockFloating follow-stop handling now lives in
`crates/fret-launch/src/runner/desktop/runner/docking/follow.rs` instead of the general
`ApplicationHandler` integration. The split preserves no-pointer-motion follow stop, canceled drag
session detection, source-window left-button fallback detection, `Instant::now()` stop timing,
non-raising stop semantics, and final effect draining from the `about_to_wait` path.

Marker summary: no-pointer-motion follow stop; source-window left-button fallback detection; non-raising stop semantics; does not close.

## Source Shape

- `crates/fret-launch/src/runner/desktop/runner/docking/follow.rs` owns
  `handle_about_to_wait_dock_follow_stop` beside DockFloating follow update/stop helpers.
- `crates/fret-launch/src/runner/desktop/runner/app_handler.rs` still owns the
  `ApplicationHandler::about_to_wait` trait hook and delegates the idle follow-stop check to the
  docking follow owner.
- The original ordering is preserved: accessibility action draining still completes first, the
  follow-stop check still runs before the broad effect drain, and platform released-outside cleanup
  still runs after that drain.

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
- Broader workspace gates were not run because M99 is a private `fret-launch` owner split with no
  public API or cross-crate behavior change; the package check, targeted nextest, and source gates
  cover this claim.

## Verdict

This keeps about-to-wait DockFloating follow-stop handling source-auditable without changing runtime
behavior. It does not close `DW-P1-linux-003`; the next true closure event remains a dated real
Linux Wayland compositor acceptance note from
`M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
