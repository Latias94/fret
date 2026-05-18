# Path Paint Surface v1 — Closeout Audit

Status: Closed
Last updated: 2026-05-17

## Scope

This closeout covers the `SceneOp::Path` paint surface: replacing solid-only path color with the
bounded scene `PaintBindingV1` surface, renderer payload encoding, gradient/material behavior, and
conformance coverage.

## Findings

### 1. The contract is accepted and implemented

`SceneOp::Path` carries `paint: PaintBindingV1` in `crates/fret-core/src/scene/mod.rs`, with
validation and fingerprint support in `scene/validate.rs` and `scene/fingerprint.rs`.

### 2. The renderer evaluates path paint through the path pipeline

`crates/fret-render-wgpu/src/renderer/render_scene/encode/draw/path.rs` encodes path paint payloads
for the wgpu path pipeline. Solid, linear gradient, radial gradient, and registered material paints
are covered by the implementation; unsupported material cases degrade deterministically.

### 3. Required conformance exists

Gradient behavior is covered by `crates/fret-render-wgpu/tests/path_paint_conformance.rs`.
Registered material paint behavior is covered by
`crates/fret-render-wgpu/tests/path_material_paint_conformance.rs`.

### 4. Optional product adoption is deferred

The renderer/core mechanism is complete. Non-solid path paint adoption in plot, node, or canvas
surfaces should be opened only when a concrete product surface needs it.

## Closure Decision

Close `path-paint-surface-v1` as complete for the framework mechanism. Future adoption or material
depth work should start as narrower follow-ons.
