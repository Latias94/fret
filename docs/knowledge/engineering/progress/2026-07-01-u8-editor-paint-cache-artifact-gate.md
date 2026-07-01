---
type: Work Progress
title: U8 editor paint cache artifact gate
tags: fret,u8,text,code-editor,diagnostics,perf,gate
timestamp: 2026-07-01
related_plan: docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md
git_branch: feat/ui-framework-convergence
---

# Summary

The editor paint contract artifact verifier now treats code-editor row cache stats as required
attribution evidence. When verifying a `--with-paint-perf` attribution directory, synced target
artifacts must include `code_editor_cache_stats` in the captured `diag stats --json` output, beside
the existing `code_editor_paint_perf` and torture-overlay-zero coverage.

This turns the previous diagnostics export into a repeatable closeout gate without changing the
Rust perf baseline schema. The verifier also projects row text and row scene cache calls, hits,
misses, evictions, resets, and computed hit rates into `decision_inputs`, so closeout reports can
distinguish Canvas/renderer paint pressure from row cache identity churn.

# Decisions

- Keep this slice in the artifact gate layer. Do not re-seed baselines or add Rust threshold schema
  fields before target-machine evidence shows which row cache bounds should be hard thresholds.
- Accept both a future top-level `code_editor_cache_stats` object and the current
  `top[0].code_editor_cache_stats` shape from `fret-diag stats --json`.
- Require the full raw row text / row scene cache counter set for attribution coverage; validation
  directories without `FRET_CODE_EDITOR_DIAG_PAINT_PERF=1` remain allowed to omit it.

# Changed Files

- `tools/perf/diag_editor_paint_contract_validate.py`
- `tools/perf/diag_editor_paint_contract_verify_artifacts.py`
- `tools/perf/test_diag_editor_paint_contract_validate.py`
- `tools/perf/test_diag_editor_paint_contract_verify_artifacts.py`
- `docs/workstreams/ui-perf-zed-smoothness-v1/editor-paint-contract-stabilization-runbook.md`

# Verification

- `PYTHONPATH=tools/perf python3 -m unittest tools.perf.test_diag_editor_paint_contract_validate tools.perf.test_diag_editor_paint_contract_verify_artifacts`
- `python3 -m py_compile tools/perf/diag_editor_paint_contract_validate.py tools/perf/diag_editor_paint_contract_verify_artifacts.py tools/perf/test_diag_editor_paint_contract_validate.py tools/perf/test_diag_editor_paint_contract_verify_artifacts.py`
- `python3 tools/perf/audit_perf_baselines.py --matrix docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-contract-matrix.md --strict`
- `python3 tools/perf/diag_editor_paint_contract_verify_artifacts.py --help`
- `cargo fmt --all --check`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`
- `git diff --check`

# Next Action

Continue U8 by adding the thin text-budget gate wrapper for text-heavy memory diagnostics and
web/wasm runtime bundle evidence, reusing the exported cache, glyph, and upload metrics instead of
expanding the baseline schema first.
