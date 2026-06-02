# M26 Docking Runtime Tear-Off Cancellation Owner Split - 2026-06-01

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split for pending DockFloating create-request
cancellation used by the multi-window parity lane. It keeps `DW-P1-linux-003` open because real
Wayland compositor acceptance still requires the M5 runbook on a qualifying Linux Wayland host.

## Claim

Docking runtime pending tear-off cancellation policy is now owned by the private tear-off child
module without changing DockOp orchestration, in-window fallback behavior, create request emission,
pending tear-off correlation, created-window completion, close-on-empty registry behavior, public
runtime hook paths, or the Wayland acceptance boundary.

## Source Shape

- `ecosystem/fret-docking/src/runtime.rs` keeps public runtime hook routing, DockOp orchestration,
  graph mutation, invalidation, close-on-empty handling, and before-close merge behavior.
- `ecosystem/fret-docking/src/runtime/tear_off.rs` owns `prune_and_cancel_for_op(...)`,
  single-panel cancellation, `cancel_for_tabs_node(...)`, TTL pruning, pending tear-off
  correlation, and DockFloating create request construction.
- `tools/gate_docking_multiwindow_workstream_source.py` now rejects direct pending cancellation
  calls from drifting back into `runtime.rs`.

## Commands Run

```powershell
cargo fmt -p fret-docking
cargo check -p fret-docking
cargo nextest run -p fret-docking --lib -E 'test(request_float_canceled_by_close_panel_closes_created_window) or test(window_created_does_not_update_drag_source_when_canceled)' --no-fail-fast
python -m py_compile tools\gate_docking_multiwindow_workstream_source.py
python tools\gate_docking_multiwindow_workstream_source.py
python -m json.tool docs\workstreams\docking-multiwindow-imgui-parity\WORKSTREAM.json
python tools\check_workstream_catalog.py
git diff --check
```

## Results

- `cargo fmt -p fret-docking`: pass.
- `cargo check -p fret-docking`: pass.
- Two pending-cancellation regressions: pass.
- `gate_docking_multiwindow_workstream_source.py`: pass.
- `check_workstream_catalog.py`: pass; 510 dedicated directories and 47 standalone markdown files.
- `WORKSTREAM.json` shape and `git diff --check`: pass.

## Verdict

This keeps pending tear-off cancellation source-auditable inside the tear-off owner while preserving
created-window cancellation behavior. It does not close `DW-P1-linux-003`; the next true closure
event is still a dated real Linux Wayland compositor acceptance note from
`M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
