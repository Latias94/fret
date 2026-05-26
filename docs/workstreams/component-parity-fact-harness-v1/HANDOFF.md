---
title: Component Parity Fact Harness v1 Handoff
status: closed
date: 2026-05-25
---

# Handoff

Current state:

- The lane is closed and narrow: it owns unified component facts and agent repair packets, not broad
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
- CPF-098/CPF-099 are complete for final packet hardening: direct `text_paint`, associated
  `text_paint`, semantic text descendants, and hotspot-sparse coverage notes are separate facts,
  and Material 3 Button has a bounded MUI contained-button upstream DOM slice.
- The current Button Group pilot records 6 direct Fret text/paint facts, 21 associated Fret
  text/paint facts, 68 semantic label facts, 160 bundle `tables.text_paint` entries, and 5532
  compactable text/paint rows from session
  `target/fret-diag-component-parity-button-group-text-paint-v1/sessions/1779671244627-41048`.
- The current Material 3 Button adapter pilot records 4 Fret semantics facts, 4 Fret interaction
  facts, 16 semantic label facts, 180 bundle `tables.text_paint` entries, and 3746 text/paint rows
  from session
  `target/fret-diag-component-parity-material3-button-live-v1/sessions/1779671892793-82708`.
- The current Material 3 Button upstream slice records 1 MUI DOM snapshot, 2 upstream DOM targets,
  4 upstream semantics facts, and 4 upstream interaction facts across the two adapter parts.
- No confirmed mechanism-layer defect was found during closeout, so no mechanism follow-on was
  created from this lane.

How to follow on:

1. Keep this lane closed unless a v1 artifact has to be corrected.
2. Start any new component parity work from the generated `agent_packet`, not screenshots.
3. If a future `repair_queue` is non-empty, fix rows by owner/layer and refresh that follow-on
   report.
4. If only `hardening_queue` is non-empty, add live source/Fret facts before broadening coverage.
5. Promote stable rows through `gate_queue` into component fixtures, diag scripts, or mechanism
   harness cases.
6. Automated Material UI browser capture and broader Material component coverage should be a new
   narrow follow-on.

Residual risk:

- Button Group is already a mostly closed shadcn seed, so the pilot proves packet shape more than it
  proves discovery power.
- Current source facts still include curated prose as context; future rows should prefer
  `upstream.dom_target_ids` when upstream DOM/CSS evidence exists.
- `tables.text_paint` is a sparse diagnostics/hotspot table. The Material 3 Button pilot proves
  bundle-level table availability and semantic label coverage while per-button direct/associated
  text/paint rows remain absent for the selected nodes. That is a diagnostics/hardening signal, not
  a proven component defect.
- The Material 3 pilot has a bounded static MUI DOM snapshot attached. Use an automated DOM
  extractor before claiming broad cross-stack visual parity.
