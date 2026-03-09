# 2026-03-07 Apple VM+Metal active same-backend paired capture

## Goal

Use Apple's launch-mode `Virtual Memory Trace + Metal Application` path on the same backend to see
whether the current active plateau can be split further than the external sampler alone allows.

This pass deliberately pairs:

- `wgpu_hello_world_control` in continuous redraw mode, and
- `hello_world_compare_demo` in the three strongest active modes:
  - `rerender-only`
  - `paint-model`
  - `layout-model`

The question is simple: does Apple-side launch tracing now expose an obvious Fret-only surface /
compositor / command-buffer explosion that could explain the large active graphics bucket?

## Setup

### Control

- Binary: `target/release/wgpu_hello_world_control`
- Env:
  - `FRET_WGPU_HELLO_WORLD_CONTROL_WINDOW_WIDTH=500`
  - `FRET_WGPU_HELLO_WORLD_CONTROL_WINDOW_HEIGHT=500`
  - `FRET_WGPU_HELLO_WORLD_CONTROL_CONTINUOUS_REDRAW=1`
  - `FRET_WGPU_HELLO_WORLD_CONTROL_EXIT_AFTER_SECS=7.25`

### Fret

- Binary: `target/release/hello_world_compare_demo`
- Shared env:
  - `FRET_HELLO_WORLD_COMPARE_WINDOW_WIDTH=500`
  - `FRET_HELLO_WORLD_COMPARE_WINDOW_HEIGHT=500`
  - `FRET_HELLO_WORLD_COMPARE_EXIT_AFTER_SECS=7.25`
- Mode-specific env:
  - `FRET_HELLO_WORLD_COMPARE_ACTIVE_MODE=rerender-only|paint-model|layout-model`

### Trace path

- Helper: `tools/capture_binary_xctrace.py`
- Record mode: `launch`
- Instruments:
  - `Virtual Memory Trace`
  - `Metal Application`
- Time limit: `8s`
- Finalization timeout: `45s`
- Summary helper: `tools/summarize_hello_world_compare_xctrace.py`

## Artifacts

Base dir:

- `target/diag/apple-vm-metal-active-samebackend-20260307-r1/`

Per-run trace metadata:

- `target/diag/apple-vm-metal-active-samebackend-20260307-r1/control-continuous/summary.json`
- `target/diag/apple-vm-metal-active-samebackend-20260307-r1/fret-rerender-only-full/summary.json`
- `target/diag/apple-vm-metal-active-samebackend-20260307-r1/fret-paint-model-full/summary.json`
- `target/diag/apple-vm-metal-active-samebackend-20260307-r1/fret-layout-model-full/summary.json`

Primary analysis summaries:

- `target/diag/apple-vm-metal-active-samebackend-20260307-r1/control-continuous/analysis/summary.json`
- `target/diag/apple-vm-metal-active-samebackend-20260307-r1/fret-rerender-only-full/analysis/summary.json`
- `target/diag/apple-vm-metal-active-samebackend-20260307-r1/fret-paint-model-full/analysis/summary.json`
- `target/diag/apple-vm-metal-active-samebackend-20260307-r1/fret-layout-model-full/analysis/summary.json`

Extra schema sweeps:

- `target/diag/apple-vm-metal-active-samebackend-20260307-r1/*/analysis/metal-extra-summary.json`
- `target/diag/apple-vm-metal-active-samebackend-20260307-r1/*/analysis/metal-app-summary.json`

## Trace health

All four captures produced full `.trace` bundles (`trace_complete_guess=true`) even though `xctrace`
returned code `54` on this machine.

Interpretation:

- treat the captures as **usable but recorded-with-issues**, and
- continue to rely on `trace_complete_guess` / bundle contents rather than raw exit code alone.

## Key findings

### 1. `metal-io-surface-access` still does not attribute the active rows to the app pid

Across all four runs:

- `metal-io-surface-access.process_filter_match_count = 0`
- `metal-command-buffer-frame-assignment.process_filter_match_count = 0`

The visible rows are still dominated by compositor/helper processes rather than the app process:

- `WindowServer`
- `Wave Helper (GPU)`
- occasionally `Clash Party Helper (GPU)`

So this launch-mode VM+Metal path still does **not** give a direct app-owned IOSurface ledger for the
active plateau.

### 2. The compositor-visible traffic is in the same rough band for control and Fret

`metal-io-surface-access` totals:

- control continuous redraw: `8757` rows, `48` surface ids
- Fret `rerender-only`: `8487` rows, `38` surface ids
- Fret `paint-model`: `8754` rows, `43` surface ids
- Fret `layout-model`: `8361` rows, `38` surface ids

`metal-command-buffer-frame-assignment` totals:

- control continuous redraw: `3389` command buffers
- Fret `rerender-only`: `3197`
- Fret `paint-model`: `3256`
- Fret `layout-model`: `3170`

This is important because the external memory baselines differ by roughly `~100 MiB`, but the
compositor-visible row volume here does **not** show a correspondingly large Fret-only explosion.

### 3. `metal-object-label` also stays in the same ballpark

Representative label counts remain close between control and Fret active modes:

- `coreanimation.Surface`
- `coreanimation.surface`
- `coreanimation.ImageOffscreen`
- `coreanimation.memoryless-texture`
- `coreanimation.offscreen-encoder`

The Fret rows are not obviously an order of magnitude larger than control on this path.

### 4. App-owned Metal application tables are still not helping under this instrument mix

For all four captures, both of these export as zero-row tables:

- `metal-application-intervals`
- `metal-application-encoders-list`

So the current `Virtual Memory Trace + Metal Application` combination is useful for launch-complete
bundle capture, but not yet for app-owned active-pass attribution.

## Working interpretation

This closes one tempting but still unproven explanation.

If the remaining active plateau were mainly an obvious Fret-only explosion in compositor-visible
surfaces or command-buffer traffic, this paired run should have shown a strong gap in:

- IOSurface-access row count,
- visible surface-id cardinality,
- frame-assignment volume, or
- coreanimation object-label volume.

It did not.

So the current evidence points away from a simple "Fret obviously creates many more visible CA
surfaces" story on this path. The unresolved bucket still looks more like:

- driver/private residency,
- OS reservation / swapchain bookkeeping that is not attributed back to the app pid here, or
- a Metal/Game Memory store we are still not decoding from the right launch template.

## What this means for the plan

This path is still worth keeping because it gives repeatable full launch traces, but it is **not** the
closure path by itself.

The next high-value tool steps are:

1. Re-run active same-backend launch captures through `Game Memory` and reuse the direct-store
   fallback path, specifically on the strong active rows.
2. Extend summarization around the `Metal Application` stores that are present but not yet useful in
   current output, especially joins involving:
   - `metal-command-buffer-frame-assignment`
   - `present-surface-id`
   - compositor processes / known compositor stores
3. Consider a follow-up capture with `Metal Resource Events` if it records resource lifecycle data that
   is missing from the current `Virtual Memory Trace + Metal Application` path.
