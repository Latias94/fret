# Renderer Upstream Semantics Parity v1 — TODO

## M0 — Setup (minimal)

- [x] Create a single “parity note” template section in this file (copy/paste friendly).
- [x] Pick 1 upstream seam to start with (recommended: scissor coordinate spaces).

## M1 — First parity note (scissor spaces)

- [x] Zed/GPUI: identify how render-target origin offsets and scissors are represented and applied.
- [x] Fret: record current representation and mapping:
  - `AbsoluteScissorRect` vs `LocalScissorRect`
  - `RenderSpace` mapping and scissor translation/clamping
- [x] Decide: gap vs deliberate difference; record rationale.
- [x] If gap: add the smallest guardrail (validator or test) *before* changing implementation.

## M2 — Clip/mask composition parity

- [x] Compare push-time capture semantics for clip path / image mask stacks.
  - Evidence: Parity note 002.
- [ ] Compare cache key strategy and reuse heuristics for mask targets.
- [x] Add one conformance test that breaks if clip capture semantics drift.
  - Evidence: `crates/fret-render-wgpu/tests/clip_path_conformance.rs` (`gpu_clip_path_is_captured_at_push_time_and_does_not_follow_later_transforms`).

## M3 — Intermediate reuse / lifetime parity

- [ ] Compare intermediate allocation/reuse strategy vs upstream:
  - lifetime model,
  - eviction/budgeting policy,
  - determinism under contention.
- [ ] Add one targeted unit test for “release after last use” stability in plan shape.

## Notes / parity template

Copy/paste for each seam:

- Seam:
- Upstream evidence anchors:
- Fret evidence anchors:
- Observed behavior:
- Differences (gap vs deliberate):
- Proposed guardrail:
- Follow-up refactor steps:

## Parity note 001 — Scissor coordinate spaces / render-target origin offsets

- Seam: scissor rectangles and pixel→clip mapping when rendering into non-fullscreen targets.
- Upstream evidence anchors (Zed / GPUI WGPU):
  - `repo-ref/zed/crates/gpui_wgpu/src/shaders.wgsl` — `to_device_position_impl()` divides by `globals.viewport_size` (no per-pass origin).
  - `repo-ref/zed/crates/gpui_wgpu/src/wgpu_renderer.rs` — `GlobalParams { viewport_size: [surface.width, surface.height] }`.
  - `repo-ref/zed/crates/gpui_wgpu/src/shaders.wgsl` — clip-rect masking uses shader-side `content_mask` distances (bounds-based), not GPU scissor.
- Upstream notes:
  - Zed/GPUI appears to assume a single full-surface viewport for device-position conversion.
  - Clipping is primarily expressed via shader distance fields (`content_mask`) rather than relying on `set_scissor_rect`.
  - As a result, there is no obvious upstream representation of “dst-local scissor” vs “absolute scissor” for scissor-sized intermediates.
- Fret evidence anchors:
  - `crates/fret-render-wgpu/src/renderer/render_plan.rs` — explicit scissor tagging: `AbsoluteScissorRect` vs `LocalScissorRect`.
  - `crates/fret-render-wgpu/src/renderer/render_scene/execute.rs` — per-pass `RenderSpaceUniform { origin_px, size_px }` upload.
  - `crates/fret-render-wgpu/src/renderer/render_scene/helpers.rs` — pass→render-space extraction (trace/meta).
  - `docs/adr/0275-render-space-and-scissor-sized-intermediates-v1.md` — normative scissor mapping and plan scissor representation.
- Observed behavior (Fret):
  - Scene geometry stays in absolute viewport pixels, but each pass has an explicit render-space origin/size.
  - RenderPlan stores either absolute (render-space) scissors or dst-local scissors depending on pass semantics.
- Differences (gap vs deliberate):
  - Deliberate: Fret must support scissor-sized intermediates and multi-pass pipelines under explicit budgeting/degradation; this requires per-pass origin/size and explicit scissor space tagging.
  - Upstream comparisons are still useful for validating clip/mask composition intent, but scissor-sized intermediate correctness needs a Fret-specific contract (`RenderSpace`).
- Proposed guardrail:
  - Keep the type-level split (`AbsoluteScissorRect` vs `LocalScissorRect`) as a hard refactor constraint.
  - Keep `RenderPlan::debug_validate()` checks that reject out-of-bounds/local scissor misuse.
- Follow-up refactor steps:
  - [Done] Centralize “apply dst-local scissor to a wgpu render pass” in one helper to avoid recorders re-implementing the same mapping pattern.
    - Evidence: `crates/fret-render-wgpu/src/renderer/fullscreen.rs` (`run_fullscreen_triangle_pass*`).
  - [Done] Extend trace/meta so it is always obvious whether a pass scissor is absolute or local (debug-only is fine).
    - Evidence: `crates/fret-render-wgpu/src/renderer/render_scene/helpers.rs` (`RenderPlanPassTraceMeta.scissor_space`),
      `crates/fret-render-wgpu/src/renderer/render_scene/execute.rs` (trace field: `scissor_space`),
      `crates/fret-render-wgpu/src/renderer/render_scene/helpers.rs` (`render_plan_trace_fingerprint` mixes scissor-space).

## Parity note 002 — Clip capture semantics (push-time vs dynamic)

- Seam: what it means to “push a clip” and whether later transforms can retroactively affect it.
- Upstream evidence anchors (Zed / GPUI):
  - Clip stack API (push/pop): `repo-ref/zed/crates/gpui/src/window.rs` (`with_content_mask`, `content_mask_stack`).
  - Per-primitive capture: `repo-ref/zed/crates/gpui/src/window.rs` (`paint_quad`, `paint_path` assign `content_mask` on insert).
  - Shader-side clip evaluation: `repo-ref/zed/crates/gpui_wgpu/src/shaders.wgsl` (`content_mask`, `distance_from_clip_rect*`, `clip_distances`).
- Fret evidence anchors:
  - Contract + plan markers: `docs/adr/0273-clip-path-and-image-mask-sources-v1.md`,
    `crates/fret-render-wgpu/src/renderer/render_plan_compiler.rs` (`EffectMarkerKind::{ClipPathPush,ClipPathPop}`).
  - GPU conformance: `crates/fret-render-wgpu/tests/clip_path_conformance.rs`
    (`gpu_clip_path_is_captured_at_push_time_and_does_not_follow_later_transforms`).
- Observed behavior:
  - Upstream (Zed/GPUI) treats “clip” as a stack of bounds masks that is intersected on push and then copied onto each primitive at insertion time.
    Shader evaluation uses those captured bounds (`content_mask`) rather than relying on GPU scissor state.
  - Fret captures clip-path state at push time and bakes it into the plan/encoding such that later transforms do not retroactively affect earlier clips
    (conformance-gated).
- Differences (gap vs deliberate):
  - Deliberate divergence: Fret’s clip-path is not limited to bounds rectangles; it supports shape-based clipping with deterministic degradation under budget.
    Upstream’s `content_mask` is bounds-based, but it still supports the same “capture-at-push-time” intuition.
- Proposed guardrail:
  - Keep the conformance test that asserts push-time capture semantics.
  - Keep plan validation and plan trace fields sufficient to debug clip stacks when refactoring.
- Follow-up refactor steps:
  - When touching clip-path encode/compile paths, always run `cargo nextest run -p fret-render-wgpu --test clip_path_conformance`.
