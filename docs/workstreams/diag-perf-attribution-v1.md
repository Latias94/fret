# Diag perf attribution v1 (fearless refactor)

Status: Draft → Active (workstream tracker)

## Context

Fret already has strong building blocks for performance work:

- Scripted repros (`tools/diag-scripts/*.json`)
- Evidence bundles (`bundle.json` + `triage.json`)
- Perf gates (`fretboard diag perf`, `tools/perf/*_gate.py`)
- Windows perf baselines (`docs/workstreams/perf-baselines/*.windows-rtx4090.*.json`)

The current gap is not “missing metrics”, but **missing interpretation and comparison affordances**:

- “Typical” perf (p50/p95) is awkward to review and compare across commits.
- Tail perf (worst frames / spikes) is captured, but attribution often requires manual, multi-step digging.
- Some hot-path work is invisible (e.g. layout-side observation recording), leading to misattribution.

This workstream proposes a **fearless refactor** of the perf diagnostics surfaces to make performance work:

1) faster to do,
2) easier to review,
3) safer to regress-proof (gates + stable schema),
4) easier to attribute (diff + heuristics + trace on demand).

## Goals

1. **Schema discipline**: a stable, versioned per-frame perf schema with predictable key naming.
2. **Typical perf first-class**: p50/p95 reporting and diff workflows become a one-command path.
3. **Tail perf first-class**: worst-frame attribution is a one-command path, with clear “why”.
4. **Diffable outputs**: compare two bundles/runs and get “what changed” ranked by impact.
5. **Low overhead by default**:
   - Always-on: cheap counters + coarse timings.
   - Opt-in: detailed spans/trace only when explicitly enabled.
6. **Evidence discipline**: changes land with a regression artifact (script + perf gate) and clickable anchors.

## Non-goals

- Replace platform profilers (PIX/ETW). We integrate with them, but do not reimplement them.
- Record/Replay GPU command streams.
- Add a brand-new script format (we remain compatible with existing JSON scripts).

## Current gaps (summary)

1. **Asymmetry in observation metrics**:
   - Paint has `paint_observation_record_time_us`, but layout does not have the equivalent.
2. **No “bundle diff” workflow**:
   - Comparing runs is largely manual (open two JSONs, eyeball).
3. **No “budget view”**:
   - Stats exist, but there is no standard “% of frame” breakdown view.
4. **Typical vs tail are not unified**:
   - “p95 total” and “worst frame breakdown” live in different tools and formats.
5. **Trace is not on-ramp’d**:
   - We have `tracing` in code, but no canonical “capture trace for this run and attach it to the bundle” workflow.

## Proposed architecture (v1)

### 1) Frame perf schema (additive, versioned)

Define a stable schema for per-frame perf stats, conceptually:

- `layout.*`
- `prepaint.*`
- `paint.*`
- `dispatch.*`
- `hit_test.*`
- `renderer.*`
- `memory/churn.*`

Rules:

- Prefer `*_time_us` and `*_calls` / `*_items` pairs when meaningful.
- Add `schema_version` (integer) to bundles / triage output.
- Keep changes additive; avoid renaming keys without a compatibility window.

### 2) “Always-on” vs “opt-in” profiling

- Always-on (for perf gates/baselines):
  - coarse phase timings,
  - key counters (calls/items),
  - churn signals (uploads/evictions/pool stats),
  - a small set of booleans (cache on/off, interactive resize, skipped engine).
- Opt-in (for deep attribution):
  - detailed spans / trace export,
  - top-N hotspots (already exists for some phases),
  - per-subsystem breakdown tables.

### 3) First-class comparison: `diag stats --diff`

Add a command that accepts two bundle paths and reports:

- top-N deltas by impact (us),
- both absolute and percent deltas,
- optional grouping by subsystem (`layout`, `paint`, `renderer`, ...),
- JSON output (for scripts/CI) + a human-readable table.

### 4) A “perf triage summary” that answers “what’s wrong?”

Enhance triage output to include:

- budget breakdown (percent of total),
- unit costs (time per call/item),
- heuristic hints (rule-based, bounded, explainable).

Example heuristic categories:

- `layout.solve_heavy`
- `layout.observation_heavy`
- `paint.text_prepare_churn`
- `renderer.atlas_upload_churn`
- `hit_test.bounds_tree_fallback_churn`

## Rollout plan

Phase 0 (M0): close the visibility gaps

- Add layout observation recording metrics (time + items).
- Ensure the new fields flow through to:
  - bundle snapshots,
  - `diag stats`,
  - perf JSON outputs (where applicable).

## Runbook (M0): reading layout observation recording

Goal: make layout-side observation recording cost visible in the same workflow as worst-frame triage.

Steps:

1. Produce a diagnostics bundle via an existing repro script (e.g. a resize stress script).
2. Run `fretboard diag stats <bundle.json> --sort time --top 30`.
3. For each top frame, inspect:
   - `layout_obs_record.us(time)=... items(models/globals)=...`
   - The corresponding `time.us(total/layout/prepaint/paint)=...` line.

Interpretation:

- High `layout_obs_record_time_us` and high `*_items` usually means observation recording is a meaningful slice of layout time.
- Near-zero `layout_obs_record_time_us` during interactive resize is expected when observation recording is intentionally skipped.

Phase 1 (M1): comparison UX

- Implement `diag stats --diff`.
- Add “budget view” to `diag stats` JSON (and optionally a table view).

## Runbook (M1): diff two bundles

1. Identify two bundle paths (directories or `bundle.json` files).
2. Run:

   - `fretboard diag stats --diff <bundle_a> <bundle_b> --top 20`
   - JSON: `fretboard diag stats --diff <bundle_a> <bundle_b> --top 50 --json`

Interpretation:

- The diff output is ranked by `|delta_us|` (largest absolute changes first).
- `avg.*` deltas approximate “typical per-frame” impact for the captured run (coarse; not a percentile).
- Use `max.*` deltas as a first-pass “tail regression” signal, then inspect the worst frames via
  `fretboard diag stats <bundle> --sort time --top 30`.

Phase 2 (M2): opt-in trace workflow

- Provide a canonical way to:
  - enable tracing for a run,
  - export a trace artifact,
  - attach it to the run output directory/bundle.

## Runbook (M2): generate a Chrome trace from a bundle

This produces a Chrome trace JSON derived from `bundle.json` stats (a synthetic, phase-based
timeline; low overhead).

- During perf runs:
  - `fretboard diag perf ... --trace`
  - The trace is written under `<out_dir>/<run_id>/trace.chrome.json` and indexed in
    `<out_dir>/<run_id>/manifest.json`.

- For an existing bundle:
  - `fretboard diag trace <bundle_dir|bundle.json>`
  - Optional output override: `--trace-out <path>`

Open the resulting JSON in Chrome tracing UI (or compatible viewers) to correlate phases with
`tick_id` / `frame_id`.

## Validation / gates

- Run existing Windows perf gates:
  - `ui-gallery-steady` baseline
  - `ui-resize-probes` (attempts=3)
  - `ui-code-editor-resize-probes` (attempts=3)
- For any new reporting/diff code:
  - add a small unit test around deterministic JSON diff ordering (where feasible).

## Risks

- Added stats increase bundle size (mitigate by keeping fields primitive and bounded).
- Instrumentation overhead (mitigate via `debug_enabled` gating and opt-in spans).
- Schema churn (mitigate via version + additive changes).
