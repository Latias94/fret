---
type: Work Progress
title: U8 code-editor text/cache diagnostics
tags: fret,u8,text,code-editor,diagnostics,perf
timestamp: 2026-07-01
related_plan: docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md
git_branch: feat/ui-framework-convergence
---

# Summary

`fret-diag` now preserves and exports code-editor row text and row scene cache stats from UI
gallery diagnostic snapshots. The existing gallery snapshot path already publishes
`app_snapshot.code_editor.torture.cache_stats`; this slice threads those counters through bundle
stats, per-run perf JSON rows, repeat-run rows, and repeat summary stats.

The exported fields make the U8 local-edit cache work observable without requiring a renderer or
editor API change. They cover row text cache calls, hits, misses, resets, hit rate, and row text
time, plus row scene cache calls, hits, misses, resets, and hit rate. Existing row scene replay and
prepaint timing fields stay intact.

# Decisions

- Keep this as diagnostics plumbing only. It does not create new cache policy or change editor
  runtime behavior.
- Preserve the snapshot schema by adding `code_editor_cache_stats` beside
  `code_editor_paint_perf`; missing cache stats default to zero in perf rows.
- Summarize repeat-run counters with the existing perf summary helper for consistency with the
  current `diag_perf` row model.

# Changed Files

- `crates/fret-diag/src/stats.rs`
- `crates/fret-diag/src/stats/bundle_stats_compute.inc.rs`
- `crates/fret-diag/src/stats/bundle_stats_report.inc.rs`
- `crates/fret-diag/src/diag_perf/code_editor_rows.rs`
- `crates/fret-diag/src/diag_perf/stats_rows.rs`
- `crates/fret-diag/src/diag_perf/runs_rows.rs`
- `crates/fret-diag/src/diag_perf/reporting.rs`

# Verification

- `cargo nextest run -p fret-diag bundle_stats_extracts_code_editor_paint_perf_from_app_snapshot top_code_editor_row_scene_fields_export_cache_stats top_code_editor_row_scene_fields_compute_replay_rate perf_json_row_exports_top_code_editor_row_scene_fields perf_repeat_run_json_row_exports_top_code_editor_row_scene_fields perf_repeat_summary_json_row_summarizes_code_editor_row_scene_fields --no-fail-fast`
- `cargo check -p fret-diag --all-targets`
- `cargo fmt --all --check`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`
- `git diff --check`

# Next Action

Use the exported metrics to add or update a repeatable text-heavy/code-editor perf gate, then close
the remaining U8 wasm text-budget smoke/perf evidence.
