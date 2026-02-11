# ADR 0095: Renderer Perf Snapshots and Stress Harnesses

Status: Proposed

## Context

Fret targets “editor-grade” UI density (toolbars, lists, docking, overlays, multiple viewports). The renderer must
preserve strict ordering semantics (ADR 0002 / ADR 0009), which limits global sorting as an optimization lever.

That makes “default performance” primarily a function of:

- CPU-side scene encoding cost (compiler time, allocations, cache hit rates),
- GPU submission cost (draw calls, pipeline switches, bind group switches),
- upload volume (uniform/vertex/instance bytes written each frame),
- atlas/page budgets and eviction behavior (SVG, glyphs, images).

We already have a dedicated scene encoding layer and an internal ordered draw stream (ADR 0088), plus a focused SVG
atlas stress app that prints perf snapshots and fragmentation/fill metrics:

- Stress harness: `apps/fret-svg-atlas-stress/src/main.rs`
- SVG perf snapshot: `crates/fret-render-wgpu/src/renderer/config.rs` (`SvgPerfSnapshot`)

But we do not yet have a unified, renderer-wide perf snapshot surface that covers draw calls, binds, and upload bytes
across all pipelines. Without stable metrics, “performance work” is easy to regress and hard to validate.

## Decision

### 1) Define a renderer-wide perf snapshot API in `fret-render`

Introduce a stable, best-effort statistics surface that can be enabled in debug builds and stress harnesses:

- `Renderer::set_perf_enabled(bool)`
- `Renderer::take_perf_snapshot() -> Option<RenderPerfSnapshot>`

The snapshot is reset-on-read so callers can sample periodically (e.g. once per second).

### 2) Standardize the minimum counters (P0)

The P0 snapshot must contain counters that map directly to common performance bottlenecks:

- Frame counts:
  - `frames` (number of rendered frames in this sampling window)
- CPU time breakdown (microseconds):
  - `encode_scene_us` (scene compiler time)
  - `prepare_svg_us` / `prepare_text_us` (if applicable)
- Submission counters:
  - `draw_calls`
  - `pipeline_switches`
  - `bind_group_switches` (at least: uniform group, texture groups, atlas groups)
- Upload volume (bytes written via `queue.write_buffer`):
  - `uniform_bytes`
  - `instance_bytes`
  - `vertex_bytes` (split by stream if helpful)
- Cache counters (best-effort):
  - `scene_encoding_cache_hits` / `scene_encoding_cache_misses`
  - per-subsystem hit/miss if already tracked (e.g. SVG raster cache)

This snapshot is explicitly “debug telemetry”; it does not change renderer semantics.

### 3) Keep subsystem-specific snapshots, but make them composable

Subsystem snapshots can remain specialized (SVG, text, images) when they need extra fields, but they should be:

- versioned and stable (so stress tools can parse them),
- optionally embedded or referenced from the unified `RenderPerfSnapshot` (e.g. `svg: Option<SvgPerfSnapshot>`).

### 4) Establish stress harness policy

We treat stress harnesses as first-class tooling for preventing regressions:

- Keep at least one deterministic “renderer stress” app in `apps/` that can run:
  - windowed, and
  - headless (fixed number of frames).
- Prefer printing a single-line, parse-friendly output per snapshot interval.
- Make the scene generation deterministic by default (seeded).

Existing harnesses:

- `apps/fret-svg-atlas-stress` (SVG raster/atlas/fragmentation focus)

Recommended next harnesses (not required for P0, but should be planned):

- `text_atlas_stress`: lots of text blobs with varying sizes and styles.
- `sprite_stress`: many images/icons mixing different bind groups and scissor regions.
- `mixed_ui_stress`: docking + overlays + virtualization + viewport surfaces.

## Consequences

- Performance work becomes measurable and repeatable.
- “Default performance” can be validated without ad-hoc profiling.
- Future renderer changes (e.g. broader batching, new atlas policies) can land safely with quantitative guardrails.

## Follow-ups

- Update ADR 0088 migration plan to reference this snapshot API as the Phase 0 requirement.
- Decide whether to add CI “soft thresholds” (warn-only) for at least draw calls and upload bytes using a headless harness.

