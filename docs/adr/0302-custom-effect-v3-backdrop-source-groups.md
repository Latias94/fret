---
title: "ADR 0302: Custom Effect V3 (Backdrop Source Groups for Liquid Glass)"
status: Draft
date: 2026-02-28
---

# ADR 0302: Custom Effect V3 (Backdrop Source Groups for Liquid Glass)

## Context

CustomV3 (ADR 0301) provides renderer-owned sources:

- `src` (current chain input),
- `src_raw` (chain root, when requested and feasible),
- optional `src_pyramid` derived from `src_raw` (bounded, budgeted, deterministically degraded).

This raises the ceiling for single-surface “liquid glass”, but it does not fully cover the real-world editor UI case:

- Many glass surfaces often coexist on screen (tool windows, panels, inspectors).
- Those surfaces frequently need to sample the **same underlying backdrop snapshot**, not a chain-local snapshot that
  changes as other surfaces are processed.
- If each surface independently captures/derives `src_raw` and `src_pyramid`, the work scales with surface count and
  the appearance can drift (e.g. refraction sampling a backdrop that already includes other glass).

The repo’s extensibility philosophy requires that such sharing be:

- **explicit** (no hidden implicit caches),
- deterministic under budgets and capability gates,
- diagnosable (requested vs applied vs degraded is visible).

## Goals

- Provide an explicit mechanism-level way for multiple CustomV3 surfaces to share:
  - a single renderer-owned **raw backdrop snapshot**, and
  - an optional bounded **pyramid** derived from that snapshot.
- Ensure sharing does not compromise determinism or budgeting.
- Keep the CustomV3 authoring model single-pass and bounded (no custom multi-pass graphs).

## Non-goals

- Cross-frame caches (temporal reuse) without explicit invalidation mechanisms.
- Automatic/heuristic sharing that may change outputs based on incidental ordering or budgets.
- Exposing arbitrary render targets or samplers in the portable ABI.

## Decision

Introduce an explicit **Backdrop Source Group** mechanism in the portable scene encoding.

Conceptually, a backdrop source group defines a region of the scene where all *backdrop-mode* CustomV3 effects can
sample a **shared group snapshot** rather than per-chain “raw” sources.

### Portable scene surface (proposed)

Add two new ops to `SceneOp`:

- `SceneOp::PushBackdropSourceGroupV1 { bounds, pyramid, quality }`
- `SceneOp::PopBackdropSourceGroup`

Where:

- `bounds: Rect` is a computation bound (not an implicit clip), consistent with other GPU-bounded ops (ADR 0117).
- `pyramid: Option<CustomEffectPyramidRequestV1>` declares the group-level pyramid request upper bounds.
  - If `None`, the group provides only a shared raw snapshot.
- `quality: EffectQuality` is a hint for budgeting/degradation (mirrors existing effect quality surfaces).

### Semantics

Within an active backdrop source group:

- For `EffectMode::Backdrop` chains that contain `EffectStep::CustomV3`:
  - `src_raw` is resolved to the group’s shared raw snapshot when:
    - the effect requests `want_raw == true` **or** requests a pyramid, and
    - the backend supports the group snapshot and can provide it safely under budgets.
  - `src_pyramid` is resolved to the group’s shared pyramid when:
    - the effect requests a pyramid, and
    - the group declared `pyramid != None`, and
    - the backend can provide it under budgets/capabilities.

Outside a backdrop source group, or when the backend degrades the group:

- CustomV3 falls back to the ADR 0301 semantics (chain-local `src_raw`/`src_pyramid` behavior and degradations).

### Determinism and degradation

Backends must be deterministic:

- Group snapshot/pyramid is either applied for the whole group or deterministically degraded with explicit reasons.
- If budgets are insufficient:
  - group pyramid deterministically degrades to `levels = 1` (aliasing to raw),
  - group snapshot may deterministically degrade to “no shared raw” (effects observe chain-local semantics).

### Budgeting rules (normative intent)

- Charge group snapshot allocation once per group.
- Charge group pyramid allocation/generation once per group (if requested and feasible).
- Sharing must reduce work relative to per-surface duplication; it must not introduce hidden per-surface costs.

### Diagnostics vocabulary (required)

Backends must surface requested vs applied vs degraded outcomes for groups and for CustomV3 sources, for example:

- `backdrop_source_groups_requested / applied`
- `backdrop_source_group_pyramid_requested / applied_levels_ge2 / degraded_to_one_*`
- `backdrop_source_group_raw_degraded_*`

Exact counter names are backend-specific, but they must be visible in perf/diagnostics snapshots.

## Consequences

Pros:

- Enables the “correct ceiling” liquid glass behavior: many surfaces share a stable backdrop snapshot.
- Predictable scaling (groups, not surfaces) and clearer budgeting.
- Determinism preserved because sharing is explicit in the scene encoding.

Cons:

- Adds a new mechanism-level surface that requires backend implementations and conformance.
- Requires careful interaction with existing stacks (clips/masks/composite groups) and multi-window.

## Tracking

- Workstream: `docs/workstreams/renderer-effects-semantics-and-extensibility-v1/custom-effect-v3/`
- Design notes: `docs/workstreams/renderer-effects-semantics-and-extensibility-v1/custom-effect-v3/m2-sharing-and-caching-design.md`

## References

- ADR 0301: Custom Effect V3 (Renderer-provided Sources: Raw + Optional Pyramid)
- ADR 0299: Custom Effect ABI (wgpu-only MVP)
- ADR 0300: Custom Effect V2 (Single User Image Input)
