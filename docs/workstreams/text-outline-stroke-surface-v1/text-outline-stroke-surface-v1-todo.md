---
title: Text Outline/Stroke Surface v1 — TODO
status: in_progress
date: 2026-02-18
---

# Text Outline/Stroke Surface v1 — TODO Tracker

Status: Active (workstream tracker)

Workstream narrative: `docs/workstreams/text-outline-stroke-surface-v1/text-outline-stroke-surface-v1.md`
Milestone board: `docs/workstreams/text-outline-stroke-surface-v1/text-outline-stroke-surface-v1-milestones.md`

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

- [x] TOS-contract-020 Decide the contract shape for `TextOutlineV1`:
  - Chosen: extend `SceneOp::Text` with `outline: Option<TextOutlineV1>`.
  - `TextOutlineV1` v1 vocabulary: `{ paint: Paint, width_px: Px }` (no join/cap/miter in v1).
  - `width_px` is logical px and is sanitized/clamped (`TextOutlineV1::MAX_WIDTH_PX`).
  - Evidence anchors:
    - `crates/fret-core/src/scene/mod.rs` (`TextOutlineV1`, `SceneOp::Text { outline }`)

- [x] TOS-degrade-030 Lock deterministic degradation rules:
  - Invalid outlines sanitize to `None` (non-finite / non-positive widths).
  - v1 supports outlines for mask and subpixel glyph runs; color/emoji deterministically degrade to
    fill-only.
  - Radius is quantized + bounded (small tap set) to stay WebGPU-uniformity-safe and avoid
    unbounded work.
  - Evidence anchors:
    - `crates/fret-core/src/scene/mod.rs` (`TextOutlineV1::sanitize`)
    - `crates/fret-render-wgpu/src/renderer/render_scene/encode/draw/text.rs` (packed params; mask+subpixel)
    - `crates/fret-render-wgpu/src/renderer/shaders.rs` (outline ring; bounded taps)

## M1 — Core contract plumbing

- [x] TOS-core-100 Add contract types + validation + fingerprinting in `fret-core`.
  - Evidence anchors:
    - `crates/fret-core/src/scene/mod.rs` (`TextOutlineV1`, `SceneOp::Text`)
    - `crates/fret-core/src/scene/validate.rs` (outline non-finite validation)
    - `crates/fret-core/src/scene/fingerprint.rs` (outline participates in fingerprint)

## M2 — Renderer implementation (wgpu)

- [x] TOS-wgpu-200 Land the chosen implementation strategy behind capabilities:
  - Chosen (v1): bounded morphology-based ring on mask and subpixel glyph atlases (no SDF/MSDF).
  - Keep WebGPU uniformity constraints explicit and gated.
  - Evidence anchors:
    - `crates/fret-render-wgpu/src/renderer/render_scene/encode/draw/text.rs`
    - `crates/fret-render-wgpu/src/renderer/pipelines/text.rs` (outline pipeline variant)
    - `crates/fret-render-wgpu/src/renderer/render_scene/render.rs` (pipeline dispatch)
    - `crates/fret-render-wgpu/src/renderer/shaders.rs` (`FRET_TEXT_OUTLINE_PRESENT`)

## M3 — Conformance + perf gates

- [x] TOS-test-300 Add a GPU readback conformance test for outlines.
  - Evidence anchors:
    - `crates/fret-render-wgpu/tests/text_outline_conformance.rs`
- [ ] TOS-perf-310 Add a small steady perf gate (only if evidence shows risk of a cliff).

## M4 — Adoption (optional)

- [x] TOS-adopt-400 Wire one real consumer (ui-gallery/editor) to exercise outlined text.
  - Evidence anchors:
    - `apps/fret-ui-gallery/src/ui/previews/pages/editors/text/outline_stroke.rs`
    - `apps/fret-ui-gallery/src/spec.rs` (`PAGE_TEXT_OUTLINE_STROKE`, `CMD_NAV_TEXT_OUTLINE_STROKE`)
    - `apps/fret-ui-gallery/src/ui/content.rs` (page routing)
