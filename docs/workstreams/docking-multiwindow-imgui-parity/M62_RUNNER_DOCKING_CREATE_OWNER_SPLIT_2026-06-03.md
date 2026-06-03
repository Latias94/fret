# M62 Runner Docking Create Owner Split - 2026-06-03

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split inside the desktop runner. It keeps
`DW-P1-linux-003` open because real Wayland compositor acceptance still requires the M5 runbook on a
qualifying Linux Wayland host.

## Claim

Desktop runner DockFloating/DockRestore post-create policy now lives in
`crates/fret-launch/src/runner/desktop/runner/docking/create.rs` instead of the general effect
dispatcher. The split preserves DockFloating registration, macOS raise behavior, cursor-grab
placement refinement, follow initialization, z-level diagnostics, driver `window_created` ordering,
and the Wayland acceptance boundary.

## Source Shape

- `crates/fret-launch/src/runner/desktop/runner/docking.rs` declares `mod create;` alongside the
  existing follow, pointer, and poll-up owners.
- `crates/fret-launch/src/runner/desktop/runner/docking/create.rs` owns
  `handle_created_docking_window`.
- `handle_created_docking_window` owns:
  - DockFloating/DockRestore registry insertion,
  - macOS source-window raise for newly created DockFloating windows,
  - post-create cursor-grab outer-position refinement,
  - reliable-position follow initialization,
  - temporary AlwaysOnTop style diagnostics for following DockFloating windows,
  - deferred front request enqueueing for the created DockFloating window.
- `crates/fret-launch/src/runner/desktop/runner/effects.rs` still owns the generic
  `WindowRequest::Create` effect flow: create the OS window, call the docking post-create owner,
  invoke the driver `window_created` hook, and request redraw.

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

This keeps DockFloating creation-time runner policy source-auditable without changing runtime
behavior. It does not close `DW-P1-linux-003`; the next true closure event remains a dated real
Linux Wayland compositor acceptance note from
`M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
