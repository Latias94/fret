# M24 Docking Runtime In-Window Fallback Owner Split - 2026-05-31

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split for the in-window fallback and recovery helpers
used by the multi-window parity lane. It keeps `DW-P1-linux-003` open because real Wayland
compositor acceptance still requires the M5 runbook on a qualifying Linux Wayland host.

## Claim

Docking runtime in-window fallback and recovery geometry is now owned by a private runtime child
module without changing DockOp orchestration, OS-window tear-off creation, created-window
completion, close-on-empty registry behavior, public runtime hook paths, or the Wayland acceptance
boundary.

## Source Shape

- `ecosystem/fret-docking/src/runtime.rs` keeps public runtime hook routing, DockOp orchestration,
  tear-off create requests, created-window completion, and close-on-empty handling.
- `ecosystem/fret-docking/src/runtime/in_window.rs` owns the visible-bounds fallback,
  `default_in_window_float_rect(...)`, rectangle clamping, and the public
  `recenter_in_window_floatings(...)` recovery hook re-exported by `runtime.rs`.
- `ecosystem/fret-docking/src/runtime/tear_off.rs` remains the private owner for dock-floating
  OS-window registry and pending tear-off state.
- `tools/gate_docking_multiwindow_workstream_source.py` now prevents in-window fallback/recovery
  geometry from drifting back into `runtime.rs`.

## Commands Run

```powershell
cargo fmt -p fret-docking
cargo check -p fret-docking
cargo nextest run -p fret-docking --lib -E 'test(request_float_degrades_to_in_window_when_multi_window_is_disabled) or test(request_float_degrades_to_in_window_when_tear_off_is_disabled) or test(request_float_degrades_to_in_window_when_window_hover_detection_is_none)' --no-fail-fast
cargo check -p fret-demo --bin imui_editor_proof_demo
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
- `cargo check -p fret-demo --bin imui_editor_proof_demo`: pass.
- `gate_docking_multiwindow_workstream_source.py`: pass.
- `check_workstream_catalog.py`: pass; 510 dedicated directories and 47 standalone markdown files.
- `WORKSTREAM.json` shape and `git diff --check`: pass.

## Verdict

This keeps Wayland-safe degradation and editor recovery behavior source-auditable while continuing
to shrink the docking runtime integration shell. It does not close `DW-P1-linux-003`; the next true
closure event is still a dated real Linux Wayland compositor acceptance note from
`M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
