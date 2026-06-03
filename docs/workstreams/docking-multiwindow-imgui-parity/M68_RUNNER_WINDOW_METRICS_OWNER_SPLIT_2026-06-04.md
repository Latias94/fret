# M68 Runner Window Metrics Owner Split - 2026-06-04

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split inside the desktop runner. It keeps
`DW-P1-linux-003` open because real Wayland compositor acceptance still requires the M5 runbook on a
qualifying Linux Wayland host.

## Claim

Desktop runner window metrics effect handling now lives in
`crates/fret-launch/src/runner/desktop/runner/window_metrics.rs` instead of the general effect
dispatcher. The split preserves diagnostic insets/preference overrides, `WindowMetricsService`
known/unknown semantics, safe-area and occlusion inset updates, color-scheme and reduced-motion
preference updates, text scale updates, redraw requests, and RAF requests.

## Source Shape

- `crates/fret-launch/src/runner/desktop/runner/mod.rs` declares `mod window_metrics;`.
- `crates/fret-launch/src/runner/desktop/runner/window_metrics.rs` owns
  `apply_window_metrics_insets_request` and `apply_window_metrics_preferences_request`.
- `apply_window_metrics_insets_request` owns:
  - `diag_window_insets_overrides`,
  - `WindowMetricsService::set_safe_area_insets`,
  - `WindowMetricsService::set_occlusion_insets`,
  - safe-area and occlusion known-state comparison before mutation.
- `apply_window_metrics_preferences_request` owns:
  - `diag_window_preference_overrides`,
  - `WindowMetricsService::set_color_scheme`,
  - `WindowMetricsService::set_prefers_reduced_motion`,
  - `WindowMetricsService::set_text_scale_factor`,
  - preference known-state comparison before mutation.
- `crates/fret-launch/src/runner/desktop/runner/effects.rs` still owns the generic effect loop, but
  now delegates `Effect::WindowMetricsSetInsets` and `Effect::WindowMetricsSetPreferences` to the
  window metrics owner.

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

This keeps desktop runner window metrics effect handling source-auditable without changing runtime
behavior. It does not close `DW-P1-linux-003`; the next true closure event remains a dated real
Linux Wayland compositor acceptance note from
`M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
