# M125 Runner Window Redraw Accessibility Owner Split - 2026-06-04

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split inside the desktop runner. It keeps
`DW-P1-linux-003` open because real Wayland compositor acceptance still requires the M5 runbook on a
qualifying Linux Wayland host.

## Claim

Desktop runner redraw-time accessibility semantics update handling now lives in
`crates/fret-launch/src/runner/desktop/runner/window_redraw_accessibility.rs`. The split moves the
active accessibility check, driver semantics snapshot request, AccessKit tree update construction,
active accessibility update, and `last_semantics_snapshot` cache maintenance out of
`app_handler.rs` while preserving redraw ordering after scene validation and before engine frame
recording.

Marker summary: redraw accessibility owner; semantics snapshot cache; app-handler dispatch only.

## Source Shape

- `crates/fret-launch/src/runner/desktop/runner/window_redraw_accessibility.rs` owns
  `update_window_redraw_accessibility_snapshot`.
- The owner explicitly receives `driver`, `app`, `app_window`, `user`, `accessibility`,
  `last_semantics_snapshot`, and `scale_factor` so the redraw block borrow structure does not
  change while `surface` is mutably borrowed.
- It owns `state.accessibility.as_mut`, `a11y.is_active`, `driver.semantics_snapshot`,
  `accessibility::tree_update_from_snapshot`, `a11y.update_if_active`, and
  `state.last_semantics_snapshot` maintenance.
- `crates/fret-launch/src/runner/desktop/runner/app_handler.rs` keeps only the redraw-time
  accessibility dispatch call between scene validation and engine-frame recording.

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
- Broader workspace gates were not run because M125 is a private `fret-launch` owner split with no
  public API or cross-crate behavior change; the package check, targeted nextest, and source gates
  cover this claim.

## Verdict

This keeps redraw-time accessibility semantics cache/update integration source-auditable in a named
owner while leaving `app_handler.rs` as dispatch plus redraw orchestration. It does not close
`DW-P1-linux-003`; the next true closure event remains a dated real Linux Wayland compositor
acceptance note from `M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
