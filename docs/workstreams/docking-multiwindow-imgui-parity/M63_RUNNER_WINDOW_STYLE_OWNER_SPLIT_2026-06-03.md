# M63 Runner Window Style Owner Split - 2026-06-03

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split inside the desktop runner. It keeps
`DW-P1-linux-003` open because real Wayland compositor acceptance still requires the M5 runbook on a
qualifying Linux Wayland host.

## Claim

Desktop runner `WindowRequest::SetStyle` platform application now lives in
`crates/fret-launch/src/runner/desktop/runner/window_style.rs` instead of the general effect
dispatcher. The split preserves z-level, hit-test, opacity, background material, surface alpha
reconfiguration, style diagnostics, redraw, and DockFloating transparent-payload follow state.

## Source Shape

- `crates/fret-launch/src/runner/desktop/runner/mod.rs` declares `mod window_style;`.
- `crates/fret-launch/src/runner/desktop/runner/window_style.rs` owns
  `apply_window_style_request`.
- `apply_window_style_request` owns:
  - z-level application with `WindowZLevelQuality` gating,
  - hit-test clamping through `RunnerWindowStyleDiagnosticsStore::clamp_hit_test_request`,
  - DockFloating follow `hit_test_passthrough_all_applied` state updates,
  - drag diagnostics `transparent_payload_hit_test_passthrough_applied` updates,
  - opacity application,
  - `RunnerWindowStyleDiagnosticsStore::apply_style_patch`,
  - background material application from the effective style snapshot,
  - composited-alpha surface reconfiguration and surface config diagnostics,
  - final redraw request.
- `crates/fret-launch/src/runner/desktop/runner/effects.rs` still owns the generic
  `WindowRequest::SetStyle` effect branch, but now delegates directly to
  `apply_window_style_request`.

## Commands Run

```powershell
cargo fmt --package fret-launch -- --check
cargo check -p fret-launch --lib
cargo nextest run -p fret-launch --lib linux_windowing_capability_posture --no-fail-fast
python -m py_compile tools\gate_docking_multiwindow_workstream_source.py tools\gate_imui_workstream_source.py
python -m json.tool docs\workstreams\docking-multiwindow-imgui-parity\WORKSTREAM.json
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
- `python tools\gate_docking_multiwindow_workstream_source.py`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Verdict

This keeps DockFloating style, transparent payload, and composited-alpha application source-auditable
without changing runtime behavior. It does not close `DW-P1-linux-003`; the next true closure event
remains a dated real Linux Wayland compositor acceptance note from
`M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
