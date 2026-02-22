# Renderer Execute Pass Recorder Modularization v1 — Milestones

## M0 — Baseline refactor gates

Exit criteria:

- Conformance anchors + layering checks are the default “before/after” gate set.

## M1 — Effect recorders modularized

Exit criteria:

- Effect passes live under `render_scene/recorders/`.
- `execute.rs` remains an orchestration-only executor loop.

Evidence anchors:

- `crates/fret-render-wgpu/src/renderer/render_scene/recorders/effects.rs`

## M2 — All pass recorders file-separated

Exit criteria:

- Each `RenderPlanPass` kind has a dedicated recorder module (or small grouped modules with a clear
  boundary).
- Shared helpers live in `render_scene/*` utilities (not ad-hoc `pub(super)` methods).

## M3 — `RenderSceneExecutor` introduced (Option C)

Exit criteria:

- Per-frame mutable state is isolated from `Renderer` (encoder, targets, cursors, perf).
- Recorders have stable, small signatures and are easy to test in isolation (where practical).

Evidence anchors:

- Design: `docs/workstreams/renderer-execute-pass-recorders-modularization-v1-refactor-design.md`
- Executor: `crates/fret-render-wgpu/src/renderer/render_scene/executor.rs`
