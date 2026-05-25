---
title: Component Parity Fact Harness v1 Milestones
status: active
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
