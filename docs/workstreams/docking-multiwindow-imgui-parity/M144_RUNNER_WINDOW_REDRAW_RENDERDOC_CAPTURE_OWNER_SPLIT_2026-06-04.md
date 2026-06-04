# M144 Runner Window Redraw RenderDoc Capture Owner Split - 2026-06-04

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split inside the desktop runner. It keeps
`DW-P1-linux-003` open because real Wayland compositor acceptance still requires the M5 runbook on a
qualifying Linux Wayland host.

## Claim

Desktop runner redraw-time RenderDoc capture begin/end now lives in
`crates/fret-launch/src/runner/desktop/runner/window_redraw_renderdoc_capture.rs`. The split moves
`begin_capture_if_requested()` and conditional `end_capture()` dispatch out of `app_handler.rs`
while preserving RenderDoc initialization, capture request hotkey handling, frame capture ordering
around redraw work, present error handling, hitch summary dispatch, runtime behavior, and public
effect surfaces.

Marker summary: redraw renderdoc capture owner; capture begin; capture end;
app-handler renderdoc-capture dispatch.

Projection marker: redraw-time renderdoc capture lifecycle.

## Source Shape

- `crates/fret-launch/src/runner/desktop/runner/window_redraw_renderdoc_capture.rs` owns
  `begin_window_redraw_renderdoc_capture` and `end_window_redraw_renderdoc_capture`.
- The owner keeps `Option<&mut RenderDocCapture>`, `begin_capture_if_requested`, the `capturing`
  boolean, and `end_capture` at the redraw capture lifecycle boundary.
- `crates/fret-launch/src/runner/desktop/runner/app_handler.rs` keeps only begin/end owner dispatch
  around the existing redraw frame work.
- `crates/fret-launch/src/runner/desktop/runner/render.rs` still owns RenderDoc initialization, and
  `crates/fret-launch/src/runner/desktop/runner/window_mapped_events.rs` still owns capture request
  hotkey handling.

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
- Broader workspace gates were not run because M144 is a private `fret-launch` owner split with no
  public API or cross-crate behavior change; the package check, targeted nextest, and source gates
  cover this claim.

## Verdict

This keeps redraw-time RenderDoc capture lifecycle source-auditable in a named owner while leaving
`app_handler.rs` responsible for redraw orchestration. It does not close `DW-P1-linux-003`; the next
true closure event remains a dated real Linux Wayland compositor acceptance note from
`M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
