# M2 Windows Mixed-DPI Capture Plan - 2026-04-13

Status: active capture runbook

Related:

- `WORKSTREAM.json`
- `M1_MIXED_DPI_ACCEPTANCE_POSTURE_2026-04-13.md`
- `docking-multiwindow-imgui-parity-todo.md`
- `tools/diag-campaigns/imui-p3-multiwindow-parity.json`
- `tools/diag-scripts/docking/arbitration/docking-arbitration-demo-multiwindow-drag-tab-back-to-main-large-outer-move.json`
- `tools/diag-scripts/docking/arbitration/local-debug/docking-arbitration-demo-multiwindow-drag-back-outer-pos-sweep.debug.json`
- `tools/diag_pick_docking_mixed_dpi_acceptance_pair.py`
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

### 3) Summarize the session and pick the best acceptance pair

Preferred helper:

```bash
python3 tools/diag_pick_docking_mixed_dpi_acceptance_pair.py \
  target/fret-diag/docking-multiwindow-imgui-parity/mixed-dpi-real-host \
  --json-out target/fret-diag/docking-multiwindow-imgui-parity/mixed-dpi-real-host/latest.acceptance-summary.json \
  --note-out target/fret-diag/docking-multiwindow-imgui-parity/mixed-dpi-real-host/latest.acceptance-note.md
```

Optional faster path when a prebuilt Windows binary is already available:

```bash
python3 tools/diag_pick_docking_mixed_dpi_acceptance_pair.py \
  target/fret-diag/docking-multiwindow-imgui-parity/mixed-dpi-real-host \
  --fretboard-bin target\\release\\fretboard-dev.exe \
  --json-out target/fret-diag/docking-multiwindow-imgui-parity/mixed-dpi-real-host/latest.acceptance-summary.json \
  --note-out target/fret-diag/docking-multiwindow-imgui-parity/mixed-dpi-real-host/latest.acceptance-note.md
```

What this helper does:

- recursively finds the three expected outer-sweep bundle labels,
- selects the latest complete session when multiple sessions exist under the out dir,
- runs `diag dock-routing --json` for each candidate bundle,
- recommends one `pre-crossing` bundle and one `post-crossing` bundle,
- writes one bounded JSON summary,
- and can also emit one Markdown evidence-note draft with host fields left as `TODO` when not supplied.

### 4) Inspect the captured bundle candidates directly when needed

The run should leave bundle directories with these labels inside the session directory:

- `multiwindow-drag-back-outer-sweep-after-tearoff`
- `multiwindow-drag-back-outer-sweep-after-outer-move-pos-x`
- `multiwindow-drag-back-outer-sweep-after-outer-move-neg-x`

Use `fretboard-dev diag dock-routing` on the candidate bundle directories when the helper output is
not enough, or when you want to inspect the losing post-crossing candidate manually:

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
   - the helper-selected `multiwindow-drag-back-outer-sweep-after-tearoff` bundle
2. `post-crossing` bundle:
   - the helper-selected bundle, or whichever of
     `multiwindow-drag-back-outer-sweep-after-outer-move-pos-x` or
     `multiwindow-drag-back-outer-sweep-after-outer-move-neg-x`
     first shows the monitor crossing evidence most clearly.

Prefer the post-crossing bundle that shows:

- `mixed_dpi_signal_observed: true`,
- `scale_factors_seen` with at least two distinct values,
- and the clearest `scr/scr_used/origin` + `sf_cur/sf_move` evidence for the boundary crossing.

The helper currently scores post-crossing candidates by:

- `mixed_dpi_signal_observed`,
- number of distinct observed scale factors,
- explicit `sf_cur != sf_move` divergence,
- presence of `origin_src=platform`,
- and whether cross-window hover evidence remained visible.

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

Recommended helper form when you already know the host summary fields:

```bash
python3 tools/diag_pick_docking_mixed_dpi_acceptance_pair.py \
  target/fret-diag/docking-multiwindow-imgui-parity/mixed-dpi-real-host \
  --fretboard-bin target\\release\\fretboard-dev.exe \
  --canonical-command "cargo run -p fretboard-dev -- diag run ..." \
  --windows-version "Windows 11 24H2" \
  --monitor-arrangement "internal panel left, external monitor right" \
  --scale-factors-used "100% + 150%" \
  --json-out target/fret-diag/docking-multiwindow-imgui-parity/mixed-dpi-real-host/latest.acceptance-summary.json \
  --note-out target/fret-diag/docking-multiwindow-imgui-parity/mixed-dpi-real-host/latest.acceptance-note.md
```

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

The generated note draft is intentionally conservative:

- it does not claim the manual checklist is fully closed,
- it marks host fields as `TODO` unless they were passed explicitly,
- and it treats the bounded routing summary as evidence, not as a substitute for human review.

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
