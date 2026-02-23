# Renderer Scene Encoding Semantics Audit v1 — TODO

Status: Active (workstream tracker)

Tracking format:

- Status: `[ ]` open, `[~]` in progress, `[x]` done, `[!]` blocked
- ID prefix: `REN-ENC-{area}-{nnn}`

## Next

- [x] REN-ENC-docs-001 Document “exact encoding cache” semantics in the profiling note.
  - Evidence: `docs/workstreams/ui-perf-renderer-profiling-v1.md`
- [x] REN-ENC-clean-001 Remove redundant `SceneEncoding::clear()` on cache miss (hygiene).
  - Evidence: `crates/fret-render-wgpu/src/renderer/render_scene/encoding_cache.rs`
- [ ] REN-ENC-observe-001 Add a trace-friendly explanation string for encoding cache misses in `RenderPerfSnapshot` (optional).
  - Example: `miss_reason=text_atlas_revision_changed|images_generation_changed|scene_fingerprint_changed`
  - Evidence target: `crates/fret-render-wgpu/src/renderer/render_scene/encoding_cache.rs`,
    `crates/fret-render-wgpu/src/renderer/types.rs`

## Nice-to-have (only with perf evidence)

- [ ] REN-ENC-cache-010 Consider a tiny multi-entry encoding cache (2–4 entries) if real workloads alternate between a small set of scenes.
  - Risk: memory blow-up if entries are large; must stay bounded.
- [ ] REN-ENC-plan-010 Investigate a structural RenderPlan cache keyed by encoding signature (Option B).
  - Prereq: write down the exact “structural signature” and add regression tests to prevent stale plan reuse.
- [ ] REN-ENC-adr-010 If we need “dynamic param handles”, draft an ADR (Option C).
