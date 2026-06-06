# M153 Runner Internal Drag Moving Window Owner Split - 2026-06-06

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split inside the desktop runner. It keeps
`DW-P1-linux-003` open because real Wayland compositor acceptance still requires the M5 runbook on a
qualifying Linux Wayland host.

## Claim

Desktop runner moving-window resolution and `HoveredWindowUnderMovingWindow` sampling now lives in
`crates/fret-launch/src/runner/desktop/runner/internal_drag_routing/moving_window.rs`. The split
removes duplicated hover/drop helper code from `internal_drag_routing.rs` while preserving the same
diagnostic semantics: dock tab/panel drags resolve a moving window from DockFloating follow state or
from a non-main drag source, sample the platform or best-effort window-under-cursor provider with
the moving window excluded, and record both the target window and source quality.

Marker summary: internal drag moving-window owner; DockFloating follow moving window; non-main dock
drag source moving window; under-moving-window target sampling; diagnostic cursor isolation
candidate expansion; `WindowUnderCursorSource` preservation.

Projection marker: ImGui-style `HoveredWindowUnderMovingWindow` evidence for docking multi-window
hand feel.

## Source Shape

- `crates/fret-launch/src/runner/desktop/runner/internal_drag_routing/moving_window.rs` owns
  `resolve_internal_drag_moving_window`, `window_under_internal_drag_moving_window`, and the private
  diagnostic cursor candidate expansion.
- `crates/fret-launch/src/runner/desktop/runner/internal_drag_routing.rs` keeps hover/drop routing
  orchestration and calls the moving-window owner from both paths.
- `MovingWindowUnderTarget` stays private to the runner module tree and does not widen any public
  runtime, app, docking, or IMUI authoring surface.

## Commands Run

```powershell
cargo fmt --package fret-launch
cargo check -p fret-launch --lib
cargo fmt --package fret-launch -- --check
cargo nextest run -p fret-launch --lib linux_windowing_capability_posture --no-fail-fast
python -m json.tool docs\workstreams\docking-multiwindow-imgui-parity\WORKSTREAM.json
python -m py_compile tools\gate_docking_multiwindow_workstream_source.py
python tools\gate_docking_multiwindow_workstream_source.py
python tools\check_layering.py
python tools\report_largest_files.py --top 30 --min-lines 800
(Get-Content crates\fret-launch\src\runner\desktop\runner\internal_drag_routing.rs | Measure-Object -Line).Lines
(Get-Content crates\fret-launch\src\runner\desktop\runner\internal_drag_routing\moving_window.rs | Measure-Object -Line).Lines
git diff --check
```

## Results

- `cargo fmt --package fret-launch`: pass.
- `cargo check -p fret-launch --lib`: pass.
- `cargo fmt --package fret-launch -- --check`: pass.
- `cargo nextest run -p fret-launch --lib linux_windowing_capability_posture --no-fail-fast`:
  pass; 2 tests passed, 94 skipped.
- `python -m json.tool docs\workstreams\docking-multiwindow-imgui-parity\WORKSTREAM.json`: pass.
- `python -m py_compile tools\gate_docking_multiwindow_workstream_source.py`: pass.
- `python tools\gate_docking_multiwindow_workstream_source.py`: pass.
- `python tools\check_layering.py`: pass.
- `python tools\report_largest_files.py --top 30 --min-lines 800`: timed out locally after both
  120s and 300s attempts; no result claimed from this helper.
- Targeted file size fallback: `internal_drag_routing.rs` is 702 lines and
  `internal_drag_routing/moving_window.rs` is 94 lines after the split.
- `git diff --check`: pass.

## Verdict

Local gates passed for the private owner split only. The split improves source auditability for the
moving-window branch of docking hover/drop routing without claiming a new Wayland compositor
acceptance result. This does not close `DW-P1-linux-003`; the next true closure event remains a
dated real Linux Wayland compositor acceptance note from
`M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
