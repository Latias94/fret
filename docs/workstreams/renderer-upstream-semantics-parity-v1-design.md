# Renderer Upstream Semantics Parity v1 — Design

Status: Draft (workstream notes; ADRs remain the source of truth)

Tracking files:

- `docs/workstreams/renderer-upstream-semantics-parity-v1-design.md`
- `docs/workstreams/renderer-upstream-semantics-parity-v1-todo.md`
- `docs/workstreams/renderer-upstream-semantics-parity-v1-milestones.md`

## Goal

Use upstream implementations under `repo-ref/` (Zed/GPUI, shadcn/ui, Radix primitives, MUI Base UI,
Compose) as **reference points** to:

- detect semantic gaps early (especially around clipping/masking, scissoring, and intermediate reuse),
- inform safe internal refactors (compile/plan/execute) without changing Fret’s public scene contract,
- and guide small, ADR-backed contract additions when parity/portability requires it.

This workstream is explicitly *evidence-driven*: every upstream claim must map to a local evidence
anchor and (when applicable) a conformance or targeted unit test.

## Scope

In scope:

- Render-plan semantics comparisons:
  - scissor/clip/mask composition rules,
  - render-space vs local-space mapping,
  - intermediate target allocation/reuse and deterministic degradation patterns.
- Renderer internal structure comparisons:
  - pass recording isolation,
  - pipeline/material caching strategies that keep determinism (no hidden “best effort”).
- “Editor workload” affordances:
  - multi-viewport and multi-window implications (batching, readbacks, and stability).

Out of scope:

- Copying component/policy behavior from Radix/shadcn into `crates/` (policy stays in `ecosystem/`).
- Changing `fret-core::SceneOp` ordering semantics (order remains authoritative).

## Sources of truth

Primary (renderer / GPU-first references):

- `repo-ref/zed` (GPUI-inspired renderer organization; editor workloads)
- `repo-ref/gpui-component` (GPUI component patterns where relevant to rendering semantics)

Secondary (interaction policy references; used only to validate “policy vs mechanism” boundaries):

- `repo-ref/primitives` (Radix primitives)
- `repo-ref/ui` (shadcn/ui)
- `repo-ref/base-ui` (MUI Base UI)
- `repo-ref/compose-multiplatform-core` (Compose runtime + rendering assumptions)
- `repo-ref/material-ui` (Material system behavior expectations)

## Method

1) Pick one “semantic seam” at a time and write a short parity note:

- upstream behavior summary (with local file anchors),
- Fret current behavior (with local file anchors),
- whether the difference is:
  - a deliberate contract choice (document in ADR/workstream), or
  - an implementation gap (add TODO + gate).

2) For implementation gaps, prefer the smallest possible guardrail:

- `RenderPlan::debug_validate()` invariant,
- a targeted unit test in `crates/fret-render-wgpu/src/renderer/render_plan/tests.rs`,
- or a minimal GPU readback conformance test under `crates/fret-render-wgpu/tests/`.

3) Only after the guardrail exists, refactor the implementation.

## Candidate seams (starting set)

- **Scissor coordinate spaces**
  - Absolute viewport-space vs dst-local scissors; mapping rules across downsample/upsample chains.
- **Clip/mask composition**
  - Clip capture semantics (push-time capture vs dynamic), mask viewport mapping, cache keys.
- **Intermediate target reuse**
  - Allocation strategy, lifetime model, eviction policy, and deterministic degradation under pressure.
- **Backdrop-style effects**
  - Strict bounds/scissoring, multi-pass correctness under render-space offsets.
- **Blend modes and isolation**
  - How bounded blend/opacity groups are represented and degraded.

## Related workstreams / ADRs

- Workstreams:
  - `docs/workstreams/renderer-vnext-fearless-refactor-v1.md`
  - `docs/workstreams/renderer-render-plan-semantics-audit-v1.md`
- ADRs (starting points):
  - ADR 0116 (RenderPlan/postprocessing substrate)
  - ADR 0275 (RenderSpace + scissor-sized intermediates)
  - ADR 0273 (clip-path + mask-image sources)
  - ADR 0281 (bounded blend modes v2)

