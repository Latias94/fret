---
title: Sweep Gradient Paint v1
status: Draft
date: 2026-02-16
---

# ADR 0280: Sweep Gradient Paint v1

## Context

Fret’s `Paint` surface currently supports:

- `Paint::Solid`
- `Paint::{LinearGradient, RadialGradient}`
- `Paint::Material { id, params }` (capability-gated + budgeted)

Many UI designs require a **sweep (conic) gradient** with multiple stops (e.g. hue rings, dials,
radar-like indicators, progress arcs with multi-stop color policies). Today this can be approximated
via materials, but that:

- is policy-heavy and not stop-based,
- is harder to share consistently across backends,
- and does not provide a stable multi-stop semantic surface for ecosystem authors.

We want a bounded, portable sweep-gradient contract that composes with existing `SceneOp::Quad`,
`SceneOp::Path`, and `SceneOp::Text` paint plumbing without introducing additional passes.

## Decision

Add a new bounded paint variant:

- `Paint::SweepGradient(SweepGradient)`

with a portable, stop-based semantic definition and deterministic degradations via `Paint::sanitize`.

## Semantics (v1)

### Coordinate space

`SweepGradient` is evaluated in the same local paint evaluation space as other paints:

- the input position is `local_pos` in the primitive’s local pixel coordinate space.

Angles are derived from:

- `v = local_pos - center`
- `angle = atan2(v.y, v.x)`

The coordinate system is Fret’s pixel space (`+x` right, `+y` down). This means:

- `angle = 0` points to `+x` (right),
- angles increase toward `+y` (down), as implied by `atan2(y, x)` in screen coordinates.

### Angles and normalization

`SweepGradient` uses turns as its public unit:

- `start_angle_turns`: start angle in turns (`1.0` = full rotation)
- `end_angle_turns`: end angle in turns

`Paint::sanitize` normalizes the angles into a stable representation:

- `start_angle_turns` is reduced into `[0, 1)`,
- the span is reduced into `(0, 1]`,
- degenerate spans (≈ 0) degrade deterministically to `Paint::Solid` using a representative stop.

### Stops

Stops reuse the existing `GradientStop` + `MAX_STOPS` contract:

- offsets are clamped to `[0, 1]` and sorted (stable, fixed-array sort),
- the shader samples linearly between stop colors, consistent with other gradient paints.

### TileMode and ColorSpace (v1)

To keep the contract portable and the shader surface bounded:

- `TileMode::{Repeat, Mirror}` degrade to `TileMode::Clamp` via `Paint::sanitize`.
- `ColorSpace::Oklab` degrades to `ColorSpace::Srgb` via `Paint::sanitize`.

Follow-up workstreams may expand these surfaces if evidence warrants (see `REN-VNEXT-sem-070`,
`REN-VNEXT-sem-080`).

## Consequences

- Ecosystem authors can express multi-stop conic gradients without relying on material policies.
- The default renderer can implement sweep gradients in the existing single-pass quad/path/text
  pipelines, preserving batching and wasm/mobile portability.
- The contract surface grows by one bounded enum variant; conformance tests are required to keep
  behavior stable across refactors and WebGPU shader validation rules.

## Evidence / implementation anchors

- Contract + normalization:
  - `crates/fret-core/src/scene/paint.rs` (`SweepGradient`, `Paint::SweepGradient`, sanitize)
  - `crates/fret-core/src/scene/{validate.rs,fingerprint.rs}`
- Renderer (wgpu default):
  - Encoding: `crates/fret-render-wgpu/src/renderer/render_scene/encode/draw/{quad.rs,path.rs,text.rs}`
  - Shader: `crates/fret-render-wgpu/src/renderer/shaders.rs` (`paint_eval*`)
- Conformance:
  - `crates/fret-render-wgpu/tests/paint_gradient_conformance.rs` (`gpu_sweep_gradient_smoke_conformance`)
