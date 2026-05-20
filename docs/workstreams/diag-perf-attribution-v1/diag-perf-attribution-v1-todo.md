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
  - [x] treats requested trace export failures as visible tooling errors instead of silently
        dropping the artifact.
    - Gate:
      `cargo nextest run -p fret-diag write_perf_chrome_trace_if_requested_writes_requested_artifact write_perf_chrome_trace_if_requested_surfaces_export_failure write_perf_chrome_trace_if_requested_noops_when_disabled --no-fail-fast`
  - [x] adds `diag trace --json` metadata output so scripts can see trace source and real-span
        counts without opening `traceEvents`.
    - Gate:
      `cargo nextest run -p fret-diag trace_command_report_json_projects_real_span_metadata trace_contract_captures_trace_out migrated_trace_builds_a_real_context contract_help_mentions_the_migrated_command_surfaces chrome_trace_merges_real_span_extension_events --no-fail-fast`
- [x] Emit initial app-loop `fret.perf.spans.v1` spans when explicitly requested.
  - `FRET_DIAG_REAL_SPANS=1` enables frame-relative View, Overlay, Layout, and Paint spans in
    `fret-bootstrap` `ui_app_driver` apps.
  - Diagnostics drive-script overhead is recorded separately as
    `fret.ui.diagnostics.drive_script`, so scripted-input cost does not disappear into app phases.
  - Diagnostics snapshot/export overhead is recorded separately as
    `fret.ui.diagnostics.snapshot`, so bundle snapshot/debug-extension/dump work does not have to
    be inferred from unaccounted frame tail time.
  - The command palette and default preferences overlays are recorded as nested app-loop spans:
    `fret.ui.view.command_palette_overlay` and `fret.ui.view.preferences_overlay`, so editor
    overlay costs can be separated from the parent View span.
  - `diag perf --trace-real-spans --launch -- <app command>` now requests a Chrome trace artifact
    and injects that environment flag into the launched app process, while preserving an explicit
    caller-provided `FRET_DIAG_REAL_SPANS` override.
  - The Chrome trace exporter merges the extension payload when a bundle contains it.
  - Gate:
    `cargo nextest run -p fret-bootstrap --features diagnostics,ui-app-driver,ui-app-command-palette real_perf_spans_extension_value_is_v1_payload record_snapshot_includes_diagnostics_snapshot_span_at_frame_relative_start perf_span_capture_records_frame_relative_driver_phase perf_span_capture_allows_nested_phase_recording perf_span_capture_records_view_command_palette_overlay_phase perf_span_capture_records_view_preferences_overlay_phase perf_span_capture_records_diagnostics_drive_script_phase --no-fail-fast`
  - CLI gate:
    `cargo nextest run -p fret-diag perf_contract_captures_threshold_and_suite_args migrated_perf_subset_builds_a_real_perf_context trace_real_spans_launch_env_injects_opt_in_flag trace_real_spans_launch_env_preserves_explicit_override trace_real_spans_launch_env_requires_launched_process --no-fail-fast`
- [ ] Future: expand real-span coverage beyond the first nested editor-overlay cases when a
      concrete attribution case needs deeper runtime spans or external profiler/Tracy correlation.

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
  - The trace exporter now also consumes the additive `debug.extensions["fret.perf.spans.v1"]`
    payload when present, marking `real_spans_included=true`, switching `trace_source` to
    `bundle_synthetic_phases_with_extension_spans`, and appending those frame-relative spans to
    Chrome `traceEvents`.
  - Gate: `cargo nextest run -p fret-diag chrome_trace_includes_trace_events chrome_trace_merges_real_span_extension_events --no-fail-fast`
  - Runtime-wide nested span capture is still not claimed complete; the first opt-in writer covers
    top-level app driver phases plus command-palette and preferences nested overlay subphases.
- [x] Keep perf regression summary rows actionable for attribution follow-ups.
  - New `diag perf` regression items now write `bundle_dir` derived from their `bundle_artifact`.
  - The shared regression-summary drill-down also recovers bundle roots from older
    `bundle_artifact` fields and threshold failure `evidence_bundle` paths, so DevTools can offer
    concrete stats/triage/hotspots/trace follow-up commands for perf-threshold failures without a
    fresh run.
  - Follow-up command projection now covers every selected bundle root instead of silently
    collapsing to the first one; the first bundle keeps the stable command ids used by GUI run
    buttons, and additional bundles get indexed labels/ids for display and MCP consumers.
  - The runnable projection now includes `diag trace <bundle> --json`, so selected failing bundles
    can produce Chrome trace metadata from the same GUI/MCP follow-up surface as stats and triage.
  - GUI-launched trace follow-up result records now include the generated
    `trace.chrome.json` under `output_artifacts`, and the selected-result summary/details render the
    artifact path for immediate reuse.
  - Successful GUI-launched trace follow-up records now also project additive `trace_report`
    metadata from the generated trace artifact (`trace_source`, real-span counts/keys, and
    `trace_event_count`), so attribution quality is visible in the selected-result summary/details
    without opening the full Chrome trace payload.
  - MCP dashboard results now include structured follow-up command rows with `diag_args`, mirroring
    the GUI runnable/manual split without forcing AI clients to parse command-line strings.
  - Gate:
    `cargo nextest run -p fret-diag regression_summary_drilldown_projects_perf_evidence regression_bundle_followup_command_lines_use_selected_bundle_dir regression_bundle_followup_commands_classify_runnable_and_baseline_required regression_bundle_followup_commands_cover_each_selected_bundle perf_row_to_regression_item_uses_single_run_bundle_artifact perf_row_to_regression_item_marks_threshold_failures --no-fail-fast`
  - GUI bridge gate:
    `cargo nextest run -p fret-devtools runnable_followup_command_action_lines_surface_indexed_bundle_commands regression_followup_trace_result_record_projects_output_artifact regression_followup_result_summary_lines_project_output_artifacts regression_followup_result_history_entry_detail_lines_surface_repro_fields load_regression_summary_drilldown_collects_perf_evidence --no-fail-fast`
  - MCP bridge gate:
    `cargo nextest run -p fret-devtools-mcp build_regression_dashboard_result_limits_top_rows_and_builds_human_summary --no-fail-fast`
- [x] Add a “field inventory” doc section (keys + meaning + where measured):
  - `docs/workstreams/diag-perf-attribution-v1/diag-perf-attribution-v1-field-inventory.md`
- [x] Ensure additive-only changes unless a migration plan is documented.
  - `schema_policy.compatibility=additive_only` is emitted by stats, stats diff, triage JSON, perf gate artifacts, and
    Chrome trace artifacts.
  - Field removals, semantic renames, or type changes now require either a schema bump or a documented compatibility
    window.
