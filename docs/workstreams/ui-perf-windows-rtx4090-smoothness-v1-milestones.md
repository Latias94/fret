# UI Perf (Windows RTX4090) — Smoothness v1 (Milestones)

## M0 — Baseline + Protocol Locked

- Measurement protocol documented and repeatable.
- Perf suites run from release binaries with stable env knobs.

## M1 — Resize Probes Majority-Pass

- `ui-resize-probes --attempts 3` passes a strict majority (>= 2/3).
- Worst bundles are attributable and do not depend on one-off startup events.

## M2 — Steady Suite Pass

- `ui-gallery-steady` passes against the Windows baseline (repeat >= 3).
- Worst bundles, if any, are explainable and actionable.

## M3 — Editor Route Guardrail

- `ui-code-editor-resize-probes` remains PASS (no regression) after landing improvements for “general UI”.

