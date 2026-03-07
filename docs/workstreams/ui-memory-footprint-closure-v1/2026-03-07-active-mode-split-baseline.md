# 2026-03-07 Active mode split baseline

## Goal

Separate three different "active" costs that were previously conflated:

1. `present-only`: the app keeps presenting, but the declarative tree is not rebuilt every frame.
2. `paint-model`: the app rebuilds every frame and mutates paint-only state.
3. `layout-model`: the app rebuilds every frame and mutates layout-affecting state.

That makes the active same-backend question much sharper:

- how much of the high active plateau is just continuous present / swapchain / driver residency,
- and how much extra is added by real per-frame rerender/layout work.

## Implementation on current head

### Compare demo modes

`hello_world_compare_demo` now accepts:

- `FRET_HELLO_WORLD_COMPARE_ACTIVE_MODE=idle`
- `FRET_HELLO_WORLD_COMPARE_ACTIVE_MODE=present-only`
- `FRET_HELLO_WORLD_COMPARE_ACTIVE_MODE=paint-model`
- `FRET_HELLO_WORLD_COMPARE_ACTIVE_MODE=layout-model`

Compatibility:

- legacy `FRET_HELLO_WORLD_COMPARE_CONTINUOUS_REDRAW=1` now maps to `present-only`.

Semantics:

- `present-only`
  - holds a declarative continuous-frames lease via `set_continuous_frames(...)`
  - does **not** request a declarative rerender every frame
- `paint-model`
  - requests `request_animation_frame()` each frame
  - mutates only paint-visible state (background color wave)
- `layout-model`
  - requests `request_animation_frame()` each frame
  - mutates layout-affecting state (an always-present same-color layout probe that changes height)

This means the internal sidecar can now distinguish a true per-frame rerender (`render_count ~ present_count`) from a pure present loop (`render_count` stays near startup while runner presents continue climbing).

### Internal report additions

`hello_world_compare_demo` internal GPU samples now also capture:

- `surface`
  - effective `present_mode`
  - `desired_maximum_frame_latency`
  - `format`
  - `alpha_mode`
  - `configure_count`
- `renderer_perf`
  - `gpu_images_bytes_estimate`
  - `gpu_render_targets_bytes_estimate`
  - `intermediate_peak_in_use_bytes`
  - `render_plan_estimated_peak_intermediate_bytes`
  - related steady-state renderer counters

### Runner/tooling support

- `crates/fret-launch/src/runner/desktop/runner/effects.rs`
  - desktop RAF pacing remains fixed: RAF intent lands in `raf_windows`, and `about_to_wait` remains the pacing point.
- `tools/run_wgpu_hello_world_control_vs_fret.py`
  - now accepts `--fret-active-mode`
- `tools/summarize_wgpu_hello_world_control_vs_fret.py`
  - summary tables now include active mode, effective surface info, and renderer-perf bytes
- New diag scripts:
  - `tools/diag-scripts/tooling/hello-world/hello-world-compare-active-memory-steady.json` (`present-only`)
  - `tools/diag-scripts/tooling/hello-world/hello-world-compare-paint-model-memory-steady.json`
  - `tools/diag-scripts/tooling/hello-world/hello-world-compare-layout-model-memory-steady.json`

## Command template

```bash
python3 tools/run_wgpu_hello_world_control_vs_fret.py \
  --out-dir target/diag/wgpu-control-vs-fret-active-redraw-<stamp> \
  --control-binary target/debug/wgpu_hello_world_control \
  --fret-binary target/debug/hello_world_compare_demo \
  --sample-at-secs 1,2,6 \
  --steady-offset-secs 6 \
  --control-env FRET_WGPU_HELLO_WORLD_CONTROL_CONTINUOUS_REDRAW_INTERVAL_MS=8 \
  --fret-active-mode paint-model \
  --fret-env FRET_DIAG_RENDERER_PERF=1
```

## Final local artifacts (2026-03-07)

- `present-only`
  - `target/diag/wgpu-control-vs-fret-active-redraw-debug-20260307-r13-present-final/summary/summary.json`
- `paint-model`
  - `target/diag/wgpu-control-vs-fret-active-redraw-debug-20260307-r14-paint-final/summary/summary.json`
- `layout-model`
  - `target/diag/wgpu-control-vs-fret-active-redraw-debug-20260307-r15-layout-final/summary/summary.json`

## Steady snapshot at 6s

| mode | case | redraws | presents | physical | graphics | internal Metal |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| `present-only` | `wgpu control` | `554` | `553` | `149.8 MiB` | `130.9 MiB` | `13.4 MiB` |
| `present-only` | `fret compare empty` | `2` | `570` | `255.6 MiB` | `227.4 MiB` | `38.3 MiB` |
| `present-only` | `fret compare full` | `2` | `526` | `263.4 MiB` | `235.1 MiB` | `42.5 MiB` |
| `paint-model` | `wgpu control` | `560` | `560` | `150.3 MiB` | `130.9 MiB` | `13.4 MiB` |
| `paint-model` | `fret compare empty` | `574` | `573` | `252.6 MiB` | `224.6 MiB` | `38.3 MiB` |
| `paint-model` | `fret compare full` | `571` | `570` | `268.4 MiB` | `238.0 MiB` | `42.5 MiB` |
| `layout-model` | `wgpu control` | `536` | `536` | `150.3 MiB` | `130.9 MiB` | `13.4 MiB` |
| `layout-model` | `fret compare empty` | `567` | `566` | `252.8 MiB` | `224.7 MiB` | `38.3 MiB` |
| `layout-model` | `fret compare full` | `564` | `563` | `270.6 MiB` | `240.2 MiB` | `42.5 MiB` |

## What this now says

### 1. The high active plateau is real, but mostly not caused by hello-world content

At `6s`, versus the same-backend `wgpu control`:

- `present-only`
  - `empty`: about `+105.8 MiB` physical / `+96.4 MiB` graphics / `+24.9 MiB` internal Metal
  - `full`: about `+113.6 MiB` physical / `+104.2 MiB` graphics / `+29.0 MiB` internal Metal
- `paint-model`
  - `empty`: about `+102.3 MiB` physical / `+93.7 MiB` graphics / `+24.9 MiB` internal Metal
  - `full`: about `+118.1 MiB` physical / `+107.1 MiB` graphics / `+29.0 MiB` internal Metal
- `layout-model`
  - `empty`: about `+102.5 MiB` physical / `+93.8 MiB` graphics / `+24.9 MiB` internal Metal
  - `full`: about `+120.3 MiB` physical / `+109.2 MiB` graphics / `+29.0 MiB` internal Metal

So the big active-vs-control gap already exists in `present-only`. Real per-frame rerender/layout increases the full scene further, but only by single-digit MiB over the `present-only` plateau.

### 2. True per-frame rerender is now verified

The old intermediate run accidentally kept `paint/layout` in a pure present loop. That is now fixed.

At the `6s` sample on current head:

- `present-only`: `render_count=2`, `runner_present.total_present_count=526`
- `paint-model`: `render_count=571`, `runner_present.total_present_count=570`
- `layout-model`: `render_count=564`, `runner_present.total_present_count=563`

So `paint/layout` are now real per-frame declarative rebuild paths, not just continuous present with static content.

### 3. Full-vs-empty still stays modest

At `6s` (`full` minus `empty`):

- `present-only`: about `+7.8 MiB` physical / `+7.8 MiB` graphics / `+4.1 MiB` internal Metal
- `paint-model`: about `+15.8 MiB` physical / `+13.4 MiB` graphics / `+4.1 MiB` internal Metal
- `layout-model`: about `+17.8 MiB` physical / `+15.4 MiB` graphics / `+4.1 MiB` internal Metal

That still does **not** look like "fonts/text alone explain the active floor." Content matters, but it is not the first-order driver of the large active baseline.

### 4. Surface + renderer-perf evidence narrows the explanation

Steady internal samples on current head show:

- effective surface config is stable:
  - `present_mode=Fifo`
  - `desired_maximum_frame_latency=2`
  - `configure_count=1`
- `renderer_perf` remains near-zero for this minimal scene:
  - `gpu_images_bytes_estimate=0`
  - `gpu_render_targets_bytes_estimate=0`
  - `intermediate_peak_in_use_bytes=0`
  - `render_plan_estimated_peak_intermediate_bytes=0`

So, for this hello-world-class case, the active delta is **not** being explained by imported images, extra renderer-owned render targets, or intermediate-effect buffers.

## Working interpretation

For the minimal compare scene on current head, the remaining active plateau now looks much more like:

- continuous present / swapchain / drawable residency,
- backend / driver / WindowServer bookkeeping that scales with an always-active window,
- plus a smaller incremental cost when Fret really rebuilds and repaints every frame.

What is still *not* closed is the large non-Metal residual inside the macOS-visible graphics bucket. The internal report now rules out several obvious Fret-owned causes for this tiny scene, but it does not yet split that residual into swapchain vs driver-private vs OS-reserved categories.

## Recommended next steps

1. Run the same active-mode matrix against a same-backend external framework control (GPUI / another `wgpu` UI) so the comparison is truly apples-to-apples beyond the raw `wgpu` control.
2. Repeat `r13/r14/r15` enough times to estimate variance and define realistic future perf/memory gates for each active mode.
3. Use Apple-side attribution (`Virtual Memory Trace + Metal Application` / `VM Tracker`) on the final active-mode matrix to split the remaining graphics residual into swapchain / Metal driver / OS reservation categories.
