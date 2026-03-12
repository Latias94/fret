---
title: Text Paint Surface v1 — TODO
status: active
date: 2026-02-17
---

# Text Paint Surface v1 — TODO Tracker

Status: Active (workstream tracker)

Workstream narrative: `docs/workstreams/text-paint-surface-v1/text-paint-surface-v1.md`
Milestone board: `docs/workstreams/text-paint-surface-v1/text-paint-surface-v1-milestones.md`

## Tracking format

- Status: `[ ]` open, `[~]` in progress, `[x]` done, `[!]` blocked
- ID: `TPS-{area}-{nnn}`

Leave 1–3 evidence anchors when completing an item (paths + key functions/tests), and prefer
renderer conformance tests for correctness-sensitive semantics.

## M0 — Contract lock-in (bounded + portable)

- [x] TPS-contract-010 Extend `SceneOp::Text` to accept `Paint` instead of solid `Color`.
  - Evidence: `crates/fret-core/src/scene/mod.rs`
  - Evidence: `crates/fret-core/src/scene/validate.rs`
  - Evidence: `crates/fret-core/src/scene/fingerprint.rs`
- [x] TPS-contract-020 Define paint coordinate semantics for text (origin + glyph local pos).
  - Evidence: `docs/adr/0279-text-paint-surface-v1.md`
  - Evidence: `crates/fret-render-wgpu/src/renderer/render_scene/encode/draw/text.rs` (`local_pos_px`)
- [x] TPS-adr-030 Add an ADR that locks semantics + degradation policy.
  - Evidence: `docs/adr/0279-text-paint-surface-v1.md`

## M1 — Renderer implementation (wgpu default)

- [x] TPS-render-100 Encode `Paint` for text draws (bounded, deterministic).
  - Evidence: `crates/fret-render-wgpu/src/renderer/render_scene/encode/draw/text.rs`
  - Evidence: `crates/fret-render-wgpu/src/renderer/render_scene/render.rs`
- [x] TPS-render-110 Implement gradient paint evaluation in the text shader/pipeline.
  - Evidence: `crates/fret-render-wgpu/src/renderer/pipelines/text.rs`
- [x] TPS-render-120 Ensure material paint is capability-gated and degrades deterministically.
  - v1: deterministically degrade `Paint::Material` → solid base color for text.
  - Evidence: `crates/fret-render-wgpu/src/renderer/render_scene/encode/draw/text.rs` (`Paint::Material` handling)

## M2 — Conformance (required)

- [x] TPS-test-190 Lock a deterministic font source for conformance gates (no system fonts).
  - Evidence: `crates/fret-render-wgpu/tests/text_font_source_determinism.rs`
- [x] TPS-test-200 Add GPU readback conformance for text paint:
  - linear gradient has expected left/right coverage on glyph shapes
  - stability across scale factors
  - uses a deterministic font source (avoid system-font flakiness)
  - Evidence: `crates/fret-render-wgpu/tests/text_paint_conformance.rs`

## M3 — Adoption (optional)

- [x] TPS-adopt-300 Wire one real consumer to use non-solid text paint:
  - pick a small demo surface (ui-gallery / editor diagnostics) to validate ergonomics.
  - Evidence: `apps/fret-ui-gallery/src/ui/previews/pages/editors/text/selection_perf.rs` (gradient text label).

## M4 — Text shadow (bounded) v1 (optional)

- [x] TPS-shadow-010 Add a bounded, portable text shadow surface (single layer, no blur).
  - ADR: `docs/adr/0283-text-shadow-surface-v1.md`
  - Evidence: `crates/fret-core/src/scene/mod.rs` (`SceneOp::Text.shadow`, `TextShadowV1`)
  - Evidence: `crates/fret-render-wgpu/src/renderer/render_scene/encode/draw/text.rs` (`encode_text_blob`, shadow prepass)
  - Evidence: `crates/fret-render-wgpu/tests/text_paint_conformance.rs` (`gpu_text_shadow_v1_renders_a_separate_layer`)
  - Gates:
    - `cargo nextest run -p fret-render-wgpu --test text_paint_conformance`
    - `cargo test -p fret-render-wgpu shaders_validate_for_webgpu`
