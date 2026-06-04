# M108 Runner Window Position Owner Split - 2026-06-04

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split inside the desktop runner. It keeps
`DW-P1-linux-003` open because real Wayland compositor acceptance still requires the M5 runbook on a
qualifying Linux Wayland host.

## Claim

Desktop runner client/screen coordinate conversion and cursor-grab window placement helpers now live
in `crates/fret-launch/src/runner/desktop/runner/window_position.rs` instead of the general
`window.rs` owner. The split preserves client-origin diagnostics, local-position projection,
cursor-grab decoration handling, mixed-DPI cursor-grab estimates, window anchor placement,
cursor-origin fallback, client-rect hit checks, and DockFloating cursor-grab outer-position behavior.

Marker summary: client/screen coordinate helpers; cursor-grab placement; does not close.

## Source Shape

- `crates/fret-launch/src/runner/desktop/runner/window_position.rs` owns
  `WindowClientOriginDiagnostics`, `client_origin_screen`, `screen_pos_in_client`,
  `local_pos_for_screen_pos`, `outer_pos_for_cursor_grab`,
  `scale_decoration_offset_for_target_scale`, `estimated_outer_pos_for_cursor_grab`,
  `compute_window_position_from_anchor`, `compute_window_position_from_cursor`,
  `compute_window_position_from_cursor_grab_estimate`,
  `compute_window_outer_position_from_cursor_grab`, `cursor_screen_pos_fallback_for_window`,
  `screen_pos_in_window`, `local_pos_for_window`, `client_origin_screen_diagnostics_for_window`,
  `window_client_rect_screen`, and `clamp_screen_pos_to_window_client`.
- `crates/fret-launch/src/runner/desktop/runner/window.rs` keeps platform window focus/style,
  platform under-cursor lookup, heuristic z-order fallback, and `WindowRuntime` state without
  defining the client/screen coordinate helper bodies or cursor-grab placement tests.
- `crates/fret-launch/src/runner/desktop/runner/event_routing.rs` and
  `crates/fret-launch/src/runner/desktop/runner/diag_cursor_override.rs` consume the new owner for
  drag diagnostics and diagnostic cursor client-origin fallback.
- Existing call sites continue to use the same `WinitRunner` helper methods, so runtime behavior and
  public effect surfaces remain unchanged.

## Commands Run

```powershell
cargo fmt --package fret-launch -- --check
cargo check -p fret-launch --lib
cargo nextest run -p fret-launch --lib window_position --no-fail-fast
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
- `cargo nextest run -p fret-launch --lib window_position --no-fail-fast`: pass.
- `python -m py_compile tools\gate_docking_multiwindow_workstream_source.py tools\gate_imui_workstream_source.py`:
  pass.
- `python -m json.tool docs\workstreams\docking-multiwindow-imgui-parity\WORKSTREAM.json`:
  pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_docking_multiwindow_workstream_source.py`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass, with the existing `WORKSTREAM.json` CRLF normalization warning.
- Broader workspace gates were not run because M108 is a private `fret-launch` owner split with no
  public API or cross-crate behavior change; the package check, targeted nextest, and source gates
  cover this claim.

## Verdict

This keeps desktop runner coordinate conversion and DockFloating cursor-grab placement
source-auditable in a dedicated window-position owner without changing runtime behavior. It does not
close `DW-P1-linux-003`; the next true closure event remains a dated real Linux Wayland compositor
acceptance note from `M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
