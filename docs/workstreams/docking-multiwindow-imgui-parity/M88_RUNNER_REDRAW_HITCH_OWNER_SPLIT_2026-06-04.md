# M88 Runner Redraw Hitch Owner Split - 2026-06-04

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split inside the desktop runner. It keeps
`DW-P1-linux-003` open because real Wayland compositor acceptance still requires the M5 runbook on a
qualifying Linux Wayland host.

## Claim

Desktop runner redraw hitch diagnostics now live in
`crates/fret-launch/src/runner/desktop/runner/redraw_hitch.rs` instead of the general
`ApplicationHandler` integration. The split preserves hitch enablement, threshold parsing,
log path selection behavior, buffered log writes, logical pixel quantization,
redraw phase tracing spans, and per-phase elapsed timing.

## Source Shape

- `crates/fret-launch/src/runner/desktop/runner/mod.rs` declares `mod redraw_hitch;`.
- `crates/fret-launch/src/runner/desktop/runner/redraw_hitch.rs` owns
  `redraw_hitch_config`, `quantize_logical_px`, `write_redraw_hitch_log`, `RedrawPhase`, and
  `measure_redraw_phase`.
- `crates/fret-launch/src/runner/desktop/runner/app_handler.rs` still owns winit event routing,
  redraw execution, renderer frame construction, surface recovery, and `ApplicationHandler`
  wiring, but now delegates redraw hitch configuration, phase timing, quantization, and logging to
  the redraw hitch owner.
- The original ordering is preserved: hitch configuration is still read before the redraw body,
  prepare/render/record/present timing wraps the same blocks, and hitch log emission still happens
  after render/present error handling and before the post-render effect drain.

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
- Broader workspace gates were not run because M88 is a private `fret-launch` owner split with no
  public API or cross-crate behavior change; the package check, targeted nextest, and source gates
  cover this claim.

## Verdict

This keeps desktop runner redraw hitch diagnostics source-auditable without changing runtime
behavior. It does not close `DW-P1-linux-003`; the next true closure event remains a dated real
Linux Wayland compositor acceptance note from
`M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
