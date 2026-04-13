# M2 Windows Mixed-DPI Capture Plan - 2026-04-13

Status: active capture runbook

Related:

- `WORKSTREAM.json`
- `M1_MIXED_DPI_ACCEPTANCE_POSTURE_2026-04-13.md`
- `docking-multiwindow-imgui-parity-todo.md`
- `tools/diag-campaigns/imui-p3-multiwindow-parity.json`
- `tools/diag-scripts/docking/arbitration/docking-arbitration-demo-multiwindow-drag-tab-back-to-main-large-outer-move.json`
- `tools/diag-scripts/docking/arbitration/local-debug/docking-arbitration-demo-multiwindow-drag-back-outer-pos-sweep.debug.json`
- `docs/ui-diagnostics-and-scripted-tests.md`

## Purpose

`DW-P0-dpi-006` now has an explicit posture (`M1`), but the lane still needs one practical answer:

> when a maintainer is on a real Windows mixed-DPI host, what exact commands and bundle-selection
> rules should they use to capture the minimum acceptance evidence without improvising?

This note freezes that runbook.

## Target host

Use this runbook only on a Windows native runner with:

- two monitors at different scale factors,
- preferred setup: `100% + 150%`,
- acceptable fallback: any two distinct scale factors,
- and working multi-window docking support.

If the host only has one monitor, or both monitors expose the same scale factor, this note does not
close `DW-P0-dpi-006`; use the bounded campaign or generic large-outer-move stress script instead.

## Canonical command set

### 1) Validate the bounded P3 package first

```bash
cargo run -p fretboard-dev -- diag campaign validate tools/diag-campaigns/imui-p3-multiwindow-parity.json --json
```

### 2) Run the local-debug outer-position sweep capture

```bash
cargo run -p fretboard-dev -- diag run \
  tools/diag-scripts/docking/arbitration/local-debug/docking-arbitration-demo-multiwindow-drag-back-outer-pos-sweep.debug.json \
  --dir target/fret-diag/docking-multiwindow-imgui-parity/mixed-dpi-real-host \
  --session-auto \
  --timeout-ms 240000 \
  --launch -- cargo run -p fret-demo --bin docking_arbitration_demo --release
```

Why this script:

- it captures one bundle immediately after tear-off,
- then one bundle after a large `+X` outer move,
- then one bundle after a large `-X` outer move,
- and finally verifies drag-back + re-dock closure.

### 3) Inspect the captured bundle candidates

The run should leave bundle directories with these labels inside the session directory:

- `multiwindow-drag-back-outer-sweep-after-tearoff`
- `multiwindow-drag-back-outer-sweep-after-outer-move-pos-x`
- `multiwindow-drag-back-outer-sweep-after-outer-move-neg-x`

Use `fretboard-dev diag dock-routing` on the candidate bundle directories:

```bash
cargo run -p fretboard-dev -- diag dock-routing <bundle_dir>
```

Optional bounded JSON view:

```bash
cargo run -p fretboard-dev -- diag dock-routing <bundle_dir> --json
```

## Bundle selection rule

For the acceptance pair, use:

1. `pre-crossing` bundle:
   - `multiwindow-drag-back-outer-sweep-after-tearoff`
2. `post-crossing` bundle:
   - whichever of
     `multiwindow-drag-back-outer-sweep-after-outer-move-pos-x` or
     `multiwindow-drag-back-outer-sweep-after-outer-move-neg-x`
     first shows the monitor crossing evidence most clearly.

Prefer the post-crossing bundle that shows:

- `mixed_dpi_signal_observed: true`,
- `scale_factors_seen` with at least two distinct values,
- and the clearest `scr/scr_used/origin` + `sf_cur/sf_move` evidence for the boundary crossing.

If neither post-move bundle shows mixed-DPI signal evidence, do not mark the run accepted.
Record whether:

- monitor arrangement never crossed the boundary,
- Windows clamped the requested outer position,
- or the host was not actually mixed-DPI.

## Acceptance checklist

The real-host acceptance pair is good enough for this lane when all of the following hold:

1. The run still completes drag-back and returns to one canonical dock graph (`floatings=[]`).
2. The selected `post-crossing` bundle reports `mixed_dpi_signal_observed: true`.
3. The selected `post-crossing` bundle reports `scale_factors_seen` with at least two distinct values.
4. `dock-routing` evidence still shows stable `scr/scr_used/origin` and `sf_cur/sf_move` rather
   than a large cursor-to-grab jump.
5. No new empty floating window or stuck-follow regression appears while crossing monitors.

## Recording rule

When a run satisfies the checklist above, record it in a new dated evidence note under this lane.

Minimum contents for that note:

- host summary:
  - Windows version,
  - monitor arrangement,
  - scale factors used,
  - whether the successful crossing came from `+X` or `-X`,
- canonical command used,
- session directory,
- chosen `pre-crossing` and `post-crossing` bundle directories,
- `dock-routing` summary lines for both bundles,
- whether `mixed_dpi_signal_observed` was present only post-crossing or in both bundles,
- and whether any follow-on automation work is still justified.

## Failure recording rule

If the run fails, or no bundle shows mixed-DPI signal, record at least:

- which bundle labels were produced,
- what `scale_factors_seen` reported,
- whether Windows clamped outer positions,
- and whether the failure looks like:
  - host/setup mismatch,
  - routing drift,
  - or initial-placement / window-decoration drift (`DW-P1-win-002`).

## Decision

From this point forward:

1. this runbook is the default real-host capture path for `DW-P0-dpi-006`,
2. the local-debug outer-position sweep script is the preferred acceptance capture helper,
3. `dock-routing` is the canonical bounded evidence readout for selecting the acceptance pair,
4. and future Windows mixed-DPI closure notes should reference this runbook instead of inventing a
   one-off command sequence.
