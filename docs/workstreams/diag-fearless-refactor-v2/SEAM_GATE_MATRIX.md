---
title: Diagnostics Seam Gate Matrix
status: draft
date: 2026-03-09
scope: diagnostics, seams, gates, tests, enforcement
---

# Diagnostics Seam Gate Matrix

Status: Draft

Purpose:

- map major diagnostics seams to their current regression protection,
- make coverage gaps explicit,
- give maintainers one place to check before declaring a seam "safe enough".

## How to read this matrix

Each row answers:

- what seam is being protected,
- what currently acts as the protecting gate,
- whether the current coverage is already acceptable,
- and what gap still remains.

Status meanings:

- `Covered`: at least one explicit gate/test/script is already named for the seam.
- `Partial`: some protection exists, but the seam category still lacks full explicit mapping.
- `Gap`: no clear protecting gate is yet recorded.

## Matrix

| Seam category | Scope | Current protection | Status | Remaining gap |
| --- | --- | --- | --- | --- |
| Aggregate dashboard projection | Shared human/machine dashboard shaping in `fret-diag` reused by CLI/GUI/MCP. | `cargo test -p fret-diag dashboard_human_lines_include_summary_and_failure_sections`; `cargo test -p fret-devtools-mcp build_regression_dashboard_result_limits_top_rows_and_builds_human_summary` | Covered | Keep future consumers on shared projection helpers instead of re-parsing. |
| DevTools regression drill-down evidence selection | GUI handling of selected summary path, bundle dirs, and capability-check artifacts. | `cargo test -p fret-devtools load_regression_summary_drilldown -- --nocapture` | Covered | Add another gate only if selection behavior grows beyond evidence-path presentation. |
| Policy-skip capability-preflight interpretation | `skipped_policy` / `capability.missing` / `capabilities_check_path` consumer-facing handling. | `cargo test -p fret-devtools load_regression_summary_drilldown -- --nocapture`; `cargo test -p fret-devtools-mcp build_regression_dashboard_result_limits_top_rows_and_builds_human_summary`; maintainer docs/checklist | Covered | Preserve wording and counters across future consumers. |
| `diag run` post-run doctor/check orchestration seam | Transport result normalization, bundle-doctor, and post-run check handling in `diag_run`. | Helper-level tests in `crates/fret-diag/src/diag_run.rs` such as `maybe_run_bundle_doctor_and_checks_requires_bundle_when_checks_are_requested` | Partial | Record a small named command-level smoke or one more explicit roadmap-linked test set for the whole seam family. |
| `diag suite` default post-run/gate planning seam | Default post-run check planning and lint/doctor orchestration in `diag_suite`. | `cargo test -p fret-diag build_suite_core_default_post_run_checks_sets_small_scroll_vlist_defaults`; `cargo test -p fret-diag finalize_suite_script_success_tail_records_row_when_lint_and_post_run_skip`; default smoke command `cargo run -p fretboard -- diag suite ui-gallery --launch -- cargo run -p fret-ui-gallery --release` | Covered | Keep future suite-seam changes tied to one named helper-level test plus one promoted suite command. |
| Campaign aggregate artifact handoff seam | Shared summary/index/report/share path shaping across single-run and batch flows in `diag_campaign`. | `cargo test -p fret-diag campaign_result_aggregate_json_uses_summary_artifact_outputs`; `cargo test -p fret-diag build_campaign_batch_artifacts_preserves_plan_paths_and_summary_outputs`; `cargo test -p fret-diag build_campaign_execution_finalize_plan_uses_failure_count_and_summary_setup`; smoke command `cargo run -p fretboard -- diag campaign run --lane smoke --tag ui-gallery --platform native --launch -- cargo run -p fret-ui-gallery --release` | Covered | Keep handoff-path changes anchored to result/finalize helper tests plus one campaign-run smoke path. |
| Artifact doctor/lint compatibility seam | Artifact normalization, doctor, lint, and bounded-share repair flow. | `cargo test -p fret-diag doctor_report_includes_normalized_capabilities_from_shared_loader`; default commands `cargo run -p fretboard -- diag doctor --check <bundle_dir> --warmup-frames 4` and `cargo run -p fretboard -- diag artifact lint <run_dir|out_dir>` | Covered | Keep doctor/lint compatibility changes tied to one helper test and one maintainer command set. |
| Legacy manifest compatibility seam | Compatibility reads for top-level `suites` / `scripts` manifest shapes. | `cargo test -p fret-diag workspace_registry_loads_legacy_top_level_suites_and_scripts_manifest`; `RETIREMENT_CRITERIA.md`; compatibility notes in `CAMPAIGN_EXECUTION_ENTRY_V1.md` | Covered | Retire only in a dedicated compatibility window after canonical manifest-only workflows are proven. |
| Legacy artifact alias seam | Canonical-vs-legacy fields such as `bundle_json` / `script_result_json`. | `cargo test -p fret-diag artifact_lint_accepts_legacy_manifest_path_aliases`; `cargo test -p fret-diag artifact_lint_reports_missing_bundle_chunks`; `RESIDUAL_NAMING_AUDIT.md`; `BUNDLE_ARTIFACT_ALIAS_AUDIT.md`; `RETIREMENT_CRITERIA.md` | Covered | Retire alias families only in a dedicated compatibility window; keep Layer A `bundle_json` stable until its contract is explicitly re-decided. |
| Documentation migration seam | Preventing old v1/v1-architecture notes from turning back into parallel planning surfaces. | `START_HERE.md`; `DOCUMENT_MIGRATION_INTENT.md`; README cross-links | Covered | Keep old notes linked, but avoid reintroducing active roadmap duplication there. |

## Priority gaps to close next

The highest-value gaps after this matrix are:

1. keep adding named protecting gates when a new major seam category appears,
2. convert covered compatibility seams into actual retirement work only when a dedicated compatibility window is opened.

## Practical maintainer rule

Before saying a seam migration is done:

1. point at the seam row here,
2. point at the protecting gate/test/script,
3. update the gap column if the seam is still only partially covered.

## Short version

If one rule is enough:

- every major seam should have one named protecting gate, and every missing gate should remain visible
  until it exists.
