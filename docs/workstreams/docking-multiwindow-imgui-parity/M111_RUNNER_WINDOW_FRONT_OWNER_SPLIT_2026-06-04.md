# M111 Runner Window Front Owner Split - 2026-06-04

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split inside the desktop runner. It keeps
`DW-P1-linux-003` open because real Wayland compositor acceptance still requires the M5 runbook on a
qualifying Linux Wayland host.

## Claim

Desktop runner pending-front retry scheduling now lives in
`crates/fret-launch/src/runner/desktop/runner/window_front.rs` instead of the general
`window.rs` state record owner or the `window_lifecycle.rs` create/insert/destroy owner. The split
preserves the `PendingFrontRequest` record, `enqueue_window_front`,
`process_pending_front_requests`, `next_pending_front_deadline`, about-to-wait scheduling,
DockFloating fronting, raise retry cadence, focused-window retry trimming, and existing call sites.

Marker summary: pending-front retry queue; about-to-wait scheduling; DockFloating fronting; does
not close.

## Source Shape

- `crates/fret-launch/src/runner/desktop/runner/window_front.rs` owns `PendingFrontRequest`,
  `enqueue_window_front`, `process_pending_front_requests`, and `next_pending_front_deadline`.
- `crates/fret-launch/src/runner/desktop/runner/window.rs` keeps `WindowRuntime`,
  `PendingWheelEvent`, `TimerEntry`, and `DockTearoffFollow` without defining pending-front records.
- `crates/fret-launch/src/runner/desktop/runner/window_lifecycle.rs` keeps window creation,
  insertion, close, force-close, and surface alpha lifecycle helpers without owning pending-front
  retry behavior.
- Existing call sites in `crates/fret-launch/src/runner/desktop/runner/window_geometry.rs`,
  `crates/fret-launch/src/runner/desktop/runner/window_under_cursor.rs`,
  `crates/fret-launch/src/runner/desktop/runner/docking/create.rs`,
  `crates/fret-launch/src/runner/desktop/runner/docking/follow.rs`, and
  `crates/fret-launch/src/runner/desktop/runner/event_loop.rs` continue to use the same private
  runner methods through the module boundary.

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
- Broader workspace gates were not run because M111 is a private `fret-launch` owner split with no
  public API or cross-crate behavior change; the package check, targeted nextest, and source gates
  cover this claim.

## Verdict

This keeps desktop runner pending-front retry behavior source-auditable in a dedicated owner and
leaves `window.rs` as the runtime state record owner. It does not close `DW-P1-linux-003`; the next
true closure event remains a dated real Linux Wayland compositor acceptance note from
`M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
