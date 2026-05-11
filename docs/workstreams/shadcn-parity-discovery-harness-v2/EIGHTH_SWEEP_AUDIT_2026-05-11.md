---
title: Shadcn Parity Discovery Eighth Sweep Audit
status: active
date: 2026-05-11
scope: shadcn parity discovery, table, row geometry harness
---

# Eighth Sweep Audit

This audit records the base Table docs-demo static slice from the v2 sweep. The slice did not find a
new mismatch, but it promoted a table-heavy surface into the fixture-driven suite before moving to
policy-heavy DataTable coverage.

## Objective Criteria

The slice required:

1. Capture the Table docs demo with layout sidecar, bundle, and screenshot evidence.
2. Compare stable shadcn `new-york-v4` row/caption/footer height facts against the upstream DOM
   snapshot.
3. Classify every non-passing result by layer.
4. Promote the slice into the v2 suite as a reusable regression gate.

## Findings

### No new Table row geometry mismatch

- Report:
  `docs/workstreams/shadcn-parity-discovery-harness-v2/artifacts/table_demo_mismatch_report_v2.json`
- Result:
  `1 pass_known` part, `3 pass_known` recipe checks, `0 mismatch`, `0 blocked`, `0 top findings`.
- Locked geometry:
  total demo table height, 37px body row height, and body row vertical cadence through the last
  invoice row.
- Owner and layer:
  `component_recipe` / `recipe`.

## Evidence

- Diag script:
  `tools/diag-scripts/ui-gallery/table/ui-gallery-table-demo-layout.json`
- Layout sidecar:
  `target/fret-diag/shadcn-parity-discovery-harness-v2-table-demo-layout/sessions/1778494087822-119804/1778494093489-ui-gallery-table-demo.layout/layout.taffy.v1.json`
- AI packet:
  `target/fret-diag/shadcn-parity-discovery-harness-v2-table-demo-layout/sessions/1778494087822-119804/1778494092251/ai.packet`
- Fixture:
  `tools/parity-discovery/fixtures/table_demo_parts_v1.json`
- Suite:
  `tools/parity-discovery/suites/shadcn_parity_discovery_v2.json`

## Gate Result

- Diag run:
  `target/debug/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/table/ui-gallery-table-demo-layout.json --dir target/fret-diag/shadcn-parity-discovery-harness-v2-table-demo-layout --session-auto --pack --ai-packet --include-screenshots --launch -- target/debug/fret-ui-gallery.exe`
  passed.
- Fixture report:
  `python tools/parity-discovery/shadcn_parity_discovery.py --mapping tools/parity-discovery/fixtures/table_demo_parts_v1.json --fret-layout-sidecar-dir target/fret-diag/shadcn-parity-discovery-harness-v2-table-demo-layout --upstream-dom-snapshot F:/SourceCodes/Rust/fret/goldens/shadcn-web/v4/new-york-v4/table-demo.json --output docs/workstreams/shadcn-parity-discovery-harness-v2/artifacts/table_demo_mismatch_report_v2.json`
  generated a passing report.

## Residual Follow-Ups

- Add a separate DataTable slice for policy-heavy sorting/filtering/pagination/column visibility.
  This base Table slice intentionally gates static recipe geometry first.
- Add cell-level test ids if a future table slice needs column-width or text-alignment parity instead
  of row-level geometry.
