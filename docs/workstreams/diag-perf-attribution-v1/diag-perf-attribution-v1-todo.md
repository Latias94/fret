# Diag perf attribution v1 (TODO)

## P0 (M0): close visibility gaps

- [x] Add layout observation recording metrics:
  - [x] `layout_observation_record_time_us`
  - [x] `layout_observation_record_models_items`
  - [x] `layout_observation_record_globals_items`
- [x] Wire metrics into bundle snapshots (`ecosystem/fret-bootstrap`).
- [x] Wire metrics into `diag stats` tables (`crates/fret-diag`).
- [x] Add a short runbook snippet to the workstream doc (“how to read these fields”).

## P1 (M1): diff + budget view

- [x] Add `fretboard-dev diag stats --diff <a> <b>`:
  - [x] stable ordering (largest delta first),
  - [x] `--json` output,
  - [x] human output.
- [x] Add “budget view” to `diag stats` JSON output:
  - [x] `avg.*` fields
  - [x] `budget_pct.*` percent breakdown
- [x] Extend “budget view” into triage output (optional):
  - [x] unit costs (e.g. `solve_us / solves`, `text_prepare_us / calls`).
- [x] Add heuristic hints (bounded, rule-based; no ML):
  - [x] `layout.observation_heavy`
  - [x] `layout.solve_heavy`
  - [x] `renderer.upload_churn`
  - [x] `paint.text_prepare_churn`

## P2 (M2): opt-in trace workflow

- [x] Define a trace artifact format and location under the run out-dir:
  - [x] `trace.chrome.json` next to per-run `bundle.json` alias (`<out_dir>/<run_id>/`).
- [x] Add a `--trace` toggle to `diag perf` that:
  - [x] exports a Chrome trace JSON (bundle-derived synthetic timeline),
  - [x] records the artifact in a run manifest (`manifest.json` file index).
- [ ] Future: enable tracing spans (not just bundle-derived phases) when explicitly requested.

## P3 (M3): optional perf hints gate

- [x] Add a `diag perf` hints gate output (`check.perf_hints.json`) and non-zero exit on failure.
- [x] Add allow/deny + severity controls:
  - [x] `--check-perf-hints-deny <code,...>`
  - [x] `--check-perf-hints-min-severity <info|warn|error>`
- [x] Wire evidence indexing + repro root artifact inclusion.
- [x] Document usage (`docs/ui-diagnostics-and-scripted-tests.md`) and CLI help.

## Maintenance / hygiene

- [ ] Add schema versioning for perf stats outputs (bundle + triage).
- [x] Add a “field inventory” doc section (keys + meaning + where measured):
  - `docs/workstreams/diag-perf-attribution-v1/diag-perf-attribution-v1-field-inventory.md`
- [ ] Ensure additive-only changes unless a migration plan is documented.
