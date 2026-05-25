---
title: Component Parity Fact Harness v1 Evidence and Gates
status: closed
date: 2026-05-25
---

# Evidence and Gates

## Baseline Commands

Validate the Python tool, Button Group pilot artifact, workstream state, and catalog:

```powershell
python -m py_compile tools/parity-discovery/shadcn_parity_discovery.py
target\debug\fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/shadcn-parity/ui-gallery-shadcn-parity-seed-layout.json --dir target/fret-diag-component-parity-button-group-text-paint-v1 --session-auto --pack --ai-packet --exit-after-run --timeout-ms 360000 --launch -- target\debug\fret-ui-gallery.exe
python tools/parity-discovery/shadcn_parity_discovery.py --mapping tools/parity-discovery/fixtures/button_group_parts_v1.json --fret-layout-sidecar-dir target/fret-diag-component-parity-button-group-text-paint-v1/sessions/1779671244627-41048 --upstream-dom-snapshot docs/workstreams/shadcn-parity-discovery-harness-v1/artifacts/upstream-dom/button-group-input.json --upstream-dom-snapshot docs/workstreams/shadcn-parity-discovery-harness-v1/artifacts/upstream-dom/button-group-select.json --output docs/workstreams/component-parity-fact-harness-v1/artifacts/button_group_agent_packet_pilot_v1.json
python -m json.tool docs/workstreams/component-parity-fact-harness-v1/artifacts/button_group_agent_packet_pilot_v1.json | Out-Null
python tools/parity-discovery/shadcn_parity_discovery.py --suite tools/parity-discovery/suites/shadcn_parity_discovery_v2.json --suite-from-existing-reports --suite-output docs/workstreams/shadcn-parity-discovery-harness-v2/artifacts/shadcn_parity_suite_report_v2.json
python -m json.tool docs/workstreams/shadcn-parity-discovery-harness-v2/artifacts/shadcn_parity_suite_report_v2.json | Out-Null
target\debug\fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/material3/button/ui-gallery-material3-button-sizes-screenshots.json --dir target/fret-diag-component-parity-material3-button-live-v1 --session-auto --pack --ai-packet --exit-after-run --timeout-ms 360000 --launch -- cargo run -p fret-ui-gallery --features gallery-full
python tools/parity-discovery/shadcn_parity_discovery.py --mapping tools/parity-discovery/fixtures/material3_button_adapter_v1.json --fret-bundle-schema2-dir target/fret-diag-component-parity-material3-button-live-v1/sessions/1779671892793-82708 --upstream-dom-snapshot docs/workstreams/component-parity-fact-harness-v1/artifacts/upstream-dom/material3-button-mui-contained.json --output docs/workstreams/component-parity-fact-harness-v1/artifacts/material3_button_adapter_pilot_v1.json
python -m json.tool docs/workstreams/component-parity-fact-harness-v1/artifacts/material3_button_adapter_pilot_v1.json | Out-Null
cargo test -p fret-bootstrap --lib --features "ui-app-driver diagnostics" schema2_exports_text_paint_facts_table_from_debug_snapshots
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
- `target/fret-diag-component-parity-button-group-text-paint-v1/sessions/1779671244627-41048/1779671250995/ai.packet`
- `target/fret-diag-component-parity-button-group-text-paint-v1/sessions/1779671244627-41048/**/layout.taffy.v1.json`
- `target/fret-diag-component-parity-button-group-text-paint-v1/sessions/1779671244627-41048/**/bundle.schema2.json`

Generated artifact:

- `docs/workstreams/component-parity-fact-harness-v1/artifacts/button_group_agent_packet_pilot_v1.json`
- `docs/workstreams/component-parity-fact-harness-v1/artifacts/shadcn_parity_suite_report_v2_agent_summary.json`
- `docs/workstreams/shadcn-parity-discovery-harness-v2/artifacts/shadcn_parity_suite_report_v2.json`

Live fact coverage:

- Button Group pilot now records 6 upstream live DOM/CSS facts from `computedStyle`.
- Button Group pilot now records 14 Fret facts from layout sidecars and bundle schema2 semantics.
- Button Group pilot now records 6 upstream semantics facts, 6 upstream interaction facts, 14 Fret
  semantics facts, and 14 Fret interaction facts in the agent packet.
- Upstream facts include class tokens, layout CSS, text metrics, paint colors, border widths,
  corner radii, icon descendant bounds, role/name/state/relation hints, and focusability hints.
- Fret facts include bounds, raw bounds, coordinate units, kind/role hints, and related `.chrome` /
  `-icon` nodes.
- The current Button Group pilot records 6 direct Fret text/paint facts, 21
  semantics-descendant-associated text/paint facts, 68 semantic label facts, plus 160 bundle
  `tables.text_paint` entries and 5532 raw text/paint rows available for future focused gates.

## Material 3 Adapter Evidence

Source facts:

- `tools/parity-discovery/fixtures/material3_button_adapter_v1.json`
- `https://m3.material.io/components/buttons/overview`
- `F:/SourceCodes/Rust/fret/repo-ref/material-ui/packages/mui-material/src/Button/Button.js`
- `F:/SourceCodes/Rust/fret/repo-ref/material-ui/docs/data/material/components/buttons/ContainedButtons.tsx`
- `F:/SourceCodes/Rust/fret/repo-ref/compose-multiplatform-core/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Button.kt`

Fret evidence:

- `ecosystem/fret-ui-material3/src/button.rs`
- `ecosystem/fret-ui-material3/src/tokens/button.rs`
- `ecosystem/fret-ui-material3/tests/radio_alignment.rs`
- `ecosystem/fret-bootstrap/src/ui_diagnostics/bundle.rs`
- `apps/fret-ui-gallery/src/ui/snippets/material3/button.rs`
- `tools/diag-scripts/ui-gallery/material3/button/ui-gallery-material3-button-sizes-screenshots.json`
- `target/fret-diag-component-parity-material3-button-live-v1/sessions/1779671892793-82708/1779671898136/ai.packet`
- `target/fret-diag-component-parity-material3-button-live-v1/sessions/1779671892793-82708/**/bundle.schema2.json`

Generated artifact:

- `docs/workstreams/component-parity-fact-harness-v1/artifacts/material3_button_adapter_pilot_v1.json`
- `docs/workstreams/component-parity-fact-harness-v1/artifacts/upstream-dom/material3-button-mui-contained.json`

Pilot packet summary:

- `status`: `needs_hardening`
- `repair_queue_count`: 0
- `hardening_queue_count`: 3
- `gate_queue_count`: 3
- Upstream coverage: 1 bounded MUI DOM snapshot, 2 upstream DOM targets, 4 upstream semantics
  facts, and 4 upstream interaction facts across the two Material adapter parts.
- Fret coverage: 4 Fret semantics facts, 4 Fret interaction facts, 16 semantic label facts, 180
  bundle `tables.text_paint` entries, and 3746 raw text/paint rows.
- Interpretation: Fret-side live evidence and a bounded upstream DOM slice are attached. Per-button
  label text remains hotspot-sparse in `tables.text_paint`, so semantic label coverage is explicit
  and distinct from direct or associated text/paint coverage.

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

2026-05-25 CPF-065/CPF-070/CPF-080 validation:

- `python -m py_compile tools/parity-discovery/shadcn_parity_discovery.py`: PASS.
- `python -m json.tool tools/parity-discovery/fixtures/material3_button_adapter_v1.json | Out-Null`:
  PASS.
- Button Group pilot generation after semantics/interaction facts: PASS.
- Button Group packet summary: 6 upstream semantics facts, 6 upstream interaction facts, 14 Fret
  semantics facts, 14 Fret interaction facts, 0 Fret text/paint facts from the historical bundle.
- Material 3 Button adapter pilot generation: PASS, produced 2 parts, 2 repair rows, and 2 gate
  rows.
- Material 3 Button adapter JSON validation: PASS.
- `cargo test -p fret-bootstrap --lib --features "ui-app-driver diagnostics" schema2_exports_text_paint_facts_table_from_debug_snapshots`:
  PASS.
- `cargo test -p fret-bootstrap --lib --features "ui-app-driver diagnostics" ui_diagnostics::bundle::tests::env_fingerprint_exports_host_monitor_topology_without_reclassifying_scale_factors_seen`:
  PASS.

2026-05-25 CPF-092/CPF-094/CPF-096 validation:

- `cargo build -p fretboard-dev -p fret-ui-gallery --features gallery-full`: PASS.
- `target\debug\fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/shadcn-parity/ui-gallery-shadcn-parity-seed-layout.json --dir target/fret-diag-component-parity-button-group-text-paint-v1 --session-auto --pack --ai-packet --exit-after-run --timeout-ms 360000 --launch -- target\debug\fret-ui-gallery.exe`:
  PASS, run id `1779671250995`.
- Button Group bundle verification: PASS, the captured bundle set contains `tables.text_paint`
  entries in all checked schema2 bundles.
- Button Group pilot regeneration against session `1779671244627-41048`: PASS.
- Button Group packet summary: `needs_hardening`, 0 repair rows, 1 hardening row, 9 gate rows,
  6 per-node Fret text/paint facts, 160 bundle `tables.text_paint` entries, and 5532 text/paint
  rows.
- `cargo build -p fret-ui-gallery --features gallery-full`: PASS after adding stable Material 3
  Button gallery `test_id`s.
- `target\debug\fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/material3/button/ui-gallery-material3-button-sizes-screenshots.json --dir target/fret-diag-component-parity-material3-button-live-v1 --session-auto --pack --ai-packet --exit-after-run --timeout-ms 360000 --launch -- cargo run -p fret-ui-gallery --features gallery-full`:
  PASS, run id `1779671898136`.
- Material 3 Button adapter regeneration against session `1779671892793-82708`: PASS.
- Material 3 Button adapter summary: `needs_hardening`, 0 repair rows, 2 hardening rows, 2 gate
  rows, 4 Fret semantics facts, 4 Fret interaction facts, 180 bundle `tables.text_paint` entries,
  and 3746 text/paint rows.
- `python tools/parity-discovery/shadcn_parity_discovery.py --suite tools/parity-discovery/suites/shadcn_parity_discovery_v2.json --suite-from-existing-reports --suite-output docs/workstreams/shadcn-parity-discovery-harness-v2/artifacts/shadcn_parity_suite_report_v2.json`:
  PASS, 9 reports, 17 parts, 0 top findings.

2026-05-25 CPF-098/CPF-099 closeout validation:

- `python -m py_compile tools/parity-discovery/shadcn_parity_discovery.py`: PASS.
- `python -m json.tool tools/parity-discovery/fixtures/material3_button_adapter_v1.json | Out-Null`:
  PASS.
- Button Group pilot regeneration against session `1779671244627-41048`: PASS.
- Button Group packet summary: 7 parts, 2 upstream DOM snapshots, 6 upstream DOM targets, 6 direct
  Fret text/paint facts, 21 semantics-descendant-associated text/paint facts, 68 semantic label
  facts, 160 bundle `tables.text_paint` entries, and 5532 text/paint rows.
- Material 3 Button adapter regeneration against session `1779671892793-82708` plus
  `docs/workstreams/component-parity-fact-harness-v1/artifacts/upstream-dom/material3-button-mui-contained.json`:
  PASS.
- Material 3 Button adapter summary: 2 parts, 1 upstream DOM snapshot, 2 upstream DOM targets, 0
  direct Fret text/paint facts, 0 associated Fret text/paint facts, 16 semantic label facts, 180
  bundle `tables.text_paint` entries, and 3746 text/paint rows.
- Packet JSON validation for both refreshed artifacts: PASS.
- `cargo test -p fret-bootstrap --lib --features "ui-app-driver diagnostics" schema2_exports_text_paint_facts_table_from_debug_snapshots`:
  PASS, 1 test passed.
- `python tools/check_workstream_catalog.py`: PASS, 437 dedicated directories and 47 standalone
  markdown files.
- Closeout decision: no confirmed mechanism-layer defect was found, so no `fret-mechanism-harness`
  follow-on was split from this lane.

Known validation note:

- `cargo nextest run -p fret-bootstrap ui_diagnostics::bundle --no-fail-fast` was not used as the
  final Rust gate because examples are built under that invocation and the existing
  `fn_driver_escape_hatch` example requires feature wiring not enabled by the command
  (`BootstrapBuilder` is behind `launch`, and `fret_launch` is not linked for that example).

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
