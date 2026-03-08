# 2026-03-07 Present-only uniform dedupe release experiment

## Goal

Test one bounded hypothesis before changing broader renderer architecture:

> Is the same-backend `present-only` memory floor mostly caused by repeated steady-state uniform-style
> uploads on an otherwise static frame?

The experiment stays deliberately narrow and reversible:

- keep the same `present-only` scene and same release binaries,
- add one opt-in env flag,
- only skip redundant `queue.write_buffer()` uploads for:
  - viewport uniforms,
  - clip buffers,
  - mask buffers,
  - render-space uniform bytes,
- then compare steady memory against the same release baseline.

The experiment flag is:

- `FRET_RENDER_WGPU_SKIP_REDUNDANT_UNIFORM_UPLOADS=1`

## Setup

### Baseline compare run

- Script: `python3 tools/run_wgpu_hello_world_control_vs_fret.py`
- Out dir: `target/diag/wgpu-control-vs-fret-present-release-baseline-20260307-r1/`
- Mode: `--fret-active-mode present-only`
- Sampling: `1s,6s`
- Build: release / release

### Experiment compare run

- Script: `python3 tools/run_wgpu_hello_world_control_vs_fret.py`
- Out dir: `target/diag/wgpu-control-vs-fret-present-uniform-dedupe-release-20260307-r1/`
- Mode: `--fret-active-mode present-only`
- Extra Fret env:
  - `FRET_RENDER_WGPU_SKIP_REDUNDANT_UNIFORM_UPLOADS=1`
- Sampling: `1s,6s`
- Build: release / release

### Apple-side verification pilot

To confirm that the experiment really removes the repeated upload event stream, we also captured one
`Metal Resource Events` pilot on the most controlled case:

- case: Fret `present-only empty`
- extra env: `FRET_RENDER_WGPU_SKIP_REDUNDANT_UNIFORM_UPLOADS=1`
- out dir: `target/diag/apple-metal-resource-events-present-uniform-dedupe-20260307-r1/fret-present-only-empty/`

## Artifacts

Release baseline:

- `target/diag/wgpu-control-vs-fret-present-release-baseline-20260307-r1/summary/summary.json`

Release experiment:

- `target/diag/wgpu-control-vs-fret-present-uniform-dedupe-release-20260307-r1/summary/summary.json`

Apple verification pilot:

- `target/diag/apple-metal-resource-events-present-uniform-dedupe-20260307-r1/fret-present-only-empty/summary.json`
- `target/diag/apple-metal-resource-events-present-uniform-dedupe-20260307-r1/fret-present-only-empty/analysis/summary.json`

## Findings

### 1. Steady release memory is effectively unchanged

At `6s`, experiment minus release baseline:

- Fret `present-only empty`
  - physical: `+0.20 MiB`
  - graphics: `+0.07 MiB`
  - internal Metal current allocated size: `+0.00 MiB`
- Fret `present-only full`
  - physical: `-0.60 MiB`
  - graphics: `-0.39 MiB`
  - internal Metal current allocated size: `+0.00 MiB`

That is noise-level movement, not a real floor reduction.

So redundant uniform-style uploads are **not** the main explanation for the steady `present-only`
residency floor.

### 2. The experiment still changes the Apple event stream dramatically

The Apple `Metal Resource Events` pilot on `present-only empty` with the experiment flag shows:

- `metal-current-allocated-size`: direct-store `row_count = 0`
- `metal-resource-allocations`: direct-store `row_count = 0`
- the relevant indexed stores are header-only (`bulkstore = 4096 B`)

This is a sharp contrast to the earlier non-experiment `present-only empty` run, where the same path
showed:

- `metal-current-allocated-size.row_count = 5200`
- `metal-resource-allocations.row_count = 5133`
- dominant label: `"(wgpu internal) Staging  ( 128.00 KiB ,  Shared )"`

So the experiment does appear to remove the repeated post-attach upload/allocation event stream that
`Metal Resource Events` was seeing.

### 3. The result cleanly separates **upload churn** from **steady residency**

Putting the two observations together:

1. The repeated upload event stream can be suppressed by redundant-uniform-write dedupe.
2. The steady `physical` / `graphics_total` / `metal_current_allocated_size` floors stay effectively the
   same.

That means the staging/uniform path we identified earlier is real, but it is a **churn problem**, not
an explanation for the large steady residency bucket.

In other words:

- this experiment closes one branch of the search tree,
- but it does **not** close the memory-floor question itself.

### 4. What the result implies about the remaining floor

The steady active floor now looks less like “we keep uploading the same small buffers forever” and more
like some combination of:

- already-live app-visible resources that remain resident after startup,
- swapchain / drawable / surface residency that is not reduced by upload dedupe,
- driver/private residency,
- WindowServer / OS-side reservation buckets outside the app-visible current-allocation ledger.

## Recommendation

Do **not** spend the next iteration polishing the redundant-uniform-upload experiment into a product
change yet.

It is useful as evidence and possibly as a perf/churn optimization, but it does not materially reduce
steady memory.

The next highest-value directions are:

1. Treat the staging/uniform result as **closed for steady-floor attribution**.
2. Shift the next bounded experiment toward live-resource residency rather than repeated upload churn.
3. Keep the Apple-side residual investigation focused on the `~60–75 MiB` bucket outside the
   app-visible Metal ledger.
