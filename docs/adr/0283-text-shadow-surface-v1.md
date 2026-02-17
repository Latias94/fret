---
title: SceneOp::Text Shadow Surface v1
status: Draft
date: 2026-02-17
---

# ADR 0283: SceneOp::Text Shadow Surface v1

## Context

Many UI surfaces want a simple “drop shadow” behind text for legibility (over images, charts, dense
diagnostic overlays, etc.). Without a contract surface, authors must approximate via:

- emitting two `SceneOp::Text` ops manually (shadow + main), or
- pre-rasterizing text into an image and drawing an `Image`/`ViewportSurface`.

These approaches are workable, but they:

- duplicate ordering logic across call sites (easy to get wrong),
- make portability and determinism harder to enforce across wasm/mobile,
- complicate conformance (we want one canonical behavior to lock and gate).

At the same time, a fully general text shadow surface (multi-layer, blur, spread, blend modes) has
real cost/complexity and is not yet justified as a core contract surface.

## Decision

Add a **bounded, portable** text shadow surface to `SceneOp::Text`:

- Extend `SceneOp::Text` with `shadow: Option<TextShadowV1>`.
- Define `TextShadowV1` as:
  - `offset: Point` (baseline-origin offset in logical pixels), and
  - `color: Color` (solid premul-ready color).

This is intentionally minimal:

- **single layer**
- **no blur**
- no additional blend modes beyond the existing text draw blend

Higher-entropy shadow recipes remain policy in ecosystem crates or future contract versions.

## Semantics (v1)

- When `shadow` is present, the renderer draws an additional text layer **behind** the main text:
  - `shadow_origin = origin + shadow.offset`
  - shadow color is applied as a **solid paint** multiplied by glyph coverage.
- `shadow` does not affect text decorations (underline/strikethrough) in v1.
- Non-finite shadow data is treated as “no shadow” (deterministic skip).

## Portability and performance notes

- This surface does not require intermediates, masks, or extra passes.
- Cost is proportional to an extra text draw (additional vertices + draw group flushes).
- Works uniformly for native and wasm/WebGPU backends (no uniformity hazards; no derivatives).

## Consequences

- Text authors get a canonical, portable shadow behavior without emitting multiple ops manually.
- The contract remains bounded; blur/multi-layer shadows remain deferred.
- Renderer ordering is more testable: “shadow behind text” can be locked by conformance.

## Evidence / implementation anchors

- Contract:
  - `crates/fret-core/src/scene/mod.rs` (`SceneOp::Text.shadow`, `TextShadowV1`)
- Validation/fingerprint:
  - `crates/fret-core/src/scene/validate.rs`
  - `crates/fret-core/src/scene/fingerprint.rs`
- Renderer (wgpu default):
  - `crates/fret-render-wgpu/src/renderer/render_scene/encode/draw/text.rs` (`encode_text_blob`, shadow prepass)
- Conformance:
  - `crates/fret-render-wgpu/tests/text_paint_conformance.rs` (`gpu_text_shadow_v1_renders_a_separate_layer`)
