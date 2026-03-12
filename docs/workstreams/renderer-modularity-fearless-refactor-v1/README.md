# Renderer Modularity (Fearless Refactor v1)

Status: Draft

Last updated: 2026-03-12

## Motivation

Fret's renderer stack is already on the right architectural path:

- `crates/fret-render-core` exists as a portable contract crate.
- `crates/fret-render` exists as a public facade.
- `crates/fret-render-wgpu` holds the default backend implementation.
- `crates/fret-render-wgpu` is protected by a strong regression net:
  - `cargo nextest run -p fret-render-wgpu`
  - `python3 tools/check_layering.py`

The current problem is not "the renderer is fundamentally wrong." The current problem is that the
public surface and the internal module boundaries are still wider than they need to be:

- `crates/fret-render` currently re-exports the entire wgpu backend surface wholesale.
- `crates/fret-render-wgpu` still exposes some types that behave more like implementation details
  than stable author-facing contracts.
- `Renderer` remains a large multi-domain owner spanning text, SVG, paths, materials, custom
  effects, intermediate budgeting, diagnostics, and GPU resource management.
- a few API seams still center the "editor-hosted" `WgpuContext` path more strongly than the
  "engine-hosted" topology described in `docs/architecture.md`.

This workstream exists to make renderer refactors boring, staged, and reversible:

- shrink the stable public surface to what we actually want to support,
- keep backend semantics stable while we modularize internals,
- preserve the host-provided GPU context topology as a first-class contract,
- and leave behind evidence/gates strong enough that we can refactor without fear.

## Goals

- Turn `crates/fret-render` into a curated facade instead of a wildcard backend dump.
- Keep `crates/fret-render-core` as the home for portable render-facing contract types.
- Make engine-hosted and editor-hosted GPU topologies equally first-class at the API level.
- Reduce `fret-render-wgpu` internal coupling so text, SVG, plan compilation, execution, and
  diagnostics can evolve independently.
- Preserve current render semantics while modularizing the implementation.

## Non-goals

- Rewriting the renderer from scratch.
- Changing shadcn/Radix/component policy behavior.
- Redesigning render-plan semantics in this workstream.
  - Existing semantic guardrails remain tracked by
    `docs/workstreams/renderer-render-plan-semantics-audit-v1/renderer-render-plan-semantics-audit-v1.md`.
- Replacing `wgpu` with another backend in v1.

## Scope

- Public render facade and contract boundaries:
  - `crates/fret-render`
  - `crates/fret-render-core`
- Default backend implementation:
  - `crates/fret-render-wgpu`
- High-churn consumers that currently depend on the facade:
  - `crates/fret-launch`
  - `ecosystem/fret`
  - demos / cookbook / stress apps under `apps/`

## Current Snapshot

As of 2026-03-12:

- `crates/fret-render/src/lib.rs` is effectively a wildcard re-export facade.
- `crates/fret-render-wgpu/src/lib.rs` re-exports a broad mix of:
  - stable-facing contracts,
  - backend bootstrap helpers,
  - diagnostics stores,
  - and types that may not need to stay public.
- `crates/fret-render-wgpu/src/renderer/mod.rs` still defines a large `Renderer` state owner.
- `crates/fret-render-wgpu/src/text/mod.rs` and `crates/fret-render-wgpu/src/renderer/shaders.rs`
  are the most obvious oversized internal modules.
- `Renderer::new(adapter, device)` and `render_scene(device, queue, ...)` already make
  host-provided GPU objects possible, but some convenience/diagnostics surfaces still privilege
  `WgpuContext`.
- The first code slice has landed:
  - `crates/fret-render` now uses an explicit facade export list instead of wildcard re-export.
  - `RendererCapabilities::from_adapter_device(...)` now exists and is used by first-party runner
    paths.
- The first internal text split has landed:
  - glyph atlas bookkeeping now lives under `crates/fret-render-wgpu/src/text/atlas.rs`
  - `text/mod.rs` no longer owns atlas/page/upload/eviction internals directly
- The second internal text split has landed:
  - text diagnostics/debug snapshot code now lives under
    `crates/fret-render-wgpu/src/text/diagnostics.rs`
  - `text/mod.rs` no longer owns atlas/debug/perf snapshot helpers directly
- The third internal text split has landed:
  - text quality state and gamma helpers now live under
    `crates/fret-render-wgpu/src/text/quality.rs`
  - `text/mod.rs` no longer owns text quality configuration/state internals directly
- The fourth internal text split has landed:
  - text tests now live under `crates/fret-render-wgpu/src/text/tests.rs`
  - `text/mod.rs` now keeps only `#[cfg(test)] mod tests;` as the test entrypoint
- The fifth internal text split has landed:
  - font catalog / fallback lifecycle helpers now live under
    `crates/fret-render-wgpu/src/text/fonts.rs`
  - `text/mod.rs` no longer owns font enumeration, locale updates, rescan flow, or font-family
    cache reset helpers directly
- The sixth internal text split has landed:
  - text blob access / release / eviction helpers now live under
    `crates/fret-render-wgpu/src/text/blobs.rs`
  - `text/mod.rs` no longer owns released-blob LRU maintenance and blob eviction helpers directly
- Slice 1 verification passed after the first facade/topology changes:
  - `cargo nextest run -p fret-render -p fret-render-wgpu`: 221/221 passed
  - `cargo check -p fret-launch -p fret-examples`: passed
  - `python3 tools/check_layering.py`: passed
- Internal text split verification remains green after the test-module extraction:
  - `cargo nextest run -p fret-render-wgpu`: 220/220 passed
  - `cargo check -p fret-launch -p fret-examples`: passed
  - `python3 tools/check_layering.py`: passed
- Internal text split verification remains green after the font/fallback extraction:
  - `cargo nextest run -p fret-render-wgpu`: 220/220 passed
  - `cargo check -p fret-launch -p fret-examples`: passed
  - `python3 tools/check_layering.py`: passed
- Internal text split verification remains green after the blob lifecycle extraction:
  - `cargo nextest run -p fret-render-wgpu`: 220/220 passed
  - `cargo check -p fret-launch -p fret-examples`: passed
  - `python3 tools/check_layering.py`: passed
- Baseline gates passed during the pre-workstream audit:
  - `cargo nextest run -p fret-render-wgpu`: 220/220 passed
  - `python3 tools/check_layering.py`: passed

## Evidence Anchors

- Architecture topology:
  - `docs/architecture.md`
- Current facade:
  - `crates/fret-render/src/lib.rs`
- Current backend facade and init path:
  - `crates/fret-render-wgpu/src/lib.rs`
  - `crates/fret-render-wgpu/src/capabilities.rs`
  - `crates/fret-render-wgpu/src/surface.rs`
- Current renderer owner and hot paths:
  - `crates/fret-render-wgpu/src/renderer/mod.rs`
  - `crates/fret-render-wgpu/src/renderer/resources.rs`
  - `crates/fret-render-wgpu/src/renderer/render_scene/render.rs`
  - `crates/fret-render-wgpu/src/renderer/render_scene/execute.rs`
  - `crates/fret-render-wgpu/src/text/mod.rs`
- Existing semantic guardrails:
  - `crates/fret-render-wgpu/src/renderer/render_plan.rs`
  - `docs/workstreams/renderer-render-plan-semantics-audit-v1/renderer-render-plan-semantics-audit-v1.md`

## Documents

- Design: `docs/workstreams/renderer-modularity-fearless-refactor-v1/DESIGN.md`
- TODO: `docs/workstreams/renderer-modularity-fearless-refactor-v1/TODO.md`
- Milestones: `docs/workstreams/renderer-modularity-fearless-refactor-v1/MILESTONES.md`
- Surface inventory: `docs/workstreams/renderer-modularity-fearless-refactor-v1/SURFACE_INVENTORY.md`

## Locked v1 Decisions

The following decisions are considered locked for the start of v1:

1. No new renderer crates in v1.
   - Work happens inside `crates/fret-render`, `crates/fret-render-core`, and
     `crates/fret-render-wgpu`.
2. `crates/fret-render` remains the default stable facade.
   - It stops using wildcard re-export, but it does not stop being the default entrypoint.
3. `crates/fret-render-core` remains value-only and portable.
   - It should not absorb backend bootstrap objects or `wgpu`-bound handles.
4. `WgpuContext` remains supported as a convenience path.
   - It is not the only first-class integration path.
5. Host-provided GPU topology closure is P0.
   - v1 must add capability/bootstrap helpers that work without forcing `WgpuContext`.
6. Render-plan semantics are treated as frozen inputs for this workstream.
   - Modularization work should not quietly redesign pass semantics.
7. The first high-value internal extraction target is `crates/fret-render-wgpu/src/text/mod.rs`.
   - `renderer/shaders.rs` is not the first breakup target.
8. Backend-only cache/registry-style exports are presumed shrink candidates until proven otherwise.

## Recommended v1 Approach

- Keep the refactor staged and behavior-preserving.
- Lock the facade surface before chasing internal cleanup.
- Make host-provided GPU topology closure a P0 seam before any deep internal extraction.
- Prefer extracting cohesive domains out of `Renderer` over inventing new abstraction layers
  prematurely.
