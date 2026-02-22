# Renderer RenderPlan Semantics Audit v1 — TODO

## Done

- [x] Add a debug-only `RenderPlan` validator to catch target lifetime, `LoadOp::Load`, scissor, and mask shape misuse early.

## Next

- [ ] Expand debug validation:
  - verify scissors are within destination bounds when provided
  - verify `MaskRef.viewport_rect` is within mask target size
  - verify `target_origin + target_size` bounds are consistent per pass
- [ ] Add “plan shape” diagnostics:
  - add a helper to dump a minimal per-pass summary (kind/src/dst/size/load/scissor) under trace
  - ensure dumps are stable enough to diff across refactors
- [ ] Add targeted semantic tests (unit or integration):
  - “LoadOp::Load requires prior init” regression
  - “ReleaseTarget inserted after last use” regression
  - “Downsample scissor mapping never expands bounds” regression
- [ ] Audit pass-by-pass semantics and document any ambiguous areas:
  - `PathMsaaBatch` initialization rules
  - `ClipMask` pass clear/load assumptions
  - mask sampling + viewport rect mapping rules for each postprocess pass

## Nice-to-have

- [ ] Compare semantics vs `repo-ref/zed`/`repo-ref/gpui-component` for:
  - intermediate target reuse
  - clip/mask composition rules
  - blend mode degradation strategy
