---
title: Custom Effect V3 (Milestones)
status: draft
date: 2026-02-28
scope: renderer, effects, extensibility, abi, budgeting
---

# Milestones

## M0 — Dual-source (raw + processed)

Outcome:

- `EffectStep::CustomV3` exists in `fret-core`.
- wgpu backend binds both `src` and `src_raw` deterministically.
- Conformance demonstrates that:
  - `src_raw` is the chain root (pre-steps),
  - `src` is the current chain input (post-previous steps),
  - scissor/mask semantics remain correct.

## M1 — Bounded blur pyramid (optional)

Outcome:

- CustomV3 can request a bounded pyramid with explicit upper bounds.
- wgpu backend builds a deterministic mip chain under budgets.
- Plan dump + counters report requested/applied levels and degradation reasons.

## M2 — Optional sharing/caching (deferred)

Outcome:

- A mechanism-level way to share pyramid/capture work across multiple glass surfaces exists,
  without hidden implicit caches.

