# Renderer Execute Pass Recorder Modularization v1 — TODO

## Done

- [x] Introduce `ExecuteCtx` to reduce pass argument explosion.
- [x] Move effect pass recorders into `render_scene/recorders/effects.rs`.

## Next

- [ ] Move remaining pass recorders into `render_scene/recorders/*`:
  - scale-nearest
  - blur
  - fullscreen blit
  - backdrop warp
  - path clip mask / MSAA path batch (as separate modules, not mixed with effects)
- [ ] Extract shared helpers out of `execute.rs` (reduce `pub(in ...)` leakage):
  - scissor mapping helpers
  - target selection helpers (output vs intermediate vs mask)
  - uniform bind-group picking (mask-image override selection)
- [ ] Implement Option C scaffold (`RenderSceneExecutor`) and migrate per-frame mutable state:
  - create `render_scene/executor.rs` and route pass recording through it
  - move per-frame cursors/transients (instance/uniform cursors) into the executor
  - keep semantics stable; record evidence anchors per pass moved
- [ ] Convert recorders to executor-based functions (`recorders/*` takes `&mut RenderSceneExecutor`).

## Gates (must stay green)

- `python3 tools/check_layering.py`
- `cargo test -p fret-render-wgpu --lib`
- `cargo test -p fret-render-wgpu --test affine_clip_conformance`
- `cargo test -p fret-render-wgpu --test viewport_surface_metadata_conformance`
- `cargo test -p fret-render-wgpu --test mask_image_conformance`
- `cargo test -p fret-render-wgpu --test composite_group_conformance`
