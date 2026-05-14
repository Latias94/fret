# M14 Launched Bounded Campaign Repair - 2026-05-13

Status: launched bounded-campaign repair; not a full Wayland or real-host hand-feel closeout.

This note records the first completed launched run of the generic bounded P3 multi-window campaign
after repairing diagnostics no-frame driving for cross-window dock drags. It does not claim Linux
Wayland compositor acceptance, and it does not close every OS-window hand-feel risk.

## Initial Failure

Command:

```powershell
cargo run -p fretboard-dev -- diag campaign run imui-p3-multiwindow-parity --launch -- cargo run -p fret-demo --bin docking_arbitration_demo --release
```

Initial campaign directory:

- `target/fret-diag/campaigns/imui-p3-multiwindow-parity/1778654204785`

Result:

- `docking-arbitration-demo-multiwindow-overlap-zorder-switch.json`: passed
- `docking-arbitration-demo-multiwindow-under-moving-window-basic.json`: failed deterministic
- `docking-arbitration-demo-multiwindow-transparent-payload-zorder-switch.json`: failed deterministic
- `docking-arbitration-demo-multiwindow-drag-tab-back-to-main-large-outer-move.json`: passed

`regression.summary.json` reported:

- `failed_deterministic: 2`
- top reason code: `timeout.no_frames`
- script failure summary: `script_stalled_no_frames`

The stalled bundles still contained useful dock-routing evidence. The failure was therefore treated
as a diagnostics runner driving gap during cross-window pointer movement, not as evidence for
widening `fret-imui`, `fret-ui-kit::imui`, or `crates/fret-ui`.

## Repair

Implementation anchor:

- `ecosystem/fret-bootstrap/src/ui_diagnostics/script_engine.rs`
- `no_frame_pointer_move_can_drive(app, active)`
- the no-frame `UiActionStepV2::PointerMove` fallback that converts generated pointer events into
  `Effect::DiagInjectEvent`, followed by redraw and animation-frame effects
- unit regression coverage:
  - `no_frame_pointer_move_can_drive_active_cross_window_dock_drag`
  - `no_frame_pointer_move_rejects_non_dock_or_inactive_drag_state`

The fix is intentionally in the diagnostics runner layer (`fret-bootstrap`). It lets a scripted
pointer move continue driving an active cross-window dock drag when a secondary window is throttled
or not producing regular frames, provided the runtime drag state is an active dock-panel or dock-tabs
drag with `cross_window_hover=true`.

## Focused Reruns

After the repair:

```powershell
cargo run -p fretboard-dev -- diag run tools/diag-scripts/docking/arbitration/docking-arbitration-demo-multiwindow-under-moving-window-basic.json --dir target/fret-diag/docking-multiwindow-imgui-parity/m14-rerun-under-moving-window --session-auto --timeout-ms 240000 --exit-after-run --launch -- cargo run -p fret-demo --bin docking_arbitration_demo --release
```

- Result: PASS
- Run ID: `1778655436281`
- Evidence directory:
  `target/fret-diag/docking-multiwindow-imgui-parity/m14-rerun-under-moving-window/sessions/1778655017526-45928`

```powershell
cargo run -p fretboard-dev -- diag run tools/diag-scripts/docking/arbitration/docking-arbitration-demo-multiwindow-transparent-payload-zorder-switch.json --dir target/fret-diag/docking-multiwindow-imgui-parity/m14-rerun-transparent-payload --session-auto --timeout-ms 240000 --exit-after-run --launch -- cargo run -p fret-demo --bin docking_arbitration_demo --release
```

- Result: PASS
- Run ID: `1778655454849`
- Evidence directory:
  `target/fret-diag/docking-multiwindow-imgui-parity/m14-rerun-transparent-payload/sessions/1778655451740-115224`

## Full Campaign

Command:

```powershell
cargo run -p fretboard-dev -- diag campaign run imui-p3-multiwindow-parity --launch -- cargo run -p fret-demo --bin docking_arbitration_demo --release
```

Result:

- `campaign: ok`
- Campaign directory: `target/fret-diag/campaigns/imui-p3-multiwindow-parity/1778655473217`
- Aggregate summary: `items_total: 4`, `passed: 4`, `failed_deterministic: 0`

Script run IDs:

- overlap z-order: `1778655476318`
- under moving window: `1778655481145`
- transparent payload z-order: `1778655488824`
- drag tab back to main after large outer move: `1778655496432`

Post-documentation verification rerun:

- Result: `campaign: ok`
- Campaign directory: `target/fret-diag/campaigns/imui-p3-multiwindow-parity/1778656624160`
- Script run IDs:
  - overlap z-order: `1778656628755`
  - under moving window: `1778656633615`
  - transparent payload z-order: `1778656641564`
  - drag tab back to main after large outer move: `1778656649140`

## Verification Commands

```powershell
cargo fmt --package fret-bootstrap -- --check
cargo check -p fret-bootstrap --features ui-app-driver,diagnostics
cargo nextest run -p fret-bootstrap --features ui-app-driver,diagnostics no_frame_pointer_move --no-fail-fast
cargo run -p fretboard-dev -- diag campaign validate tools/diag-campaigns/imui-p3-multiwindow-parity.json --json
cargo run -p fretboard-dev -- diag campaign run imui-p3-multiwindow-parity --launch -- cargo run -p fret-demo --bin docking_arbitration_demo --release
```

## Verdict

- The generic bounded P3 launched campaign is now green on the local Windows host.
- The repair stays in diagnostics orchestration, not in IMUI runtime or generic helper APIs.
- The remaining true closure path is still platform-specific real-host acceptance, especially the
  Wayland compositor runbook in `M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
