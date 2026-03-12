# Renderer Execute Pass Recorder Modularization v1 — TODO

See also:
- `docs/workstreams/renderer-execute-pass-recorders-modularization-v1/renderer-execute-pass-recorders-modularization-v1-design.md`
- `docs/workstreams/renderer-execute-pass-recorders-modularization-v1/renderer-execute-pass-recorders-modularization-v1-milestones.md`
- `docs/workstreams/renderer-render-plan-semantics-audit-v1/renderer-render-plan-semantics-audit-v1.md`

## Done

- [x] Split pass recording into `render_scene/recorders/*`.
- [x] Introduce `RenderSceneExecutor` and route pass recording through it (Option C).
- [x] Migrate effect recorders to executor-based functions.
- [x] Migrate clip-mask / composite-premul to executor-based functions.
- [x] Migrate path clip-mask / MSAA path batch to executor-based functions.
- [x] Remove `ExecuteCtx` (executor-only shared inputs).
- [x] Extract scissor mapping helpers out of `execute.rs`.
- [x] Extract uniform bind-group picking out of `execute.rs`.
- [x] Centralize plan-pass trace/render-space helpers out of `execute.rs`.
- [x] Extract target selection helpers (output vs intermediate vs mask) into `render_scene/helpers.rs`.
- [x] Pack pass resources (buffers + bind groups) into `RecordPassResources`.
- [x] Pack per-pass context (plan/index/offset) into `RecordPassCtx`.
- [x] Centralize scissor application helpers used by recorders (absolute vs dst-local).
  - Evidence: `crates/fret-render-wgpu/src/renderer/fullscreen.rs` (dst-local), `crates/fret-render-wgpu/src/renderer/render_scene/helpers.rs` (absolute mapping).
- [x] Extract a reusable composite-premul quad-pass helper to reduce recorder boilerplate.
  - Evidence: `crates/fret-render-wgpu/src/renderer/render_scene/helpers.rs` (`run_composite_premul_quad_pass`),
    `crates/fret-render-wgpu/src/renderer/render_scene/recorders/effects.rs` (`record_composite_premul_pass`).

## Next

- [x] Apply target selection helpers across remaining recorders (scene-draw, path-msaa, etc.).
- [x] Decide the final ownership shape for `SceneDrawRange`:
  - v1 decision: keep as a `Renderer` method with explicit args (`SceneDrawRangePassArgs`)
  - follow-up: revisit recorder migration only if we need uniformity for shared recorder tooling
- [x] Reduce remaining argument surface area where practical (batch args into focused structs).

## Gates (must stay green)

- `python3 tools/check_layering.py`
- `cargo test -p fret-render-wgpu --lib`
- `cargo test -p fret-render-wgpu --test affine_clip_conformance`
- `cargo test -p fret-render-wgpu --test viewport_surface_metadata_conformance`
- `cargo test -p fret-render-wgpu --test mask_image_conformance`
- `cargo test -p fret-render-wgpu --test composite_group_conformance`
