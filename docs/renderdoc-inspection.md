# RenderDoc Inspection (Scriptable)

This repo uses RenderDoc for GPU debugging, but the default GUI workflow does not scale well when we need
to repeatedly validate pass state against our rendering contracts (viewport/scissor, clip stack, mask
viewport mapping, effect boundaries).

To make this repeatable, we provide:

- `apps/fret-renderdoc`: a small CLI that runs `qrenderdoc --python`
- `tools/renderdoc/fret_dump_pass_state_json.py`: a RenderDoc Python script that dumps pass state to JSON

## Prerequisites

- RenderDoc installed.
- A capture file (`.rdc`) to inspect.

RenderDoc discovery:

- Preferred: set `RENDERDOG_RENDERDOC_DIR` to the RenderDoc install root (contains `qrenderdoc`).
- Or pass `--renderdoc-dir` to `fret-renderdoc`.

## Quick start

```bash
cargo run -p fret-renderdoc -- dump --capture .fret/renderdoc-autocap/fret_capture.rdc --marker "fret clip mask pass"
```

The command prints the path to `fret_dump_pass_state_json.response.json`.

## Frame health summary (pass breakdown)

For a quick sanity-check of pass breakdowns (without digging into uniform dumps), run with an empty marker
and a high match limit:

```bash
cargo run -p fret-renderdoc -- dump --capture .fret/renderdoc-autocap/fret_capture.rdc --marker "" --max-results 200000 --no-uniform-bytes --no-outputs-png
```

Then inspect:

- `result.summary.matches_count`: total matched draw/dispatch events (bounded by `--max-results`).
- `result.summary.top_marker_paths`: top marker paths by drawcall count (useful to confirm expected pass mix).
- `result.summary.top_leaf_markers`: top leaf marker names (useful when paths are too noisy).

Tip: if you only care about `fret-*` passes, search for `"fret"` inside `top_marker_paths` entries.

## What to look for (examples)

### Clip mask generation (`fret clip mask pass`)

Validate:

- `raster_state.scissors[0]` is the mask target sub-rect being rendered.
- `buffer_dump.named_buffers` contains `clip_mask_params.dst_size` matching the mask target size.
- `buffer_dump.selected_uniform_entry.entry.mask_viewport_origin/size` matches the effect viewport rect.

The exported PNG is a single-channel mask (`R8Unorm`), so it may look "red" in the file viewer. This is
expected.

### Pixelate scale passes (`fret downsample-nearest pass` / `fret upscale-nearest pass`)

Validate:

- `buffer_dump.named_buffers[scale_params].scale` matches the effect pixelation scale.
- `buffer_dump.named_buffers[scale_params].src_origin/dst_origin` match the intended mapping:
  - Downsample into effect-local target: `src_origin = effect_rect.xy`, `dst_origin = (0, 0)`.
  - Upscale back into full-size target: `src_origin = (0, 0)`, `dst_origin = effect_rect.xy`.
- `raster_state.scissors[0]` matches the expected effect region (for scissored passes).

Note: on some Vulkan captures, RenderDoc's python API does not reliably surface dynamic uniform buffer
offsets for WGSL pipelines. In that case, `tools/renderdoc/fret_dump_pass_state_json.py` falls back to
inferring the correct `ScaleParams` slot by counting earlier `"nearest"` drawcalls in the frame. The
JSON response includes `offset_source` to make this explicit.

If the output looks "anchored to the window origin", this usually indicates a missing origin adjustment in
the fullscreen shader when scissoring is used.
