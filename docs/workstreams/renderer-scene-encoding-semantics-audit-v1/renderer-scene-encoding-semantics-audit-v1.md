# Renderer Scene Encoding Semantics Audit v1

Status: Draft (workstream note; ADRs remain the source of truth)

This workstream complements:

- RenderPlan semantics: `docs/workstreams/renderer-render-plan-semantics-audit-v1/renderer-render-plan-semantics-audit-v1.md`
- vNext refactor tracker: `docs/workstreams/renderer-vnext-fearless-refactor-v1/renderer-vnext-fearless-refactor-v1-todo.md`

## Goal

Make fearless renderer refactors safer by documenting the semantics we rely on between:

- `fret-core::scene::Scene` (portable scene contract),
- the wgpu backend’s scene encoding (`SceneEncoding`),
- and the cache boundary (`SceneEncodingCache` / `SceneEncodingCacheKey`).

This is intentionally about *mechanism-level* semantics and observability, not about policy/recipes
(Radix/shadcn behavior stays in ecosystem crates).

## Scope

- Scene → encoding: `crates/fret-render-wgpu/src/renderer/render_scene/encode/*`
- Encoding cache + key: `crates/fret-render-wgpu/src/renderer/scene_encoding_cache.rs`,
  `crates/fret-render-wgpu/src/renderer/render_scene/encoding_cache.rs`,
  `crates/fret-render-wgpu/src/renderer/types.rs` (`SceneEncodingCacheKey`)
- How `Scene::fingerprint()` is used for caching and diagnostics:
  `crates/fret-core/src/scene/fingerprint.rs`, `crates/fret-core/src/scene/mod.rs`
- Where validation is enforced (or not):
  `crates/fret-core/src/scene/validate.rs`,
  `crates/fret-render-wgpu/src/renderer/render_scene/execute.rs`,
  `crates/fret-launch/src/runner/desktop/runner/render.rs`

## Non-goals

- No changes to `fret-core::scene` public contract in v1 of this audit.
- No changes to RenderPlan pass semantics (tracked separately).
- No attempt to “make animated scenes cache-friendly” by weakening correctness.

## Current pipeline (wgpu backend)

Per-frame high-level flow:

1) Validate scene (debug-only panic by default):
   - `crates/fret-render-wgpu/src/renderer/render_scene/execute.rs` (`render_scene_execute`)
2) Prepare frame resources (pipelines, text atlas, SVG):
   - `crates/fret-render-wgpu/src/renderer/render_scene/execute.rs`
3) Acquire encoding for frame (exact cache hit or re-encode):
   - `crates/fret-render-wgpu/src/renderer/render_scene/encoding_cache.rs`
4) Compile RenderPlan for the encoding:
   - `crates/fret-render-wgpu/src/renderer/render_scene/plan_compile.rs`
5) Upload uniforms + geometry:
   - `crates/fret-render-wgpu/src/renderer/render_scene/execute.rs`
6) Record passes and finish the command buffer:
   - `crates/fret-render-wgpu/src/renderer/render_scene/dispatch.rs`

## Key semantics / invariants (v1)

### 1) Scene encoding is a *value-carrying* transform

Encoding is not just a “structural batch plan”; it contains concrete per-draw payload:

- Paint parameters (including material params)
- Transformed geometry (path vertices, text vertices, quad instances)
- Effect markers and uniform snapshots

Therefore:

- Reusing a cached `SceneEncoding` is only correct when the *scene payload is identical* (after
  scene-layer sanitization).

### 2) Encoding cache semantics are *exact*

The scene encoding cache is a single-entry “last frame” cache:

- `crates/fret-render-wgpu/src/renderer/scene_encoding_cache.rs`

Cache hit requirements:

- `SceneEncodingCacheKey` equality (v1 includes `scene.fingerprint()` and `scene.ops_len()`):
  `crates/fret-render-wgpu/src/renderer/types.rs`
- Resource generation stability:
  - intermediate/render-target generations
  - image generation
- Text atlas revision and text quality key stability:
  - `text_atlas_revision`
  - `text_quality_key`

Implication:

- Any animation that changes scene op payload (colors, transforms, material params, etc.) will
  intentionally miss the encoding cache.
- This is correct; higher-level caching (UI paint cache / view cache) is the intended layer for
  “structural reuse with updated values”.

### 3) Validation is “debug by default”, “opt-in in release”

- Debug builds: renderer panics on `scene.validate()` errors at `render_scene_execute`.
- Release builds: desktop runner can validate if `FRET_VALIDATE_SCENE` is set:
  `crates/fret-launch/src/runner/desktop/runner/render.rs` (`validate_scene_if_enabled`).

Invariant:

- Fearless refactors must keep `Scene::validate()` meaningful and cheap enough to use in debug.

### 4) Sanitization is intentionally contract-shaping

Some canonicalization happens at the scene layer to keep backends/tests deterministic:

- Example: CSS-style “effective border radius” clamping for quads:
  `crates/fret-core/src/scene/mod.rs` (`SceneRecording::push`)
- Paint canonicalization:
  `crates/fret-core/src/scene/paint.rs` (`Paint::sanitize`, `MaterialParams::sanitize`)

This is acceptable as mechanism-level normalization *only when* it preserves portability and
deterministic semantics across backends.

## Audit findings (v1)

### A) The core flow is correct and readable

The responsibilities are well-separated:

- Scene contract: `fret-core`
- Encoding: wgpu backend only
- RenderPlan compilation: backend-internal IR with its own documented semantics
- Dispatch/recording: executor + pass recorders

This matches a typical “GPU-first UI renderer” practice (authoring → encoding → plan → record).

### B) Cache expectations should be explicit in docs & perf tooling

Because the encoding cache is exact (and single-entry), it is easy to misinterpret “cache misses”
in animated harnesses as a bug. It is not.

Actionable: document “what a miss means” and “what to optimize instead”:

- idle/static frames: encoding cache hits are expected and valuable
- animated frames: expect misses; optimize UI paint/view caching, text prep, and pass complexity

### C) Potential micro-cleanups (no semantic change)

These are purely readability/perf hygiene improvements:

- Avoid redundant encoding clears on cache miss.
- Consider documenting why the cache is single-entry (and when a small multi-entry cache would be
  justified).

## Options for “next” improvements (do not implement without evidence)

### Option A — Keep cache exact, improve observability (recommended next)

Best when:

- Most real workloads are static often enough (idle windows, editor frames with stable UI).

Work:

- Minor hygiene changes + clearer docs.

### Option B — Add a RenderPlan cache keyed by *structural* encoding signature

Idea:

- Keep `SceneEncoding` exact, but cache the compiled `RenderPlan` when only uniform payload changes.

Pros:

- Can reduce `plan_compile` CPU cost on “same structure, different colors/params” workloads.

Cons:

- Requires a careful “structural signature” definition and tight tests to avoid stale plan reuse.

### Option C — Contract-level “dynamic params handles” (bigger, ADR required)

Idea:

- Decouple frequently-changing numeric params (e.g. time) from the scene op stream via stable
  handles/uniform slots.

Pros:

- Enables deeper reuse across animation frames.

Cons:

- Requires API/contract changes and new regression gates (ADR + conformance + diag evidence).

## Evidence / gates

Keep these green while refactoring:

- Layering: `python3 tools/check_layering.py`
- Renderer conformance tests (examples):
  - `cargo nextest run -p fret-render-wgpu --test clip_path_conformance`
  - `cargo nextest run -p fret-render-wgpu --test mask_image_conformance`
  - `cargo nextest run -p fret-render-wgpu --test composite_group_conformance`
- WebGPU shader validation:
  - `cargo nextest run -p fret-render-wgpu --test shaders_validate_for_webgpu`

