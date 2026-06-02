# M44 Docking Runtime Apply Owner Split - 2026-06-02

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split of the non-request docking runtime mutation
path. It keeps `DW-P1-linux-003` open because real Wayland compositor acceptance still requires the
M5 runbook on a qualifying Linux Wayland host.

## Claim

The ordinary `DockOp` application path now lives in a private runtime child owner instead of
staying inline in `ecosystem/fret-docking/src/runtime.rs`, without changing request-to-new-window
policy, create/cancel/window-created behavior, before-close merge-back policy, empty-window
auto-close policy, layout invalidation semantics, or the Wayland acceptance boundary.

## Source Shape

- `ecosystem/fret-docking/src/runtime.rs` keeps the public runtime hook shell and declares
  `mod apply;`.
- `ecosystem/fret-docking/src/runtime/apply.rs` owns:
  - tear-off machine prune/cancel before graph mutation,
  - ordinary `DockOp` graph application,
  - cross-window tear-off mutation logging,
  - empty DockFloating OS-window scan collection,
  - post-mutation invalidation handoff and close-effect follow-through.
- Existing private runtime child owners remain responsible for request handling, window-created
  completion, before-close merge-back, auto-close close effects, and invalidation details.

## Commands Run

```powershell
cargo test -p fret-docking
python tools\gate_docking_multiwindow_workstream_source.py
python tools\gate_imui_workstream_source.py
python -m json.tool docs\workstreams\docking-multiwindow-imgui-parity\WORKSTREAM.json
git diff --check
```

## Results

- `cargo test -p fret-docking`: pass.
- `gate_docking_multiwindow_workstream_source.py`: pass after adding this note and `WORKSTREAM.json`
  markers.
- `gate_imui_workstream_source.py`: pass.
- `WORKSTREAM.json` shape and `git diff --check`: pass.

## Verdict

This keeps the public docking runtime shell narrower and more source-auditable while preserving the
existing docking behavior. It does not close `DW-P1-linux-003`; the next true closure event is
still a dated real Linux Wayland compositor acceptance note from
`M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
