---
title: Text Outline/Stroke Surface v1 — TODO
status: in_progress
date: 2026-02-18
---

# Text Outline/Stroke Surface v1 — TODO Tracker

Status: Active (workstream tracker)

Workstream narrative: `docs/workstreams/text-outline-stroke-surface-v1.md`  
Milestone board: `docs/workstreams/text-outline-stroke-surface-v1-milestones.md`

## Tracking format

- Status: `[ ]` open, `[~]` in progress, `[x]` done, `[!]` blocked
- ID: `TOS-{area}-{nnn}`

Leave 1–3 evidence anchors when completing an item (paths + key functions/tests).

## M0 — Design lock (bounded + portable)

- [x] TOS-audit-010 Audit the current text pipeline and atlas format:
  - Is the glyph atlas a coverage mask or distance field?
  - Are derivatives (`fwidth`) used in text shaders today?
  - What caches exist for glyph geometry/metrics?
  - Evidence anchors:
    - `crates/fret-render-wgpu/src/renderer/render_scene/encode/draw/text.rs`
    - `crates/fret-render-wgpu/src/renderer/pipelines/text.rs`
    - `crates/fret-render-wgpu/src/renderer/shaders.rs` (text shaders)
    - `crates/fret-render-wgpu/src/text/mod.rs` (`TextSystem::new`, atlas formats)
  - Findings (2026-02-18):
    - The mask glyph atlas is `R8Unorm` (coverage mask), not an SDF/MSDF atlas.
      - `TextSystem::new` creates:
        - mask atlas: `R8Unorm`
        - color atlas: `Rgba8UnormSrgb`
        - subpixel atlas: `Rgba8Unorm`
    - Text shaders sample `glyph_atlas` via `textureSample(...)` and do not rely on derivatives for
      coverage (no SDF edge evaluation in the text path today).
    - Implication: a “GPU SDF stroke” strategy would require an atlas format change and a new
      rasterization path; treat as v2+ unless evidence demands it.

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
