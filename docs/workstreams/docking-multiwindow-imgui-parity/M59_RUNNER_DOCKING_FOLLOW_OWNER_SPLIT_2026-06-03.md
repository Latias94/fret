# M59 Runner Docking Follow Owner Split - 2026-06-03

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split inside the desktop runner DockFloating follow
logic. It keeps `DW-P1-linux-003` open because real Wayland compositor acceptance still requires
the M5 runbook on a qualifying Linux Wayland host.

## Claim

Desktop runner DockFloating follow movement, transparent-payload style patching, follow stop, and
final settle/rollback now live in
`crates/fret-launch/src/runner/desktop/runner/docking/follow.rs`, while `docking.rs` keeps dock
drag pointer discovery, pointer-capture cancellation, and platform poll-up fallbacks. The split does
not change cursor-grab positioning, diagnostics follow freezing, transparent payload behavior,
`WindowRequest::SetStyle` emission, final settle behavior, caller paths, or the Wayland acceptance
boundary.

## Source Shape

- `crates/fret-launch/src/runner/desktop/runner/docking.rs` declares `mod follow;` and keeps:
  - `dock_drag_pointer_id`,
  - `sync_dock_drag_pointer_capture`,
  - `deliver_dock_drag_pointer_cancel`,
  - platform poll-up fallbacks for dock drags.
- `crates/fret-launch/src/runner/desktop/runner/docking/follow.rs` owns:
  - `update_dock_tearoff_follow`,
  - `stop_dock_tearoff_follow`,
  - `env_flag_is_true`,
  - transparent payload style request emission,
  - redundant outer-position suppression and final settle/rollback.
- The follow methods are visible only inside `crate::runner::desktop::runner`, preserving the
  existing runner sibling call paths without widening the public `fret-launch` API.

## Commands Run

```powershell
cargo fmt --package fret-launch
cargo check -p fret-launch --lib
cargo nextest run -p fret-launch --lib linux_windowing_capability_posture --no-fail-fast
python -m py_compile tools\gate_docking_multiwindow_workstream_source.py tools\gate_imui_workstream_source.py
python -m json.tool docs\workstreams\docking-multiwindow-imgui-parity\WORKSTREAM.json
python tools\gate_docking_multiwindow_workstream_source.py
python tools\gate_imui_workstream_source.py
python tools\check_workstream_catalog.py
git diff --check
```

## Results

- `cargo fmt --package fret-launch`: pass.
- `cargo check -p fret-launch --lib`: pass.
- `cargo nextest run -p fret-launch --lib linux_windowing_capability_posture --no-fail-fast`:
  pass.
- `python -m py_compile tools\gate_docking_multiwindow_workstream_source.py tools\gate_imui_workstream_source.py`:
  pass.
- `python -m json.tool docs\workstreams\docking-multiwindow-imgui-parity\WORKSTREAM.json`: pass.
- `python tools\gate_docking_multiwindow_workstream_source.py`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Verdict

This keeps runner DockFloating follow behavior source-auditable without changing runtime behavior.
It does not close `DW-P1-linux-003`; the next true closure event remains a dated real Linux Wayland
compositor acceptance note from
`M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
