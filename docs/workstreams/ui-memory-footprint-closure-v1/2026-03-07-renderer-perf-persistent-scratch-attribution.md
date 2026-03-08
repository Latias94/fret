# 2026-03-07 Renderer perf persistent-scratch attribution

## Goal

Close the remaining **internal diagnostics blind spot** for renderer-owned persistent scratch so the
same-backend active hello-world investigation can distinguish:

1. app-visible Metal allocations that Fret explicitly retains, versus
2. the still-unexplained hidden `Owned physical footprint (unmapped) (graphics)` plateau.

The immediate target is the previously identified `path_intermediate` suspect.

## Tooling changes

### Internal sampling surface

`apps/fret-examples/src/hello_world_compare_demo.rs` now emits additional renderer-perf fields in
`internal.gpu.json`, including:

- `path_intermediate_bytes_estimate`
- `path_intermediate_msaa_bytes_estimate`
- `path_intermediate_resolved_bytes_estimate`
- `custom_effect_v3_pyramid_scratch_bytes_estimate`
- `intermediate_pool_free_bytes`
- `intermediate_pool_free_textures`
- `path_msaa_samples_effective`
- `path_draw_calls`
- `clip_path_mask_cache_bytes_live`
- selected render-plan budget / “other live bytes” hints

### Renderer estimates

`crates/fret-render-wgpu` now computes best-effort byte estimates for:

- retained `PathIntermediate`
- retained `CustomEffectV3PyramidScratch`

These estimates are exposed through `RenderPerfSnapshot` so both direct sampling and diag bundles can
carry them.

### Compare runners / summarizers

- `tools/run_wgpu_hello_world_control_vs_fret.py`
- `tools/run_fret_vs_gpui_hello_world_compare.py`

now automatically enable `FRET_DIAG_RENDERER_PERF=1` for Fret compare runs.

- `tools/summarize_wgpu_hello_world_control_vs_fret.py`
- `tools/summarize_fret_vs_gpui_hello_world_compare.py`

now carry the new scratch fields into bounded summaries.

## Experiment

### Default path MSAA (`path_msaa=4`)

Artifact:

- `target/diag/wgpu-control-vs-fret-present-release-latency2-active-footprint-verbose-renderer-perf-scratch-single6-20260307-r1/summary/summary.json`
- `target/diag/wgpu-control-vs-fret-present-release-latency2-active-footprint-verbose-renderer-perf-scratch-single6-20260307-r1/summary/summary.md`

Command shape:

```sh
python3 tools/run_wgpu_hello_world_control_vs_fret.py \
  --out-dir target/diag/wgpu-control-vs-fret-present-release-latency2-active-footprint-verbose-renderer-perf-scratch-single6-20260307-r1 \
  --control-binary target/release/wgpu_hello_world_control \
  --fret-binary target/release/hello_world_compare_demo \
  --sample-at-secs 6 \
  --steady-offset-secs 6 \
  --fret-active-mode present-only \
  --capture-footprint-verbose \
  --shared-env FRET_RENDER_WGPU_SURFACE_DESIRED_MAX_FRAME_LATENCY=2
```

### `path_msaa=1` confirmation

Artifact:

- `target/diag/wgpu-control-vs-fret-present-release-latency2-active-footprint-verbose-renderer-perf-scratch-pathmsaa1-single6-20260307-r1/summary/summary.json`
- `target/diag/wgpu-control-vs-fret-present-release-latency2-active-footprint-verbose-renderer-perf-scratch-pathmsaa1-single6-20260307-r1/summary/summary.md`

Command shape:

```sh
python3 tools/run_wgpu_hello_world_control_vs_fret.py \
  --out-dir target/diag/wgpu-control-vs-fret-present-release-latency2-active-footprint-verbose-renderer-perf-scratch-pathmsaa1-single6-20260307-r1 \
  --control-binary target/release/wgpu_hello_world_control \
  --fret-binary target/release/hello_world_compare_demo \
  --sample-at-secs 6 \
  --steady-offset-secs 6 \
  --fret-active-mode present-only \
  --capture-footprint-verbose \
  --shared-env FRET_RENDER_WGPU_SURFACE_DESIRED_MAX_FRAME_LATENCY=2 \
  --fret-env FRET_RENDER_WGPU_PATH_MSAA_SAMPLES=1
```

## Results

### 1. The hello-world active scene allocates path-MSAA scratch even though it draws **zero** paths

At `6s`, both Fret rows report:

| case | `path_msaa_samples_effective` | `path_draw_calls` | `path_intermediate_bytes_estimate` |
| --- | ---: | ---: | ---: |
| `fret compare full` | `4` | `0` | `19.07 MiB` |
| `fret compare empty` | `4` | `0` | `19.07 MiB` |

Breakdown:

- MSAA scratch: `15.26 MiB`
- resolved scratch: `3.81 MiB`
- `custom_effect_v3_pyramid_scratch_bytes_estimate`: `0`
- `clip_path_mask_cache_bytes_live`: `0`
- `intermediate_peak_in_use_bytes`: `0`
- `intermediate_pool_free_bytes`: `0`

So the current active hello-world scene is paying a **real ~19 MiB renderer-owned Metal cost** for a
path pipeline that is not used by the scene at all.

### 2. Forcing `path_msaa=1` removes exactly that app-visible slice

At `6s`:

| case | default Metal | `path_msaa=1` Metal | delta | `path_intermediate_bytes_estimate` |
| --- | ---: | ---: | ---: | ---: |
| `fret compare full` | `42.45 MiB` | `22.30 MiB` | `-20.15 MiB` | `19.07 MiB -> 0` |
| `fret compare empty` | `38.33 MiB` | `18.17 MiB` | `-20.16 MiB` | `19.07 MiB -> 0` |

This closes the app-visible attribution loop:

- the previously observed `~20 MiB` Metal drop is the retained `path_intermediate` scratch,
- not the intermediate pool,
- not custom-effect pyramid scratch,
- and not clip-mask cache.

### 3. The hidden dirty `4 MiB × 50/52` plateau is still **not** explained by that scratch

The dirty same-backend plateau remains effectively unchanged:

| case | default owned gfx dirty | `path_msaa=1` owned gfx dirty | dominant dirty bucket |
| --- | ---: | ---: | --- |
| `fret compare full` | `217.25 MiB` | `217.25 MiB` | `4.0 MiB × 52` |
| `fret compare empty` | `204.81 MiB` | `204.81 MiB` | `4.0 MiB × 50` |

So `path_intermediate` is a **visible Metal optimization target**, but still not the root cause of the
hidden dirty plateau.

### 4. What does change in `footprint -v` is the **virtual-only** side of `owned_unmapped_graphics`

Comparing the `footprint -v` virtual-page buckets shows that default `path_msaa=4` carries two extra
rows that disappear under `path_msaa=1`:

- `1032` pages = `16,908,288` bytes (`~16.12 MiB`)
- `258` pages = `4,227,072` bytes (`~4.03 MiB`)

Those rows match the new renderer-perf estimate closely (`~19.07 MiB` total) and explain why:

- `metal_current_allocated_size_bytes` drops by ~20 MiB,
- `owned_unmapped_graphics.rows_total` falls by `2`,
- but the dominant dirty `4 MiB × 50/52` family stays flat.

Working interpretation:

- retained path scratch does leak into the broader `owned_unmapped_graphics` family,
- but primarily as **virtual / reservation** rows rather than as the dirty `4 MiB` bucket we are still
  trying to close.

## Conclusion

This closes an important diagnostic blind spot.

We can now say much more precisely:

1. `path_intermediate` is definitely real and worth optimizing.
2. On the current hello-world active scene, it is **unconditional overhead** (`path_draw_calls=0`).
3. It explains the app-visible `~20 MiB` Metal slice.
4. It does **not** explain the hidden dirty `4 MiB × 50/52` plateau.

## Recommended next step

The most actionable product change is now straightforward:

- stop eagerly allocating `path_intermediate` when the frame has no path-MSAA work,
- then rerun the same same-backend pair to turn this diagnostic finding into a shipped memory win.

That optimization should improve active Metal footprint by ~20 MiB on path-free scenes, while the
remaining hidden closure work continues separately.
