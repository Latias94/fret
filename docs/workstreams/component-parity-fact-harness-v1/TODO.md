---
title: Component Parity Fact Harness v1 TODO
status: active
date: 2026-05-25
---

# TODO

- [x] CPF-010: Open the narrow `component-parity-fact-harness-v1` workstream instead of widening
      the existing shadcn v2 coverage sweep.
- [x] CPF-020: Add `agent_packet` generation to `tools/parity-discovery` reports without changing
      fixture predicate semantics.
- [x] CPF-030: Generate a Button Group pilot artifact from upstream DOM snapshots and the latest
      Fret UI Gallery seed diagnostics evidence.
- [x] CPF-040: Document the workstream gates, evidence, and handoff state.
- [x] CPF-050: Add a live source-fact extractor for shadcn registry DOM/CSS facts so rows like
      `items-stretch`, `w-fit`, `flex-1`, text baseline, and default control chrome stop depending
      on curated prose.
- [x] CPF-060: Add text and paint facts to the packet model: rendered text bounds, baseline/center
      alignment, border radii, border widths, icon size, and color/token slots.
- [ ] CPF-070: Add interaction and semantics facts to the packet model using Radix/Base UI/APG
      outcomes and existing diagnostics bundle semantics.
- [ ] CPF-080: Add the first Material 3 adapter design slice that maps Material spec/MUI/Compose
      references into the same fact packet shape.
- [x] CPF-090: Promote agent packet summaries into the shadcn v2 suite report and coverage
      manifest workflow.
- [ ] CPF-100: Split any confirmed mechanism-level defect found by component packets into
      `fret-mechanism-harness-v1` or a narrower mechanism follow-on.
