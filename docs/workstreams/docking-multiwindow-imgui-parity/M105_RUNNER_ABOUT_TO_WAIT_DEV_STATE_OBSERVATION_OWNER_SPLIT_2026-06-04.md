# M105 Runner About-To-Wait Dev-State Observation Owner Split - 2026-06-04

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split inside the desktop runner. It keeps
`DW-P1-linux-003` open because real Wayland compositor acceptance still requires the M5 runbook on a
qualifying Linux Wayland host.

## Claim

Desktop runner about-to-wait dev-state window observation now lives in
`crates/fret-launch/src/runner/desktop/runner/dev_state.rs` instead of the general
`ApplicationHandler` integration. The split preserves the desktop-only `dev-state` feature gate,
alive-window filtering, app-export ordering, window-key snapshot iteration, surface-size to logical
size projection, outer-position sampling, and reuse of the same `turn_now` timestamp produced by
turn bookkeeping.

Marker summary: dev-state window observation; alive-window filtering; app-export ordering; turn timestamp reuse; does not close.

## Source Shape

- `crates/fret-launch/src/runner/desktop/runner/dev_state.rs` owns
  `handle_about_to_wait_dev_state_observation` beside the dev-state controller.
- `crates/fret-launch/src/runner/desktop/runner/app_handler.rs` still owns the
  `ApplicationHandler::about_to_wait` trait hook and delegates dev-state observation immediately
  after turn bookkeeping.
- The original ordering is preserved: dev-state observation still runs after tick advancement and
  environment polling, before per-window platform/a11y turn handling, and still uses the
  `turn_now` sampled by `handle_about_to_wait_turn_bookkeeping`.

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
- Broader workspace gates were not run because M105 is a private `fret-launch` owner split with no
  public API or cross-crate behavior change; the package check, targeted nextest, and source gates
  cover this claim.

## Verdict

This keeps about-to-wait dev-state window observation source-auditable without changing runtime
behavior. It does not close `DW-P1-linux-003`; the next true closure event remains a dated real
Linux Wayland compositor acceptance note from
`M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
