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
- Per-frame counters report requested vs applied raw/pyramid outcomes and deterministic degradation reasons.

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

## M2 — Sharing/caching (staged)

Outcome:

- A mechanism-level way to share capture/pyramid work across multiple glass surfaces exists, without hidden implicit
  caches, while remaining deterministic under budgets.

Current status:

- M2.0: implemented chain-local pyramid reuse (same frame, same `src_raw`, deterministic).
- M2.1: drafted contract for scene-level sharing (ADR 0302: backdrop source groups).
- M2.2: implemented group-level snapshot + shared `src_raw` (and group-bounded pyramid choice) for wgpu, with
  conformance coverage.
- M2.3: added group-level degradation counters (requested/applied/degraded) and surfaced them in per-frame perf
  snapshots.
- M2.4: implemented deterministic work bounding via ROI scissor for pyramid generation, including a shared ROI when
  a backdrop source group requests a pyramid.

## P3 — Authoring demo (apps only)

Outcome:

- A minimal “liquid glass v3” authoring-oriented demo exists for component authors.
- Demonstrates sampling:
  - `src` (processed chain input),
  - `src_raw` (crisp refraction),
  - `src_pyramid` (multi-scale sampling from raw).
- Demonstrates using a backdrop source group to share `src_raw`/pyramid across multiple glass surfaces.

Current status:

- Implemented `custom_effect_v3_web_demo` (WASM/WebGPU) and a minimal diag script for bundle/screenshot capture.
