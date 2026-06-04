# M146 Runner Window Redraw Webviews Owner Split - 2026-06-04

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split inside the desktop runner. It keeps
`DW-P1-linux-003` open because real Wayland compositor acceptance still requires the M5 runbook on a
qualifying Linux Wayland host.

## Claim

Desktop runner redraw-time webview snapshot selection and sync dispatch now live in
`crates/fret-launch/src/runner/desktop/runner/window_redraw_webviews.rs`. The split moves the
feature-gated `WebViewHost` check, `webview_has_surfaces_for_window` gate, cached
`last_semantics_snapshot` reuse, fallback `driver.semantics_snapshot`, and
`RunnerWebViewState::sync_window` dispatch out of `app_handler.rs` while preserving webview request
handling, stale-surface GC, native webview host ownership, target updates, present ordering, runtime
behavior, and public effect surfaces.

Marker summary: redraw webviews owner; webview snapshot selection; app-handler webview sync dispatch.

Evidence marker: webview request handling.

Projection marker: redraw-time webview snapshot and sync before target updates.

## Source Shape

- `crates/fret-launch/src/runner/desktop/runner/window_redraw_webviews.rs` owns
  `WindowRedrawWebViewSyncInput`, `sync_window_redraw_webviews`, and
  `window_redraw_webview_snapshot`.
- The owner keeps `WebViewHost`, `webview_has_surfaces_for_window`, `last_semantics_snapshot`,
  `driver.semantics_snapshot`, and `RunnerWebViewState::sync_window` at the redraw webview sync
  lifecycle boundary.
- `crates/fret-launch/src/runner/desktop/runner/app_handler.rs` keeps only webview sync owner
  dispatch after frame recording and before target updates.
- `crates/fret-launch/src/runner/desktop/runner/webview.rs` still owns request/event bridging,
  placement requests, stale-surface GC, and native host state.

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
- Broader workspace gates were not run because M146 is a private `fret-launch` owner split with no
  public API or cross-crate behavior change; the package check, targeted nextest, and source gates
  cover this claim.

## Verdict

This keeps redraw-time webview snapshot selection and sync source-auditable in a named owner while
leaving `app_handler.rs` responsible for redraw orchestration. It does not close
`DW-P1-linux-003`; the next true closure event remains a dated real Linux Wayland compositor
acceptance note from `M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
