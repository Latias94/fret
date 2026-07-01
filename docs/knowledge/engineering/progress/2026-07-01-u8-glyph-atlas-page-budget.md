---
type: Work Progress
title: U8 glyph atlas page budget diagnostics
tags: fret,u8,text,glyph-atlas,wasm,diagnostics
timestamp: 2026-07-01
related_plan: docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md
git_branch: feat/ui-framework-convergence
---

# Summary

U8 now exposes glyph atlas page budget as an explicit renderer-text tuning contract instead of a
wgpu-local constant. `fret_render_text::glyph_atlas_max_pages()` reads
`FRET_TEXT_GLYPH_ATLAS_MAX_PAGES`, keeps native default behavior at two pages, and sets the wasm
default to one page. The configured page budget is propagated through renderer diagnostics,
bootstrap JSON snapshots, and `fret-diag` evidence indexes.

# Decisions

- Keep this slice narrow: no glyph residency rewrite, no atlas allocator rewrite, and no
  `text_atlas_revision` scene/chunk cache-key removal.
- Report both live pages and page budget. Existing live atlas bytes gates keep their current
  semantics, while evidence indexes can now report budget byte estimates separately.
- Clamp explicit env overrides to `1..=16` pages so accidental debug settings cannot create
  unbounded GPU texture budgets.

# Changed Files

- `crates/fret-render-text/src/cache_tuning.rs`
- `crates/fret-render-text/src/lib.rs`
- `crates/fret-core/src/render_text.rs`
- `crates/fret-render-wgpu/src/text/atlas.rs`
- `crates/fret-render-wgpu/src/text/atlas_runtime_state.rs`
- `crates/fret-render-wgpu/src/text/tests.rs`
- `ecosystem/fret-bootstrap/src/ui_diagnostics/snapshot_types.rs`
- `crates/fret-diag/src/evidence_index.rs`
- `docs/adr/IMPLEMENTATION_ALIGNMENT.md`

# Verification

- `cargo fmt --all --check`
- `cargo check -p fret-render-text --lib`
- `cargo check -p fret-render-wgpu --all-targets`
- `cargo check -p fret-bootstrap --lib --features ui-app-driver,diagnostics`
- `cargo check -p fret-diag --all-targets`
- `cargo check -p fret-render-text --target wasm32-unknown-unknown --lib`
- `cargo nextest run -p fret-render-wgpu glyph_atlas_diagnostics_report_page_budget_separately_from_live_pages glyph_atlas_page_budget_blocks_growth_when_live_page_is_full text_diagnostics_report_configured_glyph_atlas_page_budget --no-fail-fast`
- `cargo nextest run -p fret-diag bundle_stats_summary_reports_text_atlas_page_budget_bytes --no-fail-fast`
- `cargo nextest run -p fret-render-text --lib --no-fail-fast`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`
- `python3 tools/perf/audit_perf_baselines.py --matrix docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-contract-matrix.md --strict`
- `git diff --check`

# Next Action

Continue U8 with visible-range glyph residency and scene/chunk invalidation analysis. Do not remove
`text_atlas_revision` from renderer cache keys until the replacement resource generation key has
correctness evidence for eviction, uploads, and cached scene replay.
