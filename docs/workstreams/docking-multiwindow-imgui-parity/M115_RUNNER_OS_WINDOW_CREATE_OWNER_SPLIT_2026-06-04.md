# M115 Runner OS Window Create Owner Split - 2026-06-04

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split inside the desktop runner. It keeps
`DW-P1-linux-003` open because real Wayland compositor acceptance still requires the M5 runbook on a
qualifying Linux Wayland host.

## Claim

Desktop runner OS window creation now lives in
`crates/fret-launch/src/runner/desktop/runner/window_os_create.rs` instead of the
`window_lifecycle.rs` create-request owner. The split preserves `create_os_window`, winit
`WindowAttributes` construction, normalized size constraints, initial visibility and accessibility
bootstrap, min/max/resize-increment handling, creation-time resizable/decorations/transparent/
activation/style posture, Windows taskbar creation attributes, macOS parent-window creation
attributes, `macos_window_log` create logging, z-level application, background material
application through `set_window_background_material`, hit-test application through
`set_window_hit_test`, and opacity application through `set_window_opacity`.

Marker summary: OS window creation; create-time style attributes; accessibility bootstrap; does
not close.

## Source Shape

- `crates/fret-launch/src/runner/desktop/runner/window_os_create.rs` owns `create_os_window`,
  winit window attributes, OS window creation, creation-time style application, accessibility
  bootstrap, Windows `TaskbarVisibility` handling, macOS `with_parent_window`, `WindowZLevel`
  setup, `set_window_background_material`, `set_window_hit_test`, and `set_window_opacity`.
- `crates/fret-launch/src/runner/desktop/runner/window_lifecycle.rs` keeps create-request
  orchestration only: request/spec resolution, dev-state spec projection, DockFloating placement
  selection, optional macOS parent handle selection, surface creation, insertion delegation,
  open-style diagnostics, dev-state key registration, and monitor topology refresh.
- Existing call sites in `crates/fret-launch/src/runner/desktop/runner/window_lifecycle.rs` and
  `crates/fret-launch/src/runner/desktop/runner/app_handler.rs` continue to call the same private
  runner method.

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
- Broader workspace gates were not run because M115 is a private `fret-launch` owner split with no
  public API or cross-crate behavior change; the package check, targeted nextest, and source gates
  cover this claim.

## Verdict

This keeps desktop runner OS window creation source-auditable in a dedicated window OS-create owner
and leaves `window_lifecycle.rs` focused on create-request orchestration. It does not close
`DW-P1-linux-003`; the next true closure event remains a dated real Linux Wayland compositor
acceptance note from `M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
