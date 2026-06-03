# M87 Runner Wheel Coalescing Owner Split - 2026-06-04

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split inside the desktop runner. It keeps
`DW-P1-linux-003` open because real Wayland compositor acceptance still requires the M5 runbook on a
qualifying Linux Wayland host.

## Claim

Desktop runner wheel coalescing math and configuration now live in
`crates/fret-launch/src/runner/desktop/runner/wheel_coalescing.rs` instead of the general
`ApplicationHandler` integration. The split preserves coalesced wheel delta accumulation,
per-axis max-abs splitting, carried remainder behavior, `FRET_WINIT_COALESCE_WHEEL` gating, and
`FRET_WINIT_COALESCE_WHEEL_MAX_ABS_PX` parsing.

## Source Shape

- `crates/fret-launch/src/runner/desktop/runner/mod.rs` declares `mod wheel_coalescing;`.
- `crates/fret-launch/src/runner/desktop/runner/wheel_coalescing.rs` owns
  `wheel_coalesce_delta`, `wheel_split_delta_by_max_abs_px`, `wheel_coalescing_enabled`, and
  `wheel_coalescing_max_abs_px`, plus the existing unit coverage for sign-change coalescing and
  max-abs remainder splitting.
- `crates/fret-launch/src/runner/desktop/runner/app_handler.rs` still owns winit event routing,
  pending wheel event insertion/delivery, redraw handling, and the `ApplicationHandler`
  implementation, but now delegates wheel math and env configuration to the wheel owner.
- The original ordering is preserved: wheel deltas are still accumulated before frame-boundary
  delivery, split by the same per-axis cap, and remainders stay pending for subsequent redraws.

## Commands Run

```powershell
cargo fmt --package fret-launch -- --check
cargo check -p fret-launch --lib
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

- `cargo fmt --package fret-launch -- --check`: pass.
- `cargo check -p fret-launch --lib`: pass.
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
- Broader workspace gates were not run because M87 is a private `fret-launch` owner split with no
  public API or cross-crate behavior change; the package check, targeted nextests, and source gates
  cover this claim.

## Verdict

This keeps desktop runner wheel coalescing source-auditable without changing runtime behavior. It
does not close `DW-P1-linux-003`; the next true closure event remains a dated real Linux Wayland
compositor acceptance note from `M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
