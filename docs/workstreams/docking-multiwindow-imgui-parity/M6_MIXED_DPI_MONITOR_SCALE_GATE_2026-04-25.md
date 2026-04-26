# M6 Mixed-DPI Monitor-Scale Gate - 2026-04-25

Status: active acceptance update

Related:

- `WORKSTREAM.json`
- `M1_MIXED_DPI_ACCEPTANCE_POSTURE_2026-04-13.md`
- `M2_WINDOWS_MIXED_DPI_CAPTURE_PLAN_2026-04-13.md`
- `M3_MIXED_DPI_AUTOMATION_DECISION_2026-04-20.md`
- `docs/workstreams/diag-environment-predicate-contract-v1/CLOSEOUT_AUDIT_2026-04-20.md`
- `tools/diag-campaigns/imui-p3-mixed-dpi-real-host.json`
- `tools/diag-scripts/docking/arbitration/local-debug/docking-arbitration-demo-multiwindow-drag-back-monitor-scale-sweep.debug.json`
- `tools/diag_pick_docking_mixed_dpi_acceptance_pair.py`
- `M7_MIXED_DPI_REAL_HOST_ACCEPTANCE_2026-04-26.md`

## Purpose

`DW-P0-dpi-006` needed one final cleanup after the diagnostics environment-predicate work landed:
the old local-debug capture script tried to move the follow window by setting outer positions while
the runner's follow state was also moving that same window from the cursor grab anchor. On a real
mixed-DPI host this could complete the drag-back flow without producing honest boundary-crossing
evidence.

This note replaces that capture path with a monitor-topology-admitted, monitor-scale sweep.

## Findings

### 1) The previous script stabilized drag-back but did not guarantee DPI crossing

The monitor-scale sweep keeps the release path deliberately narrower than the generic
large-outer-move gate: after collecting the monitor crossing bundles, it re-seeds the runner cursor
into the stable main-window tab-strip drop-end anchor, releases, and verifies the final canonical
graph. Waiting for a large outer hint after the sweep is the wrong proof primitive here because the
mixed-DPI acceptance target is the follow/cursor crossing plus final `floatings=[]` closure, not a
specific preview-zone choice. The tab-strip anchor is intentional: after the right panel is torn off,
the geometric center of the main dock space can land on an empty layout boundary, while the
drop-end anchor resolves through the normal tab-bar drop target.

However, moving the OS window directly is the wrong proof primitive for follow-mode DPI crossing:
follow-mode derives window position from the runner cursor position. A direct outer-position request
can be overwritten by the next follow update and does not prove that the cursor/follow pair crossed
the intended monitor boundary.

### 2) `host.monitor_topology` admission now exists

The closed diagnostics environment-predicate lane shipped the missing source-scoped contract:

- `requires_environment`
- `source_id: "host.monitor_topology"`
- `predicate.kind: "host_monitor_topology"`
- `monitor_count_ge`
- `distinct_scale_factor_count_ge`

In shorthand: `host.monitor_topology` + `host_monitor_topology` admission now exists.

That means the old M3 posture is now historical: the generic P3 campaign should remain portable,
but this lane can own a dedicated real-host mixed-DPI campaign that skips when the host cannot
honestly satisfy the monitor topology requirement.

### 3) The script should move the diagnostics cursor and explicitly allow follow

The new script step `set_cursor_at_host_monitor` resolves a point from the published runner monitor
topology and writes the existing runner cursor override. The mixed-DPI script drives that cursor to:

- the center of the lowest-scale monitor,
- then the center of the highest-scale monitor.

Scripted diagnostics normally freeze tear-off follow to avoid ordinary scripts chasing synthetic
cursor updates. This real-host acceptance script opts back into follow with
`FRET_DOCK_TEAROFF_FOLLOW_IN_DIAG=1`, making the intent explicit and local to this proof surface.

`dock-routing` remains the post-run proof surface. A run is accepted only when the selected
post-crossing bundle reports `mixed_dpi_signal_observed: true` and at least two observed scale
factors.

## Decisions

1. Keep `imui-p3-multiwindow-parity` generic and portable.
2. Add `imui-p3-mixed-dpi-real-host` as the dedicated mixed-DPI real-host campaign.
3. Gate that campaign with `host.monitor_topology` requiring at least two monitors and two distinct
   scale factors.
4. Replace the old outer-position local-debug script with the monitor-scale sweep script.
5. Keep `diag_pick_docking_mixed_dpi_acceptance_pair.py` as the bounded evidence selector, now using
   the monitor-sweep bundle labels.

## Canonical Commands

Validate manifests:

```bash
cargo run -p fretboard-dev -- diag campaign validate tools/diag-campaigns/imui-p3-mixed-dpi-real-host.json --json
```

Run the mixed-DPI real-host campaign:

```bash
cargo run -p fretboard-dev -- diag campaign run imui-p3-mixed-dpi-real-host --launch -- cargo run -p fret-demo --bin docking_arbitration_demo --release
```

Run the script directly when debugging:

```bash
cargo run -p fretboard-dev -- diag run \
  tools/diag-scripts/docking/arbitration/local-debug/docking-arbitration-demo-multiwindow-drag-back-monitor-scale-sweep.debug.json \
  --dir target/fret-diag/docking-multiwindow-imgui-parity/mixed-dpi-real-host \
  --session-auto \
  --timeout-ms 240000 \
  --launch -- cargo run -p fret-demo --bin docking_arbitration_demo --release
```

Pick the acceptance pair:

```bash
python3 tools/diag_pick_docking_mixed_dpi_acceptance_pair.py \
  target/fret-diag/docking-multiwindow-imgui-parity/mixed-dpi-real-host \
  --json-out target/fret-diag/docking-multiwindow-imgui-parity/mixed-dpi-real-host/latest.acceptance-summary.json \
  --note-out target/fret-diag/docking-multiwindow-imgui-parity/mixed-dpi-real-host/latest.acceptance-note.md
```

## Acceptance State

This slice created the acceptance surface. The real-host run recorded in
`M7_MIXED_DPI_REAL_HOST_ACCEPTANCE_2026-04-26.md` closes `DW-P0-dpi-006`.

The closing run records:

- one pre-crossing bundle,
- one post-crossing bundle,
- `mixed_dpi_signal_observed: true`,
- at least two observed scale factors in the selected post-crossing bundle,
- and the final drag-back state returning to one canonical dock graph with no stuck floating window.
