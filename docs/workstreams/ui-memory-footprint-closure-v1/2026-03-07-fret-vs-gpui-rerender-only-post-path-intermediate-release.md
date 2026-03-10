# 2026-03-07 Fret vs GPUI rerender-only post-path-intermediate rebaseline (release)

## Goal

Rebaseline the framework-level `Fret vs GPUI` rerender-only comparison **after** landing the
`path_intermediate` lazy-allocation optimization. The key question is:

> once the known `~20 MiB` app-visible Metal overhead is gone, how much of the framework-level gap
> is still left, and where does it still sit?

This remains a cross-framework comparison. GPUI on macOS here is still Blade/Metal rather than
`wgpu`, so this run is about comparative shape, not same-backend closure.

## Tooling updates used by this rerun

- `tools/run_fret_vs_gpui_hello_world_compare.py`
  - now waits briefly for `internal.gpu.json` before treating it as missing.
- `tools/sample_external_process_memory.py`
  - now retries transient `footprint` / `vmmap` command failures a few times before surfacing an
    error,
  - and now prioritizes `footprint -v` before the heavier `vmmap` captures so the exact bucket view
    is collected while the target is definitely still alive.
- `tools/run_fret_vs_gpui_hello_world_compare.py`
  - now keeps the target process alive a bit longer when expensive capture flags are enabled
    (`exit_after_secs` gains extra grace for `vmmap` / `footprint -v`).

The `r2`/`r3`/`r4` reruns showed that the earlier `footprint -v` failures were mostly a capture-order
/ target-exit timing problem rather than missing attribution data. After prioritizing `footprint -v`
and extending the target exit grace, the `r5` rerun recovers the exact bucket view again on all four
rows.

## Validation runs

### Primary rebaseline artifact

- `target/diag/fret-vs-gpui-hello-world-rerender-only-footprint-verbose-release-20260307-r5/summary/summary.json`
- `target/diag/fret-vs-gpui-hello-world-rerender-only-footprint-verbose-release-20260307-r5/summary/summary.md`

Command shape:

```sh
python3 tools/run_fret_vs_gpui_hello_world_compare.py \
  --out-dir target/diag/fret-vs-gpui-hello-world-rerender-only-footprint-verbose-release-20260307-r5 \
  --repo-ref-zed repo-ref/zed \
  --fret-binary target/release/hello_world_compare_demo \
  --gpui-profile release \
  --sample-at-secs 6 \
  --steady-offset-secs 6 \
  --active-mode rerender-only \
  --capture-vmmap-regions \
  --capture-footprint-verbose
```

### Earlier corroboration reruns

- `target/diag/fret-vs-gpui-hello-world-rerender-only-footprint-verbose-release-20260307-r2/summary/summary.json`
- `target/diag/fret-vs-gpui-hello-world-rerender-only-footprint-verbose-release-20260307-r3/summary/summary.json`
- `target/diag/fret-vs-gpui-hello-world-rerender-only-footprint-verbose-release-20260307-r4/summary/summary.json`

These runs exposed the timing problem clearly: coarse deltas stayed stable, but `footprint -v` often
failed on the last capture step. They are still useful as corroboration for the coarse numbers, while
`r5` is now the preferred exact-bucket artifact.

## 6s steady snapshot (coarse but stable)

The table below uses the refreshed exact-bucket metrics from the `r5` artifact plus Fret internal
renderer perf fields.

| Framework | Case | Physical | Graphics total | `Owned ... (graphics)` | category regions | Metal current | `path_intermediate` | Renders |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `GPUI` | `empty` | `151.1 MiB` | `132.0 MiB` | `112.9 MiB` | `40` | `n/a` | `n/a` | `697` |
| `GPUI` | `full` | `153.2 MiB` | `133.3 MiB` | `113.2 MiB` | `40` | `n/a` | `n/a` | `701` |
| `Fret` | `empty` | `249.2 MiB` | `221.8 MiB` | `204.0 MiB` | `83` | `18.17 MiB` | `0` | `695` |
| `Fret` | `full` | `266.7 MiB` | `236.6 MiB` | `217.3 MiB` | `86` | `22.30 MiB` | `0` | `692` |

## Findings

### 1. The shipped optimization is visible on the GPUI compare path too

Compared with the pre-optimization release comparison
`target/diag/fret-vs-gpui-hello-world-rerender-only-footprint-verbose-release-20260307-r1/summary/summary.json`:

| case | old Fret Metal | new Fret Metal | delta |
| --- | ---: | ---: | ---: |
| `Fret empty` | `38.33 MiB` | `18.17 MiB` | `-20.16 MiB` |
| `Fret full` | `42.45 MiB` | `22.30 MiB` | `-20.16 MiB` |

The new compare rows also report `path_intermediate_bytes_estimate = 0`, so the shipped renderer
change is active on this path, not just in the same-backend control workflow.

### 2. But the framework-level gap barely moves where it matters

The remaining `Fret - GPUI` deltas in `r5` are still large:

- `empty`: about `+98.1 MiB` physical / `+89.9 MiB` graphics / `+91.1 MiB` owned-unmapped dirty
- `full`: about `+113.5 MiB` physical / `+103.3 MiB` graphics / `+104.1 MiB` owned-unmapped dirty

Those numbers are effectively the same shape as the pre-optimization `r1` compare:

- `empty`: `+98.5 MiB` physical / `+89.9 MiB` graphics / `+91.1 MiB` owned
- `full`: `+112.3 MiB` physical / `+101.8 MiB` graphics / `+104.1 MiB` owned

So removing `path_intermediate` materially improves the **visible Metal** slice but does not
materially change the framework-level hidden gap against GPUI.

### 3. The remaining gap is still centered in hidden graphics-owned memory

The coarse rebaseline keeps pointing to the same family:

- `GPUI empty/full`: `Owned physical footprint (unmapped) (graphics)` stays around `~113 MiB`
- `Fret empty/full`: the same family stays around `~204–217 MiB`

So the remaining framework-level delta is still concentrated in hidden graphics-owned unmapped
memory, not in visible drawables or generic CPU-side heaps.

### 4. Exact `4 MiB` bucket refresh is now restored, and it matches the earlier evidence

The `r5` rerun refreshes the exact bucket counts directly:

- GPUI: `4.0 MiB × 28`
- Fret: `4.0 MiB × 50` (`empty`) / `4.0 MiB × 52` (`full`)

So the earlier `r1` finding was not a one-off artifact. The post-optimization framework comparison
lands on the same hidden object-count signature, which means the path-MSAA scratch win removed a
visible reservation slice, not the hidden `4 MiB` object-count inflation.

## Conclusion

This rerun sharpens the investigation in exactly the way we wanted:

1. The `path_intermediate` optimization is real and shipped.
2. It removes `~20 MiB` of app-visible Metal from the framework comparison too.
3. The large remaining `Fret vs GPUI` gap is still almost unchanged and still centered in hidden
   graphics-owned unmapped memory.

That means the next round of investigation should **stop spending time on visible Metal slices** and
focus on the extra hidden full-surface graphics objects that Fret still appears to retain.

## Recommended next steps

1. Keep using the same-backend control path for backend isolation and Fret-internal closure.
2. Use the GPUI cross-check as external proof that the remaining gap is not just “normal GPU-first
   app cost”.
3. Narrow source-side audits to the code paths that could keep extra hidden full-surface graphics
   objects alive after the `path_intermediate` win is removed.
4. Reuse this improved capture ordering / exit-grace path for future GPUI reruns so exact `4 MiB` row
   counts remain refreshable without manual babysitting.
