---
title: Shadcn Parity Discovery Ninth Sweep Audit
status: active
date: 2026-05-11
scope: shadcn parity discovery, data table, policy diagnostics suite
---

# Ninth Sweep Audit

This audit records the DataTable docs-path policy slice from the v2 sweep. The slice did not find a
new DataTable component policy mismatch, but it promoted the existing policy-heavy diagnostics suite
as a reusable gate and found multiple harness issues that could have hidden or misclassified future
component failures.

## Objective Criteria

The slice required:

1. Run policy-heavy DataTable interactions beyond static Table geometry.
2. Cover column visibility, row-actions overlays, checkbox-only selection, list-like pointer
   selection, and smoke navigation.
3. Classify every non-passing result by layer.
4. Fix confirmed high-impact harness issues and leave a reusable regression gate.

## Findings

### No new DataTable policy mismatch

- Suite:
  `tools/diag-scripts/suites/ui-gallery-data-table/suite.json`
- Result:
  `7 passed`, `0 failed`, `status: passed`.
- Covered scripts:
  guide header screenshot, basic row-actions menu toggle stability, default recipe smoke,
  guide checkbox-only selection, guide row-actions menu stability, list-like pointer selection, and
  DataTable smoke.
- Owner and layer:
  `component_policy` / `policy`.

### Checkbox-only selection script clicked an off-window target

- Pre-fix symptom:
  `ui-gallery-data-table-guide-demo-checkbox-only-selection.json` timed out waiting for
  `ui-gallery-data-table-select-row-1` to be within the window.
- Evidence:
  the failure bundle placed the checkbox at `y=754 h=16` in a `720px` tall window.
- Classification:
  diagnostics script / runner harness, not DataTable policy.
- Fix:
  the script now scrolls the row checkbox into view before asserting bounds and clicking it.

### List-like pointer selection script clicked off-window rows

- Pre-fix symptoms:
  the list-like script first timed out waiting for row 1 bounds, then failed after shift-click
  because row 3 was clamped outside the window and received no hit.
- Evidence:
  the failure trace recorded `blocking_reason=no_hit` and `clamped_outside_window=true` for
  `ui-gallery-data-table-listlike-row-3`.
- Classification:
  diagnostics script / runner harness, not DataTable policy.
- Fix:
  the script now uses small-step `scroll_into_view` with full container/window requirements before
  each row click.

### Reuse-launch suite lint used a stale bundle result

- Pre-fix symptom:
  all seven DataTable scripts passed, but the suite failed lint because
  `ui-gallery-data-table-smoke` reported a `last_bundle_dir` from the previous list-like script.
- Root cause:
  a final `capture_bundle` step wrote the per-run `script.result.json` before the asynchronous dump
  completed; the later best-effort update refreshed the bundle-local result but not the
  `run_id/script.result.json` consumed by suite lint.
- Classification:
  diagnostics runner / harness.
- Fix:
  `best_effort_update_script_result_last_bundle_artifact` now updates the per-run
  `script.result.json` alongside the bundle-local result.

## Evidence

- Patched checkbox-only selection script:
  `tools/diag-scripts/ui-gallery/data-table/ui-gallery-data-table-guide-demo-checkbox-only-selection.json`
- Patched list-like selection script:
  `tools/diag-scripts/ui-gallery/data-table/ui-gallery-data-table-listlike-pointer-selection.json`
- Runner fix:
  `ecosystem/fret-bootstrap/src/ui_diagnostics/fs_triggers.rs`
- Passing suite summary:
  `target/fret-diag/shadcn-parity-discovery-harness-v2-data-table-policy-suite-run-dir-fix/sessions/1778496500156-72456/suite.summary.json`

## Gate Result

- DataTable suite:
  `target/debug/fretboard-dev.exe diag suite tools/diag-scripts/suites/ui-gallery-data-table/suite.json --dir target/fret-diag/shadcn-parity-discovery-harness-v2-data-table-policy-suite-run-dir-fix --session-auto --timeout-ms 900000 --ai-packet --reuse-launch --launch -- target/dev-fast/fret-ui-gallery.exe`
  passed.
- JSON checks:
  `python -m json.tool tools/diag-scripts/ui-gallery/data-table/ui-gallery-data-table-guide-demo-checkbox-only-selection.json | Out-Null`
  passed.
- Registry:
  `python tools/check_diag_scripts_registry.py`
  passed.

## Residual Follow-Ups

- The `ui-gallery-data-table-retained-*` scripts still target the `DataTable (Torture)` page, which
  is gated behind `gallery-dev`. Treat those as dev-feature/perf-retained scripts, not default
  docs-path policy gates, until the suite can encode required Gallery features explicitly.
- `diag suite --help` currently exposes `--pack`, but suite execution rejects it as `diag run` only.
  That CLI/help inconsistency is a runner polish follow-up.
- The suite still reports existing `semantics.missing_label` warnings. They did not fail the gate,
  but a later accessibility-focused sweep should classify and reduce them deliberately.
