# M21 Local Wayland Guard Refresh - 2026-05-30

Status: local guard refresh; no Wayland acceptance claim.

This note records the 2026-05-30 Windows-local continuation check for the remaining
`DW-P1-linux-003` closure path. The current host can prove source-policy drift guards,
campaign manifest shape, local policy-skip behavior, Linux capability posture tests, and docking
runtime fallback. It still cannot prove real Linux Wayland compositor acceptance.

`DW-P1-linux-003` remains `[~]` in the TODO tracker, and "Manual Wayland compositor acceptance
remains open" remains unchecked.

## Assumptions-First Resume

1. Confident: the only true closure path for `DW-P1-linux-003` remains the real-host Wayland
   runbook in `M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
2. Confident: local policy-skip evidence is useful proof that non-qualifying environments stop
   before script execution, but it is not compositor acceptance.
3. Confident: refreshing the guard note and source markers is the right local action because there
   was no source drift and no qualifying Wayland compositor was available in this environment.

## Commands Run

```powershell
python -m py_compile tools\gate_docking_multiwindow_workstream_source.py tools\diag_gate_docking_wayland_policy_skip.py
python tools\gate_docking_multiwindow_workstream_source.py
python tools\diag_gate_docking_wayland_policy_skip.py --reuse-built
cargo run -p fretboard-dev -- diag campaign validate tools/diag-campaigns/imui-p3-wayland-real-host.json --json
cargo nextest run -p fret-launch --lib linux_windowing_capability_posture --no-fail-fast
cargo nextest run -p fret-docking --lib request_float_degrades_to_in_window_when_window_hover_detection_is_none --no-fail-fast
python tools\gate_imui_workstream_source.py
python -m json.tool docs\workstreams\docking-multiwindow-imgui-parity\WORKSTREAM.json
python tools\check_workstream_catalog.py
git diff --check
```

## Results

- `gate_docking_multiwindow_workstream_source.py`: pass.
- `diag_gate_docking_wayland_policy_skip.py --reuse-built`: pass; five policy-skip cases:
  - `windows-platform-mismatch`
  - `linux-wayland-multi-window-mismatch`
  - `linux-x11-tear-off-mismatch`
  - `linux-wayland-hover-detection-mismatch`
  - `linux-wayland-z-level-mismatch`
- `imui-p3-wayland-real-host` campaign validation: pass; campaign still requires Linux
  `platform.capabilities` with `ui.multi_window=true`, `ui.window_tear_off=false`,
  `ui.window_hover_detection=none`, and `ui.window_z_level=none`.
- `fret-launch` Linux windowing capability posture: pass; 2 tests, 94 skipped.
- `fret-docking` fallback behavior: pass; 1 test, 86 skipped.
- Cross-workstream source/catalog/diff checks: pass.

## Verdict

The local Wayland boundary remains healthy: non-qualifying environments policy-skip before script
execution, the source guard still rejects accidental acceptance claims, and docking still falls
back to in-window floating when window-hover detection is unavailable.

This does not close `DW-P1-linux-003`. The next true closure event remains a dated real Linux
Wayland compositor acceptance note produced from `M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
