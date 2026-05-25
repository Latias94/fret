---
title: Component Parity Fact Harness v1 Handoff
status: active
date: 2026-05-25
---

# Handoff

Current state:

- The lane is open and narrow: it owns unified component facts and agent repair packets, not broad
  shadcn coverage discovery.
- `tools/parity-discovery/shadcn_parity_discovery.py` now emits an `agent_packet` for every
  individual report and a compact packet summary for suite reports.
- The Button Group pilot artifact is the first proof target:
  `docs/workstreams/component-parity-fact-harness-v1/artifacts/button_group_agent_packet_pilot_v1.json`.
- The pilot currently reports `needs_hardening`, not `needs_repair`: all 7 Button Group parts pass,
  repair queue is empty, and the single hardening row is
  `root_source_facts_need_live_layout_extractor`.
- CPF-050 and CPF-060 are complete for the first shadcn slice: the pilot records 6 upstream
  DOM/CSS facts and 14 Fret layout/semantics facts.
- CPF-090 is complete for suite summary generation: the shadcn v2 suite report now has an
  `agent_packet`, and it can be refreshed from existing report artifacts with
  `--suite-from-existing-reports` when historical target sidecars are not available.
- CPF-065 is complete for the exporter slice: new diagnostics bundle schema2 output includes a
  first-class `tables.text_paint` table for text input, renderer text perf, widget measure, paint
  widget, and text prepare rows.
- CPF-070 is complete for the packet slice: upstream DOM role/name/state/relation/focusability
  facts and Fret bundle semantics flags/actions/relations are now included in agent packets.
- CPF-080 is complete for the first adapter slice:
  `docs/workstreams/component-parity-fact-harness-v1/artifacts/material3_button_adapter_pilot_v1.json`
  maps Material spec, MUI, Compose Material3, and Fret Material3 Button references into the same
  packet shape.

How to continue:

1. Start with the generated `agent_packet`, not screenshots.
2. If `repair_queue` is non-empty, fix rows by owner/layer and refresh the same report.
3. If only `hardening_queue` is non-empty, add live source/Fret facts before broadening coverage.
4. Promote stable rows through `gate_queue` into component fixtures, diag scripts, or mechanism
   harness cases.
5. For Material 3, attach a real Material Button diagnostics bundle and an upstream DOM snapshot
   next; the adapter already emits the right repair/gate queues.

Residual risk:

- Button Group is already a mostly closed shadcn seed, so the pilot proves packet shape more than it
  proves discovery power.
- Current source facts still include curated prose as context; future rows should prefer
  `upstream.dom_target_ids` when upstream DOM/CSS evidence exists.
- Historical Button Group bundles predate `tables.text_paint`, so the pilot has 0 Fret text/paint
  table rows even though new diagnostics exports the table. Refresh the Button Group diag evidence
  before claiming rendered text/paint parity.
- The Material 3 pilot intentionally reports `needs_repair` because it has no live upstream DOM or
  Fret diagnostics evidence attached yet; treat that as the next evidence-capture slice, not as a
  component defect.
