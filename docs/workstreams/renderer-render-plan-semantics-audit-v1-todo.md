# Renderer RenderPlan Semantics Audit v1 — TODO

## Done

- [x] Add a debug-only `RenderPlan` validator to catch target lifetime, `LoadOp::Load`, scissor, and mask shape misuse early.
- [x] Make scissor coordinate spaces explicit in the plan (`AbsoluteScissorRect` vs `LocalScissorRect`).

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
  - `ClipMask` pass clear/load assumptions (always `Clear`)
  - mask sampling + viewport rect mapping rules for each postprocess pass
  - [x] Add an explicit “Ambiguities / TODO” section to the v1 semantics doc.
    - Evidence: `docs/workstreams/renderer-render-plan-semantics-audit-v1.md` (Ambiguities / TODO).
  - [ ] Mechanically verify `ClipMask` clear/load assumptions across all plan pass variants and recorders.
  - [ ] Document “initialized within the frame” precisely for `Intermediate*` targets and clarify whether `Output` has any special casing.
  - [ ] Write a small per-pass table for `MaskRef.viewport_rect` mapping rules (dst-local space, tier expectations, downsample/upsample behavior).
- [x] Make plan-pass trace/meta preserve scissor coordinate space tags (absolute vs dst-local).
  - Evidence: `crates/fret-render-wgpu/src/renderer/render_scene/helpers.rs` (`RenderPlanPassTraceMeta.scissor_space`),
    `crates/fret-render-wgpu/src/renderer/render_scene/execute.rs` (trace field: `scissor_space`),
    `crates/fret-render-wgpu/src/renderer/render_scene/helpers.rs` (`render_plan_trace_fingerprint` mixes scissor-space).
- [x] Extend RenderPlan JSON dump to preserve scissor coordinate space tags (absolute vs dst-local).
  - Evidence: `crates/fret-render-wgpu/src/renderer/render_plan_dump.rs` (pass `*_scissor_space` fields, effect-marker `scissor_space`).

## Nice-to-have

- [ ] Compare semantics vs `repo-ref/zed`/`repo-ref/gpui-component` for:
  - intermediate target reuse
  - clip/mask composition rules
  - blend mode degradation strategy
