---
title: Custom Effect V2 (Milestones)
status: draft
date: 2026-02-26
scope: renderer, effects, extensibility, abi
---

# Milestones

## M0 — Decision locked

Exit criteria:

- One v2 binding shape is chosen (with rationale) and written down.
- Capability discovery shape is specified (what does the app learn at runtime?).

## M1 — Core surface + backend skeleton

Exit criteria:

- `fret-core` has versioned `CustomV2` surfaces (types + validation + fingerprint mixing).
- `fret-render-wgpu` has a registry skeleton + cache key inclusion and can compile an identity CustomV2.

## M2 — Extra input works (the “ceiling bump”)

Exit criteria:

- The chosen extra input is usable from WGSL and is capability-gated.
- Conformance tests cover:
  - determinism for fixed params + inputs,
  - scissor/mask correctness,
  - degradation under budgets.

## M3 — Example recipes (ecosystem)

Exit criteria:

- At least one “high-end” recipe is shipped as ecosystem policy using CustomV2 (not a renderer fork).
- A demo page exists (gallery or `fret-examples`) and has a scripted diagnostics bundle for regressions.

