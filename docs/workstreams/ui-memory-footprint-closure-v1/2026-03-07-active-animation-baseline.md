# 2026-03-07 Active animation baseline

> Follow-up: the final active-mode split baseline (`present-only` vs `paint-model` vs `layout-model`) now lives in `docs/workstreams/ui-memory-footprint-closure-v1/2026-03-07-active-mode-split-baseline.md`. This document keeps the first active baseline + RAF pacing fix evidence.

## Goal

Establish a first repeatable memory baseline for a legitimately active app:

- the app keeps presenting because it is intentionally animating,
- not because an idle-only redraw bug is leaking into the steady state.

The baseline should answer two questions:

1. How much higher is the steady footprint when the same minimal scene is deliberately kept active?
2. Does the active path plateau, or does it keep drifting upward over time?

## Setup

### Fret compare demo

`hello_world_compare_demo` now accepts:

- `FRET_HELLO_WORLD_COMPARE_CONTINUOUS_REDRAW=1`

When enabled, the demo now enables a declarative continuous-frames lease via `set_continuous_frames(...)` rather than manually re-requesting redraw from `render()`. That keeps the active path aligned with the runtime contract while still making deliberate frame-driving explicit in the sidecar (`scene.continuous_redraw=true`).

### Same-backend runner

`tools/run_wgpu_hello_world_control_vs_fret.py` now accepts per-target env overrides:

- `--control-env KEY=VALUE`
- `--fret-env KEY=VALUE`

This keeps the control/Fret comparison apples-to-apples even when the env variable names differ. For cadence-matched active runs on current head, prefer `FRET_WGPU_HELLO_WORLD_CONTROL_CONTINUOUS_REDRAW_INTERVAL_MS=8` on the control side so it matches the desktop runner's default `frame_interval`.

## First command template

```bash
python3 tools/run_wgpu_hello_world_control_vs_fret.py \
  --out-dir target/diag/wgpu-control-vs-fret-active-redraw-<stamp> \
  --control-binary target/release/wgpu_hello_world_control \
  --fret-binary target/release/hello_world_compare_demo \
  --sample-at-secs 1,2,6 \
  --steady-offset-secs 6 \
  --control-env FRET_WGPU_HELLO_WORLD_CONTROL_CONTINUOUS_REDRAW_INTERVAL_MS=8 \
  --fret-env FRET_HELLO_WORLD_COMPARE_CONTINUOUS_REDRAW=1
```

## Fret-only diag script

For a framework-owned bundle capture under deliberate activity:

- script: `tools/diag-scripts/tooling/hello-world/hello-world-compare-active-memory-steady.json`
- redirect: `tools/diag-scripts/hello-world-compare-active-memory-steady.json`

Suggested runner:

```bash
cargo run -p fretboard-dev -- diag run \
  tools/diag-scripts/tooling/hello-world/hello-world-compare-active-memory-steady.json \
  --launch -- cargo run -p fret-demo --bin hello_world_compare_demo
```

## What to look for

- `runner_present.total_present_count` should keep increasing because the app is active on purpose.
- `macos_physical_footprint_bytes`, `graphics_total_bytes`, and `metal_current_allocated_size_bytes`
  should reach a bounded plateau instead of climbing with a persistent positive slope.
- If the active path stabilizes at a higher floor, treat that as legitimate residency / buffering cost.
- If the active path keeps drifting, investigate caches, intermediate targets, or per-frame allocation churn.


## Current findings (local 2026-03-07)

### r5: deliberate activity confirmed

- Artifact: `target/diag/wgpu-control-vs-fret-active-redraw-debug-20260307-r5/summary/summary.json`
- The compare demo now clearly enters a deliberate active-present mode: `runner_present.total_present_count` climbs instead of staying flat.
- That run also exposed a desktop runner pacing bug: `Effect::RequestAnimationFrame` requested an immediate redraw in `effects.rs`, and `about_to_wait` requested another redraw again. On a light scene (`empty`), that self-spin drove about `5176` presents in `6s`, far above the control's `288` presents.

### r6: desktop RAF self-spin removed

- Artifact: `target/diag/wgpu-control-vs-fret-active-redraw-debug-20260307-r6/summary/summary.json`
- `crates/fret-launch/src/runner/desktop/runner/effects.rs` now records `Effect::RequestAnimationFrame` into `raf_windows` but leaves the actual redraw wake-up to the existing `about_to_wait` pacing path.
- Result: active cadence no longer runs away into thousands of presents per `6s`; the compare demo now settles near the desktop runner's own pacing instead of busy-spinning.

### r7: cadence-matched same-backend baseline (`control=8ms`)

- Artifact: `target/diag/wgpu-control-vs-fret-active-redraw-debug-20260307-r7-control8ms/summary/summary.json`
- Steady-state snapshot at `6s`:

| case | presents @6s | physical | graphics | internal Metal |
| --- | ---: | ---: | ---: | ---: |
| `wgpu control` | `563` | `150.3 MiB` | `130.9 MiB` | `13.4 MiB` |
| `fret compare empty` | `575` | `255.7 MiB` | `227.3 MiB` | `38.3 MiB` |
| `fret compare full` | `577` | `266.7 MiB` | `237.9 MiB` | `42.5 MiB` |

- With cadence matched, the active compare scenes are still materially higher than the plain `wgpu` control, but they now plateau instead of showing a runaway slope.
- Active same-backend delta versus control is now about:
  - `empty`: `+105.4 MiB` physical / `+96.4 MiB` graphics / `+24.9 MiB` internal Metal
  - `full`: `+116.4 MiB` physical / `+107.0 MiB` graphics / `+29.0 MiB` internal Metal
- The scene-content delta remains small even under activity (`full` vs `empty` is only about `+11.0 MiB` physical / `+10.6 MiB` graphics / `+4.1 MiB` internal Metal), so fonts/text/swatches are not the first-order explanation for the active floor in this hello-world-class repro.

## Interpretation

- Yes: a legitimately active app can hold a much higher steady memory plateau than the post-fix idle baseline.
- No: on current head, that higher active plateau no longer looks like the earlier idle-only leak/regression. After the RAF pacing fix, the active same-backend runs stabilize at a bounded level.
- This baseline is still intentionally minimal: it proves the cost of continuous present / active frame pacing. It does **not** yet measure a fully rerendered-per-frame view tree with real model churn. That needs a second active scenario if we want to separate “continuous present residency” from “true per-frame content rebuild/layout cost.”

## Next steps

1. Add a second active scenario with real per-frame content mutation:
   - one paint-only animation,
   - one layout-affecting animation,
   so active-present residency can be separated from rerender/layout cost.
2. Surface the effective Fret surface config and renderer target/intermediate bytes in the compare-demo internal report, so the remaining active delta can be split into swapchain/drawable pressure vs renderer-owned intermediates.
3. Re-run the active same-backend sweep with targeted knobs (`desired_maximum_frame_latency`, path MSAA, transparency/compositing) only after the new active baseline is in place.
