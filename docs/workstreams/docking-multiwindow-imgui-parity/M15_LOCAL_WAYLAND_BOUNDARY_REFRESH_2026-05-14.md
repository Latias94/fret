# M15 Local Wayland Boundary Refresh - 2026-05-14

Status: local boundary refresh; Wayland real-host acceptance remains open.

This note records a non-Linux local refresh for the remaining `DW-P1-linux-003` item. It keeps the
same boundary as `M11_LOCAL_NON_LINUX_CONTINUATION_BOUNDARY_2026-04-29.md`: local work can validate
source policy, campaign manifests, and non-GUI fallback behavior, but it cannot accept Linux Wayland
compositor hand-feel.

## Assumptions-First Resume

1. Confident: this lane is still the active owner for runner/backend multi-window hand-feel. Evidence:
   `WORKSTREAM.json` has `status: active`, and the IMUI gap-closure lane points P3 multi-window work
   here. If wrong, the repo would need a new owner lane before more evidence is useful.
2. Confident: the remaining explicit platform closure is Wayland real-host acceptance. Evidence:
   `docking-multiwindow-imgui-parity-todo.md` still keeps only "Manual Wayland compositor acceptance
   remains open" under `DW-P1-linux-003`. If wrong, the TODO should name a new non-Linux repro before
   implementation starts.
3. Likely: the local value on a Windows host is drift detection, not code change. Evidence: the
   Wayland posture is represented by unit tests plus a host-admitted campaign manifest. If wrong, a
   failing validation gate should drive a diagnostics or runner fix.
4. Confident: this refresh must not widen `fret-imui`, `fret-ui-kit::imui`, or `crates/fret-ui`.
   Evidence: the lane scope is runner/backend-owned and ADR 0083 keeps degradation policy outside
   generic IMUI helper growth. If wrong, an ADR-level contract update is required first.

## Commands Run

```powershell
python tools/gate_imui_workstream_source.py
cargo nextest run -p fret-launch --lib linux_windowing_capability_posture --no-fail-fast
cargo nextest run -p fret-docking --lib request_float_degrades_to_in_window_when_window_hover_detection_is_none --no-fail-fast
cargo run -p fretboard-dev -- diag campaign validate tools/diag-campaigns/imui-p3-wayland-real-host.json --json
cargo run -p fretboard-dev -- diag campaign validate tools/diag-campaigns/imui-p3-multiwindow-parity.json --json
cargo run -p fretboard-dev -- diag campaign validate tools/diag-campaigns/imui-p3-mixed-dpi-real-host.json --json
cargo run -p fretboard-dev -- diag campaign validate tools/diag-campaigns/imui-p3-windows-placement-real-host.json --json
python tools/check_workstream_catalog.py
```

## Results

- `python tools/gate_imui_workstream_source.py` passed.
- `fret-launch` Wayland/X11 capability posture tests passed: 2 tests run, 2 passed.
- `fret-docking` fallback test passed:
  `request_float_degrades_to_in_window_when_window_hover_detection_is_none`.
- `imui-p3-wayland-real-host` manifest validation passed and still requires
  `platform.capabilities` with Linux, `ui.multi_window=true`, `ui.window_tear_off=false`,
  `ui.window_hover_detection=none`, and `ui.window_z_level=none`.
- `imui-p3-multiwindow-parity`, `imui-p3-mixed-dpi-real-host`, and
  `imui-p3-windows-placement-real-host` manifest validation passed.
- `python tools/check_workstream_catalog.py` passed with 370 dedicated directories and 47 standalone
  markdown files.

## Verdict

No local drift was found in the multi-window campaign manifests, Wayland capability posture, or
docking fallback behavior.

This is not a full hand-feel closeout. `DW-P1-linux-003` remains open until a qualifying Linux
Wayland compositor run follows `M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md` and records
real-host evidence. The generic bounded P3 campaign remains the current non-Wayland regression
surface, while `imui-p3-wayland-real-host` remains the canonical Wayland admission wrapper.
