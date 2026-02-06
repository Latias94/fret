---
title: UI Performance - Zed-level Smoothness (v1) - Execution Plan
status: draft
date: 2026-02-07
scope: perf, zed, gpui, renderer, baseline, gates
---

# UI Performance: Zed-level Smoothness (v1) — Execution Plan

This document is the **execution-focused companion** to:

- Workstream plan: `docs/workstreams/ui-perf-zed-smoothness-v1.md`
- TODO tracker: `docs/workstreams/ui-perf-zed-smoothness-v1-todo.md`
- Evidence log: `docs/workstreams/ui-perf-zed-smoothness-v1-log.md`
- GPUI gap analysis: `docs/workstreams/ui-perf-gpui-gap-v1.md`
- Renderer profiling playbook: `docs/workstreams/ui-perf-renderer-profiling-v1.md`

The intent is to stop drifting into “endless experiments” by pinning:

1. what we build next,
2. how we validate it,
3. what counts as “done”, and
4. what evidence must be recorded for reversibility.

---

## Current state (as of 2026-02-07)

- Canonical steady-state baseline (macOS M4 profile): `docs/workstreams/perf-baselines/ui-gallery-steady.macos-m4.v18.json`.
- Seed policy preset (steady suite): `docs/workstreams/perf-baselines/policies/ui-gallery-steady.v2.json`.
- Baseline selection automation (anti-outlier): `tools/perf/diag_perf_baseline_select.sh`.
- Evidence: see log entry `docs/workstreams/ui-perf-zed-smoothness-v1-log.md` dated `2026-02-07 00:35`.

This means the **measurement substrate is good enough** to spend most effort on implementation rather than
baseline wrangling.

---

## Operating rules (to avoid “experiment-only” loops)

### Rule 1: Every perf change must land one of

- a new default behavior (or a narrowed, stable knob), or
- a new diagnostic signal that explains a tail hitch class, or
- a new stable gate (script + threshold + baseline update).

If a change cannot satisfy one of these, it stays out-of-tree.

### Rule 2: Evidence is mandatory and commit-addressable

For each perf-affecting PR:

1. add an entry in `docs/workstreams/ui-perf-zed-smoothness-v1-log.md` with:
   - commit hash
   - exact command(s)
   - baseline/validation paths
   - delta summary (what tightened/loosened and why)
2. update `docs/workstreams/ui-perf-zed-smoothness-v1-todo.md` checkboxes.

### Rule 3: Time split

- 70%: implementation in hot paths
- 30%: validation / triage / logging

---

## Two-week execution plan (Sprint S1)

Sprint goal: **make editor-class interactions paint-only more often**, and reduce tail latency outliers under
steady-state probes without relying on over-loose thresholds.

### Work package A — Windowed surfaces (M4)

Owner goal: scrolling and pan/zoom should avoid full relayout/rerender for most frames.

Target probes (must not regress):

- `tools/diag-scripts/ui-gallery-virtual-list-torture-steady.json`
- `tools/diag-scripts/ui-gallery-code-editor-torture-autoscroll-steady.json`
- `tools/diag-scripts/ui-gallery-window-resize-stress-steady.json`
- (optional) one 2D surface probe (canvas/node-graph) once it is stable enough for gating

Deliverables:

1. **Define a “window boundary crossing” correctness + perf probe**
   - Script should fail if it triggers full rerender too often while scrolling.
   - Acceptance: `failures=0` against the steady baseline and the probe becomes part of the acceptance suite.
2. **Reduce rerender triggers on scroll**
   - Goal: in steady scroll, most frames should be explainable as “translate cached content” rather than “re-emit”.
   - Evidence: bundle stats show improved cache-root reuse counters for the target scripts.
3. **Stabilize cache keys for windowed content**
   - Fix key instability sources (layout inputs, style resolution, subtree identity).
   - Acceptance: `view_cache_roots_cache_key_mismatch` does not spike on steady scripts.

Exit criteria:

- Acceptance probes remain `failures=0` under 3 validation runs.
- Tail outliers are explainable via diagnostics (no “mystery spikes” without churn signals).

### Work package B — Text pipeline stability (M5)

Owner goal: after warmup, “text prepare” events become rare and correlated with real changes.

Target probes:

- `tools/diag-scripts/ui-gallery-code-editor-torture-autoscroll-steady.json`
- `tools/diag-scripts/ui-gallery-menubar-keyboard-nav-steady.json` (text-first appearance sanity)

Deliverables:

1. **Document stable cache keys** for measure/shaping (wrap width, font stack, style, hinting).
2. **Make atlas eviction / re-upload observable** in perf snapshots (if not already sufficient).
3. **Add a “text cache miss” gate** (lightweight; no pixel diffs) that fails when misses spike after warmup.

Exit criteria:

- Worst frame attribution for text-dominant scripts is actionable from bundle exports alone.
- Repeat runs do not show periodic text-prepare spikes unless changes occur.

### Work package C — GPU vs CPU separation (M7)

Owner goal: classify “CPU is fine but it hitches” bugs without guessing.

Deliverables:

1. Add an opt-in GPU time signal (where supported) to diagnostics snapshots.
2. Add a short playbook entry for “GPU-stall class” triage.

Exit criteria:

- For at least one hitch case, we can label it CPU-bound vs GPU-bound using recorded evidence.

### Work package D — Reduce perf gating cost (M6-lite)

Owner goal: make “perf regressions are caught early” realistic.

Deliverables:

1. Define a reduced CI-friendly suite (3–5 scripts) and document its purpose.
2. Define baseline storage policy (per OS + hardware class).

Exit criteria:

- We can run the reduced suite locally in < 5 minutes (excluding build time).

---

## Promotion protocol (baseline updates)

Baseline updates must follow the candidate-selection workflow to avoid resize outliers:

- Script: `tools/perf/diag_perf_baseline_select.sh`
- Output: `docs/workstreams/perf-baselines/ui-gallery-steady.<machine>.vN.json`
- Evidence: `target/<work-dir>/selection-summary.json` committed or referenced in the perf log.

---

## Out of scope (for Sprint S1)

- Large-scale renderer architecture changes (render graph refactor).
- Cross-platform CI perf gating (only define the plan).
- New UI features/components not needed for perf probes.

