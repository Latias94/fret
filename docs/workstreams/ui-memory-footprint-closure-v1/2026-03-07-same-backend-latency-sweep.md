# 2026-03-07 Same-backend latency sweep

## Goal

Measure how `FRET_RENDER_WGPU_SURFACE_DESIRED_MAX_FRAME_LATENCY` changes the same-backend comparison between:

- `wgpu_hello_world_control`
- `hello_world_compare_demo` full scene
- `hello_world_compare_demo` empty scene

The runner now records the shared env override directly in `manifest.json`, so these runs are reproducible without relying on shell-local context.

## Command template

```bash
python3 tools/run_wgpu_hello_world_control_vs_fret.py \
  --out-dir <out-dir> \
  --control-binary target-codex/release/wgpu_hello_world_control \
  --fret-binary target-codex/release/hello_world_compare_demo \
  --sample-at-secs 1,2,6,12 \
  --steady-offset-secs 6 \
  --shared-env FRET_RENDER_WGPU_SURFACE_DESIRED_MAX_FRAME_LATENCY=<1|2|3>
```

## Artifacts

- Baseline / default (`2`): `target/diag/wgpu-control-vs-fret-20260307-r1/summary/summary.json`
- Explicit `latency=1`:
  - `target/diag/wgpu-control-vs-fret-latency1-20260307-r1/summary/summary.json`
  - repeat: `target/diag/wgpu-control-vs-fret-latency1-20260307-r2/summary/summary.json`
- Explicit `latency=2`:
  - `target/diag/wgpu-control-vs-fret-latency2-20260307-r1/summary/summary.json`
- Explicit `latency=3`:
  - `target/diag/wgpu-control-vs-fret-latency3-20260307-r1/summary/summary.json`
  - repeat: `target/diag/wgpu-control-vs-fret-latency3-20260307-r2/summary/summary.json`

## Steady-state summary (6s)

### Control

The plain `wgpu` control is effectively invariant across the sweep:

- steady physical footprint stays around `31.5–32.0 MiB`
- steady graphics total stays around `12.8 MiB`
- steady Metal current allocated size stays around `9.5 MiB`
- presents stay flat at `2`

This makes the knob useful: most of the visible movement is on the Fret side rather than the raw backend baseline.

### Fret full scene

- `latency=2` reference:
  - `52.3–52.5 MiB` steady physical
  - `24.5 MiB` steady graphics total
  - `42.5 MiB` Metal current allocated size
  - `11.9 MiB` `IOSurface`
  - flat presents (`4` total by the first steady sample)
- `latency=1` repeats:
  - `51.9–52.1 MiB` steady physical
  - `23.4 MiB` steady graphics total
  - `38.5 MiB` Metal current allocated size
  - `8.0 MiB` `IOSurface`
  - flat presents (`5` total by the first steady sample)
- `latency=3` repeats split into two regimes:
  - one run matched the `latency=2` footprint (`52.5 / 24.5 / 42.5 / 11.9 MiB`)
  - one run matched the lower-footprint regime (`48.3 / 20.6 / 38.5 / 8.0 MiB`)

### Fret empty scene

- `latency=2` reference:
  - `46.5–46.6 MiB` steady physical
  - `19.2 MiB` steady graphics total
  - `38.3 MiB` Metal current allocated size
  - `11.9 MiB` `IOSurface`
  - flat presents (`4` total)
- `latency=1` repeats:
  - `42.3–45.5 MiB` steady physical
  - `15.3–18.1 MiB` steady graphics total
  - `34.4 MiB` Metal current allocated size
  - `8.0 MiB` `IOSurface`
  - flat presents (`4–5` total)
- `latency=3` repeats are also split:
  - one run matched the lower-footprint regime (`42.5 / 15.3 / 34.4 / 8.0 MiB`)
  - one run matched the higher-footprint regime (`46.9 / 19.3 / 38.3 / 11.9 MiB`)

## Readout

1. `desired_maximum_frame_latency` is a **real same-backend lever** for Fret on this machine.
   - The strongest movement lines up with `IOSurface` and `metal_current_allocated_size_bytes`, not with CPU heap buckets.
   - The plain `wgpu` control barely moves, so this is not just global measurement noise.
2. The lower-footprint regime is worth about:
   - roughly `-4 MiB` Metal current allocated size for both full and empty scenes
   - roughly `-3.9 MiB` `IOSurface`
   - about `-1–4 MiB` steady physical footprint depending on scene and run
3. The sweep also exposes **runtime/driver nondeterminism** for `latency=3`.
   - Two `latency=3` repeats landed in different steady-state regimes.
   - The internal runtime samples still show flat presents after the first steady sample; the difference is not a sustained idle-present regression.
   - This points more toward drawable/surface buffering state than toward a renewed app-owned redraw loop.
4. `latency=1` is the most consistently lower-footprint configuration so far.
   - It repeatedly lands in the lower `IOSurface` / Metal regime.
   - That makes it a good next knob for controlled follow-up experiments.

## Next step

Use the same runner to test the lower-footprint regime against one more surface-family knob so we can separate:

- drawable-count / surface-latency effects
- alpha/compositing / transparency effects
- render-target or intermediate-target pressure inside Fret itself

The most practical next comparison is `latency=1` plus one additional shared surface knob (for example present mode where supported), while keeping the same `1,2,6,12` sampling layout.
