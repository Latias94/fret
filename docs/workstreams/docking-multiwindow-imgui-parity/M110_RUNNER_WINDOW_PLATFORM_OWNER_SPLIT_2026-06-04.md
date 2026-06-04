# M110 Runner Window Platform Owner Split - 2026-06-04

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split inside the desktop runner. It keeps
`DW-P1-linux-003` open because real Wayland compositor acceptance still requires the M5 runbook on a
qualifying Linux Wayland host.

## Claim

Desktop runner low-level platform window operations now live in
`crates/fret-launch/src/runner/desktop/runner/window_platform.rs` instead of the general
`window.rs` state owner. The split preserves platform raise/focus behavior, Windows foreground raising,
macOS ordered-front logging, opacity application, hit-test passthrough, region hit-test fallback,
background material application, and the high-level `window_style.rs` request pipeline.

Marker summary: platform window operations; raise/focus; style material helpers; does not close.

## Source Shape

- `crates/fret-launch/src/runner/desktop/runner/window_platform.rs` owns `bring_window_to_front`,
  `set_window_opacity`, `set_window_hit_test_passthrough_all`, `set_window_hit_test`,
  `set_window_background_material`, and the private Windows region passthrough helper.
- `crates/fret-launch/src/runner/desktop/runner/window.rs` keeps `WindowRuntime`,
  `PendingWheelEvent`, `PendingFrontRequest`, `TimerEntry`, and `DockTearoffFollow` without
  defining platform operation helper bodies.
- Existing call sites in `crates/fret-launch/src/runner/desktop/runner/window_style.rs`,
  `crates/fret-launch/src/runner/desktop/runner/window_lifecycle.rs`,
  `crates/fret-launch/src/runner/desktop/runner/window_geometry.rs`,
  `crates/fret-launch/src/runner/desktop/runner/event_routing.rs`, and
  `crates/fret-launch/src/runner/desktop/runner/docking/create.rs` continue to use the same helper
  semantics through the private runner module boundary.

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
- Broader workspace gates were not run because M110 is a private `fret-launch` owner split with no
  public API or cross-crate behavior change; the package check, targeted nextest, and source gates
  cover this claim.

## Verdict

This keeps desktop runner platform window operations source-auditable in a dedicated owner and
leaves `window.rs` as the runtime state record owner. It does not close `DW-P1-linux-003`; the next
true closure event remains a dated real Linux Wayland compositor acceptance note from
`M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
