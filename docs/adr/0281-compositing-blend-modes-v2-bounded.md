---
title: Compositing Blend Modes v2 (Bounded)
status: Draft
date: 2026-02-16
---

# ADR 0281: Compositing Blend Modes v2 (Bounded)

## Context

ADR 0247 introduced compositing groups with a minimal v1 `BlendMode` set (`Over`, `Add`, `Multiply`,
`Screen`) for isolated, ordered compositing.

As the ecosystem grows, there is demand for additional blend modes that are still:

- portable to wasm/WebGPU and mobile GPUs,
- deterministic (no backend-specific “best effort” divergence),
- and bounded (no implicit extra passes or unbounded intermediate allocation).

Many “full CSS blend mode” lists are **not** bounded in WebGPU because they require sampling the
destination (backdrop) while writing to the same render target. Implementations typically need
extra intermediates and explicit capability gating.

## Decision

Expand the `BlendMode` enum with a **small, bounded v2 set** that can be implemented using a
single fixed-function blend state (no destination sampling):

- `BlendMode::Darken` (`min(dst, src)` for color channels)
- `BlendMode::Lighten` (`max(dst, src)` for color channels)
- `BlendMode::Subtract` (`dst - src`, clamped to `[0, 1]` for color channels)

These modes apply when compositing a compositing-group intermediate back onto its parent target.

## Semantics

### Scope

Blend modes are applied only at **compositing-group boundaries**:

- a group is rendered to an offscreen intermediate (as already required by isolated opacity and
  effect stacks),
- then the group result is composited back using the requested `BlendMode`.

The ordered `SceneOp` stream remains authoritative; no reordering is introduced.

### Color math (v2)

Let `dst.rgb` be the destination color in the parent target and `src.rgb` be the group result
color (premultiplied), after the group-level opacity multiplier is applied.

For v2 modes, the color channels are combined as:

- `Darken`: `out.rgb = min(dst.rgb, src.rgb)`
- `Lighten`: `out.rgb = max(dst.rgb, src.rgb)`
- `Subtract`: `out.rgb = clamp(dst.rgb - src.rgb, 0, 1)`

### Alpha semantics

For v2 modes, alpha uses the standard premultiplied alpha-over accumulation policy:

- `out.a = src.a + dst.a * (1 - src.a)`

This keeps group coverage predictable and avoids surprising alpha growth.

### Portability and boundedness

These modes are considered “bounded” because they can be expressed as a fixed-function blend state:

- no destination sampling is required,
- no additional passes are introduced beyond the existing group composite pass,
- and pipeline variant key space remains bounded by the enum size.

Backends that cannot implement a mode must degrade deterministically to `BlendMode::Over`.

## Consequences

- A small subset of additional blend policies becomes available to ecosystem authors without
  requiring shader plugins or heavyweight effect pipelines.
- The contract remains intentionally small; it does not attempt to mirror the entire CSS blend
  mode list.

## Evidence / implementation anchors

- Contract:
  - `crates/fret-core/src/scene/composite.rs` (`BlendMode` v2 variants)
- WGPU renderer:
  - `crates/fret-render-wgpu/src/renderer/pipelines/composite.rs` (`blend_state_for_mode`)
  - `crates/fret-render-wgpu/src/renderer/render_scene/render.rs` (pipeline selection via `pipeline_index`)
- Conformance:
  - `crates/fret-render-wgpu/tests/composite_group_conformance.rs` (`gpu_composite_group_blend_modes_v2_smoke_conformance`)
