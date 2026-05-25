---
title: Component Parity Fact Harness v1 Closeout Audit
status: closed
date: 2026-05-25
---

# Closeout Audit

## Verdict

This lane is closed. The v1 target is satisfied: component parity facts now join upstream source
facts, Fret diagnostics evidence, agent repair queues, hardening queues, gate queues, and bounded
evidence anchors for both the shadcn Button Group pilot and the first Material 3 Button adapter.

No confirmed mechanism-layer defect was found during closeout, so no mechanism follow-on was split
from this lane.

## Shipped State

- `tools/parity-discovery/shadcn_parity_discovery.py` emits compact `agent_packet` data for single
  reports and suite summaries.
- Fret `bundle.schema2` text/paint evidence is surfaced through `tables.text_paint` and packet
  summaries.
- Packet facts distinguish direct `text_paint`, semantics-descendant-associated `text_paint`,
  semantic text descendants, and hotspot-sparse coverage notes.
- Button Group current evidence records 6 direct Fret text/paint facts, 21 associated text/paint
  facts, 68 semantic label facts, 160 bundle `tables.text_paint` entries, and 5532 text/paint rows.
- Material 3 Button current evidence records 1 bounded MUI contained-button DOM snapshot, 2 upstream
  DOM targets, 16 semantic label facts, 180 bundle `tables.text_paint` entries, and 3746 text/paint
  rows. The selected button nodes still have no direct or associated text/paint facts, and the
  packet now says that explicitly.

## Evidence

- `tools/parity-discovery/shadcn_parity_discovery.py`
- `tools/parity-discovery/fixtures/button_group_parts_v1.json`
- `tools/parity-discovery/fixtures/material3_button_adapter_v1.json`
- `docs/workstreams/component-parity-fact-harness-v1/artifacts/button_group_agent_packet_pilot_v1.json`
- `docs/workstreams/component-parity-fact-harness-v1/artifacts/material3_button_adapter_pilot_v1.json`
- `docs/workstreams/component-parity-fact-harness-v1/artifacts/upstream-dom/material3-button-mui-contained.json`
- `target/fret-diag-component-parity-button-group-text-paint-v1/sessions/1779671244627-41048`
- `target/fret-diag-component-parity-material3-button-live-v1/sessions/1779671892793-82708`

## Follow-ons

Start a new narrow workstream instead of reopening this lane for:

- automated upstream DOM capture for MUI/Material references,
- broader Material or shadcn component adapter coverage,
- promotion of stable packet rows into permanent component fixtures or diag scripts,
- any future confirmed mechanism defect discovered by packet evidence.

## Residual Risk

- The Material upstream DOM slice is a bounded static snapshot, not an automated browser-capture
  pipeline.
- `tables.text_paint` is hotspot-sparse by design; semantic label coverage is not equivalent to
  per-label paint geometry.
- Curated source prose remains useful context, but future broadening should prefer live upstream DOM
  targets where available.
