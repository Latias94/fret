# M29 Docking Runtime Auto-Close Owner Split - 2026-06-02

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split for DockFloating empty-window auto-close policy
used by the multi-window parity lane. It keeps `DW-P1-linux-003` open because real Wayland
compositor acceptance still requires the M5 runbook on a qualifying Linux Wayland host.

## Claim

Docking runtime DockFloating empty-window auto-close is now owned by private
`runtime/auto_close.rs` without changing DockOp orchestration, in-window fallback behavior, create
request emission, pending tear-off correlation/cancellation, window-created completion,
before-close merge-back behavior, public runtime hook paths, or the Wayland acceptance boundary.

## Source Shape

- `ecosystem/fret-docking/src/runtime.rs` keeps public runtime hook routing, DockOp orchestration,
  graph mutation, invalidation, in-window fallback dispatch, window-created routing, and
  before-close routing.
- `ecosystem/fret-docking/src/runtime/auto_close.rs` owns `collect_empty_dock_floating_windows(...)`
  and `close_empty_dock_floating_windows(...)`: DockFloating registry scanning via `reg.windows()`,
  panel-count logging via `collect_panels_in_window(window)`, empty-window detection, registry
  removal, and `WindowRequest::Close(window)` emission.
- `ecosystem/fret-docking/src/runtime/tear_off.rs` remains the private owner for registry storage
  and pending tear-off state/correlation.
- `tools/gate_docking_multiwindow_workstream_source.py` rejects empty-window auto-close bodies
  drifting back into `runtime.rs`.

## Commands Run

```powershell
cargo fmt -p fret-docking
cargo check -p fret-docking
cargo nextest run -p fret-docking --lib -E 'test(redock_from_dock_floating_window_auto_closes_empty_os_window)' --no-fail-fast
python -m py_compile tools\gate_docking_multiwindow_workstream_source.py
python tools\gate_docking_multiwindow_workstream_source.py
python -m json.tool docs\workstreams\docking-multiwindow-imgui-parity\WORKSTREAM.json
python tools\check_workstream_catalog.py
git diff --check
```

## Results

- `cargo fmt -p fret-docking`: pass.
- `cargo check -p fret-docking`: pass.
- Focused empty DockFloating auto-close regression: pass.
- `gate_docking_multiwindow_workstream_source.py`: pass.
- `check_workstream_catalog.py`: pass.
- `WORKSTREAM.json` shape and `git diff --check`: pass.

## Verdict

This keeps DockFloating empty-window auto-close source-auditable in the new private owner while
preserving the existing create, cancellation, fallback, window-created, before-close, and
invalidation behavior. It does not close `DW-P1-linux-003`; the next true closure event is still a
dated real Linux Wayland compositor acceptance note from
`M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
