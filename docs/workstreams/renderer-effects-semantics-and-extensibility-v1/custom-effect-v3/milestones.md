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
- Plan dump + counters make it visible when `src_raw` had to alias `src` (targets/budget constraints).

Current status:

- Implemented `EffectStep::CustomV3` + validation/fingerprint and a wgpu pipeline/pass for CustomV3, including a
  bounded attempt to preserve a chain-root `src_raw` scratch copy under budgets.
- Conformance exists for `src_raw` vs `src` correctness in a blur→custom chain.
- Plan dumps include raw alias/distinct summaries per effect.
- Gaps: no explicit degradation counters yet beyond plan dumps.

## M1 — Bounded blur pyramid (optional)

Outcome:

- CustomV3 can request a bounded pyramid with explicit upper bounds.
- wgpu backend builds a deterministic mip chain under budgets.
- Plan dump + counters report requested/applied levels and degradation reasons.

Current status:

- Implemented pyramid generation in wgpu via a deterministic 2×2 box downsample chain into a renderer-owned mipped
  scratch texture.
- Conformance verifies that mip level 1 sampling differs from raw near an unaligned edge.
- Plan dumps report requested count and degraded-to-1 outcomes.

## M2 — Optional sharing/caching (deferred)

Outcome:

- A mechanism-level way to share pyramid/capture work across multiple glass surfaces exists,
  without hidden implicit caches.
