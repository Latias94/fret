# 2026-03-07 Path MSAA triage + upstream transient-texture notes

## Goal

Use one source-side audit plus one minimal runtime experiment to answer two narrower questions:

1. Does Fret currently allocate any obvious full-surface renderer-owned resource that is not shown by
   the renderer's own image / render-target / intermediate counters?
2. Does that candidate explain the hidden `Owned physical footprint (unmapped) (graphics)` plateau,
   or only a smaller visible Metal slice?

## Upstream clues

Two upstream data points are directly relevant to the current search shape.

### 1. `wgpu` 28 already exposes transient textures, and Metal maps them to memoryless storage

In the local `wgpu` sources currently used by this workspace:

- `wgpu-types-28.0.0/src/texture.rs` documents `TextureUsages::TRANSIENT` as a way to reduce memory
  usage when attachment contents are not reused, especially on mobile / Apple platforms.
- `wgpu-hal-28.0.0/src/metal/device.rs` maps `TextureUses::TRANSIENT` to
  `MTLStorageMode::Memoryless` when the Metal backend reports support.

This lines up with upstream issue:

- `wgpu` issue `#8247`: "Add support for transient textures on Vulkan and Metal"
  - <https://github.com/gfx-rs/wgpu/issues/8247>

That issue explicitly notes that on Metal, transient textures can reduce texture-cache / backing
store footprint.

### 2. There are older Metal memory / leak reports, but they do not match the current plateau shape

An older upstream report exists here:

- `wgpu` issue `#5291`: "Memory leaks with Metal"
  - <https://github.com/gfx-rs/wgpu/issues/5291>

That is still useful context, but it does **not** match the current Fret evidence particularly well:

- our plateau is steady and repeatable rather than obviously unbounded,
- the dominant bucket is hidden graphics-owned unmapped memory, not a classic growing app heap leak,
- and GPUI also shows the same `4 MiB` graphics-owned family, just at a much lower count.

So the best current interpretation is not "we found a generic Metal leak". It is still "Fret keeps too
many full-surface-sized hidden graphics objects".

## Source-side audit

A local code audit points to one immediately suspicious always-sized-to-viewport resource.

### `path_intermediate` is created eagerly whenever path MSAA is enabled

Relevant local code:

- `crates/fret-render-wgpu/src/renderer/render_scene/frame_pipelines.rs`
- `crates/fret-render-wgpu/src/renderer/pipelines/path_intermediate.rs`

Current behavior:

- `ensure_frame_pipelines_and_path_samples()` computes `path_samples`
- if `path_samples > 1`, it always calls `ensure_path_intermediate(device, viewport_size, format, path_samples)`
- `ensure_path_intermediate()` allocates:
  - one full-viewport resolved texture
  - plus one full-viewport MSAA texture when `sample_count > 1`

Those resources are renderer-owned, but they do **not** flow through the registered image / render-
target accounting surfaced by `gpu_images_bytes_estimate` / `gpu_render_targets_bytes_estimate`.

So this is a real blind spot in the current internal counters.

### But current pass semantics also block the obvious transient path

In `crates/fret-render-wgpu/src/renderer/render_scene/recorders/path_msaa.rs`, the path-MSAA pass
keeps:

- `store: wgpu::StoreOp::Store`

with an in-code note that `Discard` caused Vulkan robustness problems in the presence of a resolve
attachment.

That matters because `TextureUsages::TRANSIENT` requires `StoreOp::Discard`, so the most obvious
transient / memoryless optimization path is currently blocked by cross-backend safety constraints.

## Minimal experiment

### Run

- Script: `python3 tools/run_wgpu_hello_world_control_vs_fret.py`
- Mode: release/release, cadence-matched `present-only`
- Shared env: `FRET_RENDER_WGPU_SURFACE_DESIRED_MAX_FRAME_LATENCY=2`
- Control env: `FRET_WGPU_HELLO_WORLD_CONTROL_CONTINUOUS_REDRAW_INTERVAL_MS=8`
- Fret-only env: `FRET_RENDER_WGPU_PATH_MSAA_SAMPLES=1`
- Extra runner flags:
  - `--capture-vmmap-regions`
  - `--capture-footprint-verbose`

Preferred artifact:

- `target/diag/wgpu-control-vs-fret-present-release-latency2-active-footprint-verbose-pathmsaa1-single6-20260307-r1/summary/summary.json`

## 6s steady snapshot vs the earlier `path_msaa=4` default

### `fret compare empty`

- default path MSAA (`4`):
  - internal Metal: `38.3 MiB`
  - `Owned ... (graphics)`: `204.0 MiB`
  - top bucket: `4.0 MiB × 50`
- forced `path_msaa=1`:
  - internal Metal: `18.2 MiB`
  - `Owned ... (graphics)`: `204.8 MiB`
  - top bucket: `4.0 MiB × 50`

### `fret compare full`

- default path MSAA (`4`):
  - internal Metal: `42.5 MiB`
  - `Owned ... (graphics)`: `218.1 MiB`
  - top bucket: `4.0 MiB × 52`
- forced `path_msaa=1`:
  - internal Metal: `22.3 MiB`
  - `Owned ... (graphics)`: `218.1 MiB`
  - top bucket: `4.0 MiB × 52`

## Findings

### 1. Path MSAA explains a large **visible Metal** slice

Forcing `FRET_RENDER_WGPU_PATH_MSAA_SAMPLES=1` reduces app-visible internal Metal by about:

- `~20.1 MiB` on `empty`
- `~20.2 MiB` on `full`

So the eager path-intermediate allocation is real and materially affects `metal_current_allocated_size`.

### 2. But it does **not** explain the hidden `Owned ... (graphics)` plateau

Despite that visible Metal drop, the hidden local bucket stays effectively flat:

- `Owned physical footprint (unmapped) (graphics)` remains around `~204–218 MiB`
- the dominant `4.0 MiB` bucket count remains at `×50` / `×52`

So path MSAA is a good optimization candidate for app-visible Metal, but it is **not** the main
explanation for the hidden full-surface object-count inflation.

### 3. The renderer's current diagnostics surface is missing at least one meaningful class of resources

Because the path-intermediate textures clearly move `metal_current_allocated_size`, but do not appear
in the renderer's registered image / render-target counters, the current internal diagnostics still
have a blind spot around renderer-owned scratch / pipeline-side attachments.

That is useful even though it does not close the hidden bucket itself.

## Readout

This narrows the source-side picture in a useful way.

- There **is** a concrete renderer-owned full-surface allocation path (`path_intermediate`) that the
  current internal counters do not expose.
- Disabling path MSAA removes about `~20 MiB` of visible internal Metal.
- But the large hidden `Owned physical footprint (unmapped) (graphics)` plateau and the repeated
  `4 MiB` object count remain essentially unchanged.

So path MSAA is a secondary optimization opportunity, not the root cause of the hidden bucket.

## Recommended next steps

1. Keep `path_intermediate` / transient-texture support on the optimization list, because it can
   still cut visible Metal residency substantially.
2. Do **not** treat it as closure for the hidden bucket.
3. Continue the remaining search around hidden full-surface object-count inflation, now with the
   stronger claim that the unexplained plateau survives even after the obvious path-MSAA scratch is
   removed.
