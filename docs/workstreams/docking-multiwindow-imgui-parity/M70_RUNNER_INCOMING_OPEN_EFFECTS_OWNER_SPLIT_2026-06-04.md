# M70 Runner Incoming-Open Effects Owner Split - 2026-06-04

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split inside the desktop runner. It keeps
`DW-P1-linux-003` open because real Wayland compositor acceptance still requires the M5 runbook on a
qualifying Linux Wayland host.

## Claim

Desktop runner incoming-open effect handling now lives in
`crates/fret-launch/src/runner/desktop/runner/incoming_open_effects.rs` instead of the general
effect dispatcher. The split preserves diagnostic incoming-open injection, request item projection,
read limit capping, diagnostic and startup path payload reads, unavailable-event delivery, and
release cleanup.
The owner-split gate explicitly tracks incoming-open read data and unavailable events.

## Source Shape

- `crates/fret-launch/src/runner/desktop/runner/mod.rs` declares `mod incoming_open_effects;`.
- `crates/fret-launch/src/runner/desktop/runner/incoming_open_effects.rs` owns:
  - `handle_diag_incoming_open_inject`,
  - `handle_incoming_open_read_all`,
  - `handle_incoming_open_read_all_with_limits`,
  - `handle_incoming_open_release`,
  - incoming-open limit and data-event construction helpers.
- `incoming_open_effects.rs` keeps diagnostic payload projection, startup path payload reading, and
  `PlatformCapabilities::shell.incoming_open` gating next to incoming-open event delivery.
- `crates/fret-launch/src/runner/desktop/runner/effects.rs` still owns the generic effect loop, but
  now delegates `Effect::DiagIncomingOpenInject`, `Effect::IncomingOpenReadAll`,
  `Effect::IncomingOpenReadAllWithLimits`, and `Effect::IncomingOpenRelease` to the incoming-open
  owner.

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
- `python -m json.tool docs\workstreams\docking-multiwindow-imgui-parity\WORKSTREAM.json`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_docking_multiwindow_workstream_source.py`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Verdict

This keeps desktop runner incoming-open effect handling source-auditable without changing runtime
behavior. It does not close `DW-P1-linux-003`; the next true closure event remains a dated real
Linux Wayland compositor acceptance note from
`M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
