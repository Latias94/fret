# Renderer Clip/Mask Closure v1 — TODO Tracker

Status: Draft (workstream tracker)

Tracking format:

- Status: `[ ]` open, `[~]` in progress, `[x]` done, `[!]` blocked
- ID: `CLIPMASK-{area}-{nnn}`

When completing an item, leave 1–3 evidence anchors (paths + key functions/tests), and prefer
`fretboard diag` scripts/bundles where applicable.

## Design lock

- [ ] CLIPMASK-design-010 Publish an “executable” implementation note for clip/mask stacks:
      fast paths, slow paths, cache keys, and WebGPU uniformity constraints.
  - Evidence anchors:
    - `docs/workstreams/renderer-clip-mask-closure-v1.md`
    - `docs/adr/0239-mask-layers-and-alpha-masks-v1.md` (mask semantics constraints)
    - `docs/adr/0273-clip-path-and-image-mask-sources-v1.md` (clip-path + mask sources)

## Renderer implementation (wgpu)

- [x] CLIPMASK-wgpu-020 Ensure mask/clip sampling is WGSL-uniformity-safe:
      remove divergent sampling branches and derivative hazards.
  - Evidence anchors:
    - `crates/fret-render-wgpu/src/renderer/shaders.rs` (mask_image_sample_bilinear_clamp)
    - `cargo test -p fret-render-wgpu shaders_validate_for_webgpu`
  - Implementation note:
    - For image masks, avoid `textureSample*` in `mask_eval` and use `textureLoad` + manual bilinear.

- [x] CLIPMASK-cache-030 Add caching for slow-path clip/mask intermediates where applicable:
      avoid per-frame re-rasterization of identical clip paths.
  - Notes:
    - Cache key must include transform/bounds/quality downsample.
    - Cache size must be budgeted and deterministic (eviction policy).
  - Evidence anchors:
    - `crates/fret-render-wgpu/src/renderer/clip_path_mask_cache.rs` (GPU-copy cache + LRU budget eviction)
    - `crates/fret-render-wgpu/src/renderer/render_scene/render.rs` (PathClipMask hit/copy + store)
    - `crates/fret-render-wgpu/src/renderer/render_scene/encode/ops.rs` (cache key composition)
    - `crates/fret-render-wgpu/src/renderer/render_plan_compiler.rs` (mix mask target size into key)

## Conformance + regression

- [x] CLIPMASK-test-040 Add a regression test for “clip path cache stability”:
      same scene across frames must not churn intermediates.
  - Evidence anchors:
    - `apps/fret-clip-mask-stress/src/main.rs` (prints clip-path cache counters)
    - `tools/perf/headless_clip_mask_stress_gate.py` (enforces invariants: hits>0, misses bounded, entries bounded)

- [x] CLIPMASK-perf-050 Add a perf gate for clip/mask heavy scenes:
      keep worst-frame stability and intermediate allocations bounded.
  - Evidence anchors:
    - `apps/fret-clip-mask-stress/src/main.rs`
    - `tools/perf/headless_clip_mask_stress_gate.py`
    - `docs/workstreams/perf-baselines/clip-mask-stress-headless.windows-local.v1.json`
