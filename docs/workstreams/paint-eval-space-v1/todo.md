---
title: "paint eval spaces v1 — TODO"
status: draft
date: 2026-03-01
---

# TODO

## Contract + core

- [x] Define `PaintEvalSpaceV1` in `crates/fret-core`.
- [x] Define a binding value type (e.g. `PaintBindingV1 { paint, eval_space }`).
- [x] Thread it through the scene op surfaces that carry paint:
  - [x] quads (fill + border),
  - [x] rrect strokes,
  - [x] paths,
  - [x] text.
- [x] Update `Scene::validate()` rules for unsupported combinations (deterministic degradation vs
  hard reject).
- [x] Update `Scene::fingerprint()` to include evaluation space.
- [x] Sweep in-tree callsites (tests/demos) for missing `.into()` conversions after the binding
  migration.
  - Includes updates in `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout/support/scene.rs` and
    related layout golden tests to read `PaintBindingV1.paint` when comparing colors/alpha.

## Renderer (wgpu)

- [x] Quad shader: introduce a single `paint_pos` selection based on eval space.
- [x] StrokeRRect / quad border: compute `s01` and bind `paint_pos = vec2(s01, 0)`.
- [ ] Path pipeline:
  - [x] derive per-vertex arclength from lyon `StrokeVertex::advancement()` and normalize to `s01`,
  - [x] bind `local_pos_px = vec2(s01, 0)` when `eval_space == StrokeS01` (no vertex layout change),
  - [ ] preserve dash × StrokeS01 semantics (avoid per-dash gradient reset; current behavior
    deterministically degrades StrokeS01 to LocalPx when dash is applied).
- [ ] Text pipeline: decide whether StrokeS01 is invalid (likely) and ensure Local/Viewport are
  supported.

## Gates

- [ ] Add cache/fingerprint conformance tests for eval space changes.
- [ ] Add rendering conformance tests for:
  - [x] `ViewportPx` vs `LocalPx` (should differ under transforms/panning),
  - [x] `StrokeS01` gradient directionality (monotonic along stroke).
- [ ] Add a small diag script (optional) to toggle eval spaces and record bundles.
