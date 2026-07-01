---
type: Work Progress
title: U8 text budget gate wrapper
tags: fret,u8,text,diagnostics,perf,gate,wasm
timestamp: 2026-07-01
related_plan: docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md
git_branch: feat/ui-framework-convergence
---

# Summary

`tools/perf/diag_u8_text_budget_gate.py` now provides one U8 text budget entrypoint over the
existing evidence surfaces. Native runs use `fretboard-dev diag repeat --no-compare` plus
`--check-memory-p90-max` for text-heavy and code-editor memory probes. Web/wasm runs can pass a raw
exported `bundle.json` with `--web-export-bundle`; the helper checks the wasm-sized text shape cache
entry limit, glyph atlas max-page budget, atlas live-vs-budget bytes, and renderer text upload metric
presence.

This closes the tooling gap without changing the Rust perf baseline schema. Real target-machine
native repeats and web runtime bundles are still evidence collection steps; the landed verification
uses dry-run command generation and synthetic web bundle tests.

# Decisions

- Keep native memory enforcement on the existing repeat memory p90 gate. This avoids creating a
  second distribution evaluator in Python.
- Keep code-editor memory and generic text-heavy memory as separate fresh-launch probes; suite mode
  is not used because it would reuse one app process across cold-memory samples.
- Treat web/wasm as artifact verification first. The helper validates raw bundle diagnostics instead
  of trying to automate a browser/devtools session in this slice.
- Require renderer text upload metrics to be present in web bundles, but only hard-cap atlas
  evictions by default. Upload byte thresholds should be calibrated from real web evidence.

# Changed Files

- `tools/perf/diag_u8_text_budget_gate.py`
- `tools/perf/test_diag_u8_text_budget_gate.py`
- `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-contract-matrix.md`
- `docs/workstreams/ui-memory-footprint-closure-v1/README.md`

# Verification

- `PYTHONPATH=tools/perf python3 -m unittest tools.perf.test_diag_u8_text_budget_gate`
- `python3 -m py_compile tools/perf/diag_u8_text_budget_gate.py tools/perf/test_diag_u8_text_budget_gate.py`
- `python3 tools/perf/diag_u8_text_budget_gate.py --help`
- `python3 tools/perf/diag_u8_text_budget_gate.py --dry-run --skip-code-editor --out-dir target/fret-diag-u8-text-budget-gate-dry-run`
- `python3 tools/perf/audit_perf_baselines.py --matrix docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-contract-matrix.md --strict`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`
- `cargo fmt --all --check`
- `git diff --check`

# Next Action

Run the native helper on an appropriate perf host and validate at least one web/wasm exported
runtime bundle with `--web-export-bundle`, then record the resulting evidence paths.
