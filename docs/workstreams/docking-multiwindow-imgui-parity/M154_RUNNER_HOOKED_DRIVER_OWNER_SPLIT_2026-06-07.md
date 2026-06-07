# M154 Runner Hooked Driver Owner Split - 2026-06-07

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split inside the desktop runner. It keeps
`DW-P1-linux-003` open because real Wayland compositor acceptance still requires the M5 runbook on a
qualifying Linux Wayland host.

## Claim

Desktop runner launch hook dispatch and `WinitAppDriver` forwarding now lives in
`crates/fret-launch/src/runner/desktop/runner/run/hooked_driver.rs` instead of the general run
entrypoint owner. The split preserves `on_main_window_created`, `on_gpu_ready`,
`FRET_DIAG_RENDERER_PERF`, renderer perf enabling, command/event/render forwarding, window lifecycle
forwarding, and accessibility forwarding.

Marker summary: runner hooked driver owner; launch hook dispatch; renderer perf diagnostic hook;
driver forwarding contract; run entrypoint facade remains thin.

Projection marker: desktop runner entrypoint auditability for docking and editor proof launches.

## Source Shape

- `crates/fret-launch/src/runner/desktop/runner/run.rs` keeps `run_app`,
  `run_app_with_event_loop`, `WinitRunner::new`, asset reload wiring, `WinitAppBuilder`, and the
  existing builder/unit test host.
- `crates/fret-launch/src/runner/desktop/runner/run/hooked_driver.rs` owns `HookedDriver::new` and
  the full `WinitAppDriver` forwarding implementation used by the builder launch path.
- No runtime, app, docking, IMUI authoring, or public launch API surface is widened.

## Commands Run

```powershell
cargo fmt --package fret-launch
cargo check -p fret-launch --lib
cargo fmt --package fret-launch -- --check
cargo nextest run -p fret-launch --lib winit_app_builder --no-fail-fast
cargo nextest run -p fret-launch --lib linux_windowing_capability_posture --no-fail-fast
python -m json.tool docs\workstreams\docking-multiwindow-imgui-parity\WORKSTREAM.json
python -m py_compile tools\gate_docking_multiwindow_workstream_source.py tools\gate_imui_workstream_source.py
python tools\gate_docking_multiwindow_workstream_source.py
python tools\gate_imui_workstream_source.py
python tools\check_workstream_catalog.py
python tools\check_layering.py
(Get-Content crates\fret-launch\src\runner\desktop\runner\run.rs | Measure-Object -Line).Lines
(Get-Content crates\fret-launch\src\runner\desktop\runner\run\hooked_driver.rs | Measure-Object -Line).Lines
git diff --check
```

## Results

- `cargo fmt --package fret-launch`: pass.
- `cargo check -p fret-launch --lib`: pass.
- `cargo fmt --package fret-launch -- --check`: pass.
- `cargo nextest run -p fret-launch --lib winit_app_builder --no-fail-fast`: pass.
- `cargo nextest run -p fret-launch --lib linux_windowing_capability_posture --no-fail-fast`:
  pass.
- `python -m json.tool docs\workstreams\docking-multiwindow-imgui-parity\WORKSTREAM.json`: pass.
- `python -m py_compile tools\gate_docking_multiwindow_workstream_source.py tools\gate_imui_workstream_source.py`:
  pass.
- `python tools\gate_docking_multiwindow_workstream_source.py`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `python tools\check_layering.py`: pass.
- Targeted file size fallback: `run.rs` is 596 lines and `run/hooked_driver.rs` is 298 lines
  after the split.
- `git diff --check`: pass.

## Verdict

Local gates passed for the private owner split only. The split improves source auditability for the
desktop runner launch hook and driver-forwarding path that all native docking/editor proof launches
pass through, without claiming a new Wayland compositor acceptance result.

This note does not close `DW-P1-linux-003`.

The next true closure event remains a dated real Linux Wayland compositor acceptance note from
`M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
