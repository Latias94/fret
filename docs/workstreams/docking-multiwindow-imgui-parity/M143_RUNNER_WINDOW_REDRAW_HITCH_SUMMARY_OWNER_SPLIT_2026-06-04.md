# M143 Runner Window Redraw Hitch Summary Owner Split - 2026-06-04

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split inside the desktop runner. It keeps
`DW-P1-linux-003` open because real Wayland compositor acceptance still requires the M5 runbook on a
qualifying Linux Wayland host.

## Claim

Desktop runner redraw hitch summary assembly now lives in
`crates/fret-launch/src/runner/desktop/runner/window_redraw_hitch_summary.rs`. The split moves the
threshold check, total redraw elapsed calculation, formatted hitch line, and
`write_redraw_hitch_log(...)` call out of `app_handler.rs` while preserving phase timing,
`RedrawPhase` spans, successful-present ordering, present error handling, Android soft-input follow
up, and end-of-redraw effect draining.

Marker summary: redraw hitch summary owner; total redraw elapsed; hitch threshold check;
app-handler hitch-summary dispatch.

Projection marker: redraw-time hitch summary after present success.

## Source Shape

- `crates/fret-launch/src/runner/desktop/runner/window_redraw_hitch_summary.rs` owns
  `WindowRedrawHitchSummaryInput` and `maybe_write_window_redraw_hitch_summary`.
- The owner keeps `RedrawHitchConfig`, `Instant`, `AppWindowId`, `TickId`, `FrameId`, `Rect`,
  phase elapsed values, `scene_ops`, `bounds`, and `scale_factor` at the summary boundary.
- The owner preserves `started.elapsed().as_millis() as u64`, `total_ms < config.hitch_ms`, the
  existing `redraw hitch window=...` line shape, and `write_redraw_hitch_log(&format!(...))`.
- `crates/fret-launch/src/runner/desktop/runner/app_handler.rs` keeps only
  `maybe_write_window_redraw_hitch_summary` dispatch after present-error handling and before
  Android soft-input follow-up.

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
- Broader workspace gates were not run because M143 is a private `fret-launch` owner split with no
  public API or cross-crate behavior change; the package check, targeted nextest, and source gates
  cover this claim.

## Verdict

This keeps redraw hitch summary formatting source-auditable in a named owner while leaving
`app_handler.rs` responsible for redraw orchestration. It does not close `DW-P1-linux-003`; the next
true closure event remains a dated real Linux Wayland compositor acceptance note from
`M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
