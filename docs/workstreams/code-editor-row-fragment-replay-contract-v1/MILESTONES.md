# Code Editor Row Fragment Replay Contract v1 Milestones

Status: Active
Date: 2026-05-16

## M0 - Lane Scaffold And Baseline Evidence

Status: In progress

Exit criteria:

- Workstream docs exist.
- Current local attribution and rejected micro-cleanup evidence are recorded.
- Gates are explicit.

## M1 - Target Shape Decision

Status: Pending

Decide between:

- a contiguous run descriptor over existing per-row cached scene fragments,
- or a precomposed visible-window scene fragment.

The decision must name:

- fallback behavior,
- overlay/preedit handling,
- hosted-resource touch semantics,
- boundary diagnostics fields.

## M2 - Prototype With Row-Level Fallback

Status: Pending

Implement the chosen shape without deleting the current row-level plan path. The prototype must
fall back on stale frame sequence, rect mismatch, missing row cache entry, overlay/preedit conflict,
or unsupported replay key.

## M3 - Perf Validation

Status: Pending

Run the complex-wheel local repro and generate `worst.stats.json`. A useful win should move p95
`us_row_scene_prepaint_probe` materially below the current `77us` without increasing frame paint or
renderer prepare enough to erase the gain.

## M4 - Close Or Promote

Status: Pending

If the prototype wins, promote it as the default path and update `ui-perf-zed-smoothness-v1`. If it
does not win, close this lane with a no-ship audit and choose the next owner from fresh attribution.
