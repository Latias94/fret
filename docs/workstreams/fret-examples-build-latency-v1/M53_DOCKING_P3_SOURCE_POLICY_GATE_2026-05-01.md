# Fret Examples Build Latency v1 - M53 Docking P3 Source Policy Gate - 2026-05-01

Status: complete

## Decision

Move the remaining docking parity source-policy subset named by the `docking-multiwindow-imgui-parity`
workstream out of the monolithic `fret-examples` Rust unit-test module and into
`tools/gate_imui_workstream_source.py`.

## Migrated Checks

- `immediate_mode_workstream_freezes_the_p3_docking_parity_lane_resume_surface`
- `immediate_mode_workstream_freezes_the_p3_mixed_dpi_acceptance_posture`
- `immediate_mode_workstream_freezes_the_p3_mixed_dpi_real_host_acceptance`
- `immediate_mode_workstream_freezes_the_p3_windows_placement_capture_gate`
- `immediate_mode_workstream_freezes_the_p3_window_style_opacity_capability`
- `immediate_mode_workstream_freezes_the_p3_wayland_degradation_policy_slice`
- `immediate_mode_workstream_freezes_the_p3_wayland_compositor_acceptance_runbook`

## Behavior

The IMUI workstream source gate now covers the active docking parity lane state, local non-Linux
continuation boundary, mixed-DPI accepted evidence, Windows placement/cursor-continuity evidence,
window-style opacity source-level closure, and Wayland degradation/runbook policy.

The real behavior floors stay in their owner crates and launched/campaign surfaces:
`fret-launch`, `fret-docking`, `fret-runtime`, `fret-diag`, campaign validation, and host-admitted
real runs. The mixed-DPI capture-plan, automation-decision, and monitor-scale support notes remain
in Rust for a separate smaller slice.

## Evidence

- `tools/gate_imui_workstream_source.py`
- `apps/fret-examples/src/lib.rs`
- `docs/workstreams/docking-multiwindow-imgui-parity/WORKSTREAM.json`
- `docs/workstreams/docking-multiwindow-imgui-parity/M11_LOCAL_NON_LINUX_CONTINUATION_BOUNDARY_2026-04-29.md`
- `docs/workstreams/docking-multiwindow-imgui-parity/M1_MIXED_DPI_ACCEPTANCE_POSTURE_2026-04-13.md`
- `docs/workstreams/docking-multiwindow-imgui-parity/M7_MIXED_DPI_REAL_HOST_ACCEPTANCE_2026-04-26.md`
- `docs/workstreams/docking-multiwindow-imgui-parity/M8_WINDOWS_TEAROFF_PLACEMENT_CAPTURE_GATE_2026-04-26.md`
- `docs/workstreams/docking-multiwindow-imgui-parity/M9_WINDOWS_TEAROFF_CURSOR_CONTINUITY_FIX_2026-04-26.md`
- `docs/workstreams/docking-multiwindow-imgui-parity/M10_WINDOW_STYLE_OPACITY_CAPABILITY_2026-04-26.md`
- `docs/workstreams/docking-multiwindow-imgui-parity/M4_WAYLAND_DEGRADATION_POLICY_2026-04-21.md`
- `docs/workstreams/docking-multiwindow-imgui-parity/M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`
- `docs/workstreams/docking-multiwindow-imgui-parity/docking-multiwindow-imgui-parity-todo.md`

## Result

After this migration, the Rust `#[test]` count in `apps/fret-examples/src/lib.rs` dropped from 23
to 16, and the `include_str!` count dropped from 128 to 116.

## Gates

```text
python tools/gate_imui_workstream_source.py
python -m py_compile tools/gate_imui_workstream_source.py tools/check_workstream_catalog.py
python tools/check_workstream_catalog.py
python -m json.tool docs/workstreams/docking-multiwindow-imgui-parity/WORKSTREAM.json
python -m json.tool docs/workstreams/fret-examples-build-latency-v1/WORKSTREAM.json
cargo fmt --package fret-examples --check
cargo check -p fret-examples --lib --jobs 1
```
