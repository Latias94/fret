# Renderer Execute Pass Recorder Modularization v1 — TODO

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

## Next

- [ ] Extract target selection helpers (output vs intermediate vs mask) into a dedicated helper
      surface to reduce per-pass boilerplate.
- [ ] Decide the final ownership shape for `SceneDrawRange`:
  - keep as a `Renderer` method with explicit args (status quo), or
  - migrate to an executor-based recorder function (more churn; higher uniformity)
- [ ] Reduce remaining argument surface area where practical (batch args into focused structs).

## Gates (must stay green)

- `python3 tools/check_layering.py`
- `cargo test -p fret-render-wgpu --lib`
- `cargo test -p fret-render-wgpu --test affine_clip_conformance`
- `cargo test -p fret-render-wgpu --test viewport_surface_metadata_conformance`
- `cargo test -p fret-render-wgpu --test mask_image_conformance`
- `cargo test -p fret-render-wgpu --test composite_group_conformance`
