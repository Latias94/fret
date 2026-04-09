# Diag perf attribution v1 (fearless refactor)

Status: Draft → Active (workstream tracker)

## Context

Fret already has strong building blocks for performance work:

- Scripted repros (`tools/diag-scripts/*.json`)
- Evidence bundles (`bundle.json` + `triage.json`)
- Perf gates (`fretboard-dev diag perf`, `tools/perf/*_gate.py`)
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

## Comparative notes (how other frameworks tend to do this)

Common patterns seen in mature UI stacks:

1. **Three-lane profiling**
   - Always-on cheap counters + coarse timings (for regression gates).
   - Opt-in structured spans (for attribution and review).
   - External sampling profilers (for “unknown unknowns” and OS-level context).
2. **Stable names matter**
   - Frameworks that scale treat event names/keys as a contract (renames are migrations).
3. **Frame boundaries are first-class**
   - Profilers want an explicit “frame finished” marker; otherwise traces are hard to read.

Concrete reference in `repo-ref/`:

- Zed/GPUI uses the `profiling` crate style (`#[profiling::function]`, `profiling::scope!`,
  `profiling::finish_frame!()`), which maps well to Tracy-style workflows.

Fret already has a solid base (`tracing` spans in the app driver + optional Tracy wiring via
`ecosystem/fret-bootstrap`), but we still need a tighter artifact story and a clearer field inventory
for reviewers.

## Progress (2026-02-14)

Shipped in this workstream (commit-addressable, additive changes):

1. **M0**: layout observation visibility
   - Layout observation recording stats (time + item counts) flow into bundle snapshots and `diag stats`.
2. **M1**: diff + budget view
   - `fretboard-dev diag stats --diff <a> <b>` and a standard JSON budget view (`avg.*`, `budget_pct.*`).
3. **M2**: opt-in trace artifacts
   - `fretboard-dev diag perf ... --trace` writes `<out_dir>/<run_id>/trace.chrome.json` and indexes it in `manifest.json`.
   - `fretboard-dev diag trace <bundle>` produces a bundle-derived Chrome trace artifact.
4. **M3**: explainability + optional gate
   - `triage.json` includes rule-based hints and unit-cost estimates.
   - `fretboard-dev diag perf ... --check-perf-hints` can turn selected hints into an explicit CI-style gate (`check.perf_hints.json`).

Remaining gaps / follow-ups:

- Perf schema versioning for perf stats outputs (bundle + triage + perf checks).
- Opt-in “real spans” tracing (beyond synthetic phase timelines), with a stable artifact story.
- A field inventory doc (keys + meaning + where measured) to reduce tribal knowledge:
  - `docs/workstreams/diag-perf-attribution-v1/diag-perf-attribution-v1-field-inventory.md`

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
- `layout.build_roots_heavy`
- `layout.roots_heavy`
- `view_cache.layout_invalidated`
- `paint.text_prepare_churn`
- `renderer.upload_churn`
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
2. Run `fretboard-dev diag stats <bundle.json> --sort time --top 30`.
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

   - `fretboard-dev diag stats --diff <bundle_a> <bundle_b> --top 20`
   - JSON: `fretboard-dev diag stats --diff <bundle_a> <bundle_b> --top 50 --json`

Interpretation:

- The diff output is ranked by `|delta_us|` (largest absolute changes first).
- `avg.*` deltas approximate “typical per-frame” impact for the captured run (coarse; not a percentile).
- Use `max.*` deltas as a first-pass “tail regression” signal, then inspect the worst frames via
  `fretboard-dev diag stats <bundle> --sort time --top 30`.

Phase 2 (M2): opt-in trace workflow

- Provide a canonical way to:
  - enable tracing for a run,
  - export a trace artifact,
  - attach it to the run output directory/bundle.

## Runbook (M2): generate a Chrome trace from a bundle

This produces a Chrome trace JSON derived from `bundle.json` stats (a synthetic, phase-based
timeline; low overhead).

- During perf runs:
  - `fretboard-dev diag perf ... --trace`
  - The trace is written under `<out_dir>/<run_id>/trace.chrome.json` and indexed in
    `<out_dir>/<run_id>/manifest.json`.

- For an existing bundle:
  - `fretboard-dev diag trace <bundle_dir|bundle.json>`
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
