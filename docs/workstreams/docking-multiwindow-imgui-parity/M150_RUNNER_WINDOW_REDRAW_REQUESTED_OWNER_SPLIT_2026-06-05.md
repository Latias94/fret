# M150 Runner Window Redraw Requested Owner Split - 2026-06-05

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split inside the desktop runner. It keeps
`DW-P1-linux-003` open because real Wayland compositor acceptance still requires the M5 runbook on a
qualifying Linux Wayland host.

## Claim

Desktop runner `WindowEvent::RedrawRequested` frame-drive orchestration now lives in
`crates/fret-launch/src/runner/desktop/runner/window_redraw.rs`. The split moves redraw span setup,
pending wheel drain, window environment refresh, pre-render effect drain,
diagnostic screenshot request polling, pending surface-resize fallback, Android text-input handoff, frame prepare,
app render, text-input sync, scene validation, accessibility snapshot update, engine frame record,
webview sync, render-target update application, present owner dispatch, RenderDoc capture bracketing,
present error recovery, hitch summary reporting, Android soft-input forcing, and post-render effect
drain out of `app_handler.rs` while preserving runtime behavior and public effect surfaces.

Marker summary: redraw request owner; ApplicationHandler RedrawRequested dispatch; redraw span; pre-render effect drain; pending wheel drain; surface resize fallback; text-input sync; accessibility snapshot; engine frame record; webview sync; target updates; present owner dispatch; present error recovery; hitch summary; post-render effect drain.

Projection marker: redraw-time frame-drive orchestration from RedrawRequested dispatch through
post-render effect drain.

## Source Shape

- `crates/fret-launch/src/runner/desktop/runner/window_redraw.rs` owns
  `handle_window_redraw_requested`.
- The owner sequences existing child owners for frame prepare, app render, text input,
  accessibility, engine record, webviews, target updates, present, RenderDoc capture, present error
  recovery, and hitch summaries.
- `crates/fret-launch/src/runner/desktop/runner/app_handler.rs` keeps only the winit
  `WindowEvent::RedrawRequested` match arm and dispatches to `handle_window_redraw_requested`.
- Existing focused redraw child owners remain intact; this split adds a workflow owner above them
  rather than moving child policy back into `app_handler.rs`.

## Commands Run

```powershell
cargo fmt --package fret-launch
cargo check -p fret-launch --lib
cargo check -p fret-launch --features diag-screenshots --lib
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
- `cargo check -p fret-launch --features diag-screenshots --lib`: pass.
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
- Broader workspace gates were not run because M150 is a private `fret-launch` owner split with no
  public API or cross-crate behavior change; the package checks, targeted nextest, and source gates
  cover this claim.

## Verdict

This keeps redraw-time frame-drive orchestration source-auditable in a named owner while leaving
`app_handler.rs` responsible for winit event dispatch. It does not close `DW-P1-linux-003`; the next
true closure event remains a dated real Linux Wayland compositor acceptance note from
`M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
