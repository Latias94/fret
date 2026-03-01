---
title: "paint eval spaces v1 — TODO"
status: draft
date: 2026-02-28
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

## Renderer (wgpu)

- [ ] Quad shader: introduce a single `paint_pos` selection based on eval space.
- [ ] StrokeRRect: compute `s01` and bind `paint_pos = vec2(s01, 0)`.
- [ ] Path pipeline:
  - [ ] extend `PathVertex` with an `s01` attribute for stroke-prepared paths,
  - [ ] normalize `s01` consistently across scale factors,
  - [ ] preserve dash × StrokeS01 semantics (avoid per-dash gradient reset).
- [ ] Text pipeline: decide whether StrokeS01 is invalid (likely) and ensure Local/Viewport are
  supported.

## Gates

- [ ] Add cache/fingerprint conformance tests for eval space changes.
- [ ] Add rendering conformance tests for:
  - [ ] `ViewportPx` vs `LocalPx` (should differ under transforms/panning),
  - [ ] `StrokeS01` gradient directionality (monotonic along stroke).
- [ ] Add a small diag script (optional) to toggle eval spaces and record bundles.
