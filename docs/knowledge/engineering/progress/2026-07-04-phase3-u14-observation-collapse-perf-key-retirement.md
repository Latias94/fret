---
type: Work Progress
title: Phase 3 U14 observation-collapse perf key retirement
tags: fret,phase3,u14,diagnostics,perf-keys,compatibility
timestamp: 2026-07-04
related_plan: ../../../plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md
---

# Summary

This U14 slice retires the historical observation-collapse frame-stat keys from current
diagnostics output while preserving old-bundle readability.

Changes:

- Removed `layout_collapse_layout_observations_time_us` and
  `paint_collapse_observations_time_us` from the current `fret-diag` perf-key registry.
- Removed both keys from current bundle-stats JSON output (`sum`, `max`, `avg`, `p50`, `p95`, and
  top rows), text report summaries, and trace synthetic subphase exports.
- Added the missing registered `layout.request_build_roots.*` and `layout.roots.*` timing keys to
  the Chrome trace synthetic layout subphase list so the trace-exported timing registry is covered
  by generated events.
- Kept `bundle_stats_compute` reads and internal aggregate fields so historical bundles containing
  those keys still deserialize and aggregate.
- Added tests that define the retired keys as compatibility inputs only: they are readable from old
  bundle stats, but absent from `registered_perf_keys` and report JSON.
- Regenerated `docs/workstreams/diag-perf-profiling-infra-v1/perf-key-registry.frame-stats.json`.
- Updated ADR implementation alignment for ADR 0327 with the U14 diagnostics cutover evidence.

# Verification

Passed:

- `cargo nextest run -p fret-diag retired_observation_collapse_keys_are_compat_inputs_only full_registered_perf_key_registry_covers_consumed_debug_stats_fields bundle_stats_reads_retired_observation_collapse_inputs_without_reporting_current_keys registered_perf_key_inventory_doc_is_in_sync registered_perf_key_contract_keeps_stats_and_gate_keys_additive trace_exported_perf_key_registry_contains_core_timeline_keys --no-fail-fast`
- `cargo nextest run -p fret-diag chrome_trace_synthetic_ui_subphases_cover_registered_timing_events retired_observation_collapse_keys_are_compat_inputs_only bundle_stats_reads_retired_observation_collapse_inputs_without_reporting_current_keys registered_perf_key_inventory_doc_is_in_sync --no-fail-fast`
- `cargo nextest run -p fret-diag --no-fail-fast`
- `cargo fmt --all --check`
- `cargo run -p fretboard -- diag stats --perf-keys-json > docs/workstreams/diag-perf-profiling-infra-v1/perf-key-registry.frame-stats.json`
- `python3 tools/check_consumption_profiles.py`
- `python3 tools/check_adr_numbers.py`
- `python3 tools/check_workstream_catalog.py`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_execution_surface.py`
- `git diff --check`
- `python3 /Users/frankorz/.codex/skills/engineering-wiki-memory/scripts/wiki_memory.py validate --root docs/knowledge/engineering` passed with existing migration warnings only.

# Deletion Gate

Current registry/report/trace output no longer publishes the retired observation-collapse keys.
Static search over the current registry JSON, `perf_keys.rs`, and `trace.rs` leaves only the
`RETIRED_COMPAT_FRAME_STATS_KEYS` test list in `perf_keys.rs`.

# Next Action

Commit this U14 diagnostics slice, then write the Phase 3 closeout/bridge ledger that classifies
every remaining match from the final closeout searches.

# Citations

- [Phase 3 retained bridge deletion plan](../../../plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md)
- [Phase 2 closeout retained bridge](../../../plans/2026-07-02-001-refactor-ui-framework-phase2-closeout.md)
