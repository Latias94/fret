# 2026-03-07 Fret vs GPUI same-scene active matrix (debug/debug)

Release/release follow-up: `docs/workstreams/ui-memory-footprint-closure-v1/2026-03-07-fret-vs-gpui-same-scene-active-matrix-release.md`.

## Goal

Cross-check the current Fret `hello_world_compare_demo` active-memory story against a real GPU-first UI framework on the **same scene shape**, not just against the raw `wgpu` hello-world control.

This run is intended to answer:

1. How much of Fret's active plateau remains when the comparison target is another declarative UI stack instead of a raw backend control?
2. Does the gap stay small in idle but reopen once both frameworks rerender every frame?
3. Is the remaining delta mostly an "always-active window" floor, or does content (`full` vs `empty`) matter differently across frameworks?

## Setup

### Fret side

- Binary: `target/debug/hello_world_compare_demo`
- Scene knobs:
  - `FRET_HELLO_WORLD_COMPARE_ACTIVE_MODE=idle|rerender-only|paint-model|layout-model`
  - `FRET_HELLO_WORLD_COMPARE_NO_TEXT=1`
  - `FRET_HELLO_WORLD_COMPARE_NO_SWATCHES=1`

### GPUI side

- Reference checkout: `repo-ref/zed` @ `f4aad4bb27a846ff0ef4ba4d31875c61ef78b0c4`
- Materialized example template: `tools/external-templates/gpui_hello_world_compare.rs`
- Runner script copies that template into `repo-ref/zed/crates/gpui/examples/fret_hello_world_compare.rs`
- Build mode: `debug`
- Build flag: `--features runtime_shaders`

Why `runtime_shaders`?

- On this machine, the default GPUI macOS build path fails during `xcrun metal` because the standalone Metal Toolchain component is missing.
- Enabling `runtime_shaders` avoids the build-time metallib compilation step and keeps the comparison runnable.

### Runner

- Comparison driver: `tools/run_fret_vs_gpui_hello_world_compare.py`
- Summarizer: `tools/summarize_fret_vs_gpui_hello_world_compare.py`
- External memory sampler: `tools/sample_external_process_memory.py`

The runner captures four cases per mode:

- GPUI `empty`
- GPUI `full`
- Fret `empty`
- Fret `full`

with the same window size (`500x500`) and the same external sampling offsets (`1s`, `2s`, `6s`).

## Command template

```bash
python3 tools/run_fret_vs_gpui_hello_world_compare.py \
  --out-dir target/diag/fret-vs-gpui-hello-world-same-scene-debug-<stamp>-rerender-only \
  --gpui-profile debug \
  --fret-binary target/debug/hello_world_compare_demo \
  --sample-at-secs 1,2,6 \
  --steady-offset-secs 6 \
  --active-mode rerender-only
```

## Final local artifacts (2026-03-07)

- `idle`
  - `target/diag/fret-vs-gpui-hello-world-same-scene-debug-20260307-r2-idle/summary/summary.json`
- `rerender-only`
  - `target/diag/fret-vs-gpui-hello-world-same-scene-debug-20260307-r2-rerender-only/summary/summary.json`
- `paint-model`
  - `target/diag/fret-vs-gpui-hello-world-same-scene-debug-20260307-r2-paint-model/summary/summary.json`
- `layout-model`
  - `target/diag/fret-vs-gpui-hello-world-same-scene-debug-20260307-r2-layout-model/summary/summary.json`

## Steady snapshot at 6s

| mode | framework | case | physical | graphics | owned | renders |
| --- | --- | --- | ---: | ---: | ---: | ---: |
| `idle` | GPUI | `empty` | `32.6 MiB` | `13.8 MiB` | `2.7 MiB` | `2` |
| `idle` | GPUI | `full` | `34.7 MiB` | `15.2 MiB` | `3.0 MiB` | `3` |
| `idle` | Fret | `empty` | `46.4 MiB` | `19.2 MiB` | `5.8 MiB` | `2` |
| `idle` | Fret | `full` | `48.8 MiB` | `20.6 MiB` | `11.1 MiB` | `2` |
| `rerender-only` | GPUI | `empty` | `151.0 MiB` | `132.0 MiB` | `114.7 MiB` | `580` |
| `rerender-only` | GPUI | `full` | `152.9 MiB` | `133.3 MiB` | `115.0 MiB` | `561` |
| `rerender-only` | Fret | `empty` | `249.8 MiB` | `221.9 MiB` | `205.8 MiB` | `579` |
| `rerender-only` | Fret | `full` | `270.9 MiB` | `240.3 MiB` | `221.8 MiB` | `581` |
| `paint-model` | GPUI | `empty` | `155.7 MiB` | `136.8 MiB` | `117.5 MiB` | `586` |
| `paint-model` | GPUI | `full` | `158.0 MiB` | `138.5 MiB` | `118.1 MiB` | `221` |
| `paint-model` | Fret | `empty` | `250.0 MiB` | `221.9 MiB` | `205.8 MiB` | `579` |
| `paint-model` | Fret | `full` | `271.6 MiB` | `240.7 MiB` | `221.8 MiB` | `578` |
| `layout-model` | GPUI | `empty` | `150.6 MiB` | `132.0 MiB` | `114.7 MiB` | `589` |
| `layout-model` | GPUI | `full` | `153.1 MiB` | `133.3 MiB` | `115.0 MiB` | `560` |
| `layout-model` | Fret | `empty` | `252.3 MiB` | `224.6 MiB` | `208.5 MiB` | `580` |
| `layout-model` | Fret | `full` | `265.5 MiB` | `235.2 MiB` | `219.1 MiB` | `579` |

## What this says

### 1. Idle same-scene Fret-vs-GPUI is much closer than the active story

At `6s`, on the same static scene:

- `empty`: Fret is only about `+13.8 MiB` physical / `+5.4 MiB` graphics above GPUI.
- `full`: Fret is only about `+14.1 MiB` physical / `+5.4 MiB` graphics above GPUI.

That is a very different picture from the active runs. So the headline gap is **not** simply "Fret is always huge even on the same trivial scene".

### 2. Once both frameworks stay active, a large Fret plateau reappears

At `6s`:

- `rerender-only`
  - `empty`: Fret is about `+98.8 MiB` physical / `+89.9 MiB` graphics above GPUI.
  - `full`: Fret is about `+118.0 MiB` physical / `+106.9 MiB` graphics above GPUI.
- `layout-model`
  - `empty`: Fret is about `+101.7 MiB` physical / `+92.6 MiB` graphics above GPUI.
  - `full`: Fret is about `+112.4 MiB` physical / `+101.9 MiB` graphics above GPUI.

So the large active gap **survives** when the target is another real GPU-first UI framework on the same scene shape.

### 3. The active content delta is small in GPUI, but much larger in Fret

`full - empty` at `6s`:

- GPUI
  - `idle`: about `+2.1 MiB` physical / `+1.4 MiB` graphics
  - `rerender-only`: about `+1.9 MiB` physical / `+1.3 MiB` graphics
  - `paint-model`: about `+2.3 MiB` physical / `+1.6 MiB` graphics
  - `layout-model`: about `+2.5 MiB` physical / `+1.3 MiB` graphics
- Fret
  - `idle`: about `+2.4 MiB` physical / `+1.4 MiB` graphics
  - `rerender-only`: about `+21.1 MiB` physical / `+18.3 MiB` graphics
  - `paint-model`: about `+21.6 MiB` physical / `+18.7 MiB` graphics
  - `layout-model`: about `+13.2 MiB` physical / `+10.6 MiB` graphics

This is one of the clearest new signals from the same-scene framework comparison:

- in idle, Fret and GPUI content cost is similarly small;
- under active rerender/layout, Fret picks up a noticeably larger **content-sensitive** delta.

### 4. The active gap is still dominated by `owned unmapped memory`

At `6s`:

- GPUI active `owned unmapped memory` sits around `~114.7–118.1 MiB`.
- Fret active `owned unmapped memory` sits around `~205.8–221.8 MiB`.

So the framework-to-framework gap still shows up primarily in the same macOS-visible graphics bucket that dominated the earlier Fret-vs-control runs.

### 5. The most reliable apples-to-apples rows are `rerender-only` and `layout-model`

The `paint-model` row has one caveat:

- GPUI `full` only reached about `221` renders by `6s`, while:
  - GPUI `empty` was about `586`,
  - GPUI `layout-model full` was about `560`,
  - Fret `paint-model full` was about `578`.

So GPUI `paint-model full` is **not cadence-matched** in this debug/debug matrix and should be treated as a lower-confidence cell.

By contrast:

- `rerender-only`: GPUI/Fret full are about `561` vs `581`
- `layout-model`: GPUI/Fret full are about `560` vs `579`

Those two modes are close enough in render cadence to support the main same-scene memory conclusion.

## Working interpretation

This closes an important ambiguity that the raw `wgpu` control could not close by itself.

The earlier same-backend control already showed that Fret's active plateau was present before real paint/layout mutation. This GPUI same-scene run adds a different dimension:

- the large active gap is **not** only "Fret vs raw backend control";
- another mature GPU-first declarative stack on the same scene still lands about `~100 MiB` lower in the active cases that are cadence-aligned;
- the remaining Fret gap is not purely an always-active window floor either, because Fret's `full - empty` delta under active rerender/layout is much larger than GPUI's.

So the current evidence points to two separate unresolved pieces on the Fret side:

1. a large active residency floor visible in the macOS graphics bucket, and
2. an additional active **content-sensitive** cost that grows much more than GPUI's on the same scene.

## What this does **not** close yet

This is still **not** a same-backend comparison.

- GPUI on macOS here is Blade/Metal, not `wgpu`.
- So this matrix is a strong **framework-level behavior baseline**, but it cannot by itself say how much of the active gap is backend-specific versus Fret-specific.

That means the previous same-backend `wgpu` control work remains necessary.

## Recommended next steps

1. Re-run the same-scene GPUI matrix in `release/release` once the Fret release binary is rebuilt, to remove debug-only cadence distortion as much as possible.
2. Keep using `rerender-only` and `layout-model` as the primary cross-framework active comparison rows; treat `paint-model full` as diagnostic-only until its cadence is understood.
3. Pair the strongest rows (`rerender-only`, `layout-model`) with Apple-side attribution (`Virtual Memory Trace + Metal Application`) so the remaining graphics bucket can be split into swapchain / driver-private / OS reservation contributions.
4. Keep the same-backend `wgpu` control matrix as the backend-attribution baseline, and use this GPUI matrix as the framework-level reality check.
