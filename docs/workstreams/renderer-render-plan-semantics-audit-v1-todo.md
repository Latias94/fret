# Renderer RenderPlan Semantics Audit v1 — TODO

## Done

- [x] Add a debug-only `RenderPlan` validator to catch target lifetime, `LoadOp::Load`, scissor, and mask shape misuse early.

## Next

- [ ] Expand debug validation:
  - verify scissors are within destination bounds when provided (in progress)
  - verify `MaskRef.viewport_rect` is within mask target size (in progress)
  - verify `target_origin + target_size` bounds are consistent per pass
- [x] Add “plan shape” diagnostics:
  - per-pass trace spans include kind/src/dst/load/scissor/render-space
  - render-scene trace span includes `plan_fingerprint`
- [x] Add targeted semantic tests (unit or integration):
  - “LoadOp::Load requires prior init” regression (validator unit test)
  - “ReleaseTarget inserted after last use” regression (unit test)
  - “Downsample scissor mapping never expands bounds” regression (unit test)
- [ ] Audit pass-by-pass semantics and document any ambiguous areas:
  - `PathMsaaBatch` initialization rules
  - `ClipMask` pass clear/load assumptions
  - mask sampling + viewport rect mapping rules for each postprocess pass

## Nice-to-have

- [ ] Compare semantics vs `repo-ref/zed`/`repo-ref/gpui-component` for:
  - intermediate target reuse
  - clip/mask composition rules
  - blend mode degradation strategy
