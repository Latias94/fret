# M30 Docking Runtime Request Owner Split - 2026-06-02

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split for DockFloating request-to-new-window policy
used by the multi-window parity lane. It keeps `DW-P1-linux-003` open because real Wayland
compositor acceptance still requires the M5 runbook on a qualifying Linux Wayland host.

## Claim

Docking runtime DockFloating request-to-new-window policy is now owned by private
`runtime/request.rs` without changing DockOp orchestration, in-window fallback behavior, create
request emission, pending tear-off correlation/cancellation, window-created completion,
before-close merge-back behavior, empty-window auto-close behavior, public runtime hook paths, or
the Wayland acceptance boundary.

## Source Shape

- `ecosystem/fret-docking/src/runtime.rs` keeps public runtime hook routing, graph mutation,
  invalidation, window-created routing, before-close routing, and empty-window auto-close routing.
  `handle_dock_op(...)` delegates DockFloating request ops to the private owner.
- `ecosystem/fret-docking/src/runtime/request.rs` owns `handle_request_float_to_new_window(...)`,
  `RequestFloatPanelToNewWindow`, `RequestFloatTabsToNewWindow`, capability fallback through
  `dock_tear_off_supported`, in-window fallback rectangles through `default_in_window_float_rect`,
  active drag pointer correlation, pending request registration, and
  `push_dock_floating_window_create`.
- `ecosystem/fret-docking/src/runtime/in_window.rs` remains the private owner for in-window fallback
  and recovery geometry.
- `ecosystem/fret-docking/src/runtime/tear_off.rs` remains the private owner for create-request
  construction, registry storage, and pending tear-off state/correlation.
- `tools/gate_docking_multiwindow_workstream_source.py` rejects request/fallback bodies drifting
  back into `runtime.rs`.

## Commands Run

```powershell
cargo fmt -p fret-docking
cargo check -p fret-docking
cargo nextest run -p fret-docking --lib -E 'test(request_float_creates_window_and_window_created_moves_panel) or test(request_float_degrades_to_in_window_when_multi_window_is_disabled) or test(request_float_degrades_to_in_window_when_tear_off_is_disabled) or test(request_float_degrades_to_in_window_when_window_hover_detection_is_none) or test(request_float_is_idempotent_until_window_created)' --no-fail-fast
python -m py_compile tools\gate_docking_multiwindow_workstream_source.py
python tools\gate_docking_multiwindow_workstream_source.py
python -m json.tool docs\workstreams\docking-multiwindow-imgui-parity\WORKSTREAM.json
python tools\check_workstream_catalog.py
git diff --check
```

## Results

- `cargo fmt -p fret-docking`: pass.
- `cargo check -p fret-docking`: pass.
- Five request/create/fallback/idempotence regressions: pass.
- `gate_docking_multiwindow_workstream_source.py`: pass.
- `check_workstream_catalog.py`: pass.
- `WORKSTREAM.json` shape and `git diff --check`: pass.

## Verdict

This keeps DockFloating request-to-new-window policy source-auditable in the new private owner while
preserving the existing create, fallback, cancellation, window-created, before-close, auto-close,
and invalidation behavior. It does not close `DW-P1-linux-003`; the next true closure event is still
a dated real Linux Wayland compositor acceptance note from
`M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
