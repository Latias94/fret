# Renderer Execute Pass Recorder Modularization v1 — Design

## Goal

Make `render_scene` pass recording easier to evolve (fearless refactors) without changing rendering semantics:
- Reduce the argument surface area between `execute.rs` and per-pass recorders.
- Centralize shared pass concerns (target selection, scissor mapping, bind-group picking, trace/render-space metadata).
- Keep the “pass recorder” layer locally testable via existing conformance tests.

## Non-goals

- No change to public contracts or cross-crate APIs.
- No rendering behavior changes unless a bug is discovered and explicitly scoped as a follow-up.
- No component/policy work (Radix/shadcn behaviors remain in ecosystem).

## Current shape (Option C)

- `RenderSceneExecutor` owns the stable per-frame context required to record passes:
  - device/queue/encoder, frame targets, encoding, format/usage/viewport, perf.
- Each `RenderPlanPass` is recorded via an executor-based recorder function in `render_scene/recorders/*`.
- Shared helpers live in `render_scene/helpers.rs` (e.g. plan target view selection).

## Key refactor: resource bundling

Pass recorders frequently need the same GPU resources:
- vertex buffers: viewport/text/path
- bind groups: quad instances, text paint, path paint

Instead of threading these through `record_pass` as many parameters, they are bundled into:
- `RecordPassResources`

This keeps the call site in `execute.rs` stable while allowing recorders to evolve independently.

## Risks and mitigations

- **Risk:** accidental semantic changes while reorganizing code.
  - **Mitigation:** keep conformance tests + WebGPU shader validation green during each step.
- **Risk:** layering drift (wgpu/platform deps leaking into core).
  - **Mitigation:** run `tools/check_layering.py` when touching boundaries (not required for pure refactors, but should stay green).

## Follow-ups (v1.x)

- Decide whether `SceneDrawRange` remains a `Renderer` method (explicit args) or becomes a recorder function for uniformity.
- If we need to change “hard-to-change” runtime behavior, write/extend an ADR and add evidence anchors.

