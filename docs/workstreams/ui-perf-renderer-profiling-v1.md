# UI Performance: Bottom-Up Renderer Profiling (v1)

Status: Draft (workstream note; ADRs remain the source of truth)

This note captures a practical, repeatable way to profile “editor-class” smoothness starting from **rendering
primitives** (text, SVG, path, batching) and working upward into the UI runtime.

Related:

- Zed smoothness workstream: `docs/workstreams/ui-perf-zed-smoothness-v1.md`
- Perf log: `docs/workstreams/ui-perf-zed-smoothness-v1-log.md`
- GPUI gap analysis: `docs/workstreams/ui-perf-gpui-gap-v1.md`
- Tracy guide: `docs/tracy.md`

---

## 1) Why bottom-up here

Fret’s UI runtime has more “mechanisms” than Zed’s GPUI substrate (view caching, retained tree, invalidation,
diagnostics, policies in ecosystem crates). When we see a page become **paint-dominant**, it’s easy to spend time
micro-optimizing the wrong layer.

Bottom-up profiling answers:

- Is the cost dominated by **text preparation** (shaping, atlas churn)?
- Is it dominated by **scene encoding** (CPU work to build the render plan)?
- Is it dominated by **draw submission complexity** (draw calls / pipeline switches / uploads)?

---

## 2) Recommended profiling stack (layered)

Use the smallest tool that answers the question.

### 2.1 Coarse budget: `fretboard diag perf`

Use this to decide which slice dominates the frame:

- `layout_time_us`, `prepaint_time_us`, `paint_time_us`

If `paint_time_us` dominates, switch to renderer-level signals below.

### 2.2 Renderer primitive signals: `RenderPerfSnapshot` (UI gallery logging)

UI gallery now supports periodic renderer perf logging (disabled by default):

- Enable: `FRET_UI_GALLERY_RENDERER_PERF=1`
- Optional breakdown: `FRET_RENDERER_PERF_PIPELINES=1`

Output includes:

- CPU: `encode_scene_us`, `prepare_text_us`, `prepare_svg_us`
- Complexity proxies: `draw_calls`, per-primitive draw calls, pipeline switches, bind group switches, uniform/instance/vertex bytes
- Scene encoding cache hit/miss counts

This is the fastest “primitive-level” sanity check before deeper tracing.

### 2.3 Deep CPU attribution: Tracy (`diag repro --with tracy`)

Tracy is the best way to see *where* CPU time goes across the runtime + renderer call stack.

Use it when:

- a page is paint-dominant but renderer perf numbers are not explanatory enough, or
- you suspect “glue cost” between UI → scene → renderer is the real culprit.

### 2.4 GPU / pipeline inspection: RenderDoc (`diag repro --with renderdoc`)

Use RenderDoc when:

- CPU looks fine but you still see hitches (GPU stalls, overdraw, heavy passes),
- you need pass-level detail (blur passes, clip masks, intermediate pool pressure).

---

## 3) Recommended workflows

### 3.1 “Editor-class” workload + primitive signals (fastest loop)

Run a steady scripted workload (example: code editor autoscroll) and enable renderer perf logging:

```bash
cargo run -p fretboard -- diag repro tools/diag-scripts/ui-gallery-code-editor-torture-autoscroll-steady.json \
  --env FRET_UI_GALLERY_RENDERER_PERF=1 \
  --env FRET_RENDERER_PERF_PIPELINES=1 \
  --env FRET_UI_GALLERY_VIEW_CACHE=1 \
  --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 \
  --launch -- target/release/fret-ui-gallery
```

What to look for:

- `prepare_text_us` spikes: likely glyph atlas churn / shaping reuse gaps.
- `encode_scene_us` spikes: likely scene encoding cache misses or expensive effect planning.
- High `text_draw_calls` + high pipeline switches: batching/material fragmentation problem.
- `scene_encoding_cache_misses` unexpectedly high on steady scripts: cache key instability.

### 3.2 Same workload, but with Tracy capture

```bash
cargo run -p fretboard -- diag repro tools/diag-scripts/ui-gallery-code-editor-torture-autoscroll-steady.json \
  --with tracy \
  --env FRET_TRACY=1 \
  --env FRET_UI_GALLERY_RENDERER_PERF=1 \
  --env FRET_UI_GALLERY_VIEW_CACHE=1 \
  --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 \
  --launch -- target/release/fret-ui-gallery
```

Notes:

- `--with tracy` already enables `FRET_TRACY=1` and may auto-inject `--features fret-bootstrap/tracy` for `cargo run` launches.
- Capture is saved from the Tracy UI (not automatically by `fretboard` yet).

### 3.3 GPU capture with RenderDoc (frame-level)

```bash
cargo run -p fretboard -- diag repro tools/diag-scripts/ui-gallery-code-editor-torture-autoscroll-steady.json \
  --with renderdoc \
  --renderdoc-after-frames 120 \
  --renderdoc-marker fret.runner.render_scene \
  --env FRET_UI_GALLERY_VIEW_CACHE=1 \
  --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 \
  --launch -- target/release/fret-ui-gallery
```

---

## 4) Next “make it actionable” upgrades (follow-ups)

- Export renderer perf snapshot into diagnostics bundles (so perf logs can include it next to `paint_time_us`).
- Add GPU timestamp queries (where supported) and export `gpu_render_us` for the “CPU looks fine but it hitches” class of bugs.
- Add explicit text atlas / eviction / upload counters (bytes + events) to correlate with spikes.

