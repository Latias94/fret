# M89 Runner Monitor Topology Owner Split - 2026-06-04

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split inside the desktop runner. It keeps
`DW-P1-linux-003` open because real Wayland compositor acceptance still requires the M5 runbook on a
qualifying Linux Wayland host.

## Claim

Desktop runner monitor topology diagnostics now live in
`crates/fret-launch/src/runner/desktop/runner/monitor_topology.rs` instead of the general
`ApplicationHandler` integration. The split preserves monitor enumeration, scale-factor fallback,
stable monitor sorting, virtual desktop bounds construction, and
`RunnerMonitorTopologyDiagnosticsStore` updates.

## Source Shape

- `crates/fret-launch/src/runner/desktop/runner/mod.rs` declares `mod monitor_topology;`.
- `crates/fret-launch/src/runner/desktop/runner/monitor_topology.rs` owns
  `collect_runner_monitor_topology_snapshot` and
  `refresh_runner_monitor_topology_diagnostics`.
- `crates/fret-launch/src/runner/desktop/runner/app_handler.rs` still owns winit event routing,
  surface lifecycle entry points, redraw execution, and `ApplicationHandler` wiring, but now
  delegates monitor topology snapshot construction and diagnostics publication to the monitor
  topology owner.
- The original ordering is preserved: topology diagnostics are refreshed from the same
  `can_create_surfaces`, window-event, and `about_to_wait` paths, and window creation paths that
  need a refresh still call the same runner method.

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
- `python -m py_compile tools\gate_docking_multiwindow_workstream_source.py tools\gate_imui_workstream_source.py`:
  pass.
- `python -m json.tool docs\workstreams\docking-multiwindow-imgui-parity\WORKSTREAM.json`:
  pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_docking_multiwindow_workstream_source.py`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass, with the existing `WORKSTREAM.json` CRLF normalization warning.
- Broader workspace gates were not run because M89 is a private `fret-launch` owner split with no
  public API or cross-crate behavior change; the package check, targeted nextest, and source gates
  cover this claim.

## Verdict

This keeps desktop runner monitor topology diagnostics source-auditable without changing runtime
behavior. It does not close `DW-P1-linux-003`; the next true closure event remains a dated real
Linux Wayland compositor acceptance note from
`M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
