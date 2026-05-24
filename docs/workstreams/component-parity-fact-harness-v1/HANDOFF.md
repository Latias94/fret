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

How to continue:

1. Start with the generated `agent_packet`, not screenshots.
2. If `repair_queue` is non-empty, fix rows by owner/layer and refresh the same report.
3. If only `hardening_queue` is non-empty, add live source/Fret facts before broadening coverage.
4. Promote stable rows through `gate_queue` into component fixtures, diag scripts, or mechanism
   harness cases.
5. Keep Material 3 support as a source adapter into the same packet shape; do not fork a separate
   harness unless the fact model proves incompatible.

Residual risk:

- Button Group is already a mostly closed shadcn seed, so the pilot proves packet shape more than it
  proves discovery power.
- Current source facts still include curated prose as context; future rows should prefer
  `upstream.dom_target_ids` when upstream DOM/CSS evidence exists.
- Fret paint/text facts are still hints from layout sidecar labels and bundle semantics. Diagnostics
  needs first-class paint/text tables before claiming full rendered paint parity.
- CPF-070 should add interaction/semantics facts next.
