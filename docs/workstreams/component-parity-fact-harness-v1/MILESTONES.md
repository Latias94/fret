---
title: Component Parity Fact Harness v1 Milestones
status: closed
date: 2026-05-25
---

# Milestones

## M0 - Lane Opened

Status: complete on 2026-05-25.

Evidence:

- `docs/workstreams/component-parity-fact-harness-v1/WORKSTREAM.json`
- `docs/workstreams/component-parity-fact-harness-v1/DESIGN.md`
- `docs/workstreams/component-parity-fact-harness-v1/TODO.md`
- `docs/workstreams/component-parity-fact-harness-v1/EVIDENCE_AND_GATES.md`
- `docs/workstreams/component-parity-fact-harness-v1/HANDOFF.md`

## M1 - Agent Packet Pilot

Status: complete on 2026-05-25 after the Button Group pilot artifact was generated.

Exit criteria:

- `tools/parity-discovery` emits `agent_packet` for individual reports.
- Suite reports summarize per-report agent packet readiness.
- Button Group pilot artifact contains source truth, Fret wiring, repair/hardening/gate queues, and
  evidence contexts.

## M2 - Live Source Facts

Status: complete on 2026-05-25 for the first shadcn DOM/CSS extractor slice.

Completed criteria:

- shadcn source facts can be extracted mechanically from upstream DOM snapshots for mapped
  `upstream.dom_target_ids`.
- The Button Group pilot now records 6 upstream DOM/CSS facts and 14 Fret layout/semantics facts.
- Extracted facts include class tokens, computed layout values, text metrics, paint values,
  border/radius values, and icon descendant bounds.

Remaining hardening:

- Curated prose is still retained as context.
- Fret paint/text facts are currently layout/bundle hints until diagnostics exports first-class
  paint/text tables.

## M2b - Shadcn v2 Agent Summary

Status: complete on 2026-05-25.

Completed criteria:

- `tools/parity-discovery` can generate suite summaries from already generated report artifacts via
  `--suite-from-existing-reports`.
- `docs/workstreams/shadcn-parity-discovery-harness-v2/artifacts/shadcn_parity_suite_report_v2.json`
  now contains a suite-level `agent_packet`.
- The component parity lane also archives the generated summary at
  `docs/workstreams/component-parity-fact-harness-v1/artifacts/shadcn_parity_suite_report_v2_agent_summary.json`.

## M3 - Material Adapter

Status: complete for the first adapter slice on 2026-05-25.

Completed criteria:

- A Material 3 component slice maps spec, MUI, and Compose facts into the same report/agent packet
  shape.
- The adapter keeps Material policy in the ecosystem layer and promotes only reusable mechanism
  gaps inward.

Evidence:

- `tools/parity-discovery/fixtures/material3_button_adapter_v1.json`
- `docs/workstreams/component-parity-fact-harness-v1/artifacts/material3_button_adapter_pilot_v1.json`
- `ecosystem/fret-ui-material3/src/button.rs`
- `ecosystem/fret-ui-material3/src/tokens/button.rs`

## M4 - Semantics and Text/Paint Facts

Status: complete for the first schema2 + packet slice on 2026-05-25.

Completed criteria:

- Agent packets now include upstream DOM semantics/interaction facts from role/name/state/relation
  attributes and implicit control roles.
- Agent packets now include Fret semantics/interaction facts from bundle schema2 role, label,
  flags, actions, relations, active-descendant, and scroll fields.
- Fret diagnostics schema2 now exports a first-class `tables.text_paint` table with text input,
  renderer text perf, widget measure, paint widget, and text prepare rows.

## M5 - Live Diagnostics Packet Closure

Status: complete for the Button Group and first Material 3 Button live slices on 2026-05-25.

Completed criteria:

- Agent packets compact repeated bundle semantics/text-paint observations into stable facts with
  observed counts, node samples, and bounded evidence paths.
- Button Group pilot evidence was refreshed from a current UI Gallery seed diagnostics run, and the
  pilot now records both per-node text/paint facts and bundle-level `tables.text_paint` row counts.
- Material 3 Button gallery rows now expose stable live `test_id`s for filled/default and
  filled/disabled buttons.
- Material 3 Button adapter checks can use `live_fact_requirements` so live semantics and
  bundle-level text/paint evidence move checks out of repair queues without pretending sparse
  hotspot rows are per-button paint geometry.

## M6 - Closeout and Follow-on Decision

Status: complete on 2026-05-25.

Completed criteria:

- Agent packets now keep direct `text_paint`, semantics-ancestor-associated `text_paint`, semantic
  text descendants, and hotspot-sparse coverage notes separate.
- Button Group regenerated from current evidence with 6 direct text/paint facts, 21 associated
  text/paint facts, and 68 semantic label facts.
- Material 3 Button regenerated from current evidence with 1 bounded MUI upstream DOM snapshot, 2
  upstream DOM targets, 16 semantic label facts, 3746 bundle text/paint rows, and no direct or
  associated text/paint facts for the selected button nodes.
- No confirmed mechanism-level defect was found, so this lane closes without creating a mechanism
  follow-on.

Follow-on boundary:

- Automated MUI browser capture, broader Material component coverage, or a future mechanism defect
  should start a new narrow workstream instead of reopening this v1 lane.
