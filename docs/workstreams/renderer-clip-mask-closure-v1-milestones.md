# Renderer Clip/Mask Closure v1 — Milestones

Status: Done (implementation note + gates aligned)

Tracking files:

- `docs/workstreams/renderer-clip-mask-closure-v1.md`
- `docs/workstreams/renderer-clip-mask-closure-v1-todo.md`
- `docs/workstreams/renderer-clip-mask-closure-v1-milestones.md`

## M0 — Design lock (portable + bounded)

Exit criteria:

- Implementation note is explicit about:
  - fast paths vs slow paths,
  - cache keys + eviction budget,
  - WebGPU uniformity constraints,
  - deterministic degradation rules.

Evidence anchors:

- `docs/workstreams/renderer-clip-mask-closure-v1.md`

## M1 — WGSL uniformity closure + correctness

Exit criteria:

- `cargo test -p fret-render-wgpu shaders_validate_for_webgpu` is green.
- Existing clip/mask conformance tests are green.
- Web demo shaders validate (no WGSL uniformity violations in masked paths).

Evidence anchors:

- `crates/fret-render-wgpu/tests/clip_path_conformance.rs`
- `crates/fret-render-wgpu/tests/mask_*_conformance.rs`

## M2 — Cache + perf baselines

Exit criteria:

- Slow-path intermediates are cached (budgeted, deterministic eviction).
- Perf gate exists with a checked-in baseline.

Evidence anchors:

- `crates/fret-render-wgpu/src/renderer/clip_path_mask_cache.rs`
- `tools/perf/headless_clip_mask_stress_gate.py`
- `docs/workstreams/perf-baselines/clip-mask-stress-headless.windows-local.v1.json`
