---
title: Component Parity Fact Harness v1 TODO
status: closed
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
- [x] CPF-065: Promote Fret diagnostics bundle text/paint evidence into a first-class
      `tables.text_paint` schema2 table so future component packets do not have to scrape
      per-snapshot debug arrays.
- [x] CPF-070: Add interaction and semantics facts to the packet model using Radix/Base UI/APG
      outcomes and existing diagnostics bundle semantics.
- [x] CPF-080: Add the first Material 3 adapter design slice that maps Material spec/MUI/Compose
      references into the same fact packet shape.
- [x] CPF-090: Promote agent packet summaries into the shadcn v2 suite report and coverage
      manifest workflow.
- [x] CPF-092: Compact bundle semantics and text/paint evidence in agent packets by stable fact
      signatures, preserving observed counts and evidence-path samples instead of duplicating every
      captured snapshot node.
- [x] CPF-094: Refresh the Button Group seed diagnostics evidence with current schema2 bundles so
      `tables.text_paint` rows enter the pilot packet.
- [x] CPF-096: Attach the Material 3 Button adapter to its first live UI Gallery diagnostics slice
      using stable gallery `test_id`s and explicit `live_fact_requirements`.
- [x] CPF-098: Harden `text_paint` packet association by reporting direct per-node hits,
      semantics-descendant ancestor associations, semantic text descendants, and explicit sparse
      hotspot coverage notes as separate facts.
- [x] CPF-099: Attach a bounded Material UI contained-button upstream DOM slice for enabled and
      disabled buttons to the Material 3 Button adapter.
- [x] CPF-100: Evaluate whether the final packets expose a confirmed mechanism-level defect. No
      mechanism defect was found in this slice, so no mechanism follow-on was created from this
      lane.
