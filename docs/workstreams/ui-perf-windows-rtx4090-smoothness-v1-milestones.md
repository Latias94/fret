---
title: UI Performance (Windows RTX 4090) - Smoothness Closure v1 (Milestones)
status: draft
date: 2026-02-12
scope: performance, regression-gates, windows
---

# UI Performance (Windows RTX 4090) - Smoothness Closure (v1) — Milestones

Related:

- Plan: `docs/workstreams/ui-perf-windows-rtx4090-smoothness-v1.md`
- TODO: `docs/workstreams/ui-perf-windows-rtx4090-smoothness-v1-todo.md`

---

## Milestone table

| Milestone | Status | Goal | Acceptance criteria (gates) | Evidence anchors |
| --- | --- | --- | --- | --- |
| M0 | In progress | Clean, repeatable Windows perf protocol | Baseline runs are commit-addressable and reproducible | `target/fret-diag-perf*` dirs + log entry |
| M1 | In progress | Reduce resize tail latency (stress + drag-jitter) | `ui-resize-probes` attempts=3 majority PASS; worst bundles explainable | `target/fret-diag-perf-samples/ui-resize-probes.windows-rtx4090.after-semantics-gate.20260212-153145` |
| M2 | In progress | Reduce steady-suite tail spikes | `ui-gallery-steady` baseline PASS; no new regressions in code-editor probes | `target/fret-diag-perf-samples/ui-gallery-steady.windows-rtx4090.after-semantics-gate3.20260212-155616/check.perf_thresholds.json` |
| M3 | Planned | CPU vs GPU attribution closure | Tail spikes classified (CPU vs renderer churn) with evidence | `FRET_DIAG_RENDERER_PERF=1` bundles + optional RenderDoc capture |

Notes:

- Keep milestones small and reversible: one hitch class per landable step.
- Do not tighten baselines until the underlying hitch class is removed (avoid chasing noise).
