# M132 Runner Window Redraw Pending Wheel Owner Split - 2026-06-04

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split inside the desktop runner. It keeps
`DW-P1-linux-003` open because real Wayland compositor acceptance still requires the M5 runbook on a
qualifying Linux Wayland host.

## Claim

Desktop runner redraw-time pending wheel drain now lives in
`crates/fret-launch/src/runner/desktop/runner/window_redraw_pending_wheel.rs`. The split moves
diagnostic wheel burst injection, pending wheel coalescing, frame-boundary max-abs splitting,
remainder carry-over redraw requests, and final wheel event delivery out of `app_handler.rs` while
preserving the existing redraw ordering before window-environment refresh and frame preparation.

Marker summary: redraw pending wheel owner; frame-boundary wheel drain; app-handler dispatch only.

Projection marker: frame-boundary wheel drain.

## Source Shape

- `crates/fret-launch/src/runner/desktop/runner/window_redraw_pending_wheel.rs` owns
  `handle_window_redraw_pending_wheel`.
- The owner keeps using `poll_diag_wheel_burst_inject`, `PendingWheelEvent`,
  `wheel_coalesce_delta`, `wheel_split_delta_by_max_abs_px`, `wheel_coalescing_enabled`,
  `wheel_coalescing_max_abs_px`, `WindowRuntime::pending_wheel`, `app.request_redraw`, and
  `deliver_window_event_now` in the same sequence as the previous redraw path.
- It owns the local `deliver_pending_wheel_now` helper for constructing
  `Event::Pointer(PointerEvent::Wheel { .. })`.
- `crates/fret-launch/src/runner/desktop/runner/app_handler.rs` keeps only
  `self.handle_window_redraw_pending_wheel(app_window);` before window-environment refresh.
- `wheel_coalescing.rs` continues to own math/env configuration, and `window_mapped_events.rs`
  continues to own catchall mapped wheel accumulation into `pending_wheel`.

## Commands Run

```powershell
cargo fmt --package fret-launch
cargo check -p fret-launch --lib
cargo fmt --package fret-launch -- --check
cargo nextest run -p fret-launch --lib wheel_coalescing --no-fail-fast
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
- `cargo nextest run -p fret-launch --lib wheel_coalescing --no-fail-fast`: pass.
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
- Broader workspace gates were not run because M132 is a private `fret-launch` owner split with no
  public API or cross-crate behavior change; the package checks, targeted nextests, and source gates
  cover this claim.

## Verdict

This keeps redraw-time pending wheel drain source-auditable in a named owner while leaving
`app_handler.rs` responsible for redraw orchestration. It does not close `DW-P1-linux-003`; the next
true closure event remains a dated real Linux Wayland compositor acceptance note from
`M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
