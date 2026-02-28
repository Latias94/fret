---
title: Custom Effect V3 — M2 Sharing/Caching (Design)
status: draft
date: 2026-02-28
scope: renderer, effects, extensibility, budgeting, determinism
---

# Custom Effect V3 — M2 Sharing/Caching (Design)

CustomV3 M0/M1 raised the ceiling by adding renderer-provided sources (`src_raw` + optional `src_pyramid`).
The remaining “liquid glass ceiling” work is **not** just better shaders — it is **source sharing**:

- Multiple glass surfaces often want the *same* raw backdrop snapshot.
- Multiple surfaces may also want the *same* pyramid derived from that snapshot.
- Without sharing, the backend can end up doing redundant capture + downsample work, and budget pressure becomes
  “per-surface”, which is frequently the wrong scaling behavior for editor-grade UIs.

This document proposes an M2 shape that keeps the repo’s non-negotiables:

- **No hidden implicit caches** that change output in surprising ways.
- Deterministic behavior under budgets and capability gates.
- Mechanism-level surfaces in `crates/`, authoring/policy in `apps/` / `ecosystem/`.

## Terms

- **Backdrop snapshot**: a read-only texture capturing the backdrop before applying a set of glass surfaces.
- **Glass group**: an explicit mechanism-level grouping that defines which surfaces share the same snapshot/pyramid.
- **Chain-local**: sharing that only applies within a single effect chain compilation/execution.

## Goal

Provide a mechanism-level way to share backdrop preparation work across multiple surfaces, while preserving:

- deterministic outputs (no “sometimes shared” based on incidental ordering),
- explicit budgeting (sharing reduces cost, it must not create negative/hidden costs),
- clear diagnostics (requested vs applied vs shared vs degraded).

## Non-goals

- A fully general multi-pass custom effect graph ABI (still bounded single-pass authoring).
- Cross-frame temporal caches (would need an explicit invalidation mechanism and would complicate determinism).

## Current state (M0/M2.2)

- `src_raw` may be distinct (chain snapshot) or deterministically aliased to `src`.
- When requested, `src_pyramid` is generated at record-time into a renderer-owned scratch (mipped) texture derived
  from `src_raw`, with deterministic degradation to `levels = 1` under budget pressure.
- M2.0 chain-local pyramid reuse exists (same frame, same `src_raw`, deterministic).
- M2.1/ M2.2 backdrop source groups are drafted (ADR 0302) and implemented for wgpu to share a stable group snapshot.

## Design space

### Option A — Chain-local sharing only (no core changes)

Share within a single chain execution:

- If multiple CustomV3 steps in the same chain request a pyramid derived from the same `src_raw`, build it once.

Pros:

- No `fret-core` changes.
- Low risk, fully reversible.

Cons:

- Does not solve the “many glass surfaces” case (each surface is typically its own chain).

### Option B — Explicit “glass group” snapshot (mechanism-level, contract-backed)

Introduce a **scene-level grouping primitive** that makes sharing explicit and deterministic:

- The renderer captures a backdrop snapshot **once per group** (before applying any glass surfaces in that group).
- All CustomV3 steps inside the group sample `src_raw` (and `src_pyramid`) from that group snapshot, not from the
  progressively-updated `srcdst`.

This is the closest match to “correct” liquid-glass behavior in practice: surfaces see the same background “behind
the group”, and the group’s ordering only affects compositing, not what the backdrop samples contain.

Pros:

- Scales well: `O(groups)` snapshot cost instead of `O(surfaces)`.
- Sharing is explicit, not incidental.
- Determinism is easier to specify (group boundaries are in the scene graph).

Cons:

- Requires a new mechanism-level surface (ADR + `fret-core` contract).
- Requires careful interaction with blend/composite groups, clip stacks, and multi-window.

### Option C — Heuristic backend sharing (no contract changes)

Backend automatically detects identical `src_raw` requests and shares opportunistically.

Pros:

- No `fret-core` changes.

Cons:

- Easy to violate “no hidden implicit caches”.
- Hard to guarantee determinism when budgets are tight (sharing changes which surfaces get the pyramid).

## Recommendation

Pursue **Option B** as the long-term “correct ceiling” for liquid glass, but land it in stages:

1) **M2.0 (now):** land Option A’s **chain-local pyramid reuse** as a small, reversible optimization with explicit
   per-frame counters.
2) **M2.1 (design+ADR):** define a contract-backed **glass group** primitive (Option B), including budgeting and
   diagnostics vocabulary, without committing to an implementation.
   - Tracking ADR: `docs/adr/0305-custom-effect-v3-backdrop-source-groups.md`
3) **M2.2 (implementation):** implement group snapshot + shared pyramid in the wgpu backend under budgets, with
   conformance + `fretboard diag` evidence.

Status (this worktree):

- M2.0: implemented and instrumented (frame-local cache hit/miss counters).
- M2.1: ADR 0302 exists (Draft).
- M2.2: implemented for wgpu with conformance (`effect_custom_v3_conformance` group snapshot test).

## Budgeting rules (M2.0)

- Pyramid generation reuse is allowed **only** when:
  - `src_raw` is the same plan target,
  - `src_raw` has not been written since the last pyramid build in the same frame,
  - `(format, size, levels)` match.
- Reuse is a pure optimization: it must not change outputs.
- Reuse must be observable (per-frame counters).

## Budgeting rules (Option B, draft)

If we add a contract-backed group snapshot, budgeting must be explicit:

- Charge snapshot allocation once per group.
- Charge pyramid allocation/generation once per group (if requested by any surface in the group).
- Under budget pressure, degrade deterministically:
  - snapshot may alias to the live backdrop (if safe) or disable group sharing (tracked),
  - pyramid may degrade to `levels = 1` (tracked),
  - the group must never silently fall back to “per-surface snapshots” without counters.

## Diagnostics vocabulary (proposed)

For M2.0:

- `custom_effect_v3_pyramid_cache_hits`
- `custom_effect_v3_pyramid_cache_misses`

For Option B (future):

- `glass_group_snapshot_requested / applied / degraded_*`
- `glass_group_pyramid_requested / applied_levels_ge2 / degraded_to_one_*`
- `glass_group_shared_surfaces` (count of surfaces that benefited from sharing)

## Evidence anchors

- CustomV3 contract: `docs/adr/0304-custom-effect-v3-renderer-provided-sources.md`
- V3 workstream index: `docs/workstreams/renderer-effects-semantics-and-extensibility-v1/custom-effect-v3/README.md`
