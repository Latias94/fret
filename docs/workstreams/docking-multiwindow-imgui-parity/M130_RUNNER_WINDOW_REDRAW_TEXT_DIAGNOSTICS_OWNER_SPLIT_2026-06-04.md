# M130 Runner Window Redraw Text Diagnostics Owner Split - 2026-06-04

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split inside the desktop runner. It keeps
`DW-P1-linux-003` open because real Wayland compositor acceptance still requires the M5 runbook on a
qualifying Linux Wayland host.

## Claim

Desktop runner redraw-time renderer text diagnostics publication now lives in
`crates/fret-launch/src/runner/desktop/runner/window_redraw_text_diagnostics.rs`. The split moves
`FRET_RENDER_TEXT_DEBUG` and `FRET_DIAG_DIR` mode detection, `begin_text_diagnostics_frame`,
SVG text bridge diagnostics publication, renderer text diagnostics/font-trace/fallback-policy
snapshot publication, and the debug vs untracked global-write policy out of `app_handler.rs` while
preserving redraw ordering around driver render and renderer perf diagnostics.

Marker summary: redraw text diagnostics owner; renderer text diagnostics publication;
app-handler dispatch only.

Projection marker: renderer text diagnostics publication.

## Source Shape

- `crates/fret-launch/src/runner/desktop/runner/window_redraw_text_diagnostics.rs` owns
  `WindowRedrawTextDiagnosticsMode`, `window_redraw_text_diagnostics_mode_from_env`,
  `begin_window_redraw_text_diagnostics_frame`, and `publish_window_redraw_text_diagnostics`.
- The owner explicitly receives `app`, `renderer`, `frame_id`, and a copyable mode value, so it
  does not borrow the whole redraw state while `surface` remains mutably borrowed.
- It owns `FRET_RENDER_TEXT_DEBUG`, `FRET_DIAG_DIR`, `renderer.begin_text_diagnostics_frame`,
  `publish_renderer_svg_text_bridge_diagnostics`, `text_diagnostics_snapshot`,
  `text_font_trace_snapshot`, `text_fallback_policy_snapshot`,
  `RendererTextPerfSnapshot::default`, `RendererTextFontTraceSnapshot::default`,
  `RendererTextFallbackPolicySnapshot::default`, `app.set_global`, and
  `app.with_global_mut_untracked`.
- `crates/fret-launch/src/runner/desktop/runner/app_handler.rs` keeps only redraw-time text
  diagnostics mode creation plus begin/publish dispatch around render and renderer perf
  diagnostics.
- Web runner text diagnostics remain unchanged; M130 is a desktop private owner split, not a
  cross-runner helper unification.

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
- Broader workspace gates were not run because M130 is a private `fret-launch` owner split with no
  public API or cross-crate behavior change; the package check, targeted nextest, and source gates
  cover this claim.

## Verdict

This keeps redraw-time renderer text diagnostics publication source-auditable in a named owner
while leaving `app_handler.rs` as dispatch plus redraw orchestration. It does not close
`DW-P1-linux-003`; the next true closure event remains a dated real Linux Wayland compositor
acceptance note from `M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
