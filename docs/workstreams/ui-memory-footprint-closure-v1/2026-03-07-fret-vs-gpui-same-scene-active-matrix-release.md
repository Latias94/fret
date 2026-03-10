# 2026-03-07 Fret vs GPUI same-scene active matrix (release/release)

## Goal

Repeat the same-scene Fret-vs-GPUI compare run with optimized binaries on both sides so the
framework-level memory picture is not biased by debug-only render cadence distortion.

This run answers two follow-up questions from the earlier debug/debug matrix:

1. Does the large active Fret plateau survive in `release/release`?
2. Does the debug-only caveat on `paint-model full` disappear once both binaries are optimized?

## Setup

### Fret side

- Binary: `target/release/hello_world_compare_demo`
- Scene knobs:
  - `FRET_HELLO_WORLD_COMPARE_ACTIVE_MODE=idle|rerender-only|paint-model|layout-model`
  - `FRET_HELLO_WORLD_COMPARE_NO_TEXT=1`
  - `FRET_HELLO_WORLD_COMPARE_NO_SWATCHES=1`

### GPUI side

- Reference checkout: `repo-ref/zed` @ `f4aad4bb27a846ff0ef4ba4d31875c61ef78b0c4`
- Materialized example template: `tools/external-templates/gpui_hello_world_compare.rs`
- Runner copies that template into `repo-ref/zed/crates/gpui/examples/fret_hello_world_compare.rs`
- Build mode: `release`
- Build flag: `--features runtime_shaders`

### Driver

- Comparison runner: `tools/run_fret_vs_gpui_hello_world_compare.py`
- Summarizer: `tools/summarize_fret_vs_gpui_hello_world_compare.py`
- External memory sampler: `tools/sample_external_process_memory.py`
- Sample offsets: `1s, 2s, 6s`
- Steady comparison point: `6s`

Why still use `6s`?

- Both frameworks still show transient startup peaks that collapse before steady state.
- The closure question here is the sustained active plateau, not the first-second launch spike.

## Artifacts

- `target/diag/fret-vs-gpui-hello-world-same-scene-release-20260307-r1-idle/summary/summary.json`
- `target/diag/fret-vs-gpui-hello-world-same-scene-release-20260307-r1-rerender-only/summary/summary.json`
- `target/diag/fret-vs-gpui-hello-world-same-scene-release-20260307-r1-paint-model/summary/summary.json`
- `target/diag/fret-vs-gpui-hello-world-same-scene-release-20260307-r1-layout-model/summary/summary.json`

## 6s steady snapshot

| Mode | Framework | Scene | Physical footprint | Graphics total | Owned unmapped | Renders |
| --- | --- | --- | ---: | ---: | ---: | ---: |
| `idle` | GPUI | `empty` | `32.2 MiB` | `13.8 MiB` | `2.7 MiB` | `2` |
| `idle` | GPUI | `full` | `34.8 MiB` | `15.2 MiB` | `3.0 MiB` | `3` |
| `idle` | Fret | `empty` | `49.3 MiB` | `22.1 MiB` | `8.5 MiB` | `2` |
| `idle` | Fret | `full` | `48.1 MiB` | `20.6 MiB` | `11.1 MiB` | `2` |
| `rerender-only` | GPUI | `empty` | `157.9 MiB` | `139.1 MiB` | `117.5 MiB` | `585` |
| `rerender-only` | GPUI | `full` | `160.4 MiB` | `140.7 MiB` | `118.1 MiB` | `580` |
| `rerender-only` | Fret | `empty` | `252.4 MiB` | `224.7 MiB` | `208.5 MiB` | `580` |
| `rerender-only` | Fret | `full` | `268.0 MiB` | `238.0 MiB` | `221.8 MiB` | `574` |
| `paint-model` | GPUI | `empty` | `150.8 MiB` | `132.0 MiB` | `114.7 MiB` | `585` |
| `paint-model` | GPUI | `full` | `158.0 MiB` | `138.5 MiB` | `118.1 MiB` | `583` |
| `paint-model` | Fret | `empty` | `254.7 MiB` | `227.4 MiB` | `208.5 MiB` | `576` |
| `paint-model` | Fret | `full` | `270.3 MiB` | `240.7 MiB` | `224.5 MiB` | `578` |
| `layout-model` | GPUI | `empty` | `150.5 MiB` | `132.0 MiB` | `114.7 MiB` | `583` |
| `layout-model` | GPUI | `full` | `152.5 MiB` | `133.3 MiB` | `115.0 MiB` | `582` |
| `layout-model` | Fret | `empty` | `252.0 MiB` | `224.6 MiB` | `208.5 MiB` | `577` |
| `layout-model` | Fret | `full` | `270.8 MiB` | `240.7 MiB` | `221.8 MiB` | `577` |

## Release/release deltas

### Fret minus GPUI at 6s

- `idle`
  - `empty`: about `+17.1 MiB` physical / `+8.2 MiB` graphics / `+5.8 MiB` owned
  - `full`: about `+13.3 MiB` physical / `+5.4 MiB` graphics / `+8.1 MiB` owned
- `rerender-only`
  - `empty`: about `+94.5 MiB` physical / `+85.7 MiB` graphics / `+91.0 MiB` owned
  - `full`: about `+107.6 MiB` physical / `+97.4 MiB` graphics / `+103.7 MiB` owned
- `paint-model`
  - `empty`: about `+103.9 MiB` physical / `+95.4 MiB` graphics / `+93.8 MiB` owned
  - `full`: about `+112.3 MiB` physical / `+102.3 MiB` graphics / `+106.4 MiB` owned
- `layout-model`
  - `empty`: about `+101.5 MiB` physical / `+92.6 MiB` graphics / `+93.8 MiB` owned
  - `full`: about `+118.3 MiB` physical / `+107.3 MiB` graphics / `+106.8 MiB` owned

### `full - empty` content delta at 6s

- GPUI
  - `idle`: about `+2.6 MiB` physical / `+1.4 MiB` graphics
  - `rerender-only`: about `+2.5 MiB` physical / `+1.6 MiB` graphics
  - `paint-model`: about `+7.2 MiB` physical / `+6.5 MiB` graphics
  - `layout-model`: about `+2.0 MiB` physical / `+1.3 MiB` graphics
- Fret
  - `idle`: effectively noise-level on this run (`-1.2 MiB` physical / `-1.5 MiB` graphics); treat idle content cost as approximately zero here
  - `rerender-only`: about `+15.6 MiB` physical / `+13.3 MiB` graphics
  - `paint-model`: about `+15.6 MiB` physical / `+13.4 MiB` graphics
  - `layout-model`: about `+18.8 MiB` physical / `+16.0 MiB` graphics

## What changed versus debug/debug

1. The main conclusion survives optimized binaries.
   - Idle remains relatively close.
   - Active rows still leave Fret roughly `~+95–118 MiB` higher in physical footprint and `~+86–107 MiB` higher in the macOS-visible graphics buckets.

2. The `paint-model full` caveat is now gone.
   - Debug/debug had one under-paced GPUI cell (`paint-model full` ≈ `221` renders by `6s`).
   - In release/release, `paint-model full` is cadence-aligned enough to trust (`GPUI≈583`, `Fret≈578`).

3. Startup peak and steady-state floor are clearly separate issues.
   - Idle still shows transient high `physical_footprint_peak` values before the `6s` steady sample collapses.
   - That means the active plateau conclusion should continue to rely on the steady rows, while startup needs separate Apple-side attribution.

## Main takeaways

### 1. Idle is still not the headline problem

The optimized run still shows only a modest same-scene idle gap:

- about `+13–17 MiB` physical, and
- about `+5–8 MiB` graphics.

So the release data agrees with the debug matrix that Fret is not simply "always huge" on the same trivial scene.

### 2. The active plateau is still real after removing debug distortion

Across all three active rows:

- Fret still lands around `252–271 MiB` physical and `224–241 MiB` graphics.
- GPUI still lands around `150–160 MiB` physical and `132–141 MiB` graphics.

That keeps the active framework-level gap very large even when both binaries are optimized.

### 3. The active content-sensitive delta is still much larger on Fret

The `full - empty` rows remain one of the most actionable signals:

- GPUI active content cost stays around `~2–7 MiB` physical.
- Fret active content cost stays around `~16–19 MiB` physical.

So the unresolved Fret overhead is not just an always-active window floor; there is also a content-sensitive component that grows more than GPUI on the same scene.

### 4. The remaining active gap is still dominated by `owned unmapped memory`

At `6s`:

- GPUI active `owned unmapped memory` sits around `~114.7–118.1 MiB`.
- Fret active `owned unmapped memory` sits around `~208.5–224.5 MiB`.

That keeps the center of gravity in the same graphics-facing bucket that also dominated the earlier same-backend control work.

## What this still does not close

This is still a framework-level cross-check, not same-backend closure:

- GPUI on macOS here is Blade/Metal, not `wgpu`.
- So this matrix confirms that the active plateau is not merely "Fret vs raw backend control", but it still cannot separate backend cost from framework cost on its own.

## Recommended next steps

1. Use the release/release `rerender-only`, `paint-model`, and `layout-model` rows as the stable framework-level baseline going forward.
2. Pair the strongest rows with Apple-side attribution (`Virtual Memory Trace + Metal Application`) so the remaining graphics bucket can be split into swapchain / driver-private / OS reservation contributions.
3. Keep the same-backend `wgpu` control matrix as the primary closure path, and treat GPUI as the external reality check that says the active gap is still too large relative to another mature GPU-first UI stack.
