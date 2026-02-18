---
title: Text Outline/Stroke Surface v1 — TODO
status: draft
date: 2026-02-18
---

# Text Outline/Stroke Surface v1 — TODO Tracker

Status: Draft (workstream tracker)

Workstream narrative: `docs/workstreams/text-outline-stroke-surface-v1.md`  
Milestone board: `docs/workstreams/text-outline-stroke-surface-v1-milestones.md`

## Tracking format

- Status: `[ ]` open, `[~]` in progress, `[x]` done, `[!]` blocked
- ID: `TOS-{area}-{nnn}`

Leave 1–3 evidence anchors when completing an item (paths + key functions/tests).

## M0 — Design lock (bounded + portable)

- [ ] TOS-audit-010 Audit the current text pipeline and atlas format:
  - Is the glyph atlas a coverage mask or distance field?
  - Are derivatives (`fwidth`) used in text shaders today?
  - What caches exist for glyph geometry/metrics?
  - Evidence anchors:
    - `crates/fret-render-wgpu/src/renderer/render_scene/encode/draw/text.rs`
    - `crates/fret-render-wgpu/src/renderer/pipelines/text.rs`
    - `crates/fret-render-wgpu/src/renderer/shaders.rs` (text shaders)

- [ ] TOS-contract-020 Decide the contract shape for `TextOutlineV1`:
  - Extend `SceneOp::Text` vs introduce a dedicated `SceneOp::TextOutline`.
  - Decide whether to reuse `StrokeStyleV2` vocabulary or introduce a smaller outline-only struct.
  - Define `width_px` coordinate semantics and sanitization.

- [ ] TOS-degrade-030 Lock deterministic degradation rules:
  - capability gated (backend/feature unsupported),
  - budget gated (intermediate pressure),
  - and “never unbounded work” constraints.

## M1 — Core contract plumbing

- [ ] TOS-core-100 Add contract types + validation + fingerprinting in `fret-core`.
  - Evidence anchors:
    - `crates/fret-core/src/scene/mod.rs`
    - `crates/fret-core/src/scene/validate.rs`
    - `crates/fret-core/src/scene/fingerprint.rs`

## M2 — Renderer implementation (wgpu)

- [ ] TOS-wgpu-200 Land the chosen implementation strategy behind capabilities:
  - Vector outline path (path pipeline) **or** SDF/MSDF atlas evaluation.
  - Keep WebGPU uniformity constraints explicit and gated.
  - Evidence anchors:
    - `crates/fret-render-wgpu/src/renderer/render_scene/encode/draw/text.rs`
    - `crates/fret-render-wgpu/src/renderer/shaders.rs`

## M3 — Conformance + perf gates

- [ ] TOS-test-300 Add a GPU readback conformance test for outlines.
- [ ] TOS-perf-310 Add a small steady perf gate (only if evidence shows risk of a cliff).

## M4 — Adoption (optional)

- [ ] TOS-adopt-400 Wire one real consumer (ui-gallery/editor) to exercise outlined text.
