---
title: "paint eval spaces v1 — milestones"
status: draft
date: 2026-02-28
---

# Milestones

## M0 — Contract locked

- [ ] ADR 0306 is accepted and indexed.
- [ ] Workstream notes exist (`README.md`, `todo.md`, `milestones.md`).
- [ ] Evidence anchors for baseline gradient strokes exist (wgpu conformance test).

## M1 — Core contract implemented

- [ ] Add `PaintEvalSpaceV1` + binding type in `crates/fret-core`.
- [ ] Update scene ops to bind eval space per paint usage site (breaking change allowed).
- [ ] Update validation + fingerprint to include eval space.
- [ ] Add conformance tests for fingerprint/cache invalidation behavior.

## M2 — WGPU: viewport-space evaluation

- [ ] Quad/text/path pipelines support `ViewportPx` evaluation (or deterministic degradation).
- [ ] Add a conformance test that distinguishes Local vs Viewport evaluation.

## M3 — WGPU: stroke-space evaluation (StrokeS01)

- [ ] `SceneOp::StrokeRRect` supports `StrokeS01` via perimeter `s01`.
- [ ] Path stroke supports `StrokeS01` via per-vertex arclength attributes.
- [ ] Define and implement dash × StrokeS01 semantics (no gradient reset per dash).
- [ ] Add conformance tests for StrokeS01 correctness and stability across scale factors.

## M4 — Ecosystem adoption gates

- [ ] A small demo/diag script exercises:
  - [ ] a viewport shimmer,
  - [ ] a StrokeS01 wire gradient.
- [ ] Node graph skin layer can opt into StrokeS01 without backend-specific code.

