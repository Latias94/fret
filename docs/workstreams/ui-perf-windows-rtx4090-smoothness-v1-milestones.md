# UI perf: Windows RTX 4090 smoothness v1 — Milestones

## M0 — Repeatable P0 runbook

Exit criteria:

- P0 commands are stable, copy-pastable, and produce the expected artifacts in `target/fret-diag/`.
- A reviewer can reproduce a failing bundle and generate `diag stats --diff` in < 2 minutes.

## M1 — Stable gates (attempts=3)

Exit criteria:

- `ui-gallery-steady` passes `docs/workstreams/perf-baselines/ui-gallery-steady.windows-rtx4090.v1.json` with `--repeat 3` reliably.
- `ui-resize-probes` passes `docs/workstreams/perf-baselines/ui-resize-probes.windows-rtx4090.v1.json` with `--repeat 3` reliably.
- `ui-code-editor-resize-probes` does not regress.

## M2 — Tail attribution is one-command

Exit criteria:

- From `check.perf_thresholds.json`, we can jump to the failing metric and identify the responsible phase/hotspot with 1–2 commands.
- At least one real “tail spike” case is documented with:
  - the bundle path
  - `diag stats --diff` output highlights
  - one trace artifact (Tracy or Chrome trace)

## M3 — Typical perf becomes reviewable

Exit criteria:

- Typical perf (p50/p95) is reported and diffed as a first-class review surface (not just max).
- Baseline seeding policy and headroom rationale are documented for Windows smoothness.

## M4 — Regression-proof guardrails

Exit criteria:

- A change that regresses Windows tail perf is caught by a gate and has a clear rollback path.
- A change that improves typical perf but harms tail perf is surfaced explicitly (policy decision, not accidental).

