# M126 Runner Window Redraw Text Input Owner Split - 2026-06-04

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split inside the desktop runner. It keeps
`DW-P1-linux-003` open because real Wayland compositor acceptance still requires the M5 runbook on a
qualifying Linux Wayland host.

## Claim

Desktop runner redraw-time text-input snapshot application now lives in
`crates/fret-launch/src/runner/desktop/runner/window_redraw_text_input.rs`. The split moves
`WindowTextInputSnapshotService` lookup, IME allowed-state synchronization, Android soft-input
request forwarding, cursor-area synchronization, surrounding-text synchronization,
`FRET_IME_DEBUG` snapshot logging, and follow-up `prepare_frame` out of `app_handler.rs` while
preserving redraw ordering after render and before scene validation/accessibility update.

Marker summary: redraw text-input owner; IME snapshot application; Android soft-input forwarding;
app-handler dispatch only.

## Source Shape

- `crates/fret-launch/src/runner/desktop/runner/window_redraw_text_input.rs` owns
  `apply_window_redraw_text_input_snapshot`.
- The owner explicitly receives `app`, `app_window`, `platform`, `window`, and on Android
  `android_soft_input_request` so the redraw block borrow structure stays field-scoped while
  `surface` is mutably borrowed.
- It owns `WindowTextInputSnapshotService` lookup, `set_ime_allowed`, Android
  `android_soft_input_request` forwarding, `set_ime_cursor_area`, `ImeSurroundingTextUpdate`,
  `set_ime_surrounding_text`, `FRET_IME_DEBUG` snapshot logging, and `platform.prepare_frame`.
- `crates/fret-launch/src/runner/desktop/runner/app_handler.rs` keeps only cfg-gated redraw-time
  text-input snapshot dispatch after render and before scene validation.

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
- Broader workspace gates were not run because M126 is a private `fret-launch` owner split with no
  public API or cross-crate behavior change; the package check, targeted nextest, and source gates
  cover this claim.

## Verdict

This keeps redraw-time text-input/IME snapshot integration source-auditable in a named owner while
leaving `app_handler.rs` as dispatch plus redraw orchestration. It does not close
`DW-P1-linux-003`; the next true closure event remains a dated real Linux Wayland compositor
acceptance note from `M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
