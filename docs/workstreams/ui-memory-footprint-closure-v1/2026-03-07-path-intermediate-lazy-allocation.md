# 2026-03-07 Path-intermediate lazy allocation

## Goal

Turn the previously isolated `path_intermediate` overhead into a shipped optimization instead of a
pure diagnosis. The target is the hello-world active scene where Fret keeps path-MSAA scratch alive
while `path_draw_calls=0`.

## Implementation

### Renderer changes

- `crates/fret-render-wgpu/src/renderer/render_scene/frame_pipelines.rs`
  - stop eagerly calling `ensure_path_intermediate(...)` during pipeline warm-up,
  - keep the path-MSAA and composite pipelines eagerly created (low-risk behavior-preserving part).
- `crates/fret-render-wgpu/src/renderer/render_scene/execute.rs`
  - compile the render plan first,
  - then call `sync_path_intermediate_for_plan(...)`,
  - release the retained scratch (`self.path_intermediate = None`) when the compiled plan contains no
    `RenderPlanPass::PathMsaaBatch`.
- `crates/fret-render-wgpu/src/renderer/pipelines/path_intermediate.rs`
  - add the plan-pass detector used by the new sync point,
  - keep the existing byte-estimate helpers,
  - add a focused unit test covering the pass detector.
- `crates/fret-render-wgpu/src/renderer/config.rs`
  - drop stale `path_intermediate` scratch immediately when the requested path-MSAA sample count
    changes, so runtime config changes do not keep the old scratch alive until the next rebuild.

### Gate hardening

The compare runners previously assumed `internal.gpu.json` existed immediately after the external
sampler exited. In practice the compare demo writes that file from a background internal-sampling
thread, so the gate could fail on a small race even when the sample itself succeeded.

- `tools/run_wgpu_hello_world_control_vs_fret.py`
- `tools/run_fret_vs_gpui_hello_world_compare.py`

now wait up to a short configurable grace window (`--internal-report-wait-secs`, default `2.0`)
before treating a missing internal report as fatal.

## Validation

### Code-level checks

```sh
cargo test -p fret-render-wgpu path_msaa --lib
cargo build -p fret-demo --release --bin hello_world_compare_demo --bin wgpu_hello_world_control
python3 -m py_compile \
  tools/run_wgpu_hello_world_control_vs_fret.py \
  tools/run_fret_vs_gpui_hello_world_compare.py
```

### Same-backend active compare (authoritative memory numbers)

Artifact:

- `target/diag/wgpu-control-vs-fret-present-release-latency2-active-footprint-verbose-renderer-perf-scratch-lazy-path-intermediate-single6-20260307-r2/summary/summary.json`
- `target/diag/wgpu-control-vs-fret-present-release-latency2-active-footprint-verbose-renderer-perf-scratch-lazy-path-intermediate-single6-20260307-r2/summary/summary.md`

Command shape:

```sh
python3 tools/run_wgpu_hello_world_control_vs_fret.py \
  --out-dir target/diag/wgpu-control-vs-fret-present-release-latency2-active-footprint-verbose-renderer-perf-scratch-lazy-path-intermediate-single6-20260307-r2 \
  --control-binary target/release/wgpu_hello_world_control \
  --fret-binary target/release/hello_world_compare_demo \
  --sample-at-secs 6 \
  --steady-offset-secs 6 \
  --post-sample-wait-secs 2 \
  --fret-active-mode present-only \
  --capture-footprint-verbose \
  --shared-env FRET_RENDER_WGPU_SURFACE_DESIRED_MAX_FRAME_LATENCY=2
```

### Gate-race retest (no extra sampler wait)

Artifact:

- `target/diag/wgpu-control-vs-fret-present-release-latency2-active-footprint-verbose-renderer-perf-scratch-lazy-path-intermediate-single6-20260307-r3/summary/summary.json`

Command shape:

```sh
python3 tools/run_wgpu_hello_world_control_vs_fret.py \
  --out-dir target/diag/wgpu-control-vs-fret-present-release-latency2-active-footprint-verbose-renderer-perf-scratch-lazy-path-intermediate-single6-20260307-r3 \
  --control-binary target/release/wgpu_hello_world_control \
  --fret-binary target/release/hello_world_compare_demo \
  --sample-at-secs 6 \
  --steady-offset-secs 6 \
  --fret-active-mode present-only \
  --capture-footprint-verbose \
  --shared-env FRET_RENDER_WGPU_SURFACE_DESIRED_MAX_FRAME_LATENCY=2
```

This retest finishes successfully without the earlier `missing internal report` false negative.

## Results

### 1. Path-free hello-world rows now retain **zero** path scratch

At `6s`, both active Fret rows now report:

- `path_msaa_samples_effective = 4`
- `path_draw_calls = 0`
- `path_intermediate_bytes_estimate = 0`
- `path_intermediate_msaa_bytes_estimate = 0`
- `path_intermediate_resolved_bytes_estimate = 0`

So the renderer now keeps path MSAA enabled as a capability, but no longer pays the full-viewport
scratch cost when the compiled plan has no path-MSAA work.

### 2. The app-visible Metal win lands exactly where expected

Compared against the previous pre-change baseline
`target/diag/wgpu-control-vs-fret-present-release-latency2-active-footprint-verbose-renderer-perf-scratch-single6-20260307-r1/summary/summary.json`:

| case | pre-change Metal | post-change Metal | delta | path scratch |
| --- | ---: | ---: | ---: | ---: |
| `fret compare full` | `42.45 MiB` | `22.30 MiB` | `-20.16 MiB` | `20,000,000 -> 0` |
| `fret compare empty` | `38.33 MiB` | `18.17 MiB` | `-20.16 MiB` | `20,000,000 -> 0` |

This is the shipped version of the previously diagnosed `path_intermediate` win.

### 3. The hidden dirty plateau remains a separate problem

The dominant dirty `Owned physical footprint (unmapped) (graphics)` bucket does not materially move:

- `fret compare full`: still dominated by `4.0 MiB × 52` (`217.25 MiB -> 218.11 MiB`, effectively
  noise at this level)
- `fret compare empty`: still dominated by `4.0 MiB × 50` (`204.81 MiB -> 204.81 MiB`)

So this change closes the **visible Metal** slice but does not close the remaining hidden graphics
object-count inflation.

## Conclusion

This work lands two concrete outcomes:

1. A real renderer optimization: path-free scenes recover about `~20 MiB` of active app-visible Metal.
2. A more reliable compare gate: internal-report collection no longer fails on the small post-sampler
   race that surfaced during validation.

The remaining memory-closure question is now narrower and cleaner: after removing the known
`path_intermediate` overhead, why does Fret still keep roughly `22–24` extra `4 MiB` graphics-owned
unmapped objects compared with the cadence-matched control / GPUI baselines?

## Recommended next step

Re-run the same-scene `Fret vs GPUI` matrix with this optimization landed so the remaining delta is
re-baselined after the `~20 MiB` visible Metal slice is gone.
