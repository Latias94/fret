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

Status: pending.

Exit criteria:

- shadcn source facts can be extracted or measured mechanically from upstream DOM/CSS/golden
  artifacts.
- Curated prose remains useful context but is not the only source for layout/chrome truth.
- At least one prior Button Group curated fact becomes a live measured predicate.

## M3 - Material Adapter

Status: pending.

Exit criteria:

- A Material 3 component slice maps spec, MUI, and Compose facts into the same report/agent packet
  shape.
- The adapter keeps Material policy in the ecosystem layer and promotes only reusable mechanism
  gaps inward.
