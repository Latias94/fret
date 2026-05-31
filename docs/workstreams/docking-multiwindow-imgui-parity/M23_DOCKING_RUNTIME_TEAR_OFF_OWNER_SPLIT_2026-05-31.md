# M23 Docking Runtime Tear-Off Owner Split - 2026-05-31

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable docking runtime split for the multi-window parity lane.
It keeps `DW-P1-linux-003` open because real Wayland compositor acceptance still requires the M5
runbook on a qualifying Linux Wayland host.

## Claim

Docking runtime tear-off state is now owned by a private runtime child module without changing
DockOp orchestration, in-window fallback behavior, created-window completion, close-on-empty
registry behavior, before-close merge behavior, public docking runtime APIs, or the Wayland
acceptance boundary.

## Source Shape

- `ecosystem/fret-docking/src/runtime.rs` keeps public runtime hooks:
  `handle_dock_op`, `handle_dock_window_created`, `handle_dock_before_close_window`, and
  `recenter_in_window_floatings`.
- `ecosystem/fret-docking/src/runtime/tear_off.rs` owns `DockFloatingOsWindowRegistry`,
  `DockTearOffMachine`, pending tear-off correlation, cancellation, TTL pruning, and the
  crate-visible `is_dock_floating_os_window(...)` helper used by declarative docking policy.
- `tools/gate_docking_multiwindow_workstream_source.py` now prevents the registry and pending
  tear-off state machine from drifting back into `runtime.rs`.

## Commands Run

```powershell
cargo fmt -p fret-docking
cargo check -p fret-docking
cargo nextest run -p fret-docking --lib request_float_degrades_to_in_window_when_window_hover_detection_is_none --no-fail-fast
python -m py_compile tools\gate_docking_multiwindow_workstream_source.py
python tools\gate_docking_multiwindow_workstream_source.py
python -m json.tool docs\workstreams\docking-multiwindow-imgui-parity\WORKSTREAM.json
python tools\check_workstream_catalog.py
git diff --check
```

## Results

- `cargo fmt -p fret-docking`: pass.
- `cargo check -p fret-docking`: pass.
- `request_float_degrades_to_in_window_when_window_hover_detection_is_none`: pass; 1 passed,
  86 skipped.
- `gate_docking_multiwindow_workstream_source.py`: pass.
- `check_workstream_catalog.py`: pass; 510 dedicated directories and 47 standalone markdown files.
- `WORKSTREAM.json` shape and `git diff --check`: pass.

## Verdict

This moves the docking multi-window runtime toward smaller, auditable owners while preserving the
existing local fallback proof. It does not close `DW-P1-linux-003`; the next true closure event is
still a dated real Linux Wayland compositor acceptance note from
`M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
