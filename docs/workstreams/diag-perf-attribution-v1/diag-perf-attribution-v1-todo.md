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
  - [x] `paint.widget_heavy`
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

- [x] Add schema versioning for perf stats outputs (stats + stats diff + triage).
  - Contract constants: `crates/fret-diag/src/perf_schema.rs`
  - Outputs now include `kind`, `schema_version`, `schema_policy`, and source/nested schema links where applicable.
  - Gate: `cargo nextest run -p fret-diag stats_json_includes_avg_and_budget stats_diff_json_is_versioned_and_additive_only triage_includes_hints_and_unit_costs_for_worst_frame --no-fail-fast`
- [x] Add schema policy to perf gate artifacts.
  - `check.perf_thresholds.json` and `check.perf_hints.json` now share `PERF_GATE_SCHEMA_VERSION`.
  - Both gate artifacts emit `schema_policy.compatibility=additive_only`.
  - Gate: `cargo nextest run -p fret-diag perf_thresholds_json_projects_renderer_thresholds perf_hints_json_is_versioned_and_additive_only --no-fail-fast`
- [x] Add schema policy to Chrome trace artifacts.
  - `trace.chrome.json` now emits `kind=perf_trace_chrome`, `schema_version`, `schema_policy`,
    `source_bundle_schema_version`, `trace_source=bundle_synthetic_phases`, and `real_spans_included=false`.
  - Gate: `cargo nextest run -p fret-diag chrome_trace_includes_trace_events --no-fail-fast`
- [x] Keep perf regression summary rows actionable for attribution follow-ups.
  - New `diag perf` regression items now write `bundle_dir` derived from their `bundle_artifact`.
  - The shared regression-summary drill-down also recovers bundle roots from older
    `bundle_artifact` fields and threshold failure `evidence_bundle` paths, so DevTools can offer
    concrete stats/triage/hotspots follow-up commands for perf-threshold failures without a fresh
    run.
  - Follow-up command projection now covers every selected bundle root instead of silently
    collapsing to the first one; the first bundle keeps the stable command ids used by GUI run
    buttons, and additional bundles get indexed labels/ids for display and MCP consumers.
  - Gate:
    `cargo nextest run -p fret-diag regression_summary_drilldown_projects_perf_evidence regression_bundle_followup_command_lines_use_selected_bundle_dir regression_bundle_followup_commands_classify_runnable_and_baseline_required regression_bundle_followup_commands_cover_each_selected_bundle perf_row_to_regression_item_uses_single_run_bundle_artifact perf_row_to_regression_item_marks_threshold_failures --no-fail-fast`
  - GUI bridge gate:
    `cargo nextest run -p fret-devtools runnable_followup_command_action_lines_surface_indexed_bundle_commands load_regression_summary_drilldown_collects_perf_evidence --no-fail-fast`
  - MCP bridge gate:
    `cargo nextest run -p fret-devtools-mcp build_regression_dashboard_result_limits_top_rows_and_builds_human_summary --no-fail-fast`
- [x] Add a “field inventory” doc section (keys + meaning + where measured):
  - `docs/workstreams/diag-perf-attribution-v1/diag-perf-attribution-v1-field-inventory.md`
- [x] Ensure additive-only changes unless a migration plan is documented.
  - `schema_policy.compatibility=additive_only` is emitted by stats, stats diff, triage JSON, perf gate artifacts, and
    Chrome trace artifacts.
  - Field removals, semantic renames, or type changes now require either a schema bump or a documented compatibility
    window.
