---
title: "paint eval spaces v1 — milestones"
status: draft
date: 2026-02-28
---

# Milestones

## M0 — Contract locked

- [x] ADR 0306 is accepted and indexed.
- [x] Workstream notes exist (`README.md`, `todo.md`, `milestones.md`).
- [x] Evidence anchors for baseline gradient strokes exist (wgpu conformance test).

## M1 — Core contract implemented

- [x] Add `PaintEvalSpaceV1` + binding type in `crates/fret-core`.
- [x] Update scene ops to bind eval space per paint usage site (breaking change allowed).
- [x] Update validation + fingerprint to include eval space.
- [ ] Add conformance tests for fingerprint/cache invalidation behavior.

## M2 — WGPU: viewport-space evaluation

- [x] Quad/text/path pipelines support `ViewportPx` evaluation (or deterministic degradation).
- [x] Add a conformance test that distinguishes Local vs Viewport evaluation.

## M3 — WGPU: stroke-space evaluation (StrokeS01)

- [x] `SceneOp::StrokeRRect` / quad borders support `StrokeS01` via perimeter `s01`.
- [x] Path stroke supports `StrokeS01` via per-vertex arclength (`lyon` advancement) normalized to `s01`.
- [ ] Define and implement dash × StrokeS01 semantics (no gradient reset per dash).
- [x] Add conformance tests for StrokeS01 correctness and stability across scale factors.

## M4 — Ecosystem adoption gates

- [ ] A small demo/diag script exercises:
  - [ ] a viewport shimmer,
  - [ ] a StrokeS01 wire gradient.
- [ ] Node graph skin layer can opt into StrokeS01 without backend-specific code.
