# UI Perf (Windows RTX4090) — Smoothness v1 (Milestones)

## M0 — Baseline + Protocol Locked

- Measurement protocol documented and repeatable.
- Perf suites run from release binaries with stable env knobs.
- UI gallery perf suites make cache+shell + VirtualList known-heights defaults implicit (caller-overridable).

## M1 — Resize Probes Majority-Pass

- `ui-resize-probes --attempts 3` passes a strict majority (>= 2/3).
- Worst bundles are attributable and do not depend on one-off startup events.
  - Evidence: 2026-02-13 PASS (out-dir: `target/fret-diag-perf/ui-resize-probes.hoverstrip.3x.20260213-151459`)

## M2 — Steady Suite Pass

- `ui-gallery-steady` passes against the Windows baseline (repeat >= 3).
- Worst bundles, if any, are explainable and actionable.
  - Evidence: 2026-02-13 PASS (out-dir: `target/fret-diag-perf/ui-gallery-steady.hoverstrip.3x.20260213-152340`)

## M3 — Editor Route Guardrail

- `ui-code-editor-resize-probes` remains PASS (no regression) after landing improvements for “general UI”.
  - Evidence: 2026-02-13 PASS (out-dir: `target/fret-diag-perf/ui-code-editor-resize-probes.hoverstrip.3x.20260213-151711`)
