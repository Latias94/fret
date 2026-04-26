# M9 Windows Tear-Off Cursor Continuity Fix - 2026-04-26

Status: accepted real-host fix for `DW-P1-win-002`

Related:

- `M8_WINDOWS_TEAROFF_PLACEMENT_CAPTURE_GATE_2026-04-26.md`
- `tools/diag-scripts/docking/arbitration/local-debug/docking-arbitration-demo-windows-tearoff-placement-capture.debug.json`
- `tools/diag-campaigns/imui-p3-windows-placement-real-host.json`
- `crates/fret-launch/src/runner/desktop/runner/diag_cursor_override.rs`
- `ecosystem/fret-bootstrap/src/ui_diagnostics/service.rs`
- `ecosystem/fret-bootstrap/src/ui_diagnostics/script_runner.rs`

## Problem

The first real-host placement run with the M8 diagnostics surface proved a large scripted
tear-off drift:

- session: `target/fret-diag/docking-multiwindow-imgui-parity/windows-placement-real-host/sessions/1777185128550-39852`
- settled bundle: `1777185131319-windows-tearoff-placement-after-tearoff-settled`
- bounded readout: `move_local=(802.7,14.0)`, `move_grab_delta=(786.7,0.0)`,
  `move_grab_error=786.7`

This was not a docking policy problem. The runner was allowed to migrate the active dock drag source
to the newly created tear-off window, but diagnostics cursor overrides were still interpreting the
first post-migration local coordinate against the new window origin. That made the synthetic cursor
move with the tear-off window instead of behaving like a physical OS cursor in screen space.

## Fix

The runner diagnostics cursor override now preserves screen-space continuity when an active dock
drag remaps its source window to a newly created tear-off window:

- same-window scripted cursor updates still integrate small local deltas into screen space,
- cross-window dock-drag owner remaps preserve the previous screen position for the first handoff,
- subsequent same-window updates integrate normally from the remapped window,
- explicit cursor placements to a non-drag-source target still snap to that target's client origin.

The diagnostics script migration layer keeps its existing ImGui-style drag-owner remap behavior so
multi-window scripts do not starve when the newly created window becomes the active render source.
Focused tests lock the distinction between ordinary migration and dock-drag migration.

## Acceptance Evidence

Accepted run:

```text
cargo run -p fretboard-dev -- diag run tools/diag-scripts/docking/arbitration/local-debug/docking-arbitration-demo-windows-tearoff-placement-capture.debug.json --dir target/fret-diag/docking-multiwindow-imgui-parity/windows-placement-real-host --session-auto --timeout-ms 180000 --launch -- target/release/docking_arbitration_demo.exe
```

Result:

- PASS run id: `1777187534717`
- session: `target/fret-diag/docking-multiwindow-imgui-parity/windows-placement-real-host/sessions/1777187533293-68088`
- settled bundle: `1777187535921-windows-tearoff-placement-after-tearoff-settled`
- final bundle: `1777187536078-docking-arbitration-demo-windows-tearoff-placement-capture`

Bounded `dock-routing` readout for the settled bundle:

```text
window=4294967298 frame=67 ... scr=(2616.0,247.0) outer=(2580.0,180.0) deco=(12.0,46.0)
origin=(2592.0,226.0) origin_src=platform move_outer=(2580.0,180.0)
move_deco=(12.0,46.0) move_origin=(2592.0,226.0) move_origin_src=platform
move_local=(16.0,14.0) move_grab_delta=(0.0,0.0) move_grab_error=0.0
sf_move_run=1.500 sf_move=1.500
```

The same script completed drag-back and returned to a one-window canonical graph (`floatings=[]`),
so this is accepted as the Windows placement closure for `DW-P1-win-002`.

## Gates

```text
cargo nextest run -p fret-bootstrap --features ui-app-driver,diagnostics -E 'test(migration_remaps_drag_pointer_until_before_pointer_down) or test(ordinary_migration_keeps_drag_pointer_until_window_after_pointer_down) or test(dock_drag_migration_remaps_drag_pointer_until_after_pointer_down)' --no-fail-fast --jobs 2
cargo check -p fret-launch -p fret-bootstrap --features fret-bootstrap/ui-app-driver,fret-bootstrap/diagnostics --jobs 2
cargo build -p fret-demo --bin docking_arbitration_demo --release --jobs 2
cargo run -p fretboard-dev -- diag run tools/diag-scripts/docking/arbitration/local-debug/docking-arbitration-demo-windows-tearoff-placement-capture.debug.json --dir target/fret-diag/docking-multiwindow-imgui-parity/windows-placement-real-host --session-auto --timeout-ms 180000 --launch -- target/release/docking_arbitration_demo.exe
cargo run -p fretboard-dev -- diag dock-routing target/fret-diag/docking-multiwindow-imgui-parity/windows-placement-real-host/sessions/1777187533293-68088/1777187535921-windows-tearoff-placement-after-tearoff-settled
```

## Decision

Mark `DW-P1-win-002` done. Keep M8 as the reusable proof surface and M9 as the accepted fix note.
Do not move this behavior into `fret-ui`; cursor continuity across OS-window source remaps is a
runner/diagnostics transport invariant.
