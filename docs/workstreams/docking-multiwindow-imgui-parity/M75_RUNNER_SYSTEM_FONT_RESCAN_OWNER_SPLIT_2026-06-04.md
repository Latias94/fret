# M75 Runner System Font Rescan Owner Split - 2026-06-04

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split inside the desktop runner. It keeps
`DW-P1-linux-003` open because real Wayland compositor acceptance still requires the M5 runbook on a
qualifying Linux Wayland host.

## Claim

Desktop runner system-font rescan state handling now lives in
`crates/fret-launch/src/runner/desktop/runner/text_effects.rs` instead of the general effect
dispatcher module. The split preserves async startup gating, explicit rescan request behavior,
state publication, completed-result application, resize deferral, redraw follow-up, and pending
restart behavior. The general effect loop still drives `apply_pending_system_font_rescan_result(...)`
each drain turn.

## Source Shape

- `crates/fret-launch/src/runner/desktop/runner/text_effects.rs` owns:
  - `system_font_rescan_async_enabled`,
  - `system_font_catalog_startup_async_enabled`,
  - `publish_system_font_rescan_state`,
  - `request_system_font_rescan`,
  - `rescan_system_fonts_sync`,
  - `finish_system_font_rescan_result_state`,
  - `apply_pending_system_font_rescan_result`.
- `text_effects.rs` also owns the resize-deferral helpers used by completed system-font rescan
  application.
- `crates/fret-launch/src/runner/desktop/runner/effects.rs` still owns the generic effect loop, but
  no longer owns the system-font rescan state machine.

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

## Verdict

This keeps desktop runner system-font rescan state handling source-auditable without changing runtime behavior. It does not close `DW-P1-linux-003`; the next true closure event remains a dated real Linux Wayland compositor acceptance note from `M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
