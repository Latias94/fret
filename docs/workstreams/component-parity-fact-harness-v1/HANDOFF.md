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
- CPF-092/CPF-094/CPF-096 are complete for the live diagnostics closure slice: bundle semantics and
  text/paint facts are compacted before entering the packet, Button Group now consumes current
  `tables.text_paint` evidence, and Material 3 Button uses live UI Gallery bundle facts through
  explicit `live_fact_requirements`.
- The current Button Group pilot records 6 per-node Fret text/paint facts, 160 bundle
  `tables.text_paint` entries, and 5532 compactable text/paint rows from session
  `target/fret-diag-component-parity-button-group-text-paint-v1/sessions/1779671244627-41048`.
- The current Material 3 Button adapter pilot records 4 Fret semantics facts, 4 Fret interaction
  facts, 180 bundle `tables.text_paint` entries, and 3746 text/paint rows from session
  `target/fret-diag-component-parity-material3-button-live-v1/sessions/1779671892793-82708`.

How to continue:

1. Start with the generated `agent_packet`, not screenshots.
2. If `repair_queue` is non-empty, fix rows by owner/layer and refresh the same report.
3. If only `hardening_queue` is non-empty, add live source/Fret facts before broadening coverage.
4. Promote stable rows through `gate_queue` into component fixtures, diag scripts, or mechanism
   harness cases.
5. For Material 3, attach an upstream Material UI DOM snapshot next. The Fret live side is now
   hardening-only, not repair-only.

Residual risk:

- Button Group is already a mostly closed shadcn seed, so the pilot proves packet shape more than it
  proves discovery power.
- Current source facts still include curated prose as context; future rows should prefer
  `upstream.dom_target_ids` when upstream DOM/CSS evidence exists.
- `tables.text_paint` is a sparse diagnostics/hotspot table. The Material 3 Button pilot proves
  bundle-level table availability while per-button text/paint row association remains a hardening
  target, not a proven component defect.
- The Material 3 pilot still has no upstream Material UI DOM snapshot attached; use the local
  Material UI repo-ref source and a future DOM extractor before claiming cross-stack visual parity.
