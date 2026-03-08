# 2026-03-07 Same-backend control baseline

## Goal

Record a reproducible apples-to-apples comparison between:

- `wgpu_hello_world_control` (plain `wgpu` + `fret-render` surface wiring)
- `hello_world_compare_demo` full scene
- `hello_world_compare_demo` empty scene (`FRET_HELLO_WORLD_COMPARE_NO_TEXT=1`, `FRET_HELLO_WORLD_COMPARE_NO_SWATCHES=1`)

The purpose of this run is to separate **startup peak** from **steady-state floor** on the same backend.

## Command

```bash
CARGO_TARGET_DIR=target-codex cargo build --release -p fret-demo \
  --bin wgpu_hello_world_control \
  --bin hello_world_compare_demo

python3 tools/run_wgpu_hello_world_control_vs_fret.py \
  --out-dir target/diag/wgpu-control-vs-fret-20260307-r1 \
  --control-binary target-codex/release/wgpu_hello_world_control \
  --fret-binary target-codex/release/hello_world_compare_demo \
  --sample-at-secs 1,2,6,12 \
  --steady-offset-secs 6
```

## Primary artifacts

- Session manifest: `target/diag/wgpu-control-vs-fret-20260307-r1/manifest.json`
- Summary JSON: `target/diag/wgpu-control-vs-fret-20260307-r1/summary/summary.json`
- Summary Markdown: `target/diag/wgpu-control-vs-fret-20260307-r1/summary/summary.md`

## Steady-state comparison (6s)

| Case | Peak MiB | Peak→Steady Drop MiB | Steady Physical MiB | Steady Graphics MiB | Steady Metal MiB | Residual MiB | Presents |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wgpu control` | 226.0 | 194.5 | 31.5 | 12.8 | 9.5 | 3.3 | 2 |
| `fret compare full` | 263.3 | 211.0 | 52.3 | 24.5 | 42.5 | -17.9 | 4 |
| `fret compare empty` | 249.4 | 202.9 | 46.5 | 19.2 | 38.3 | -19.1 | 4 |

## Deltas vs same-backend control

At the steady `6s` sample:

- `fret compare full` vs control:
  - `+20.8 MiB` physical footprint
  - `+11.7 MiB` macOS-visible graphics total
  - `+33.0 MiB` Metal current allocated size
  - `+37.3 MiB` more startup peak and `+16.5 MiB` more peak→steady collapse
- `fret compare empty` vs control:
  - `+15.0 MiB` physical footprint
  - `+6.4 MiB` macOS-visible graphics total
  - `+28.8 MiB` Metal current allocated size
  - `+23.4 MiB` more startup peak and `+8.4 MiB` more peak→steady collapse

## Readout

1. The post-fix steady-state question remains bounded: the same-backend residual is in the **tens of MiB**, not the earlier hundreds-of-MiB story.
2. Startup peak is now closed as a **separate** problem. Both the plain control and Fret climb far above their steady floors before collapsing.
3. The remaining steady-state difference is not explained by visible macOS graphics buckets alone:
   - Fret steady `Metal current allocated size` is already larger than the macOS-visible graphics total.
   - That makes the old “just explain `owned unmapped memory (graphics)`” framing incomplete for current-head steady-state closure.
4. The content delta (`full` vs `empty`) is still small compared with the framework/runtime delta:
   - about `+5.8 MiB` physical
   - about `+5.3 MiB` graphics total
   - about `+4.0 MiB` Metal current allocated size

## Caveats

- The earliest `1s` sample can still catch pre-surface / partially initialized startup states, especially for the Fret full case.
- For this reason, treat:
  - `physical_footprint_peak_bytes_max` as the startup-peak signal,
  - the `6s` sample as the steady-state anchor,
  - and the `12s` sample as a sanity check that the floor stays flat.

## Next step

Use the same runner for targeted same-backend bisects (drawable count, surface latency, alpha/compositing, transparency, or render-target/intermediate knobs) and correlate those runs against both:

- startup peak / peak→steady collapse
- steady-state `Metal current allocated size` vs macOS-visible graphics total
