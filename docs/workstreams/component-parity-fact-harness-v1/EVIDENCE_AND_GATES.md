---
title: Component Parity Fact Harness v1 Evidence and Gates
status: active
date: 2026-05-25
---

# Evidence and Gates

## Baseline Commands

Validate the Python tool, Button Group pilot artifact, workstream state, and catalog:

```powershell
python -m py_compile tools/parity-discovery/shadcn_parity_discovery.py
python tools/parity-discovery/shadcn_parity_discovery.py --mapping tools/parity-discovery/fixtures/button_group_parts_v1.json --fret-layout-sidecar-dir target/fret-diag-shadcn-parity-seed-codex/sessions/1779639738088-60664 --upstream-dom-snapshot docs/workstreams/shadcn-parity-discovery-harness-v1/artifacts/upstream-dom/button-group-input.json --upstream-dom-snapshot docs/workstreams/shadcn-parity-discovery-harness-v1/artifacts/upstream-dom/button-group-select.json --output docs/workstreams/component-parity-fact-harness-v1/artifacts/button_group_agent_packet_pilot_v1.json
python -m json.tool docs/workstreams/component-parity-fact-harness-v1/artifacts/button_group_agent_packet_pilot_v1.json | Out-Null
python tools/parity-discovery/shadcn_parity_discovery.py --suite tools/parity-discovery/suites/shadcn_parity_discovery_v2.json --suite-from-existing-reports --suite-output docs/workstreams/shadcn-parity-discovery-harness-v2/artifacts/shadcn_parity_suite_report_v2.json
python -m json.tool docs/workstreams/shadcn-parity-discovery-harness-v2/artifacts/shadcn_parity_suite_report_v2.json | Out-Null
python -m json.tool docs/workstreams/component-parity-fact-harness-v1/WORKSTREAM.json | Out-Null
python tools/check_workstream_catalog.py
```

## Button Group Pilot Evidence

Source facts:

- `tools/parity-discovery/fixtures/button_group_parts_v1.json`
- `repo-ref/ui/apps/v4/registry/new-york-v4/ui/button-group.tsx`
- `repo-ref/ui/apps/v4/registry/new-york-v4/examples/button-group-input.tsx`
- `repo-ref/ui/apps/v4/registry/new-york-v4/examples/button-group-dropdown.tsx`
- `repo-ref/ui/apps/v4/registry/new-york-v4/examples/button-group-select.tsx`
- `repo-ref/ui/apps/v4/registry/new-york-v4/examples/input-group-button-group.tsx`

Fret evidence:

- `docs/workstreams/shadcn-parity-harness-v1/README.md`
- `tools/diag-scripts/ui-gallery/shadcn-parity/ui-gallery-shadcn-parity-seed-layout.json`
- `target/fret-diag-shadcn-parity-seed-codex/sessions/1779639738088-60664/1779640214741/ai.packet`
- `target/fret-diag-shadcn-parity-seed-codex/sessions/1779639738088-60664/**/layout.taffy.v1.json`
- `target/fret-diag-shadcn-parity-seed-codex/sessions/1779639738088-60664/**/bundle.schema2.json`

Generated artifact:

- `docs/workstreams/component-parity-fact-harness-v1/artifacts/button_group_agent_packet_pilot_v1.json`
- `docs/workstreams/component-parity-fact-harness-v1/artifacts/shadcn_parity_suite_report_v2_agent_summary.json`
- `docs/workstreams/shadcn-parity-discovery-harness-v2/artifacts/shadcn_parity_suite_report_v2.json`

Live fact coverage:

- Button Group pilot now records 6 upstream live DOM/CSS facts from `computedStyle`.
- Button Group pilot now records 14 Fret facts from layout sidecars and bundle schema2 semantics.
- Upstream facts include class tokens, layout CSS, text metrics, paint colors, border widths,
  corner radii, and icon descendant bounds.
- Fret facts include bounds, raw bounds, coordinate units, kind/role hints, and related `.chrome` /
  `-icon` nodes. Fret paint/text facts remain hints until diagnostics exports first-class paint/text
  tables.

## Gate Policy

Use the report queues as the promotion rule:

- `repair_queue`: fix before claiming parity. Use the row's owner/layer to choose recipe, policy,
  mechanism, app-demo, or diagnostics ownership.
- `hardening_queue`: already passing but not strong enough as a future-proof gate. Prefer live
  measurement or higher-confidence source facts before broadening coverage.
- `gate_queue`: promote only after the owner/layer is confirmed. Component recipe rows become
  component fixtures, diagnostics rows become diag scripts, and reusable mechanism rows become
  mechanism harness cases.

## Validation Notes

2026-05-25 local validation:

- `python -m py_compile tools/parity-discovery/shadcn_parity_discovery.py`: PASS.
- `python -m json.tool docs/workstreams/component-parity-fact-harness-v1/WORKSTREAM.json | Out-Null`:
  PASS.
- `python -m json.tool tools/parity-discovery/fixtures/button_group_parts_v1.json | Out-Null`:
  PASS.
- Button Group pilot generation: PASS, produced 7 parts, 4 layout sidecars, 4 bundle schema2 files,
  and 2 upstream DOM snapshots.
- Pilot JSON validation: PASS.
- `python tools/check_workstream_catalog.py`: PASS, 437 dedicated directories and 47 standalone
  markdown files.
- `git diff --check`: PASS.

Pilot packet summary:

- `status`: `needs_hardening`
- `repair_queue_count`: 0
- `hardening_queue_count`: 1
- `gate_queue_count`: 9
- First hardening row: `root_source_facts_need_live_layout_extractor`
- Fret wiring: 12 stable `test_id`s

2026-05-25 CPF-050/CPF-060/CPF-090 validation:

- `python -m py_compile tools/parity-discovery/shadcn_parity_discovery.py`: PASS.
- `python -m json.tool tools/parity-discovery/fixtures/button_group_parts_v1.json | Out-Null`:
  PASS.
- Button Group pilot generation after live facts: PASS.
- Pilot summary: 6 upstream live facts, 14 Fret live facts, 0 repair rows, 1 hardening row, and 9
  gate rows.
- `python tools/parity-discovery/shadcn_parity_discovery.py --suite tools/parity-discovery/suites/shadcn_parity_discovery_v2.json --suite-from-existing-reports --suite-output docs/workstreams/shadcn-parity-discovery-harness-v2/artifacts/shadcn_parity_suite_report_v2.json`:
  PASS, 9 reports, 17 parts, 0 top findings.
- v2 suite agent summary: `regression_locked`, 0 repair rows, 0 hardening rows.
- `tools/parity-discovery/manifests/shadcn_parity_coverage_v2.json` now records the suite agent
  summary artifact and refresh command under `agent_summary`.

Suite smoke:

- Direct replay of `tools/parity-discovery/suites/shadcn_parity_discovery_v1.json` was not used as
  this lane's final gate because the manifest references a historical target sidecar directory that
  is absent in this worktree:
  `target/fret-diag/shadcn-parity-discovery-sweep-v1/button-group-select-after-select-padding/sessions/1778337694097-135816`.
- A current-evidence smoke suite was generated under `target/component-parity-fact-harness-v1/` and
  validated the new suite-level `agent_packet` summary:
  `target/component-parity-fact-harness-v1/suite_smoke_report.json`.
- Smoke summary: 1 report, 7 parts, 0 top findings, status `needs_hardening`, 0 repair rows, 1
  hardening row, and 9 gate rows.

The initial Button Group seed diagnostics run also exposed two unrelated warning surfaces that are
outside this lane:

- unexpected cfg `unstable-retained-bridge` in `crates/fret-ui/src/tree/layout/clean_geometry.rs`
- dead code `current_effective_opacity` in `crates/fret-ui/src/elements/runtime.rs`

Treat those as cleanup candidates, not parity harness blockers.
