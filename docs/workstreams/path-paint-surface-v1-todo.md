---
title: Path Paint Surface v1 — TODO
status: active
date: 2026-02-16
---

# Path Paint Surface v1 — TODO Tracker

Status: Active (workstream tracker)

Workstream narrative: `docs/workstreams/path-paint-surface-v1.md`  
Milestone board: `docs/workstreams/path-paint-surface-v1-milestones.md`

## Tracking format

- Status: `[ ]` open, `[~]` in progress, `[x]` done, `[!]` blocked
- ID: `PPS-{area}-{nnn}`

Leave 1–3 evidence anchors when completing an item (paths + key functions/tests), and prefer
renderer conformance tests for correctness-sensitive semantics.

## M0 — Contract lock-in (bounded + portable)

- [x] PPS-contract-010 Extend `SceneOp::Path` to accept `Paint` instead of solid `Color`.
  - Evidence: `crates/fret-core/src/scene/mod.rs` (`SceneOp::Path { paint }`)
  - Evidence: `crates/fret-core/src/scene/validate.rs` (path `paint_is_finite`)
  - Evidence: `crates/fret-core/src/scene/fingerprint.rs` (path `mix_paint`)
- [x] PPS-contract-020 Define paint coordinate semantics for paths (origin + local pos).
  - Evidence: `docs/adr/0278-path-paint-surface-v1.md` (Coordinate space)
  - Evidence: `crates/fret-render-wgpu/src/renderer/render_scene/encode/draw/path.rs` (`local_pos_px`)
- [x] PPS-adr-030 Add an ADR that locks semantics + degradation policy.
  - Evidence: `docs/adr/0278-path-paint-surface-v1.md`

## M1 — Renderer implementation (wgpu default)

- [x] PPS-render-100 Encode `Paint` for path draws (bounded, deterministic).
  - Evidence: `crates/fret-render-wgpu/src/renderer/render_scene/encode/draw/path.rs` (`paint_to_gpu`, `state.path_paints`)
  - Evidence: `crates/fret-render-wgpu/src/renderer/types.rs` (`PathDraw.paint_index`, `SceneEncoding.path_paints`)
  - Evidence: `crates/fret-render-wgpu/src/renderer/buffers.rs` (`path_paint_buffers`)
- [x] PPS-render-110 Implement gradient paint evaluation in the path shader/pipeline.
  - Evidence: `crates/fret-render-wgpu/src/renderer/shaders.rs` (`PATH_SHADER` `paint_eval`)
  - Evidence: `crates/fret-render-wgpu/src/renderer/pipelines/path.rs` (bind groups)
  - Evidence: `crates/fret-render-wgpu/src/renderer/render_scene/render.rs` (upload + bind path paints)
- [x] PPS-render-120 Ensure material paint is capability-gated and degrades deterministically.
  - Evidence: `crates/fret-render-wgpu/src/renderer/render_scene/encode/draw/path.rs` (`Paint::Material` deterministic degrade)

## M2 — Conformance (required)

- [x] PPS-test-200 Add GPU readback conformance for path paint:
  - linear gradient has expected left/right coverage
  - radial gradient has expected center/edge coverage (optional)
  - stability across scale factors
  - Evidence: `crates/fret-render-wgpu/tests/path_paint_conformance.rs`

## M3 — Adoption (optional)

- [ ] PPS-adopt-300 Wire one real consumer to use non-solid path paint:
  - pick a small demo surface (plot/node graph/canvas) to validate ergonomics.
