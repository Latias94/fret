# M71 Runner File Transfer Effects Owner Split - 2026-06-04

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split inside the desktop runner. It keeps
`DW-P1-linux-003` open because real Wayland compositor acceptance still requires the M5 runbook on a
qualifying Linux Wayland host.

## Claim

Desktop runner external-drop and file-dialog effect handling now lives in
`crates/fret-launch/src/runner/desktop/runner/file_transfer_effects.rs` instead of the general
effect dispatcher. The split preserves external-drop read completion, file-dialog open selection/cancel,
read-limit capping, capability gating, and release cleanup. The owner-split gate explicitly tracks
external-drop and file-dialog completion events.

## Source Shape

- `crates/fret-launch/src/runner/desktop/runner/mod.rs` declares `mod file_transfer_effects;`.
- `crates/fret-launch/src/runner/desktop/runner/file_transfer_effects.rs` owns:
  - `handle_external_drop_read_all`,
  - `handle_external_drop_read_all_with_limits`,
  - `handle_external_drop_release`,
  - `handle_file_dialog_open`,
  - `handle_file_dialog_read_all`,
  - `handle_file_dialog_read_all_with_limits`,
  - `handle_file_dialog_release`.
- `file_transfer_effects.rs` keeps the native external-drop and file-dialog provider imports next to
  the platform calls and platform completion dispatch.
- `crates/fret-launch/src/runner/desktop/runner/effects.rs` still owns the generic effect loop, but
  now delegates `Effect::ExternalDropReadAll`, `Effect::ExternalDropReadAllWithLimits`,
  `Effect::ExternalDropRelease`, `Effect::FileDialogOpen`, `Effect::FileDialogReadAll`,
  `Effect::FileDialogReadAllWithLimits`, and `Effect::FileDialogRelease` to the file-transfer owner.

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
- `python -m json.tool docs\workstreams\docking-multiwindow-imgui-parity\WORKSTREAM.json`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_docking_multiwindow_workstream_source.py`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Verdict

This keeps desktop runner external-drop and file-dialog effect handling source-auditable without
changing runtime behavior. It does not close `DW-P1-linux-003`; the next true closure event remains
a dated real Linux Wayland compositor acceptance note from
`M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
