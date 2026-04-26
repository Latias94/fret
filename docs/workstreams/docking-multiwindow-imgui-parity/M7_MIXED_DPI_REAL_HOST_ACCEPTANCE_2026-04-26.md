# M7 Mixed-DPI Real-Host Acceptance - 2026-04-26

Status: accepted real-host evidence; closes `DW-P0-dpi-006`

Related:

- `WORKSTREAM.json`
- `M1_MIXED_DPI_ACCEPTANCE_POSTURE_2026-04-13.md`
- `M2_WINDOWS_MIXED_DPI_CAPTURE_PLAN_2026-04-13.md`
- `M6_MIXED_DPI_MONITOR_SCALE_GATE_2026-04-25.md`
- `tools/diag-campaigns/imui-p3-mixed-dpi-real-host.json`
- `tools/diag-scripts/docking/arbitration/local-debug/docking-arbitration-demo-multiwindow-drag-back-monitor-scale-sweep.debug.json`
- `tools/diag_pick_docking_mixed_dpi_acceptance_pair.py`

## Host Summary

- Windows version: Microsoft Windows 11 Pro, version `10.0.26200`, build `26200`, 64-bit.
- Monitor arrangement:
  - Virtual desktop bounds: `x=-2560 y=0 width=6400 height=2160`.
  - Lowest-scale monitor: `x=-2560 y=533 width=2560 height=1440`, scale factor `1.25`.
  - Highest-scale monitor: `x=0 y=0 width=3840 height=2160`, scale factor `1.50`.
- Successful crossing target: highest-scale monitor.

## Commands

Direct script run with a prebuilt demo binary:

```bash
cargo run -p fretboard-dev -- diag run \
  tools/diag-scripts/docking/arbitration/local-debug/docking-arbitration-demo-multiwindow-drag-back-monitor-scale-sweep.debug.json \
  --dir target/fret-diag/docking-multiwindow-imgui-parity/mixed-dpi-real-host \
  --session-auto \
  --timeout-ms 240000 \
  --launch -- target/release/docking_arbitration_demo.exe
```

Acceptance pair selection:

```bash
python tools/diag_pick_docking_mixed_dpi_acceptance_pair.py \
  target/fret-diag/docking-multiwindow-imgui-parity/mixed-dpi-real-host/sessions/1777176979919-77232 \
  --json-out target/fret-diag/docking-multiwindow-imgui-parity/mixed-dpi-real-host/sessions/1777176979919-77232/acceptance-summary.json \
  --note-out target/fret-diag/docking-multiwindow-imgui-parity/mixed-dpi-real-host/sessions/1777176979919-77232/acceptance-note.md
```

## Selected Bundles

- Session directory: `target/fret-diag/docking-multiwindow-imgui-parity/mixed-dpi-real-host/sessions/1777176979919-77232`
- `pre-crossing`: `1777176982333-multiwindow-drag-back-monitor-sweep-after-tearoff`
- `post-crossing`: `1777176982631-multiwindow-drag-back-monitor-sweep-after-highest-scale-monitor`
- Final state bundle: `1777176982787-docking-arbitration-demo-multiwindow-drag-back-monitor-scale-sweep`

Campaign wrapper rerun also passed:
`target/fret-diag/campaigns/imui-p3-mixed-dpi-real-host/1777177090420`.

## Dock-Routing Summary

- `pre-crossing`: `mixed_dpi=false scale_factors=1.500 entries=4 sf_run=1.500 sf_cur=1.500 sf_move=1.500 scr_used=(5033.6,247.0) origin=(3812.0,226.0) origin_src=platform cross=1`
- `post-crossing`: `mixed_dpi=true scale_factors=1.250, 1.500 entries=6 sf_run=1.500 sf_cur=1.500 sf_move=1.500 scr_used=(1920.0,1080.0) origin=(1901.0,1069.0) origin_src=platform cross=1`
- Losing but still valid post-crossing candidate: `multiwindow-drag-back-monitor-sweep-after-lowest-scale-monitor` also reported `mixed_dpi=true` with `scale_factors=1.250, 1.500`.

## Final Dock Graph

`fretboard-dev diag dock-graph` on the final bundle reported:

```text
windows_total: 1
canonical_ok=true
signature: dock(root=split(v,[tabs(a=1:[demo.viewport.left,demo.viewport.right]),tabs([demo.controls])]);floatings=[])
stats: nodes=3 tabs=2 splits=1 floatings=0 max_depth=2
```

`fretboard-dev diag windows` on the same final bundle reported one remaining window with reliable
hover detection and docking diagnostics present.

## Acceptance Checklist

- Drag-back completion to one canonical dock graph (`floatings=[]`): yes.
- Post-crossing bundle reports `mixed_dpi_signal_observed: true`: yes.
- Post-crossing bundle reports at least two distinct scale factors: yes.
- `dock-routing` keeps stable `scr/scr_used/origin` and `sf_cur/sf_move` evidence: yes.
- No empty floating window or stuck-follow regression while crossing monitors: yes; final bundle has one window and `floatings=[]`.

## Verdict

`DW-P0-dpi-006` is accepted for this Windows mixed-DPI host. Keep the dedicated
`imui-p3-mixed-dpi-real-host` campaign as a manual, host-admitted proof surface; do not fold this
requirement back into the generic `imui-p3-multiwindow-parity` smoke campaign.
