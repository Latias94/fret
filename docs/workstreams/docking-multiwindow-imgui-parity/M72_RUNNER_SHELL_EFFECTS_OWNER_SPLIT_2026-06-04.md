# M72 Runner Shell Effects Owner Split - 2026-06-04

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split inside the desktop runner. It keeps
`DW-P1-linux-003` open because real Wayland compositor acceptance still requires the M5 runbook on a
qualifying Linux Wayland host.

## Claim

Desktop runner shell action handling now lives in
`crates/fret-launch/src/runner/desktop/runner/shell_effects.rs` instead of the general effect
dispatcher. The split preserves macOS about-panel and app hide/unhide actions, open URL capability gating,
and share-sheet unavailable completion dispatch. The owner-split gate explicitly tracks
shell action handling and share-sheet completion events.

## Source Shape

- `crates/fret-launch/src/runner/desktop/runner/mod.rs` declares `mod shell_effects;`.
- `crates/fret-launch/src/runner/desktop/runner/shell_effects.rs` owns:
  - `handle_show_about_panel`,
  - `handle_hide_app`,
  - `handle_hide_other_apps`,
  - `handle_unhide_all_apps`,
  - `handle_open_url`,
  - `handle_share_sheet_show`.
- `shell_effects.rs` keeps the native open-url provider import and the shell capability checks next to
  the platform calls and share-sheet completion dispatch.
- `crates/fret-launch/src/runner/desktop/runner/effects.rs` still owns the generic effect loop, but
  now delegates `Effect::ShowAboutPanel`, `Effect::HideApp`, `Effect::HideOtherApps`,
  `Effect::UnhideAllApps`, `Effect::OpenUrl`, and `Effect::ShareSheetShow` to the shell owner.

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

This keeps desktop runner shell actions source-auditable without changing runtime behavior.
It does not close `DW-P1-linux-003`; the next true closure event remains a dated real Linux
Wayland compositor acceptance note from `M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
