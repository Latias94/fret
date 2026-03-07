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
    error.

Even after retries, this machine still intermittently returns `footprint -v` exit code `66` during
GPUI/Fret compare captures, so the exact row-bucket refresh is still not fully reliable. The coarse
`footprint -j` / `vmmap` comparison is stable enough to rebaseline the remaining gap.

## Validation runs

### Primary rebaseline artifact

- `target/diag/fret-vs-gpui-hello-world-rerender-only-footprint-verbose-release-20260307-r4/summary/summary.json`
- `target/diag/fret-vs-gpui-hello-world-rerender-only-footprint-verbose-release-20260307-r4/summary/summary.md`

Command shape:

```sh
python3 tools/run_fret_vs_gpui_hello_world_compare.py \
  --out-dir target/diag/fret-vs-gpui-hello-world-rerender-only-footprint-verbose-release-20260307-r4 \
  --repo-ref-zed repo-ref/zed \
  --fret-binary target/release/hello_world_compare_demo \
  --gpui-profile release \
  --sample-at-secs 6 \
  --steady-offset-secs 6 \
  --active-mode rerender-only \
  --capture-vmmap-regions \
  --capture-footprint-verbose
```

### Partial corroboration reruns

- `target/diag/fret-vs-gpui-hello-world-rerender-only-footprint-verbose-release-20260307-r2/summary/summary.json`
- `target/diag/fret-vs-gpui-hello-world-rerender-only-footprint-verbose-release-20260307-r3/summary/summary.json`

These two runs hit the same `footprint -v` flake on multiple rows, but they agree on the important
coarse deltas: Fret loses the visible `~20 MiB` Metal slice while the large framework-level hidden
gap remains.

## 6s steady snapshot (coarse but stable)

The table below uses the stable coarse metrics from the `r4` artifact plus Fret internal renderer
perf fields. `Owned ... (graphics)` is read from the top category in the raw `footprint -j` output.

| Framework | Case | Physical | Graphics total | `Owned ... (graphics)` | category regions | Metal current | `path_intermediate` | Renders |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `GPUI` | `empty` | `150.9 MiB` | `132.0 MiB` | `112.9 MiB` | `40` | `n/a` | `n/a` | `701` |
| `GPUI` | `full` | `152.9 MiB` | `133.3 MiB` | `113.2 MiB` | `40` | `n/a` | `n/a` | `700` |
| `Fret` | `empty` | `251.4 MiB` | `224.2 MiB` | `204.0 MiB` | `83` | `18.17 MiB` | `0` | `695` |
| `Fret` | `full` | `268.0 MiB` | `237.9 MiB` | `217.3 MiB` | `86` | `22.30 MiB` | `0` | `696` |

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

The remaining `Fret - GPUI` deltas in `r4` are still large:

- `empty`: about `+100.5 MiB` physical / `+92.2 MiB` graphics / `+91.1 MiB` owned-unmapped dirty
- `full`: about `+115.1 MiB` physical / `+104.5 MiB` graphics / `+104.1 MiB` owned-unmapped dirty

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

### 4. Exact `4 MiB` bucket refresh is still blocked by Apple tooling flakiness, not by the product change

The previous successful release cross-check
`docs/workstreams/ui-memory-footprint-closure-v1/2026-03-07-fret-vs-gpui-rerender-only-footprint-verbose-release.md`
still provides the clean exact-bucket evidence:

- GPUI: `4.0 MiB × 28`
- Fret: `4.0 MiB × 50` (`empty`) / `4.0 MiB × 52` (`full`)

The new same-backend post-optimization run also still shows Fret at `4.0 MiB × 50/52`, so the
working interpretation remains stable: the path-MSAA scratch win removed a visible reservation slice,
not the hidden `4 MiB` object-count inflation.

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
4. Stabilize `footprint -v` bucket capture further (or add a fallback path) so future GPUI reruns can
   refresh exact `4 MiB` row counts instead of relying on prior successful captures.
