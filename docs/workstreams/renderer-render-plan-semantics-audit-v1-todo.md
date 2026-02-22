# Renderer RenderPlan Semantics Audit v1 — TODO

## Done

- [x] Add a debug-only `RenderPlan` validator to catch target lifetime, `LoadOp::Load`, scissor, and mask shape misuse early.

## Next

- [x] Expand debug validation:
  - verify scissors are within destination bounds when provided
  - verify `MaskRef.viewport_rect` and `MaskRef.size` are consistent per pass
  - reject integer overflow in scissor/rect bounds math
- [x] Add “plan shape” diagnostics:
  - per-pass trace spans include kind/src/dst/load/scissor/render-space
  - render-scene trace span includes `plan_fingerprint`
- [x] Add targeted semantic tests (unit or integration):
  - “LoadOp::Load requires prior init” regression (validator unit test)
  - “ReleaseTarget inserted after last use” regression (unit test)
  - “Downsample scissor mapping never expands bounds” regression (unit test)
- [ ] Audit pass-by-pass semantics and document any ambiguous areas:
  - `PathMsaaBatch` initialization rules (validated as `LoadOp::Load`)
  - `ClipMask` pass clear/load assumptions
  - mask sampling + viewport rect mapping rules for each postprocess pass

## Nice-to-have

- [ ] Compare semantics vs `repo-ref/zed`/`repo-ref/gpui-component` for:
  - intermediate target reuse
  - clip/mask composition rules
  - blend mode degradation strategy
