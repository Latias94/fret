---
title: Renderer Paint GPU Storage Unification v1
status: active
date: 2026-02-16
scope: fret-render-wgpu internal refactor (no contract changes)
---

# Renderer Paint GPU Storage Unification v1

This workstream consolidates the renderer-internal GPU storage plumbing used for `Paint` data
uploads (currently duplicated across the path and text pipelines).

## Why this exists

Today, `fret-render-wgpu` maintains **two parallel implementations** for:

- allocating per-frame storage buffers for `PaintGpu`,
- creating their bind groups,
- resizing them when capacity grows,
- and selecting the per-frame buffer/bind-group pair during rendering.

The duplication increases refactor risk (easy to fix one path but not the other), and makes future
paint expansions (e.g. more primitives consuming paints, or more paint variants) harder to land
fearlessly.

This workstream keeps the public scene contract stable while making the implementation easier to
evolve.

## Non-goals (v1)

- No changes to `fret-core` contracts (`SceneOp`, `Paint`, etc.).
- No shader semantic changes.
- No changes to bind group indices visible to WGSL (`@group` / `@binding` stay stable).

## Scope (v1)

- Introduce a small, reusable “storage ring buffer” utility for `wgpu::Buffer + wgpu::BindGroup`
  pairs.
- Use it to back:
  - path paint storage (`@group(1) @binding(0)` in the path pipeline)
  - text paint storage (`@group(2) @binding(0)` in text pipelines)
- Keep behavior identical:
  - capacity growth policy remains `next_power_of_two` with a bounded multiplicative bump,
  - per-frame rotation remains `FRAMES_IN_FLIGHT`-based,
  - perf counters remain stable (bytes uploaded and draw counts).

## Always-run gates

- `python3 tools/check_layering.py`
- `cargo test -p fret-render-wgpu shaders_validate_for_webgpu`
- `cargo test -p fret-render-wgpu --test text_paint_conformance`

## Tracking

- TODOs: `docs/workstreams/renderer-paint-gpu-storage-unification-v1-todo.md`
- Milestones: `docs/workstreams/renderer-paint-gpu-storage-unification-v1-milestones.md`

