# Diag perf attribution v1: field inventory (draft)

This document is a **living index** of the perf fields that show up in:

- `bundle.json` (per-run evidence bundle)
- `triage.json` (explainability summary derived from a bundle)
- `check.perf_thresholds.json` / `check.perf_hints.json` (optional gate evidence)

It is intentionally **pragmatic**: it focuses on the fields that are currently the most useful for
UI smoothness work (especially on Windows), and points you at where those fields are measured and
how to interpret them.

## Reading guide: typical vs tail

- **Typical perf**: use percentiles (`p50`, `p95`) from `fretboard-dev diag perf ... --repeat N` and/or
  use `triage.json` → `stats.avg.*` (per-frame averages over the considered snapshots).
- **Tail perf / spikes**: use `triage.json` → `stats.max.*` and the worst-frame hints, then drill
  into the bundle with `fretboard-dev diag stats <bundle> --sort time --top 30`.

## Where the fields come from

High level pipeline for perf diagnostics:

1. UI runtime records per-frame stats during the app loop.
2. `ecosystem/fret-bootstrap` serializes those stats into `bundle.json` snapshots.
3. `crates/fret-diag` reads `bundle.json` and produces:
   - `triage.json` (hints + unit costs + budget view),
   - `diag stats` tables and diffs,
   - optional check outputs (perf thresholds / perf hints).

Key wiring points:

- Snapshot schema (what gets written into `bundle.json`):
  - `ecosystem/fret-bootstrap/src/ui_diagnostics.rs`
- Stats summarization / diff / budget view:
  - `crates/fret-diag/src/stats.rs`
- Perf triage (hints + unit costs):
  - `crates/fret-diag/src/lib.rs` (triage section)

## Output schema contract

The perf attribution JSON outputs are versioned independently from the raw diagnostics bundle
schema:

- `diag stats <bundle> --json`
  - `kind`: `perf_stats`
  - `schema_version`: `1`
  - `source_bundle_schema_version`: copied from the source bundle when available.
- `diag stats --diff <a> <b> --json`
  - `kind`: `perf_stats_diff`
  - `schema_version`: `1`
- `triage.json`
  - `kind`: `perf_triage`
  - `schema_version`: `1`
  - `source_bundle_schema_version`: sniffed from the source bundle file.
  - `stats_schema_version`: copied from the nested `stats.schema_version`.
- `check.perf_thresholds.json`
  - `kind`: `perf_thresholds`
  - `schema_version`: `1`
- `check.perf_hints.json`
  - `kind`: `perf_hints`
  - `schema_version`: `1`
- `trace.chrome.json`
  - `kind`: `perf_trace_chrome`
  - `schema_version`: `1`
  - `source_bundle_schema_version`: copied from the source bundle when available.
  - `trace_source`: `bundle_synthetic_phases` unless the source bundle also contains supported
    real-span extensions, then `bundle_synthetic_phases_with_extension_spans`.
  - `real_spans_included`: `false` for synthetic-only traces, `true` when supported real spans
    were merged into `traceEvents`.
  - `real_span_extension_keys`: sorted list of debug-extension keys that contributed real spans.
  - `real_span_event_count`: count of merged real-span events.
- `diag trace --json`
  - `kind`: `diag_trace_report`
  - `schema_version`: `1`
  - `schema_policy.compatibility`: `additive_only`
  - `trace_chrome_json_path`: written Chrome trace artifact.
  - `trace_source`, `real_spans_included`, `real_span_event_count`, and
    `real_span_extension_keys`: copied from the generated trace so automation can inspect metadata
    without loading the full `traceEvents` array.
  - Regression selected-bundle follow-ups surface this as a runnable `trace` command with direct
    `diag_args=["trace", <bundle_dir>, "--json"]`; indexed bundle dirs use `trace-2`, `trace-3`,
    and so on.
  - DevTools follow-up result records expose the generated trace as
    `output_artifacts[].kind="trace.chrome.json"` plus `output_artifacts[].path`, and normalize the
    stored path to `/` separators for stable GUI/MCP evidence.
  - Successful DevTools trace follow-up records also include additive `trace_report` metadata read
    from the generated trace artifact: `trace_chrome_json_path`, `trace_source`,
    `real_spans_included`, `real_span_event_count`, `real_span_extension_keys`, and
    `trace_event_count`. The GUI summary/details render these fields so maintainers can tell
    whether a selected trace includes real spans without opening the full `traceEvents` payload.
  - DevTools selected-result actions resolve the trace artifact path from
    `trace_report.trace_chrome_json_path` first, then fall back to the `trace.chrome.json`
    `output_artifacts[]` row, before copying or opening the resolved artifact path.
  - MCP regression dashboards expose the same commands as structured rows under
    `followup_commands`, `runnable_followup_commands`, and `manual_followup_commands`, preserving
    `diag_args` for clients that should not parse shell text.

Supported real-span extension payload:

- `debug.extensions["fret.perf.spans.v1"]`
  - `schema_version`: `"v1"`
  - `spans`: bounded array of frame-relative spans.
  - Each span supports:
    - `name`: Chrome trace event name.
    - `cat` or `category`: Chrome trace category.
    - `start_us` (or `ts_us`): microseconds after the synthetic frame start.
    - `dur_us`: span duration in microseconds; zero-duration spans are ignored.
    - `tid`: optional Chrome trace thread id override; defaults to the diagnostics window id.
    - `args`: optional JSON payload nested under `traceEvents[].args.span_args`.

This is an additive adapter only: the exporter consumes the extension when present, but runtime-wide
real-span capture must stay explicitly opt-in in the app/runtime owner layer.

Current writer:

- Set `FRET_DIAG_REAL_SPANS=1` for `fret-bootstrap` `ui_app_driver` apps to write
  `fret.perf.spans.v1` for the View, Overlay, Layout, and Paint driver phases.
- The command palette and default preferences overlays are recorded as nested View subphases:
  `fret.ui.view.command_palette_overlay` and `fret.ui.view.preferences_overlay`, so editor overlay
  costs can be correlated separately from the parent View span.
- Diagnostics-enabled `ui_app_driver` frames also record `fret.ui.diagnostics.drive_script` so
  scripted-input overhead is visible without folding it into View/Layout/Paint.
- Diagnostics-enabled `ui_app_driver` frames also record `fret.ui.diagnostics.snapshot` using the
  same frame-relative clock before writing the current snapshot extension, so snapshot/debug
  extension/dump overhead is visible in the same `trace.chrome.json` as the app phases.
- Prefer `fretboard-dev diag perf ... --trace-real-spans --launch -- <app command>` when the tool
  launches the app; this also requests a Chrome trace artifact and injects
  `FRET_DIAG_REAL_SPANS=1` into the launched process unless the caller already set the variable
  explicitly.
- These are frame-relative real spans measured in the app loop, not synthetic subdivisions derived
  from aggregate stats.
- Broader nested spans (for example additional runtime hot paths or external profiler/Tracy
  correlation) remain follow-up work.

All six outputs include `schema_policy` with `compatibility=additive_only`. Field additions are
allowed inside the current schema version. Field removals, semantic renames, or type changes require
either a schema version bump or a documented migration/compatibility window.

Evidence anchors:

- Contract constants: `crates/fret-diag/src/perf_schema.rs`
- Stats output: `crates/fret-diag/src/stats/bundle_stats_report.inc.rs`
- Stats diff output: `crates/fret-diag/src/stats.rs`
- Triage output: `crates/fret-diag/src/triage_json.rs`
- Perf gate outputs: `crates/fret-diag/src/diag_perf/outputs.rs`
- Chrome trace output: `crates/fret-diag/src/trace.rs`
- Focused gate:
  `cargo nextest run -p fret-diag stats_json_includes_avg_and_budget stats_diff_json_is_versioned_and_additive_only triage_includes_hints_and_unit_costs_for_worst_frame --no-fail-fast`
- Gate artifact focused gate:
  `cargo nextest run -p fret-diag perf_thresholds_json_projects_renderer_thresholds perf_hints_json_is_versioned_and_additive_only --no-fail-fast`
- Trace artifact focused gate:
  `cargo nextest run -p fret-diag chrome_trace_includes_trace_events chrome_trace_merges_real_span_extension_events --no-fail-fast`
- Requested trace artifact reliability gate:
  `cargo nextest run -p fret-diag write_perf_chrome_trace_if_requested_writes_requested_artifact write_perf_chrome_trace_if_requested_surfaces_export_failure write_perf_chrome_trace_if_requested_noops_when_disabled --no-fail-fast`
- Trace JSON report focused gate:
  `cargo nextest run -p fret-diag trace_command_report_json_projects_real_span_metadata trace_contract_captures_trace_out migrated_trace_builds_a_real_context contract_help_mentions_the_migrated_command_surfaces chrome_trace_merges_real_span_extension_events --no-fail-fast`
- Regression follow-up trace action gate:
  `cargo nextest run -p fret-diag regression_bundle_followup_command_lines_use_selected_bundle_dir regression_bundle_followup_commands_classify_runnable_and_baseline_required regression_bundle_followup_commands_cover_each_selected_bundle --no-fail-fast`
- DevTools GUI/MCP trace projection gate:
  `cargo nextest run -p fret-devtools runnable_followup_command_action_lines_surface_indexed_bundle_commands regression_followup_command_returns_direct_diag_args regression_followup_trace_result_record_projects_output_artifact regression_followup_result_summary_lines_project_output_artifacts regression_followup_result_history_entry_detail_lines_surface_repro_fields --no-fail-fast`
  and `cargo nextest run -p fret-devtools-mcp build_regression_dashboard_result_limits_top_rows_and_builds_human_summary --no-fail-fast`
- Runtime extension writer focused gate:
  `cargo nextest run -p fret-bootstrap --features diagnostics,ui-app-driver,ui-app-command-palette real_perf_spans_extension_value_is_v1_payload record_snapshot_includes_diagnostics_snapshot_span_at_frame_relative_start perf_span_capture_records_frame_relative_driver_phase perf_span_capture_allows_nested_phase_recording perf_span_capture_records_view_command_palette_overlay_phase perf_span_capture_records_view_preferences_overlay_phase perf_span_capture_records_diagnostics_drive_script_phase --no-fail-fast`
- CLI opt-in focused gate:
  `cargo nextest run -p fret-diag perf_contract_captures_threshold_and_suite_args migrated_perf_subset_builds_a_real_perf_context trace_real_spans_launch_env_injects_opt_in_flag trace_real_spans_launch_env_preserves_explicit_override trace_real_spans_launch_env_requires_launched_process --no-fail-fast`

## Core timing fields (per frame, in microseconds)

These are the “first line” metrics that explain where a frame went:

- `total_time_us`
  - Meaning: end-to-end frame time captured by diagnostics.
  - Typical usage: baseline gates, p50/p95 review.
- `layout_time_us`
  - Meaning: total layout time for the frame.
  - Typical usage: smoothness regressions often show up here first.
- `prepaint_time_us`
  - Meaning: prepaint work (building paint primitives, layout-dependent prep).
- `paint_time_us`
  - Meaning: paint work (scene encoding, draw list construction, etc).
- `dispatch_time_us`
  - Meaning: input/command dispatch cost attributed to the frame (when captured).
- `hit_test_time_us`
  - Meaning: hit-test cost attributed to the frame (when captured).

Measurement:

- These are ultimately recorded by the UI runtime, then surfaced into snapshots by
  `ecosystem/fret-bootstrap/src/ui_diagnostics.rs`.

## Layout breakdown fields (why is layout heavy?)

When `layout_time_us` is high but `layout_engine_solve_time_us` is low, you’re usually paying for
mechanism-level work around the solver: root selection, tree walking, observation recording,
view-cache invalidations, etc.

The current sub-breakdown (all in microseconds) is:

- `layout_request_build_roots_time_us`
  - Meaning: time spent building the list of layout roots for this frame.
  - “Bad smell”: large share of layout (`layout.build_roots_heavy` hint).
- `layout_roots_time_us`
  - Meaning: time spent processing layout roots (tree walking + applying layout).
  - “Bad smell”: dominates layout (`layout.roots_heavy` hint).
- `layout_engine_solve_time_us`
  - Meaning: layout solver time (Taffy solve).
  - “Bad smell”: solver dominates layout (`layout.solve_heavy` hint).
- `layout_observation_record_time_us`
  - Meaning: recording layout observation data for the frame.
  - “Bad smell”: recording dominates layout (`layout.observation_heavy` hint).
- `layout_view_cache_time_us`
  - Meaning: time attributed to view-cache work in the layout path.
  - “Bad smell”: view-cache roots become layout-invalidated (`view_cache.layout_invalidated` hint).
- `layout_expand_view_cache_invalidations_time_us`
  - Meaning: time spent expanding view-cache invalidations (if present).

Where measured / wired:

- Layout segmentation is recorded in the UI layout pipeline:
  - `crates/fret-ui/src/tree/layout.rs`
- Values are written into bundle snapshots by:
  - `ecosystem/fret-bootstrap/src/ui_diagnostics.rs`
- Human output “layout_breakdown.us(...)” and JSON stats keys come from:
  - `crates/fret-diag/src/stats.rs`

## Layout observation recording (time + item counts)

If observation recording is a meaningful slice, you should also look at the item counts:

- `layout_observation_record_models_items`
- `layout_observation_record_globals_items`

Interpretation:

- High `layout_observation_record_time_us` with high item counts usually means observation recording
  is on the critical path (not solver time).
- Near-zero observation recording during interactive resize is expected when observation recording
  is intentionally skipped.

## View cache reuse signals (root-level)

These help answer “why did a cached view still relayout?”:

- `view_cache_roots_total`
- `view_cache_roots_reused`
- `view_cache_roots_layout_invalidated`
- `view_cache_roots_cache_key_mismatch`
- `view_cache_roots_not_marked_reuse_root`

Typical workflows:

- Use `triage.json` hints first; then confirm via `diag stats` top frames.
- If `view_cache_roots_layout_invalidated > 0`, the worst frame may be paying a relayout despite
  reuse (expected for some state changes; suspicious if it happens during “toggle-only” actions).

## Invalidation walk (how much work to discover dirtiness?)

- `invalidation_walk_calls`
- `invalidation_walk_nodes`

Interpretation:

- A rising `invalidation_walk_nodes` often correlates with tail spikes during high-frequency input
  (mouse move, resize drag), especially when combined with layout root churn.

## Renderer churn signals (GPU-first, but CPU-visible)

Renderer-related keys are typically surfaced as `top_renderer_*` in perf runs.
Common signals:

- `top_renderer_prepare_text_us` / `top_renderer_text_atlas_upload_bytes`
- `top_renderer_prepare_svg_us` / `top_renderer_svg_upload_bytes`
- `top_renderer_image_upload_bytes`
- `top_renderer_scene_encoding_cache_misses`

Interpretation:

- Upload bytes and cache misses are “churn indicators”: they often correlate with frame spikes and
  should be triaged with a trace/profiler when they regress.

## Practical commands

Typical perf (p50/p95):

- `target/release/fretboard.exe diag perf ui-gallery-steady --repeat 5 --json`

Tail perf (worst frames + attribution):

- `target/release/fretboard.exe diag perf ui-gallery-steady --repeat 3`
- `target/release/fretboard.exe diag triage <bundle.json> --sort time --top 10`
- `target/release/fretboard.exe diag stats <bundle.json> --sort time --top 30`
- `target/release/fretboard.exe diag stats --diff <bundle_a> <bundle_b> --top 30`

Opt-in artifact for timeline correlation:

- `target/release/fretboard.exe diag perf ui-gallery-steady --repeat 1 --trace`
- `target/release/fretboard.exe diag trace <bundle.json> --json`

Current trace artifacts are bundle-derived synthetic phase timelines, optionally enriched with
`fret.perf.spans.v1` real spans when `real_spans_included=true`. They are useful for correlating
`fret.frame`, layout, prepaint, paint, and renderer-adjacent bundle stats in a Chrome trace viewer.
Treat broader runtime spans and Tracy correlation as separate opt-in profiling work until the source
bundle provides those spans explicitly.
