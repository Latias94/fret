# M20 Local Wayland Guard Refresh - 2026-05-26

Status: local guard refresh; no Wayland acceptance claim.

This note records the 2026-05-26 Windows-local continuation check for the remaining
`DW-P1-linux-003` closure path. The current host can prove source-policy drift guards,
capability-posture tests, campaign manifest shape, local policy-skip behavior, and docking runtime
fallback. It cannot prove real Linux Wayland compositor acceptance.

`DW-P1-linux-003` remains `[~]` in the TODO tracker, and "Manual Wayland compositor acceptance
remains open" remains unchecked.

## Assumptions-First Resume

1. Confident: the only true closure path for `DW-P1-linux-003` is still the real-host Wayland
   runbook in `M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
2. Confident: local policy-skip evidence is useful guard evidence, but it is not compositor
   acceptance because no qualifying Wayland compositor ran the degradation script.
3. Confident: keeping the source gate pointed at this invariant is better than recording another
   ambiguous "green local" note that could be mistaken for platform acceptance.

## Commands Run

```powershell
python -m py_compile tools\gate_docking_multiwindow_workstream_source.py tools\diag_gate_docking_wayland_policy_skip.py
python tools\gate_docking_multiwindow_workstream_source.py
python tools\gate_imui_workstream_source.py
cargo nextest run -p fret-launch --lib linux_windowing_capability_posture --no-fail-fast
python tools\diag_gate_docking_wayland_policy_skip.py
cargo run -p fretboard-dev -- diag campaign validate tools/diag-campaigns/imui-p3-wayland-real-host.json --json
cargo nextest run -p fret-docking --lib request_float_degrades_to_in_window_when_window_hover_detection_is_none --no-fail-fast
```

## Results

- `gate_docking_multiwindow_workstream_source.py`: pass.
- `gate_imui_workstream_source.py`: pass.
- `fret-launch` Linux windowing capability posture: pass; 2 tests, 94 skipped.
- `diag_gate_docking_wayland_policy_skip.py`: pass; five policy-skip cases:
  - `windows-platform-mismatch`
  - `linux-wayland-multi-window-mismatch`
  - `linux-x11-tear-off-mismatch`
  - `linux-wayland-hover-detection-mismatch`
  - `linux-wayland-z-level-mismatch`
- `imui-p3-wayland-real-host` campaign validation: pass; campaign still requires Linux
  `platform.capabilities` with `ui.multi_window=true`, `ui.window_tear_off=false`,
  `ui.window_hover_detection=none`, and `ui.window_z_level=none`.
- `fret-docking` fallback behavior: pass; 1 test, 86 skipped.

Gate note:

- The first `fret-docking` fallback run timed out while waiting behind package-cache/build
  contention and did not produce a test result. No cargo/nextest/rustc process remained afterward;
  the same focused test passed when rerun serially.

## Verdict

The local Wayland boundary remains healthy: non-qualifying environments policy-skip before script
execution, source guards still reject accidental acceptance claims, and docking falls back to
in-window floating when window-hover detection is unavailable.

This does not close `DW-P1-linux-003`. The next true closure event remains a dated real Linux
Wayland compositor acceptance note produced from `M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
