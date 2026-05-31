# M25 Docking Runtime Tear-Off Create Request Owner Split - 2026-06-01

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split for the DockFloating OS-window create request
path used by the multi-window parity lane. It keeps `DW-P1-linux-003` open because real Wayland
compositor acceptance still requires the M5 runbook on a qualifying Linux Wayland host.

## Claim

Docking runtime tear-off capability checks and DockFloating OS-window create request construction
are now owned by the private tear-off child module without changing DockOp orchestration,
in-window fallback behavior, pending tear-off correlation, created-window completion,
close-on-empty registry behavior, public runtime hook paths, or the Wayland acceptance boundary.

## Source Shape

- `ecosystem/fret-docking/src/runtime.rs` keeps public runtime hook routing, DockOp orchestration,
  in-window fallback dispatch, created-window completion, close-on-empty handling, and before-close
  merge behavior.
- `ecosystem/fret-docking/src/runtime/tear_off.rs` owns `dock_tear_off_supported(...)`,
  `push_dock_floating_window_create(...)`, `WindowRequest::Create` construction for
  `CreateWindowKind::DockFloating`, dock-floating OS-window registry, and pending tear-off state.
- `ecosystem/fret-docking/src/runtime/in_window.rs` remains the private owner for visible-bounds
  in-window fallback and recovery geometry.
- `tools/gate_docking_multiwindow_workstream_source.py` now prevents DockFloating create request
  emission from drifting back into `runtime.rs`.

## Commands Run

```powershell
cargo fmt -p fret-docking
cargo check -p fret-docking
cargo nextest run -p fret-docking --lib -E 'test(request_float_degrades_to_in_window_when_multi_window_is_disabled) or test(request_float_degrades_to_in_window_when_tear_off_is_disabled) or test(request_float_degrades_to_in_window_when_window_hover_detection_is_none)' --no-fail-fast
python -m py_compile tools\gate_docking_multiwindow_workstream_source.py
python tools\gate_docking_multiwindow_workstream_source.py
python -m json.tool docs\workstreams\docking-multiwindow-imgui-parity\WORKSTREAM.json
python tools\check_workstream_catalog.py
git diff --check
```

## Results

- `cargo fmt -p fret-docking`: pass.
- `cargo check -p fret-docking`: pass.
- Three in-window fallback regressions: pass.
- `gate_docking_multiwindow_workstream_source.py`: pass.
- `check_workstream_catalog.py`: pass; 510 dedicated directories and 47 standalone markdown files.
- `WORKSTREAM.json` shape and `git diff --check`: pass.

## Verdict

This keeps OS-window tear-off request construction source-auditable inside the tear-off owner while
preserving Wayland-safe degradation and editor recovery behavior. It does not close `DW-P1-linux-003`;
the next true closure event is still a dated real Linux Wayland compositor acceptance note from
`M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
