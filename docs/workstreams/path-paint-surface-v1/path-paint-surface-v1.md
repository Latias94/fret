---
title: Path Paint Surface v1 — Workstream
status: completed
date: 2026-02-16
scope: fret-core SceneOp::Path, renderer path pipeline, portability + conformance
---

# Path Paint Surface v1 — Workstream

This workstream upgrades `SceneOp::Path` from **solid-only** rendering to a bounded, portable
**`Paint` surface** (solid + gradients + material, with deterministic degradations).

## Why this exists

Today, `SceneOp::Path` is limited to:

- prepared geometry (`PathId`)
- `origin`
- a solid `Color`

This forces common “icon fill gradient” / “chart stroke gradient” / “materialized vector shapes”
into approximation patterns (quad overlays, pre-rasterization, many ops), which are harder to batch
and harder to keep deterministic across wasm/mobile backends.

## Goals (v1)

1. Make `SceneOp::Path` accept a `Paint` value (same contract as `SceneOp::Quad` / `StrokeRRect`).
2. Define paint coordinate semantics for paths (stable and deterministic).
3. Keep the surface bounded and portable:
   - capability-gated behavior is explicit
   - degradations are deterministic (no hidden backend heuristics)
4. Leave at least one hard regression gate:
   - GPU readback conformance for path gradient paint

## Non-goals (v1)

- Full CSS parity (all blend modes, all tile modes, full `clip-path` masking semantics, etc.).
- “Constant pixel” stroke width under non-uniform transforms (explicit follow-up item).
- Making `fret-ui` a component library: this is a renderer/contract mechanism surface only.

## Contract + semantics

Normative contract: ADR 0278 (`docs/adr/0278-path-paint-surface-v1.md`).

Key semantics to lock:

- `Paint` is evaluated in **path local space**:
- `Paint` is evaluated in **logical scene space** (consistent with quad `local_pos` semantics):
  - `local_pos = origin + prepared_path_vertex_pos`
  - gradients/material params use the same coordinate space as other `Paint` surfaces (pre-transform)
- clip/mask/effect stacks operate in pixel space as today
- backends must degrade deterministically when a `Paint` variant is unsupported

## Tracking

Detailed TODOs: `docs/workstreams/path-paint-surface-v1/path-paint-surface-v1-todo.md`
Milestones: `docs/workstreams/path-paint-surface-v1/path-paint-surface-v1-milestones.md`
