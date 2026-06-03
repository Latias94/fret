# M60 Runner Docking Pointer/Poll-Up Owner Split - 2026-06-03

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split inside the desktop runner docking module. It
keeps `DW-P1-linux-003` open because real Wayland compositor acceptance still requires the M5
runbook on a qualifying Linux Wayland host.

## Claim

Desktop runner dock-drag pointer discovery/capture-cancel logic now lives in
`crates/fret-launch/src/runner/desktop/runner/docking/pointer.rs`, and platform release-outside
poll-up fallbacks now live in
`crates/fret-launch/src/runner/desktop/runner/docking/poll_up.rs`. The root
`docking.rs` is now a three-module facade over `follow`, `pointer`, and `poll_up`.

The split does not change dock-drag pointer matching, pointer-cancel delivery,
macOS release-outside polling, Windows poll-up diagnostics, cursor override preference, drop
routing, follow stop, caller paths, or the Wayland acceptance boundary.

## Source Shape

- `crates/fret-launch/src/runner/desktop/runner/docking.rs` declares only:
  - `mod follow;`,
  - `mod pointer;`,
  - `mod poll_up;`.
- `crates/fret-launch/src/runner/desktop/runner/docking/pointer.rs` owns:
  - `dock_drag_pointer_id`,
  - `sync_dock_drag_pointer_capture`,
  - `deliver_dock_drag_pointer_cancel`.
- `crates/fret-launch/src/runner/desktop/runner/docking/poll_up.rs` owns:
  - `maybe_finish_dock_drag_released_outside`,
  - `maybe_finish_dock_drag_released_outside_windows`,
  - macOS `macos_is_left_mouse_down` release polling,
  - Windows `win32::is_left_mouse_down` / `cursor_pos_physical` poll-up routing,
  - follow-stop cleanup after Windows poll-up.
- The runner-facing methods are visible only inside `crate::runner::desktop::runner`, preserving
  existing sibling call paths without widening the public `fret-launch` API.

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

This keeps runner dock-drag pointer and release-outside fallback behavior source-auditable without
changing runtime behavior. It does not close `DW-P1-linux-003`; the next true closure event remains
a dated real Linux Wayland compositor acceptance note from
`M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
