# UI Memory Footprint Closure (v1)

## Problem

Fret-based apps (even simple demos) can appear to have a high memory footprint on macOS. We need a
repeatable, diagnosable evidence chain that answers:

1) **CPU**: where does the resident/dirty memory come from and why does it not return to the OS?
2) **GPU**: how much memory is actually allocated on the GPU/device side, and which subsystems
   contribute?
3) Which optimizations are worth landing (high impact, low risk), with gates that prevent regressions.

This workstream is about **measurement → attribution → bounded optimizations**, not one-off profiling.

## Snapshot (2026-03-05)

Using `tools/diag-scripts/todo-memory-steady.json` on macOS/Metal:

- Tooling helper:
  - Summarize multiple `--session-auto` samples under a base dir:
    - `fretboard diag memory-summary --dir target/fret-diag-mem-todo-steady`
    - `fretboard diag memory-summary --dir target/fret-diag-mem-todo-steady --sort-key wgpu_metal_current_allocated_size_bytes_max --top 5`
    - Linear fit helper (least squares; outputs intercept + slope + suggested `slope_ppm`):
      - `fretboard diag memory-summary --dir target/diag/mem-sweep-count-20260305 --fit-linear macos_owned_unmapped_memory_dirty_bytes:renderer_gpu_images_bytes_estimate`
      - `fretboard diag memory-summary --dir target/diag/mem-sweep-count-20260305 --fit-linear wgpu_metal_current_allocated_size_bytes_max:renderer_gpu_images_bytes_estimate`
    - Renderer-side attribution fields (from `bundle_last_frame_stats`) are also surfaced, so you can sort/compare by:
      - `--sort-key renderer_gpu_images_bytes_estimate`
      - `--sort-key renderer_gpu_render_targets_bytes_estimate`
      - `--sort-key renderer_intermediate_peak_in_use_bytes`
      - `--sort-key render_text_shape_cache_bytes_estimate_total`
      - `--sort-key render_text_blob_paint_palette_bytes_estimate_total`
      - `--sort-key render_text_blob_decorations_bytes_estimate_total`
  - Optional macOS-only hint for the largest `vmmap` buckets:
    - `fretboard diag memory-summary --dir target/fret-diag-mem-todo-steady --vmmap-regions-sorted-top`
  - Aggregate macOS `vmmap -sortBySize` top-dirty regions across samples (helps attribute `owned unmapped memory`):
    - `fretboard diag memory-summary --dir target/fret-diag-mem-todo-steady --vmmap-regions-sorted-agg`
    - If pointing at a parent dir with multiple dated runs, recursion is enabled by default (bounded); disable via `--no-recursive`.
  - Break down the aggregated regions further by a normalized `detail` key (e.g. de-addressed malloc zones, IOSurface kind):
    - `fretboard diag memory-summary --dir target/fret-diag-mem-todo-steady --vmmap-regions-sorted-detail-agg`
  - Aggregate Apple `footprint` categories (dirty bytes) across samples (macOS-only):
    - `fretboard diag memory-summary --dir target/fret-diag-mem-todo-steady --footprint-categories-agg`
    - Useful for cross-checking whether the large `vmmap` buckets (e.g. `owned unmapped memory`) correspond to a stable
      `footprint` category (e.g. untagged VM allocations, page tables, GPU carveout reservations).

- Fresh baseline batch (local, 2026-03-05; outputs under `target/diag/mem-baseline-20260305/`):
  - `empty-idle` (N=10; `target/release/empty_idle_demo`):
    - `macos_physical_footprint_peak_bytes` p50=283,534,950 (~270.4 MiB), p90=286,156,390 (~273.0 MiB)
    - `macos_owned_unmapped_memory_dirty_bytes` p50=213,594,931 (~203.7 MiB), p90=216,321,229 (~206.3 MiB)
    - `wgpu_metal_current_allocated_size_bytes_max` p50=23,511,040 (~22.4 MiB)
    - `vmmap_regions_sorted_agg` p90 top: `owned unmapped memory` (~206.3 MiB), `MALLOC_SMALL` (~24.0 MiB), `MALLOC_LARGE (empty)` (8.0 MiB)
  - `text-heavy` (N=10; `target/release/text_heavy_memory_demo`):
    - `macos_physical_footprint_peak_bytes` p50=361,863,578 (~345.1 MiB), p90=367,211,315 (~350.2 MiB)
    - `macos_owned_unmapped_memory_dirty_bytes` p50=249,036,800 (~237.5 MiB), p90=254,384,538 (~242.6 MiB)
    - `wgpu_metal_current_allocated_size_bytes_max` p50=126,500,864 (~120.6 MiB)
    - `vmmap_regions_sorted_agg` p90 top: `owned unmapped memory` (~242.6 MiB), `IOSurface` (~32.8 MiB), `MALLOC_SMALL` (~29.4 MiB)
  - `image-heavy` (N=5; `target/release/image_heavy_memory_demo`):
    - `macos_physical_footprint_peak_bytes` p50=483,393,536 (~461.0 MiB)
    - `macos_owned_unmapped_memory_dirty_bytes` p50=337,222,042 (~321.6 MiB)
    - `wgpu_metal_current_allocated_size_bytes_max` p50=204,914,688 (~195.4 MiB), max=294,207,488 (~280.6 MiB)
  - `todo` (N=5; `target/release/todo_demo`):
    - `macos_physical_footprint_peak_bytes` p50=358,193,562 (~341.6 MiB)
    - `macos_owned_unmapped_memory_dirty_bytes` p50=238,655,898 (~227.6 MiB)
    - `wgpu_metal_current_allocated_size_bytes_max` p50=83,755,008 (~79.9 MiB)
  - Summary JSON files:
    - `target/diag/mem-baseline-20260305/empty-idle.memory-summary.json`
    - `target/diag/mem-baseline-20260305/text-heavy.memory-summary.json`
    - `target/diag/mem-baseline-20260305/image-heavy.memory-summary.json`
    - `target/diag/mem-baseline-20260305/todo.memory-summary.json`

- Cross-check: Apple `footprint` categories (local, 2026-03-05; outputs under `target/diag/mem-baseline-20260305-footprint/`):
  - Key observation: `footprint_categories_agg` top bucket is consistently `Owned physical footprint (unmapped) (graphics)`, which closely tracks `vmmap` `owned unmapped memory` dirty.
  - `empty-idle` (N=10): `Owned physical footprint (unmapped) (graphics)` p50=~201.8 MiB, p90=~202.6 MiB
  - `text-heavy` (N=10): `Owned physical footprint (unmapped) (graphics)` p50=~235.0 MiB, p90=~237.1 MiB
  - `image-heavy` (N=5): `Owned physical footprint (unmapped) (graphics)` p50=~315.6 MiB, p90=~315.6 MiB
  - `todo` (N=5): `Owned physical footprint (unmapped) (graphics)` p50=~221.6 MiB, p90=~221.6 MiB
  - Summary JSON files:
    - `target/diag/mem-baseline-20260305-footprint/empty-idle.memory-summary.json`
    - `target/diag/mem-baseline-20260305-footprint/text-heavy.memory-summary.json`
    - `target/diag/mem-baseline-20260305-footprint/image-heavy.memory-summary.json`
    - `target/diag/mem-baseline-20260305-footprint/todo.memory-summary.json`

- Attribution experiments (local, 2026-03-05; outputs under `target/diag/mem-attrib-20260305/`):
  - Surface frame latency sweep (empty-idle; N=5 each; see scripts under `tools/diag-scripts/tooling/empty/`):
    - `desired_maximum_frame_latency=1`: `Owned physical footprint (unmapped) (graphics)` p90=~202.6 MiB
    - `desired_maximum_frame_latency=2`: `Owned physical footprint (unmapped) (graphics)` p90=~202.6 MiB
    - `desired_maximum_frame_latency=3`: `Owned physical footprint (unmapped) (graphics)` p90=~201.8 MiB
    - Conclusion: no material change observed for the headline bucket in this baseline.
  - WGPU memory hints sweep (text-heavy; N=5 each; see scripts under `tools/diag-scripts/tooling/text/`):
    - `FRET_WGPU_MEMORY_HINTS=performance`: `Owned physical footprint (unmapped) (graphics)` p90=~236.6 MiB
    - `FRET_WGPU_MEMORY_HINTS=memory`: `Owned physical footprint (unmapped) (graphics)` p90=~237.1 MiB
    - Conclusion: small deltas only; does not explain the baseline headline.
  - Image resource drop + idle (image-heavy; keep vs after-drop; N=5 each; outputs under `target/diag/mem-attrib-drop-20260305/`):
    - Keep (`tools/diag-scripts/image-heavy-memory-steady.json`):
      - `Owned physical footprint (unmapped) (graphics)` p50=~315.6 MiB
      - `macos_owned_unmapped_memory_dirty_bytes` p50=~321.6 MiB
      - `wgpu_metal_current_allocated_size_bytes_max` p50=~195.4 MiB
      - `renderer_gpu_images_bytes_estimate` p50=~96.0 MiB (24×1024×1024×RGBA8)
    - After drop + idle (`tools/diag-scripts/image-heavy-memory-steady-after-drop.json`, drops registered images at frame 200):
      - `Owned physical footprint (unmapped) (graphics)` p50=~218.8 MiB
      - `macos_owned_unmapped_memory_dirty_bytes` p50=~221.8 MiB
      - `wgpu_metal_current_allocated_size_bytes_max` p50=~98.7 MiB
      - `renderer_gpu_images_bytes_estimate` p50=0
    - Conclusion: the headline `owned unmapped memory` / `Owned physical footprint (unmapped) (graphics)` bucket is sensitive to live texture pressure and can return close to baseline after releasing images (not a one-way leak signature).
  - Summary JSON files:
    - `target/diag/mem-attrib-20260305/latency1.memory-summary.json`
    - `target/diag/mem-attrib-20260305/latency2.memory-summary.json`
    - `target/diag/mem-attrib-20260305/latency3.memory-summary.json`
    - `target/diag/mem-attrib-20260305/memory-hints-performance.memory-summary.json`
    - `target/diag/mem-attrib-20260305/memory-hints-memory.memory-summary.json`
    - `target/diag/mem-attrib-drop-20260305/image-heavy-keep.memory-summary.json`
    - `target/diag/mem-attrib-drop-20260305/image-heavy-drop.memory-summary.json`

- Repeat sample (N=5; `target/release/todo_demo`; `--env FRET_DIAG_WGPU_ALLOCATOR_REPORT=1`):
  - `macos_vmmap_steady.physical_footprint_peak_bytes`: 358,612,992 .. 419,325,542 (~342.0 .. 399.9 MiB)
    - Note: 4/5 runs clustered at ~342–346 MiB; 1/5 outlier correlated with higher GPU/driver-backed regions.
  - `macos_vmmap_steady.regions.owned_unmapped_memory_dirty_bytes`: 238,655,898 (~227.6 MiB; stable)
  - `macos_vmmap_steady.regions.malloc_small_dirty_bytes`: 63,753,421 .. 67,738,010 (~60.8 .. 64.6 MiB)
  - `macos_vmmap_steady.regions.malloc_dirty_bytes_total`: 76,909,773 .. 81,025,434 (~73.3 .. 77.3 MiB)
  - `macos_vmmap_steady.tables.malloc_zones.total.frag_bytes`: 24,285,594 .. 28,165,325 (~23.2 .. 26.9 MiB)
  - `wgpu_metal_current_allocated_size_bytes`: 83,755,008 .. 137,232,384 (~79.9 .. 130.9 MiB)
  - `render_text_atlas_bytes_live_estimate_total`: 4,194,304 (4 MiB; mask atlas 1 page)
- Evidence index notes:
  - `evidence.index.json.resources.bundle_last_frame_stats.wgpu_metal_current_allocated_size_bytes_{min,max}` reports the range across captured snapshots.
  - `evidence.index.json.resources.bundle_last_frame_stats.wgpu_hub_*` counters are available when `--env FRET_DIAG_WGPU_REPORT=1` is enabled.
  - `evidence.index.json.resources.bundle_last_frame_stats.render_text_registered_font_blobs_{total_bytes,count}` helps validate whether font blobs are dominating the baseline (requires app-side fields present in the bundle).
  - `check.wgpu_metal_allocated_size.json` evaluates the max value across captured snapshots (not just the last frame).
  - `check.wgpu_hub_counts.json` evaluates the max values across captured snapshots (not just the last frame):
    - `--max-wgpu-hub-buffers`
    - `--max-wgpu-hub-textures`
    - `--max-wgpu-hub-render-pipelines`
    - `--max-wgpu-hub-shader-modules`

 - Hub+allocator report sampling note (N=5; `--env FRET_DIAG_WGPU_ALLOCATOR_REPORT_EVERY_N_FRAMES=60` + `--env FRET_DIAG_WGPU_REPORT_EVERY_N_FRAMES=60`):
   - Observed a repeatable-ish outlier pattern where `wgpu_metal_current_allocated_size_bytes` and vmmap GPU-backed regions spike while hub counts remain stable:
     - Typical: `wgpu_metal_current_allocated_size_bytes_max` ~83.8 MiB and `io_surface_dirty_bytes` ~26.6 MiB.
     - Outlier example: `wgpu_metal_current_allocated_size_bytes_max` 119,341,056 (~113.8 MiB) with `io_surface_dirty_bytes` 44,354,765 (~42.3 MiB) and `io_accelerator_dirty_bytes` 18,884,198 (~18.0 MiB), while `wgpu_hub_textures_max` stayed at 16 and `wgpu_hub_render_pipelines_max` at 52.
   - The Metal size transitions aligned with the report cadence (around frames ~480 and ~540), suggesting the reporting path may perturb driver allocations. Next experiment: sweep `*_EVERY_N_FRAMES` (e.g. 60 → 600) to measure the trade-off between attribution granularity and measurement stability.
 - Cadence sweep (N=10 each; same script; env-only change):
   - Cadence 60 (`FRET_DIAG_WGPU_{ALLOCATOR_,}REPORT_EVERY_N_FRAMES=60`): 2/10 runs triggered the outlier shape (IOAccelerator/IOSurface growth; `physical_footprint_peak_bytes` up to 479,828,378 and `wgpu_metal_current_allocated_size_bytes_max` up to 137,232,384).
   - Cadence 600 (`FRET_DIAG_WGPU_{ALLOCATOR_,}REPORT_EVERY_N_FRAMES=600`): 0/10 outliers; Metal allocated size and vmmap GPU-backed regions were stable across runs.
   - Recommendation: default memory scripts to cadence 600 for stable baselines; keep cadence 60 only for deep-dive attribution when needed.
   - Script defaults:
     - Baseline: `tools/diag-scripts/tooling/todo/todo-memory-steady.json` now sets cadence 600 via `meta.env_defaults`.
     - Deep dive: `tools/diag-scripts/tooling/todo/todo-memory-steady-wgpu-highfreq.json` sets cadence 60.
     - Other memory steady scripts (`empty-idle`, `text-heavy`, `image-heavy`) also default to cadence 600 via `meta.env_defaults`.

Using `tools/diag-scripts/empty-idle-memory-steady.json` on macOS/Metal (baseline):

- Without UI diagnostics enabled (manual `vmmap -summary` on a plain run):
  - Physical footprint (peak): ~241 MiB
  - `owned unmapped memory` dirty: ~204 MiB
  - Default malloc zone: ~13.6 MiB allocated, ~4.0 MiB frag
  - With `fretboard diag repro` (UI diagnostics enabled, plus tool-side `vmmap` capture):
  - Repeat sample (N=5):
    - `macos_vmmap_steady.physical_footprint_peak_bytes`: 279,550,362 .. 282,591,232 (~266.6 .. 269.5 MiB)
    - `macos_vmmap_steady.regions.owned_unmapped_memory_dirty_bytes`: 213,594,931 (~203.7 MiB)
    - `macos_vmmap_steady.regions.malloc_small_dirty_bytes`: 34,036,122 .. 36,120,166 (~32.5 .. 34.4 MiB)
    - `macos_vmmap_steady.regions.malloc_dirty_bytes_total`: 46,307,738 .. 48,391,782 (~44.2 .. 46.2 MiB)
    - `macos_vmmap_steady.tables.malloc_zones.total.frag_bytes`: 14,533,837 .. 16,630,989 (~13.9 .. 15.9 MiB)
  - malloc zones total: ~23.8 .. 24.5 MiB allocated, ~13.9 .. 15.9 MiB frag (system allocator)
  - `wgpu_metal_current_allocated_size_bytes`: 32,161,792 (~30.7 MiB; requires `--env FRET_DIAG_WGPU_ALLOCATOR_REPORT=1`)

Using `tools/diag-scripts/text-heavy-memory-steady.json` on macOS/Metal (fonts + emoji stress):

- Repeat sample (N=5):
  - `macos_vmmap_steady.physical_footprint_peak_bytes`: 359,976,141 .. 366,162,739 (~343.3 .. 349.3 MiB)
  - `macos_vmmap_steady.regions.owned_unmapped_memory_dirty_bytes`: 249,036,800 .. 254,384,538 (~237.5 .. 242.6 MiB)
  - `render_text_atlas_bytes_live_estimate_total`: ~20 MiB (after lazy mask atlas page allocation)
- Text cache byte estimates (`resource_caches.render_text`, last snapshot; local 2026-03-05):
  - `shape_cache_bytes_estimate_total`: ~0.26 MiB (best-effort; excludes allocator overhead)
- Default malloc zone: ~26.6 MB allocated, ~20.9 MB frag (system allocator)
- `wgpu_metal_current_allocated_size_bytes`: 127,418,368 (~121.6 MiB; requires `--env FRET_DIAG_WGPU_ALLOCATOR_REPORT=1`)

Using `tools/diag-scripts/image-heavy-memory-steady.json` on macOS/Metal (texture upload stress):

- Repeat sample (N=5, defaults: `FRET_IMAGE_HEAVY_DEMO_COUNT=24`, `FRET_IMAGE_HEAVY_DEMO_SIZE_PX=1024`):
  - `macos_vmmap_steady.physical_footprint_peak_bytes`: 483,917,824 .. 501,324,186 (~461.6 .. 478.2 MiB)
  - `macos_vmmap_steady.regions.owned_unmapped_memory_dirty_bytes`: 331,874,304 .. 337,222,042 (~316.5 .. 321.6 MiB)
  - `macos_vmmap_steady.regions.io_surface_dirty_bytes`: 34,393,293 (~32.8 MiB; stable)
  - `macos_vmmap_steady.regions.io_accelerator_dirty_bytes`: 5,980,160 .. 7,372,800 (~5.7 .. 7.0 MiB)
  - `macos_vmmap_steady.regions.malloc_small_dirty_bytes`: 41,104,179 .. 44,774,195 (~39.2 .. 42.7 MiB)
  - `wgpu_metal_current_allocated_size_bytes`: 204,914,688 (~195.4 MiB; stable; requires `--env FRET_DIAG_WGPU_ALLOCATOR_REPORT=1`)
  - `renderer_gpu_images_bytes_estimate`: 100,663,296 (~96.0 MiB; stable; derived from registered image descriptors)

Using `tools/diag-scripts/image-heavy-memory-steady-after-drop.json` on macOS/Metal (drop registered images + idle):

- Repeat sample (N=5; defaults: `FRET_IMAGE_HEAVY_DEMO_COUNT=24`, `FRET_IMAGE_HEAVY_DEMO_SIZE_PX=1024`):
  - `macos_vmmap_steady.regions.owned_unmapped_memory_dirty_bytes` p50=232,574,157 (~221.8 MiB)
  - Apple `footprint` category `Owned physical footprint (unmapped) (graphics)` p50=229,457,920 (~218.8 MiB)
  - `wgpu_metal_current_allocated_size_bytes_{min,max}`: 103,464,960 (~98.7 MiB; stable)
  - `renderer_gpu_images_bytes_estimate`: 0 (post-drop steady state)
- Notes:
  - This script intentionally uses a “grow, then drop, then idle” shape; avoid gating on `macos_vmmap_steady.physical_footprint_peak_bytes` because it includes the pre-drop peak.
  - The primary signal here is whether the post-drop steady state returns close to `text-heavy` / `todo` levels, not whether the peak phase was large.
  - `FRET_IMAGE_HEAVY_DEMO_POLL_AFTER_DROP` is an optional knob; an A/B (idle 1200 frames) showed no material delta with `poll=1` vs `poll=0` in the post-drop steady state.

Using `tools/diag-scripts/ui-gallery/memory/ui-gallery-code-editor-torture-memory-steady.json` on macOS/Metal (UI Gallery, editor-grade stress):

- Note: this page is behind `fret-ui-gallery`'s `gallery-dev` feature; launch with that enabled (or `gallery-full`) or the nav item will not exist.
- Repeat sample (N=5; captured via `fretboard diag repro --launch`):
  - `macos_vmmap_steady.physical_footprint_peak_bytes`: 387,343,974 .. 390,804,275 (~369.4 .. 372.7 MiB)
  - `macos_vmmap_steady.regions.owned_unmapped_memory_dirty_bytes`: 236,349,030 .. 236,978,176 (~225.4 .. 226.0 MiB)
  - `macos_vmmap_steady.regions.malloc_small_dirty_bytes`: 79,475,507 .. 83,745,178 (~75.8 .. 79.9 MiB)
  - `macos_vmmap_steady.regions.malloc_dirty_bytes_total`: 95,967,641 .. 100,204,544 (~91.5 .. 95.6 MiB)
  - `macos_vmmap_steady.tables.malloc_zones.total.frag_bytes`: 14,675,558 .. 18,765,005 (~14.0 .. 17.9 MiB)
  - `macos_vmmap_steady.regions.io_surface_dirty_bytes`: 37,748,736 (36.0 MiB; stable)
  - `macos_vmmap_steady.regions.io_accelerator_dirty_bytes`: 5,324,800 (5.1 MiB; stable)
  - `wgpu_metal_current_allocated_size_bytes`: 118,308,864 (~112.8 MiB; stable)
- App-side attribution (`app_snapshot.code_editor.torture.cache_sizes`, last snapshot):
  - `row_text_cache_entries`: 429
  - `row_text_cache_text_bytes_estimate_total`: ~29–30 KiB
  - `row_rich_cache_entries`: 429
  - `row_rich_cache_line_bytes_estimate_total`: ~29–30 KiB
- App-side attribution (`app_snapshot.code_editor.torture.memory`, last snapshot; single run):
  - `buffer_len_bytes`: 1,477,870 (~1.4 MiB)
  - `buffer_line_count`: 20,004
  - `undo_len`: 0 (limit 512)
  - `undo_text_bytes_estimate_total`: 0
- Text system attribution (`resource_caches.render_text`, last snapshot; single run):
  - `registered_font_blobs_total_bytes`: 0 (no injected memory-backed fonts observed)
  - `baseline_metrics_cache_entries`: 5
  - `shape_cache_bytes_estimate_total`: ~6.3 MiB (best-effort; excludes allocator overhead)
  - `blob_paint_palette_bytes_estimate_total`: ~0.06 MiB (best-effort)

Interpretation:

- This workload's headline "high memory" is **not explained by GPU allocation** (stable ~113 MiB) nor by
  the measured code editor paint caches (tens of KiB). The dominant CPU-side contributors remain:
  - `owned unmapped memory` dirty (allocator retention / sticky reservations), and
  - `MALLOC_SMALL` dirty (heap allocations + fragmentation).
- The new best-effort text cache byte estimates (shape cache + blob payload slices) are **single-digit MiB** even in the editor torture scenario, which further weakens the hypothesis that “font/text caches explain the high footprint”.
- Allocator A/B spot-check (single-run, `apps/fret-demo` `ui_gallery` binary, `--release`):
  - `system`:
    - `owned unmapped memory` dirty: 236,349,030 (~225.4 MiB)
    - `MALLOC_SMALL` dirty: 79,491,891 (~75.8 MiB)
    - `malloc_dirty_bytes_total`: 95,984,025 (~91.5 MiB)
    - malloc zones total frag: 14,505,165 (~13.8 MiB)
  - `mimalloc`:
    - `owned unmapped memory` dirty: 236,978,176 (~226.0 MiB)
    - `MALLOC_SMALL` dirty: 81,317,069 (~77.5 MiB)
    - `malloc_dirty_bytes_total`: 97,671,578 (~93.1 MiB)
    - malloc zones total frag: 16,006,963 (~15.3 MiB)
  - `jemalloc`:
    - `owned unmapped memory` dirty: 236,978,176 (~226.0 MiB)
    - `MALLOC_SMALL` dirty: 81,631,642 (~77.9 MiB)
    - `malloc_dirty_bytes_total`: 98,212,250 (~93.7 MiB)
    - malloc zones total frag: 16,725,606 (~16.0 MiB)
  - In this workload, switching Rust's global allocator does not materially change the headline
    `owned unmapped memory` bucket, and slightly increases `MALLOC_*` / frag signals.
- Diagnostics stability note:
  - Full debug snapshot capture can make bundle dumps prohibitively expensive in editor torture scenarios.
    This script therefore sets env defaults (via `meta.env_defaults`) to record **stats-only** debug
    snapshots: `FRET_DIAG_DEBUG_SNAPSHOT=0`.
  - Text/font attribution note:
    - `resource_caches.render_text` now also reports best-effort font DB/cache counters:
      - `registered_font_blobs_{count,total_bytes}` (injected memory-backed fonts)
      - `family_id_cache_entries`, `baseline_metrics_cache_entries` (shaper caches)

Allocator A/B (empty idle, `--release`, `fretboard diag repro`, same script):

- System allocator:
  - `macos_vmmap_steady.physical_footprint_peak_bytes`: 284,164,096
  - `owned unmapped memory` dirty: 213,594,931
  - Default malloc zone: 23,907,533 allocated, 15,623,782 frag (~40%)
  - `wgpu_metal_current_allocated_size_bytes`: 32,161,792
- `mimalloc`:
  - `macos_vmmap_steady.physical_footprint_peak_bytes`: 285,212,672 (Δ +1,048,576 vs system)
  - `owned unmapped memory` dirty: 213,594,931 (Δ 0 vs system)
  - Default malloc zone: 7,843,840 allocated, 5,574,656 frag (~42%)
  - `wgpu_metal_current_allocated_size_bytes`: 32,161,792
- `jemalloc`:
  - `macos_vmmap_steady.physical_footprint_peak_bytes`: 280,494,080 (Δ -3,670,016 vs system)
  - `owned unmapped memory` dirty: 216,321,229 (Δ +2,726,298 vs system)
  - Default malloc zone: 7,814,144 allocated, 4,572,160 frag (~37%)
  - `wgpu_metal_current_allocated_size_bytes`: 32,161,792

macOS allocator knob spot-check (ui-gallery code editor torture, single run):

- Default (no `MallocNanoZone` override):
  - `owned unmapped memory` dirty: ~225.4 MiB
  - `MALLOC_SMALL` dirty: ~78.1 MiB
  - malloc zones total frag: ~16.1 MiB
- `MallocNanoZone=0`:
  - `owned unmapped memory` dirty: ~226.0 MiB (no improvement observed)
  - `MALLOC_SMALL` dirty: ~80.2 MiB (slightly higher)
  - malloc zones total frag: ~18.0 MiB (higher)

Interpretation:

- GPU memory can be substantial but may not be reflected by `physical footprint` in a stable way.
- The largest CPU-side “mystery” is `owned unmapped memory` dirty, which likely reflects allocator
  behavior, caching, or sticky runtime allocations.
- The allocator choice strongly affects the default malloc zone (allocated + frag), but does not
  materially change the `owned unmapped memory` headline in this baseline.
- Diagnostics stability note:
  - `diag repro --launch` previously had a rare timeout where the script bundle existed but tooling
    never observed a final `script.result` stage. Root cause was a tooling write-back race to the
    runtime-owned `<out_dir>/script.result.json` in filesystem mode; fixed by avoiding writes to
    that path from tooling.
- Text atlas optimization note:
  - After `perf(render): lazily allocate mask glyph atlas pages`, the `render_text` mask atlas no
    longer preallocates all pages. Observed impact (same scripts):
    - `text-heavy`: `render_text_atlas_bytes_live_estimate_total` is ~20 MiB (mask pages `1`).
  - vmmap region attribution note:
    - `resource.footprint.json.macos_vmmap_steady.regions` now also includes:
      - `io_surface_dirty_bytes` (Metal-backed surfaces/textures)
      - `io_accelerator_dirty_bytes` (GPU driver allocations)
      - `malloc_small_dirty_bytes` (CPU heap bucket)
      - `malloc_dirty_bytes_total` (sum of `MALLOC_*` vmmap regions)
    - `resource.footprint.json.macos_vmmap_steady.tables.malloc_zones` now includes:
      - `default_zone` (best-effort `DefaultMallocZone` row)
      - `total` (allocated/frag/dirty sums across all zones)
    - These are intended to support more actionable macOS gates than “just physical footprint”.
    - `diag repro` now supports additional allocator-focused thresholds:
      - `--max-macos-malloc-dirty-bytes-total`
      - `--max-macos-malloc-zones-total-allocated-bytes`
      - `--max-macos-malloc-zones-total-frag-bytes`
      - `--max-macos-malloc-zones-total-dirty-bytes`
  - wgpu allocator sampling note:
    - Bundles may now report `wgpu_allocator_sample_present=true` even when
      `wgpu_allocator_report_present=false` (e.g. Metal-only `currentAllocatedSize` path).

## Goals

- Make “memory footprint” a first-class, scriptable diagnostic with stable outputs.
- Separate CPU vs GPU budgets and make both observable in `diag repro` bundles.
- Provide at least one additional measurable reduction (beyond text atlas prealloc), backed by a gate.

## Non-goals

- Perfect leak detection (e.g. full heap tracing) in v1.
- Cross-platform parity for every metric (we focus on macOS first, then extend).
- Eliminating all caching (editor-grade UIs need caches; we want **bounded** caches).

## Principles

- Prefer evidence that is cheap to collect in CI-like runs (seconds, not minutes).
- “One number” is not enough: always pair **a headline metric** with **top contributors**.
- Track “peak during lifetime” vs “steady state” explicitly.
- Avoid relying on `cargo run` compilation cost; use `--launch` with a prebuilt binary.

## Work Plan

### 0) Close the loop (v1 tracks)

For each track, the closure deliverable is: **(script) + (bundle fields) + (gate)**.

1) **Allocator / retained pages track (CPU)**
   - Explain and bound `owned unmapped memory` dirty + malloc fragmentation/dirty (`MALLOC_*`).
   - Primary evidence: `resource.footprint.json.macos_vmmap_steady.{regions,tables}`.
2) **Text / fonts track (CPU + GPU)**
   - Explain and bound text shaping caches and glyph atlas growth.
   - Primary evidence: `resource_caches.render_text.*` + text atlas bytes gate.
3) **Renderer / wgpu track (GPU + driver-backed regions)**
   - Explain and bound Metal/wgpu allocations and render target budgets.
   - Primary evidence: `wgpu_metal_current_allocated_size_bytes`, render target/image budgets, and
     `macos_vmmap_steady.regions.{io_surface_dirty_bytes,io_accelerator_dirty_bytes}`.

### 1) Improve attribution (tool-side)

- Parse `resource.vmmap_summary.txt` into a structured summary and persist it into
  `resource.footprint.json` (so comparisons do not require manual text parsing):
  - Regions: top resident/dirty + key buckets (`owned unmapped`, `IOSurface`, `IOAccelerator`,
    `MALLOC_*`).
  - `MALLOC ZONE`: default zone row (best-effort) + totals + top allocated/frag.
- Implemented (to reduce exit-time bias): two tool-side `vmmap -summary` captures:
  - Pre-exit “still running” (preferred for gates):
    - Raw: `resource.vmmap_summary.steady.txt`
    - Structured: `resource.footprint.json.macos_vmmap_steady`
  - Post-exit-signal (after writing the exit trigger, best-effort):
    - Raw: `resource.vmmap_summary.txt`
    - Structured: `resource.footprint.json.macos_vmmap`
  - Tooling gates and evidence summaries prefer `macos_vmmap_steady` when present.

### 2) Improve attribution (app-side)

- Add app-side stats for major caches (bytes + counts) where feasible:
  - Text system:
    - Already present: `resource_caches.render_text` cache entry counters.
    - Added: font DB counters (`registered_font_blobs_*`, baseline/family caches).
    - Next: add one “bytes” signal for text caches (even if approximate) to correlate with
      `MALLOC_SMALL` and malloc zone fragmentation.
  - Image cache bytes and “live texture” estimates (already partially present).
  - Code editor:
    - Added: buffer/undo best-effort memory snapshots (`app_snapshot.code_editor.torture.memory`).
    - Next: rope chunk + undo payload distribution estimates, and syntax parse/shaping caches (where
      applicable), so `MALLOC_SMALL` vs `owned unmapped` can be explained with app-level counters.
- Keep all fields “best effort” and clearly labeled (estimate vs exact).

### 3) Build a minimal baseline matrix

Add scripted repros that isolate hypotheses:

- **Empty idle**: minimal window, no text, no images (baseline CPU + GPU).
- **Text heavy**: many font faces, emoji, and diverse glyphs (forces atlas growth).
- **Image heavy**: representative image decode + texture upload path (forces texture cache).
  - Added: `apps/fret-demo --bin image_heavy_memory_demo` + `tools/diag-scripts/image-heavy-memory-steady.json`
  - Optional knobs:
    - `FRET_IMAGE_HEAVY_DEMO_COUNT` (default `24`)
    - `FRET_IMAGE_HEAVY_DEMO_SIZE_PX` (default `1024`)

Each script must:

- Wait for a steady window (N frames).
- Export a bundle + tool footprint.
- Be stable under `--release` builds.

### 4) Land bounded optimizations with gates

Optimizations should be:

- Small and localized.
- Verified by a script + evidence index summary.
- Guarded by a gate (macOS footprint, and/or Metal allocated size).

## Initial candidate optimizations (ordered)

1) **Allocator/retained pages investigation** (largest CPU-side unknown):
   - Try a “switch allocator” A/B (mimalloc/jemalloc) locally to validate the hypothesis before
     shipping anything.
   - If allocator choice materially changes `owned unmapped memory` dirty or fragmentation, decide
     whether to keep it as a dev-only knob or a shipping default.
2) **Text cache bounds**:
   - Add explicit upper bounds / eviction policies where unbounded maps exist.
3) **Render target budgets**:
   - Ensure intermediate target pools are not over-reserving for small demos.

## Evidence & Gates

Primary evidence sources:

- `resource.footprint.json` (tool-side process sampling + vmmap summary)
- `resource.vmmap_regions_sorted.steady.txt` (tool-side; address-level region list, sorted by size; truncated)
- `evidence.index.json.resources.bundle_last_frame_stats` (app-side last-frame stats)

Candidate gates:

- `--max-macos-physical-footprint-peak-bytes`
- `--max-macos-owned-unmapped-memory-dirty-bytes`
- `--max-macos-owned-unmapped-memory-dirty-bytes-linear-vs-renderer-gpu-images` (format: `<intercept_bytes>,<slope_ppm>[,<headroom_bytes>]`)
- `--max-macos-io-surface-dirty-bytes`
- `--max-macos-io-accelerator-dirty-bytes`
- `--max-macos-malloc-small-dirty-bytes`
- `--max-renderer-gpu-images-bytes-estimate`
- `--max-renderer-gpu-render-targets-bytes-estimate`
- `--max-renderer-intermediate-peak-in-use-bytes`
- `--max-wgpu-metal-current-allocated-size-bytes` (macOS/Metal; best-effort)
- `--max-wgpu-metal-current-allocated-size-bytes-linear-vs-renderer-gpu-images` (format: `<intercept_bytes>,<slope_ppm>[,<headroom_bytes>]`)
- `--max-wgpu-hub-buffers` (requires `--env FRET_DIAG_WGPU_REPORT=1`; best-effort)
- `--max-wgpu-hub-textures` (requires `--env FRET_DIAG_WGPU_REPORT=1`; best-effort)
- `--max-wgpu-hub-render-pipelines` (requires `--env FRET_DIAG_WGPU_REPORT=1`; best-effort)
- `--max-wgpu-hub-shader-modules` (requires `--env FRET_DIAG_WGPU_REPORT=1`; best-effort)
- `--max-render-text-atlas-bytes-live-estimate-total` (text-heavy attribution; stable, derived from `resource_caches.render_text`)
- `--max-render-text-registered-font-blobs-total-bytes` (guards memory-backed font injection growth; `resource_caches.render_text`)
- `--max-render-text-registered-font-blobs-count` (guards memory-backed font injection churn; `resource_caches.render_text`)
- `--max-render-text-shape-cache-entries` (guards unbounded text shaping cache growth; `resource_caches.render_text`)
- `--max-render-text-blob-cache-entries` (guards unbounded text blob cache growth; `resource_caches.render_text`)
- `--max-render-text-shape-cache-bytes-estimate-total` (best-effort; `resource_caches.render_text`)
- `--max-render-text-blob-paint-palette-bytes-estimate-total` (best-effort; `resource_caches.render_text`)
- `--max-render-text-blob-decorations-bytes-estimate-total` (best-effort; `resource_caches.render_text`)
- `--max-code-editor-buffer-len-bytes` (UI Gallery; `app_snapshot.code_editor.torture.memory`)
- `--max-code-editor-undo-text-bytes-estimate-total` (UI Gallery; `app_snapshot.code_editor.torture.memory`)
- `--max-code-editor-row-text-cache-entries` (UI Gallery; `app_snapshot.code_editor.torture.cache_sizes`)
- `--max-code-editor-row-rich-cache-entries` (UI Gallery; `app_snapshot.code_editor.torture.cache_sizes`)

Repeat gates (distribution-aware; recommended for CI / flake-resistant baselines):

- `fretboard diag repeat ... --check-memory-p90-max <key>:<bytes>`
  - Uses all `evidence.index.json` samples under the repeat output dir and fails if:
    - any sample is missing the requested `<key>`, or
    - the p90 value exceeds `<bytes>`.
- `fretboard diag repeat ... --no-compare`
  - Skips bundle-to-baseline diffing so memory/distribution runs can pass even when payloads are intentionally non-deterministic across runs.

Recommended local gate baselines (macOS, 2026-03-04):

- `empty-idle-memory-steady`:
  - `--max-macos-physical-footprint-peak-bytes 335544320` (320 MiB)
  - `--max-macos-owned-unmapped-memory-dirty-bytes 241172480` (230 MiB)
  - `--max-render-text-atlas-bytes-live-estimate-total 16777216` (16 MiB)
  - Optional (requires `--env FRET_DIAG_WGPU_ALLOCATOR_REPORT=1`):
    - `--max-wgpu-metal-current-allocated-size-bytes 52428800` (50 MiB)
- `text-heavy-memory-steady`:
  - `--max-macos-physical-footprint-peak-bytes 440401920` (420 MiB)
  - `--max-macos-owned-unmapped-memory-dirty-bytes 304087040` (290 MiB)
  - `--max-render-text-atlas-bytes-live-estimate-total 50331648` (48 MiB)
  - Optional (best-effort; monitor-only until we have a stable distribution):
    - `--max-render-text-shape-cache-bytes-estimate-total 33554432` (32 MiB)
  - Optional (requires `--env FRET_DIAG_WGPU_ALLOCATOR_REPORT=1`):
    - `--max-wgpu-metal-current-allocated-size-bytes 167772160` (160 MiB)
- `image-heavy-memory-steady`:
  - `--max-macos-physical-footprint-peak-bytes 536870912` (512 MiB)
  - `--max-macos-owned-unmapped-memory-dirty-bytes 402653184` (384 MiB)
  - `--max-macos-io-surface-dirty-bytes 67108864` (64 MiB)
  - `--max-macos-io-accelerator-dirty-bytes 16777216` (16 MiB)
  - `--max-macos-malloc-small-dirty-bytes 67108864` (64 MiB)
  - `--max-renderer-gpu-images-bytes-estimate 134217728` (128 MiB)
  - `--max-renderer-gpu-render-targets-bytes-estimate 67108864` (64 MiB)
  - `--max-renderer-intermediate-peak-in-use-bytes 67108864` (64 MiB)
  - Optional (requires `--env FRET_DIAG_WGPU_ALLOCATOR_REPORT=1`):
    - `--max-wgpu-metal-current-allocated-size-bytes 268435456` (256 MiB)
  - Optional (when scripts vary image pressure; requires `--env FRET_DIAG_WGPU_ALLOCATOR_REPORT=1`):
    - `--max-macos-owned-unmapped-memory-dirty-bytes-linear-vs-renderer-gpu-images 251658240,1010000,33554432` (240 MiB + 1.01 * images + 32 MiB)
    - `--max-wgpu-metal-current-allocated-size-bytes-linear-vs-renderer-gpu-images 117440512,1010000,33554432` (112 MiB + 1.01 * images + 32 MiB)
- `image-heavy-memory-steady-after-drop`:
  - Note: do not set `--max-macos-physical-footprint-peak-bytes` for this scenario; the script includes a pre-drop peak by design.
  - `--max-macos-owned-unmapped-memory-dirty-bytes 293601280` (280 MiB)
  - `--max-renderer-gpu-images-bytes-estimate 1048576` (1 MiB; should be ~0 after `unregister_image`)
  - Optional (requires `--env FRET_DIAG_WGPU_ALLOCATOR_REPORT=1`):
    - `--max-wgpu-metal-current-allocated-size-bytes 134217728` (128 MiB)
- `todo-memory-steady`:
  - `--max-macos-physical-footprint-peak-bytes 419430400` (400 MiB)
  - `--max-macos-owned-unmapped-memory-dirty-bytes 268435456` (256 MiB)
  - `--max-macos-malloc-small-dirty-bytes 100663296` (96 MiB)
  - `--max-macos-io-surface-dirty-bytes 50331648` (48 MiB)
  - `--max-macos-io-accelerator-dirty-bytes 33554432` (32 MiB)
  - `--max-render-text-atlas-bytes-live-estimate-total 16777216` (16 MiB)
- `ui-gallery-code-editor-torture-memory-steady`:
  - `--max-macos-physical-footprint-peak-bytes 603979776` (576 MiB)
  - `--max-macos-owned-unmapped-memory-dirty-bytes 268435456` (256 MiB)
  - `--max-macos-malloc-small-dirty-bytes 234881024` (224 MiB)
  - `--max-macos-io-surface-dirty-bytes 67108864` (64 MiB)
  - `--max-macos-io-accelerator-dirty-bytes 16777216` (16 MiB)
  - Optional (best-effort; monitor-only until we have a stable distribution):
    - `--max-render-text-atlas-bytes-live-estimate-total 16777216` (16 MiB)
    - `--max-render-text-shape-cache-bytes-estimate-total 16777216` (16 MiB)
    - `--max-render-text-blob-paint-palette-bytes-estimate-total 8388608` (8 MiB)
    - `--max-render-text-blob-decorations-bytes-estimate-total 4194304` (4 MiB)
  - `--max-wgpu-metal-current-allocated-size-bytes 150994944` (144 MiB)

Note: these numbers are intentionally conservative and should be revisited when:

- the script payload changes (fonts/emoji coverage),
- the renderer backend changes (wgpu/wgpu-core bumps),
- or the measurement surface changes (new diagnostics fields enabled by default).

### Repeat distributions (local 2026-03-06)

These runs were captured via `fretboard diag repeat` and summarized with `fretboard diag memory-summary`.

Note:

- `diag repeat` currently compares each run's bundle against the first passing run. For editor-grade
  workloads like UI Gallery, this can produce `differing_runs > 0` even when every script run
  itself passes. Treat the memory samples as valid when `stage_counts.passed == repeat`.
- For memory-only calibration, prefer `--no-compare` so `repeat.summary.json` reflects only script/tooling failures and explicit memory gates.

- `empty-idle-memory-steady` (`target/diag/mem-empty-idle-repeat3-20260306/`):
  - `macos_physical_footprint_peak_bytes`: p90 ≈ 267.0 MiB
  - `macos_owned_unmapped_memory_dirty_bytes`: p90 ≈ 204.0 MiB
  - `macos_malloc_small_dirty_bytes`: p90 ≈ 32.8 MiB
  - Top `footprint` category: `Owned physical footprint (unmapped) (graphics)` ≈ 202.1 MiB
- `text-heavy-memory-steady` (`target/diag/mem-text-heavy-repeat3-20260306/`; N=3):
  - `macos_physical_footprint_peak_bytes`: p90 ≈ 344.4 MiB
  - `macos_owned_unmapped_memory_dirty_bytes`: p90 ≈ 237.5 MiB
  - `render_text_atlas_bytes_live_estimate_total`: p90 ≈ 20.0 MiB
  - `render_text_shape_cache_bytes_estimate_total`: p90 ≈ 0.26 MiB
  - Top `footprint` category: `Owned physical footprint (unmapped) (graphics)` ≈ 235.0 MiB
- `todo-memory-steady` (`target/diag/mem-todo-repeat10-20260306/`; N=10):
  - `macos_physical_footprint_peak_bytes`: p50 ≈ 349.7 MiB, p90 ≈ 356.6 MiB
  - `macos_owned_unmapped_memory_dirty_bytes`: p50/p90 ≈ 227.6 MiB
  - `macos_malloc_small_dirty_bytes`: p50 ≈ 68.7 MiB, p90 ≈ 74.4 MiB
  - `render_text_atlas_bytes_live_estimate_total`: p50/p90 ≈ 4.0 MiB
  - Top `footprint` category: `Owned physical footprint (unmapped) (graphics)` p90 ≈ 221.7 MiB
  - Second `footprint` category: `MALLOC_SMALL` p90 ≈ 74.4 MiB
- `ui-gallery-code-editor-torture-memory-steady` (`target/diag/mem-ui-gallery-editor-repeat10-20260306/`; N=10):
  - `macos_physical_footprint_peak_bytes`: p50 ≈ 472.5 MiB, p90 ≈ 495.4 MiB
  - `macos_owned_unmapped_memory_dirty_bytes`: p50/p90 ≈ 228.1 MiB
  - `macos_malloc_small_dirty_bytes`: p50 ≈ 175.1 MiB, p90 ≈ 187.5 MiB
  - `render_text_shape_cache_bytes_estimate_total`: p50 ≈ 5.90 MiB, p90 ≈ 5.91 MiB
  - `render_text_atlas_bytes_live_estimate_total`: p50/p90 ≈ 4.0 MiB
  - Top `footprint` category: `Owned physical footprint (unmapped) (graphics)` p90 ≈ 224.5 MiB
  - Second `footprint` category: `MALLOC_SMALL` p90 ≈ 187.6 MiB

Interpretation:

- `todo` stays close to the previously observed framework baseline: the dominant bucket is still the
  graphics-owned unmapped footprint, with a moderate `MALLOC_SMALL` heap.
- `ui-gallery` does **not** materially raise the headline graphics bucket versus `todo`; instead, it
  raises `MALLOC_SMALL` by ~110 MiB at p90. This is now the clearest app/framework-managed target
  for optimization.

### UI Gallery shell bisect (local 2026-03-06)

Using one-off repeat runs on the `ui_gallery` binary:
Repo suite:

- `tools/diag-scripts/suites/ui-gallery-memory-bisect/suite.json`
  - Includes `card`, `minimal_root`, `simple_sidebar`, and `simple_content` steady-state scripts.

Using one-off repeat runs on the `ui_gallery` binary:

- `card` page steady (`target/diag/mem-ui-gallery-card-repeat3-20260306/`; N=3):
  - `macos_physical_footprint_peak_bytes` p90 ≈ 526.8 MiB
  - `macos_malloc_small_dirty_bytes` p90 ≈ 233.0 MiB
  - `macos_owned_unmapped_memory_dirty_bytes` p90 ≈ 227.7 MiB
- `FRET_UI_GALLERY_BISECT=1` (minimal root; `target/diag/mem-ui-gallery-minroot-repeat3-20260306/`; N=3):
  - `macos_physical_footprint_peak_bytes` p90 ≈ 314.1 MiB
  - `macos_malloc_small_dirty_bytes` p90 ≈ 40.8 MiB
  - `macos_owned_unmapped_memory_dirty_bytes` p90 ≈ 219.7 MiB
- `FRET_UI_GALLERY_BISECT=16` (`simple_sidebar`; `target/diag/mem-ui-gallery-simple-sidebar-repeat3-20260306/`; N=3):
  - `macos_malloc_small_dirty_bytes` p90 ≈ 199.3 MiB
- `FRET_UI_GALLERY_BISECT=32` (`simple_content`; `target/diag/mem-ui-gallery-simple-content-repeat3-20260306/`; N=3):
  - `macos_malloc_small_dirty_bytes` p90 ≈ 214.7 MiB

Interpretation:

- The large `MALLOC_SMALL` heap in `ui_gallery` is **not code-editor-specific**; the `card` page shows the same pattern.
- The heap drops sharply under `BISECT_MINIMAL_ROOT`, which means the dominant retained heap is tied to the normal gallery shell/render path rather than the Metal baseline.
- `simple_sidebar` and `simple_content` remain high individually, which suggests the next attribution step should target the **shared non-minimal shell path** (e.g. top bar / workspace frame / command palette / settings sheet / other common subtree state), not just page-specific demos.
- New `app_snapshot.shell` counters are tiny in the `card` sample (command registry strings ~26 KiB, page spec strings ~23 KiB, workspace tab strings <0.1 KiB), which means the large retained heap is **not** explained by app-managed shell metadata strings. The next measurement step likely needs widget/runtime-level counters rather than more page metadata counters.
- New bundle-derived tree counters also line up with heap size: a fresh `card` sample shows `ui_semantics_nodes`≈1152 and `ui_prepaint_nodes_visited`/`ui_interaction_records`≈1155, while `minimal_root` drops to `ui_semantics_nodes`=2 and zero prepaint/interaction records together with `MALLOC_SMALL` dropping back to ~43 MiB. That makes shared shell/tree scale the strongest current suspect.
- Subtree-level semantics counts narrow it further:
  - `card` sample: `ui_gallery_nav_scroll_semantics_subtree_nodes`≈848, `ui_gallery_page_overlay_semantics_subtree_nodes`≈220
  - explicit `FRET_UI_GALLERY_BISECT=16` (`simple_sidebar`): `MALLOC_SMALL` drops to ~235 MiB and total semantics nodes drop to ~714
  - explicit `FRET_UI_GALLERY_BISECT=32` (`simple_content`): `MALLOC_SMALL` drops to ~273 MiB while the nav subtree remains ≈848 and total semantics nodes remain ~934
  - command palette / settings button subtrees stay tiny (≈6 nodes each)
  This makes the sidebar/nav shell the strongest single retained-tree suspect, with the main page content shell as the next-largest contributor.

### UI Gallery nav cold-start sweep (local 2026-03-06)

To turn that suspicion into a cold-start experiment, the gallery now accepts `FRET_UI_GALLERY_NAV_QUERY` and exports `app_snapshot.shell.nav_visible_*` counters (groups / items / tags / bytes). These absolute numbers are lower than the first shell-bisect samples above because this worktree has since landed several refactors; treat the sweep below as the current branch-local baseline. Running the same `card` page script with different startup queries gives:

- Empty query (`target/diag/mem-ui-gallery-card-nav-empty-repeat3-20260306/`; N=3):
  - `ui_gallery_nav_visible_items_count` p50/p90 = 61
  - `ui_gallery_nav_scroll_semantics_subtree_nodes` p50/p90 = 321
  - `macos_malloc_small_dirty_bytes` p50/p90 ≈ 129.6 MiB
  - `macos_physical_footprint_peak_bytes` p50/p90 ≈ 408.9 MiB
- `FRET_UI_GALLERY_NAV_QUERY=card` (`target/diag/mem-ui-gallery-card-nav-card-repeat3-20260306/`; N=3):
  - `ui_gallery_nav_visible_items_count` p50/p90 = 2
  - `ui_gallery_nav_scroll_semantics_subtree_nodes` p50/p90 = 20
  - `macos_malloc_small_dirty_bytes` p50/p90 ≈ 107.7 MiB
  - `macos_physical_footprint_peak_bytes` p50/p90 ≈ 387.0 MiB
- `FRET_UI_GALLERY_NAV_QUERY=__none__` (`target/diag/mem-ui-gallery-card-nav-none-repeat3-20260306/`; N=3):
  - `ui_gallery_nav_visible_items_count` p50/p90 = 0
  - `ui_gallery_nav_scroll_semantics_subtree_nodes` p50/p90 = 7
  - `macos_malloc_small_dirty_bytes` p50/p90 ≈ 100.4 MiB
  - `macos_physical_footprint_peak_bytes` p50/p90 ≈ 379.2 MiB
- `FRET_UI_GALLERY_BISECT=16` (`target/diag/mem-ui-gallery-card-simple-sidebar-repeat3-20260306/`; N=3):
  - `ui_semantics_nodes` p50/p90 ≈ 161
  - `macos_malloc_small_dirty_bytes` p50/p90 ≈ 109.9 MiB

Interpretation:

- Hiding almost the entire nav list at startup (`61 -> 0` visible items) removes about **29.2 MiB** of `MALLOC_SMALL` and about **314 nav semantics nodes** (`321 -> 7`).
- Most of that drop already happens once the list shrinks from `61 -> 2` visible items: about **21.9 MiB** of `MALLOC_SMALL` disappears while the nav subtree falls from `321 -> 20` nodes.
- This confirms the retained nav list is a **real heap contributor**, not just a proxy metric. However, the empty-nav cold start still sits around **100 MiB `MALLOC_SMALL`**, so the nav list is only part of the remaining shell cost.
- `simple_sidebar` on the same page lands in the same rough band (~110 MiB), which means the next attribution step should focus on the **content / shared shell intercept** rather than spending more time on string metadata or the last few sidebar controls.

### UI Gallery card subtree attribution sweep (analysis-only; local 2026-03-06)

A follow-up pass fixed one diagnostics correctness issue first: `fretboard diag repeat --launch` was not forwarding `script.meta.env_defaults` into the launch environment. That meant the earlier "card" runs could silently fall back to the diag default page (`overlay`) instead of the intended `FRET_UI_GALLERY_START_PAGE=card`. The repeat launcher now honors script env defaults.

For subtree attribution we also need a different semantics export policy than the raw steady-memory gates. The analysis runs below used:

- `--env FRET_DIAG_DEBUG_SNAPSHOT=1`
- `--env FRET_DIAG_SEMANTICS_TEST_IDS_ONLY=0`
- `--env FRET_DIAG_BUNDLE_DUMP_SEMANTICS_TEST_IDS_ONLY=1`

This keeps the bundle on the real `card` page while preserving `test_id` roots **plus their ancestors** in `bundle.schema2.json`, so subtree counts remain connected. The default steady-memory scripts still use `FRET_DIAG_DEBUG_SNAPSHOT=0`; do not mix those raw-memory baselines with the subtree-analysis numbers below.

Using the same `ui-gallery-card-memory-steady.json` script with those overrides (N=3 each):

- Full card shell (`target/diag/mem-ui-gallery-card-analysis-empty-r3-20260306/`; all runs confirmed `selected_page=card`):
  - `MALLOC_SMALL` p50/p90 ≈ 395.7 / 407.0 MiB
  - peak working set (`resource.footprint.json`) p50 ≈ 522.5 MiB
  - `ui_semantics_nodes` p50 = 468
  - `ui_gallery_workspace_frame_semantics_subtree_nodes` p50 = 466
  - `ui_gallery_content_shell_semantics_subtree_nodes` p50 = 284
  - `ui_gallery_current_page_semantics_subtree_nodes` p50 = 282
  - `ui_gallery_nav_scroll_semantics_subtree_nodes` p50 = 133
  - `ui_gallery_top_bar_semantics_subtree_nodes` p50 = 33
- `FRET_UI_GALLERY_NAV_QUERY=__none__` (`target/diag/mem-ui-gallery-card-analysis-navnone-r3-20260306/`; `selected_page=card`):
  - `MALLOC_SMALL` p50/p90 ≈ 222.6 / 229.5 MiB
  - peak working set p50 ≈ 342.4 MiB
  - `ui_semantics_nodes` p50 = 336
  - `ui_gallery_workspace_frame_semantics_subtree_nodes` p50 = 334
  - `ui_gallery_content_shell_semantics_subtree_nodes` p50 = 284
  - `ui_gallery_current_page_semantics_subtree_nodes` p50 = 282
  - `ui_gallery_nav_scroll_semantics_subtree_nodes` p50 = 1
  - `ui_gallery_top_bar_semantics_subtree_nodes` p50 = 33
- `FRET_UI_GALLERY_BISECT=32` (`simple_content`; `target/diag/mem-ui-gallery-card-analysis-simplecontent-r3-20260306/`; `selected_page=card`):
  - `MALLOC_SMALL` p50/p90 ≈ 131.2 / 168.1 MiB
  - peak working set p50 ≈ 235.6 MiB
  - `ui_semantics_nodes` p50 = 183
  - `ui_gallery_workspace_frame_semantics_subtree_nodes` p50 = 181
  - `ui_gallery_top_bar_semantics_subtree_nodes` p50 = 33
  - `ui_gallery_content_shell_semantics_subtree_nodes` / `ui_gallery_current_page_semantics_subtree_nodes` are absent here by design because `BISECT_SIMPLE_CONTENT` replaces the normal content subtree.
- `FRET_UI_GALLERY_BISECT=32` + `FRET_UI_GALLERY_NAV_QUERY=__none__` (`target/diag/mem-ui-gallery-card-analysis-simplecontent-navnone-r3-20260306/`; `selected_page=card`):
  - `MALLOC_SMALL` p50/p90 ≈ 90.9 / 90.9 MiB
  - peak working set p50 ≈ 191.9 MiB
  - `ui_semantics_nodes` p50 = 51
  - `ui_gallery_workspace_frame_semantics_subtree_nodes` p50 = 49
  - `ui_gallery_top_bar_semantics_subtree_nodes` p50 = 33

Interpretation:

- On the real `card` page, the retained nav subtree is much more expensive than the earlier overlay-skewed samples suggested: removing the nav list from the full shell drops `MALLOC_SMALL` by about **173.1 MiB** (`395.7 -> 222.6 MiB`).
- After removing nav, replacing the normal card content with `simple_content` still removes another **131.8 MiB** of `MALLOC_SMALL` (`222.6 -> 90.9 MiB`). That makes the **card content / content shell** the next major heap contributor after nav, not a small residual.
- Even after both simplifications, there is still a shared-shell floor around **90.9 MiB** (`simple_content + nav none`). That floor now has a much smaller tree (`workspace_frame` p50 = 49, `top_bar` p50 = 33), so the next attribution step should split the remaining frame chrome / status bar / other always-on surfaces rather than fonts.
- The content subtree is clearly large enough to justify focused instrumentation: in the full-card sample the `content_shell` subtree is ~284 nodes and the current page subtree is ~282 nodes, versus ~133 nodes for the nav subtree and ~33 nodes for the top bar.
- Earlier 2026-03-06 `card` samples collected before the `diag repeat` env-default fix should be treated as **preliminary shell-shape evidence only**; the corrected `selected_page=card` analysis above supersedes them for page-specific attribution.

#### Card content-shell section sweep (local 2026-03-06)

To keep digging into the corrected `card` path, the gallery content tree now exports explicit subtree roots for:

- `ui-gallery-content-header`
- `ui-gallery-page-preview`
- `ui-gallery-status-bar`
- `ui-gallery-card-section-{demo,usage,size,card-content,meeting-notes,image,rtl,compositions,notes}`

`fret-diag` / `fretboard diag memory-summary` now surface matching `*_semantics_subtree_nodes` keys, so the analysis runs can attribute the stable `card` shell without opening raw bundles. Re-running the corrected analysis recipe (N=3 each):

- Full card shell (`target/diag/mem-ui-gallery-card-sections-empty-r3-20260306/`; all runs confirmed `selected_page=card`):
  - `MALLOC_SMALL` p50 ≈ **370.6 MiB**
  - peak physical footprint p50 ≈ **661.3 MiB**
  - `ui_semantics_nodes` p50 = `484`
  - `ui_gallery_workspace_frame_semantics_subtree_nodes` p50 = `482`
  - `ui_gallery_nav_scroll_semantics_subtree_nodes` p50 = `133`
  - `ui_gallery_content_shell_semantics_subtree_nodes` p50 = `300`
  - `ui_gallery_current_page_semantics_subtree_nodes` p50 = `298`
  - `ui_gallery_page_preview_semantics_subtree_nodes` p50 = `277`
  - `ui_gallery_content_header_semantics_subtree_nodes` p50 = `11`
  - `ui_gallery_status_bar_semantics_subtree_nodes` p50 = `4`
- `FRET_UI_GALLERY_NAV_QUERY=__none__` (`target/diag/mem-ui-gallery-card-sections-navnone-r3-20260306/`; all runs confirmed `selected_page=card`):
  - `MALLOC_SMALL` p50 ≈ **226.8 MiB**
  - peak physical footprint p50 ≈ **515.9 MiB**
  - `ui_semantics_nodes` p50 = `352`
  - `ui_gallery_workspace_frame_semantics_subtree_nodes` p50 = `350`
  - `ui_gallery_nav_scroll_semantics_subtree_nodes` p50 = `1`
  - `ui_gallery_content_shell_semantics_subtree_nodes` p50 = `300`
  - `ui_gallery_current_page_semantics_subtree_nodes` p50 = `298`
  - `ui_gallery_page_preview_semantics_subtree_nodes` p50 = `277`
  - `ui_gallery_content_header_semantics_subtree_nodes` p50 = `11`
  - `ui_gallery_status_bar_semantics_subtree_nodes` p50 = `4`

The section-level subtree sizes are stable across both runs because they live entirely under the page preview, not under nav. For the full-card sample, the largest section roots are:

- `ui_gallery_card_section_image_semantics_subtree_nodes` p50 = `51`
- `ui_gallery_card_section_compositions_semantics_subtree_nodes` p50 = `42`
- `ui_gallery_card_section_meeting_notes_semantics_subtree_nodes` p50 = `41`
- `ui_gallery_card_section_demo_semantics_subtree_nodes` p50 = `30`
- `ui_gallery_card_section_usage_semantics_subtree_nodes` / `ui_gallery_card_section_card_content_semantics_subtree_nodes` p50 = `25` each

Cross-checking the subtree totals gives a tighter picture of the content shell shape:

- The summed card-section roots account for **264 / 277** preview nodes, leaving only about **13** preview-card chrome / wrapper nodes outside the sections.
- `current_page` exceeds `page_preview` by **21** nodes; `content_header` alone accounts for **11** of those, leaving roughly **10** nodes for the scroll/container glue.
- `content_shell` exceeds `current_page` by only **2** nodes, so the remaining shell wrappers are negligible compared with the page preview body itself.
- `nav none` still removes about **143.8 MiB** of `MALLOC_SMALL` (`370.6 -> 226.8 MiB`) while the content-shell subtree counts remain unchanged. That is strong evidence that the new probes cleanly isolate the page/content region from the retained nav heap.

This does **not** assign bytes per section yet, but it narrows the next memory step considerably: the heap-heavy part of the corrected `card` page is overwhelmingly inside the preview body, and most of that preview body is in the section content rather than outer chrome or the pinned header. The next useful lever is a section-level bisect / startup knob so each large section can be hidden independently and translated into byte deltas.

#### Card section bisect byte sweep (local 2026-03-06)

The card page now supports startup-level section masks via `FRET_UI_GALLERY_BISECT`, and the first dedicated section scripts live under:

- `tools/diag-scripts/ui-gallery/memory/ui-gallery-card-no-image-memory-steady.json`
- `tools/diag-scripts/ui-gallery/memory/ui-gallery-card-no-compositions-memory-steady.json`
- `tools/diag-scripts/ui-gallery/memory/ui-gallery-card-no-meeting-notes-memory-steady.json`
- `tools/diag-scripts/ui-gallery/memory/ui-gallery-card-no-heavy-sections-memory-steady.json`
- `tools/diag-scripts/suites/ui-gallery-card-section-memory-bisect/suite.json`

Using the same analysis override (`FRET_DIAG_DEBUG_SNAPSHOT=1`, `FRET_DIAG_SEMANTICS_TEST_IDS_ONLY=0`, `FRET_DIAG_BUNDLE_DUMP_SEMANTICS_TEST_IDS_ONLY=1`) plus `FRET_UI_GALLERY_NAV_QUERY=__none__`:

- Baseline (`target/diag/mem-ui-gallery-card-sections-navnone-r3-20260306/`):
  - `MALLOC_SMALL` p50 ≈ **226.8 MiB**
  - `ui_gallery_page_preview_semantics_subtree_nodes` p50 = `277`
  - `ui_gallery_current_page_semantics_subtree_nodes` p50 = `298`
- No image (`target/diag/mem-ui-gallery-card-no-image-navnone-r3-20260306/`; all runs confirmed `selected_page=card`, `card_sections_hidden_count=1`):
  - `MALLOC_SMALL` p50 ≈ **188.1 MiB** (**-38.7 MiB** vs baseline)
  - `ui_gallery_page_preview_semantics_subtree_nodes` p50 = `226` (**-51**)
  - `ui_gallery_current_page_semantics_subtree_nodes` p50 = `247` (**-51**)
- No compositions (`target/diag/mem-ui-gallery-card-no-compositions-navnone-r3-20260306/`; all runs confirmed `selected_page=card`, `card_sections_hidden_count=1`):
  - `MALLOC_SMALL` p50 ≈ **185.3 MiB** (**-41.5 MiB** vs baseline)
  - `ui_gallery_page_preview_semantics_subtree_nodes` p50 = `235` (**-42**)
  - `ui_gallery_current_page_semantics_subtree_nodes` p50 = `256` (**-42**)
- No meeting notes (`target/diag/mem-ui-gallery-card-no-meeting-notes-navnone-r5-20260306/`; rerun at N=5 because the first N=3 pass was noisy):
  - `MALLOC_SMALL` p50 ≈ **188.6 MiB** (**-38.2 MiB** vs baseline)
  - `MALLOC_SMALL` p90 ≈ **190.1 MiB**
  - `ui_gallery_page_preview_semantics_subtree_nodes` p50 = `236` (**-41**)
  - `ui_gallery_current_page_semantics_subtree_nodes` p50 = `257` (**-41**)
- No heavy sections (`image + compositions + meeting notes`; `target/diag/mem-ui-gallery-card-no-heavy-sections-navnone-r3-20260306/`):
  - `MALLOC_SMALL` p50 ≈ **144.7 MiB** (**-82.1 MiB** vs baseline)
  - peak physical footprint p50 ≈ **433.3 MiB** (**-82.6 MiB** vs baseline)
  - `ui_gallery_page_preview_semantics_subtree_nodes` p50 = `143` (**-134**)
  - `ui_gallery_current_page_semantics_subtree_nodes` p50 = `164` (**-134**)

Interpretation:

- These three "heavy" sections account for **134 / 277** preview nodes, but already remove about **82.1 MiB** of `MALLOC_SMALL` on the `nav none` card page. That makes them the first concrete page-content heap hotspots, not just large subtree counts.
- The individual section deltas are **not additive**: `no_image` + `no_compositions` + `no_meeting_notes` sum to about **118.4 MiB**, while disabling all three together removes about **82.1 MiB**. That implies a substantial shared allocation overlap across those sections (likely shared doc/code-shell retained objects, layout state, or strings), not three fully independent heaps.
- The tracked GPU/text cache counters do **not** explain the delta by themselves: between the `nav none` baseline and `no heavy sections`, `renderer_gpu_images_bytes_estimate` stays at `0`, `render_text_atlas_bytes_live_estimate_total` stays at `4 MiB`, and `render_text_shape_cache_bytes_estimate_total` only moves by about `0.13 MiB`. So the dominant savings are in retained UI/tree/heap allocations that current cache counters do not yet attribute directly.
- Compared with the earlier `simple_content + nav none` floor (~**90.9 MiB** `MALLOC_SMALL`), removing the heavy trio still leaves about **53.8 MiB** of content-specific heap on the table (`144.7 -> 90.9 MiB`). The next cut should therefore target the remaining `demo / usage / size / card_content / rtl / notes` cluster and the shared docs/code-tab shell rather than nav, fonts, or the pinned header.

#### Card overlap / code-tab sweep (local 2026-03-06)

The second-stage card sweep extends the same `FRET_UI_GALLERY_BISECT` path with:

- `tools/diag-scripts/ui-gallery/memory/ui-gallery-card-no-light-sections-memory-steady.json`
- `tools/diag-scripts/ui-gallery/memory/ui-gallery-card-preview-only-memory-steady.json`
- `tools/diag-scripts/suites/ui-gallery-card-section-memory-bisect/suite.json`

Using the same analysis override (`FRET_DIAG_DEBUG_SNAPSHOT=1`, `FRET_DIAG_SEMANTICS_TEST_IDS_ONLY=0`, `FRET_DIAG_BUNDLE_DUMP_SEMANTICS_TEST_IDS_ONLY=1`) plus `FRET_UI_GALLERY_NAV_QUERY=__none__`:

- No light sections (`demo + usage + size + card_content + rtl + notes`; `target/diag/mem-ui-gallery-card-no-light-sections-navnone-r3-20260306/`):
  - `MALLOC_SMALL` p50 ≈ **149.7 MiB** (**-77.1 MiB** vs baseline)
  - peak physical footprint p50 ≈ **438.2 MiB** (**-77.7 MiB** vs baseline)
  - `ui_gallery_page_preview_semantics_subtree_nodes` p50 = `147` (**-130**)
  - `macos_malloc_zones_total_allocated_bytes` p50 ≈ **111.0 MiB** (**-51.5 MiB** vs baseline)
  - `macos_malloc_zones_total_frag_bytes` p50 ≈ **43.2 MiB** (**-25.9 MiB** vs baseline)
- Preview only (`card_code_tabs_disabled=true`; `target/diag/mem-ui-gallery-card-preview-only-navnone-r3-20260306/`):
  - `MALLOC_SMALL` p50 ≈ **169.5 MiB** (**-57.3 MiB** vs baseline)
  - peak physical footprint p50 ≈ **458.3 MiB** (**-57.6 MiB** vs baseline)
  - `ui_gallery_page_preview_semantics_subtree_nodes` p50 = `197` (**-80**)
  - `macos_malloc_zones_total_allocated_bytes` p50 ≈ **161.3 MiB** (**-1.2 MiB** vs baseline)
  - `macos_malloc_zones_total_frag_bytes` p50 ≈ **12.8 MiB** (**-56.3 MiB** vs baseline)

Interpretation:

- The remaining "light" cluster is almost the same order of live heap as the earlier heavy trio: `no heavy sections` and `no light sections` both reduce `macos_malloc_zones_total_allocated_bytes` by about **50–52 MiB**, so the page has two similarly expensive preview-body halves rather than one dominant residual tail.
- `preview only` removes exactly **80** preview nodes, and every code-backed section drops by exactly **10** nodes (`30 -> 20`, `25 -> 15`, `20 -> 10`, ...). That is strong evidence that the shared `DocSection` code-tab shell contributes a fixed semantics shape per section.
- The byte pattern is more important than the node pattern: `preview only` barely changes `macos_malloc_zones_total_allocated_bytes` (**-1.2 MiB**) but collapses `macos_malloc_zones_total_frag_bytes` by about **56.3 MiB**. This strongly suggests the code-tab / code-view path is currently a **heap-fragmentation amplifier**, not just a holder of large live allocations.
- This fragmentation signal likely explains much of the earlier non-additivity: the three individual heavy-section sweeps mostly reduced `MALLOC_SMALL` by shrinking fragmented small-allocation slabs, while the grouped `no heavy sections` sweep also exposed the real live-allocation drop (~**50.5 MiB** allocated + ~**31.8 MiB** fragmentation).
- The tracked GPU/text counters still do **not** explain the effect (`renderer_gpu_images_bytes_estimate` remains `0`; text atlas stays at `4 MiB`; shape cache only moves slightly). The next diagnostic step should focus on code-view / docs-shell runtime counters and combined preview-body masks rather than fonts or GPU residency.

#### Combined preview-body sweep (local 2026-03-06)

To separate "preview body live bytes" from the code-tab fragmentation tax, two combined masks were added:

- `tools/diag-scripts/ui-gallery/memory/ui-gallery-card-no-heavy-sections-preview-only-memory-steady.json`
- `tools/diag-scripts/ui-gallery/memory/ui-gallery-card-no-light-sections-preview-only-memory-steady.json`

Using the same `nav none` + analysis override recipe:

- No heavy sections + preview only (`target/diag/mem-ui-gallery-card-no-heavy-sections-preview-only-navnone-r3-20260306/`):
  - `MALLOC_SMALL` p50 ≈ **117.3 MiB** (**-52.3 MiB** vs `preview only`)
  - peak physical footprint p50 ≈ **405.4 MiB** (**-52.9 MiB** vs `preview only`)
  - `macos_malloc_zones_total_allocated_bytes` p50 ≈ **108.9 MiB** (**-52.4 MiB** vs `preview only`)
  - `macos_malloc_zones_total_frag_bytes` p50 ≈ **12.8 MiB** (**~flat** vs `preview only`)
  - `ui_gallery_page_preview_semantics_subtree_nodes` p50 = `93` (**-104** vs `preview only`)
- No light sections + preview only (`target/diag/mem-ui-gallery-card-no-light-sections-preview-only-navnone-r3-20260306/`):
  - `MALLOC_SMALL` p50 ≈ **118.5 MiB** (**-51.0 MiB** vs `preview only`)
  - peak physical footprint p50 ≈ **407.1 MiB** (**-51.2 MiB** vs `preview only`)
  - `macos_malloc_zones_total_allocated_bytes` p50 ≈ **108.9 MiB** (**-52.4 MiB** vs `preview only`)
  - `macos_malloc_zones_total_frag_bytes` p50 ≈ **14.0 MiB** (**+1.2 MiB** vs `preview only`)
  - `ui_gallery_page_preview_semantics_subtree_nodes` p50 = `117` (**-80** vs `preview only`)

Interpretation:

- Once code tabs are removed, the "heavy" and "light" halves of the card preview are almost perfectly symmetric in **live allocated bytes**: each half accounts for about **52.4 MiB** of `macos_malloc_zones_total_allocated_bytes`.
- The fragmentation story is now much clearer: both combined masks stay near the same **13–14 MiB** frag floor as `preview only`, which means the earlier `~56 MiB` frag drop really does belong to the shared code-tab / code-view path rather than the preview bodies themselves.
- The subtree counts are **not** proportional to live bytes once code tabs are removed. The heavy half removes **104** preview nodes while the light half removes only **80**, yet both delete essentially the same live allocation. So subtree nodes remain a useful shape probe, but not a byte proxy.
- The remaining unknown is the shared preview/page shell floor that survives after one half is removed. The next isolating cut should therefore be an "all sections hidden" / page-shell-only run (or equivalent runtime counters inside the doc/code shell) rather than more one-off section sweeps.

#### Card shared floor sweep (local 2026-03-06)

To measure the residual doc/page scaffold directly, the suite now also includes:

- `tools/diag-scripts/ui-gallery/memory/ui-gallery-card-page-shell-only-memory-steady.json`
- `tools/diag-scripts/ui-gallery/memory/ui-gallery-card-scaffold-only-memory-steady.json`

Using the same `nav none` + analysis override recipe:

- Page shell only (`all sections hidden`; `target/diag/mem-ui-gallery-card-page-shell-only-navnone-r3-20260306/`):
  - `MALLOC_SMALL` p50 ≈ **94.1 MiB**
  - `macos_malloc_zones_total_allocated_bytes` p50 ≈ **84.1 MiB**
  - `macos_malloc_zones_total_frag_bytes` p50 ≈ **14.3 MiB**
  - `ui_gallery_page_preview_semantics_subtree_nodes` p50 = `9`
- Scaffold only (`all sections hidden + intro hidden`; `target/diag/mem-ui-gallery-card-scaffold-only-navnone-r3-20260306/`):
  - `MALLOC_SMALL` p50 ≈ **90.8 MiB** (**-3.3 MiB** vs `page shell only`)
  - `macos_malloc_zones_total_allocated_bytes` p50 ≈ **83.9 MiB** (**-0.2 MiB** vs `page shell only`)
  - `macos_malloc_zones_total_frag_bytes` p50 ≈ **11.3 MiB** (**-3.0 MiB** vs `page shell only`)
  - `ui_gallery_page_preview_semantics_subtree_nodes` p50 = `9` (**flat**)
- Branch-local `simple_content + nav none` rerun (`target/diag/mem-ui-gallery-simple-content-navnone-r3-20260306-branchlocal/`):
  - `MALLOC_SMALL` p50 ≈ **81.1 MiB**
  - `macos_malloc_zones_total_allocated_bytes` p50 ≈ **75.8 MiB**
  - `macos_malloc_zones_total_frag_bytes` p50 ≈ **9.4 MiB**

Interpretation:

- Starting from `preview only`, hiding **all** card sections drops `macos_malloc_zones_total_allocated_bytes` by about **77.2 MiB** (`161.3 -> 84.1 MiB`) while `ui_gallery_page_preview_semantics_subtree_nodes` falls from `197` to `9`. So once code tabs are gone, almost all remaining live preview-body bytes are indeed in the sections rather than hidden caches.
- Hiding the intro on top of that only changes live allocated bytes by about **0.2 MiB**; most of its effect lands in fragmentation / physical footprint instead. That means the intro text is *not* the main reason the page-shell floor stays above the generic floor.
- Even the stripped `scaffold only` card page still sits about **8.1 MiB** above the current branch-local `simple_content + nav none` floor in `macos_malloc_zones_total_allocated_bytes` (`83.9 -> 75.8 MiB`). The remaining residual now looks like **doc-page scaffold / content-shell integration cost**, not card sections, code tabs, fonts, or GPU caches.

#### Card doc scaffold counters (local 2026-03-06)

To avoid over-using more page bisects for obviously tiny payloads, the `ui-gallery` app snapshot now exports card-doc scaffold counters through `memory-summary`:

- `ui_gallery_card_doc_section_slots_total`
- `ui_gallery_card_doc_visible_sections_count`
- `ui_gallery_card_doc_visible_sections_with_code_count`
- `ui_gallery_card_doc_visible_sections_with_shell_count`
- `ui_gallery_card_doc_intro_len_bytes`
- `ui_gallery_card_doc_visible_static_text_bytes_estimate_total`
- `ui_gallery_card_doc_visible_code_bytes_estimate_total`
- `ui_gallery_card_doc_visible_code_lines_estimate_total`

Validation runs (N=1 is enough here because these counters are derived from static page structure, not noisy memory buckets):

- Full card (`target/diag/mem-ui-gallery-card-navnone-r1-20260306-doccounters-v2/`):
  - visible sections = `9` / `9`, sections with code = `8`, sections with shell = `1`
  - intro bytes = `121`
  - visible static text bytes ≈ **681 B**
  - visible code bytes ≈ **31.2 KiB** across `817` lines
- Page shell only (`target/diag/mem-ui-gallery-card-page-shell-only-navnone-r1-20260306-doccounters-v2/`):
  - visible sections = `0`, code bytes = `0`
  - intro bytes = `121`
  - visible static text bytes = `121 B`
- Scaffold only (`target/diag/mem-ui-gallery-card-scaffold-only-navnone-r1-20260306-doccounters-v2/`):
  - visible sections = `0`, intro bytes = `0`
  - visible static text bytes = `0 B`, code bytes = `0`

Interpretation:

- The entire card docs payload that is *obviously attributable to source/text* is tiny: even the full page only exposes about **31 KiB** of code excerpt text and less than **1 KiB** of static labels/descriptions.
- The stripped scaffold still holds ~**84 MiB** allocated with **zero** visible section text/code payload, so the remaining residual is not explained by doc strings, snippet source text, or intro copy.
- This narrows the next instrumentation target further: the missing attribution is almost certainly in retained widget/runtime/layout state around `render_doc_page` and shared content-shell scaffolding, not in source blobs or user-visible copy.

#### Card scaffold lazy-build check (local 2026-03-06)

A follow-up experiment changed `preview_card(...)` so hidden card sections are only constructed when their bisect flag is active, rather than eagerly rendering every snippet before masking sections out.

Using the current branch-local reruns (`target/diag/mem-ui-gallery-card-*-navnone-r3-20260306-lazybuild/`):

- `simple_content + nav none`:
  - `MALLOC_SMALL` p50 ≈ **78.5 MiB**
  - `macos_malloc_zones_total_allocated_bytes` p50 ≈ **75.2 MiB**
- `page shell only`:
  - `MALLOC_SMALL` p50 ≈ **106.2 MiB**
  - `macos_malloc_zones_total_allocated_bytes` p50 ≈ **87.2 MiB**
- `scaffold only`:
  - `MALLOC_SMALL` p50 ≈ **108.5 MiB**
  - `macos_malloc_zones_total_allocated_bytes` p50 ≈ **87.3 MiB**
- Full card:
  - `MALLOC_SMALL` p50 ≈ **221.9 MiB**
  - `macos_malloc_zones_total_allocated_bytes` p50 ≈ **163.2 MiB**
- `preview only`:
  - `MALLOC_SMALL` p50 ≈ **207.5 MiB**
  - `macos_malloc_zones_total_allocated_bytes` p50 ≈ **161.6 MiB**

Interpretation:

- Removing the eager construction of hidden snippets does **not** collapse the doc-shell floor. The current branch still leaves about **12 MiB** allocated above the `simple_content + nav none` floor (`87.3 -> 75.2 MiB`).
- The residual remains overwhelmingly a **CPU heap / `MALLOC_SMALL`** story rather than a GPU one; the Metal/IOSurface buckets stay almost flat across the page-shell/scaffold/simple comparison.
- This demotes the earlier "hidden snippets are still eagerly built" hypothesis from likely root cause to, at best, a secondary allocator-layout effect.

#### `heap` / `MallocStackLogging` attribution (local 2026-03-06)

To get callsite-level CPU heap hints, local one-off runs were also sampled with macOS `heap -s -H` under `MallocStackLogging=1`.
These runs are **not** suitable for absolute memory gates (stack logging perturbs the process), but they are useful for identifying which allocation families grow between `simple`, `scaffold`, `preview`, and `full`.

Key deltas from the saved local heap summaries (`/tmp/simple.heap.txt`, `/tmp/scaffold.heap.txt`, `/tmp/preview.heap.txt`, `/tmp/full.heap.txt`):

- `scaffold` vs `simple` is already dominated by framework-side retained structures:
  - `UiTree::create_node`: **+196 KiB**
  - `RawVecInner::finish_grow` families: **+240 KiB**, **+105 KiB**, **+84 KiB**, **+64 KiB**
  - `hashbrown::RawTableInner::fallible_with_capacity`: **+52 KiB**
  - `WindowElementState::take_scratch_element_children_vec`: **+26 KiB**
- `preview` vs `scaffold` scales those same families sharply:
  - `UiTree::create_node`: **+1.64 MiB**
  - `RawVecInner::finish_grow` families: roughly **+0.5–3.0 MiB** depending on callsite
  - `hashbrown::RawTableInner::fallible_with_capacity`: **+489 KiB** and **+836 KiB** on two hot callsites
  - `WindowElementState::take_scratch_element_children_vec`: **+440 KiB**
  - `TextSystem::prepare_with_key`: **+207 KiB**
- `full` vs `preview` adds only a small amount of directly attributable code/doc payload:
  - `DocSection::code_rust_from_file_region`: only **+32 KiB**
  - the larger extra growth still lands in `UiTree::create_node` (**+448 KiB**) and table/vector growth callsites.

Interpretation:

- The dominant live-heap growth is in **framework-owned nodes, vectors, and hash tables** (`UiTree::create_node`, `RawVec`, `hashbrown`) plus some `WindowElementState` scratch vector retention, not in raw snippet strings.
- Fonts do **not** look like the main floor driver in these samples: `TextMeasureCaches::new` is a roughly flat **~592 KiB** item, and `fontique::family::FamilyInfo::new` stays around **~166 KiB** across modes.
- Text preparation is real but secondary: `TextSystem::prepare_with_key` grows by only a few hundred KiB between `simple/scaffold` and `preview/full`, which is far smaller than the node/vector/table growth.
- This aligns with the earlier payload counters: the remaining `card` residual is much more likely to be **retained UI/runtime structure capacity** than fonts or visible code/text.

#### Retained-state counter rerun (`FRET_DIAG_DEBUG_SNAPSHOT=1`, local 2026-03-06)

One follow-up confusion was that the steady-memory scripts intentionally pin `FRET_DIAG_DEBUG_SNAPSHOT=0` in their `meta.env_defaults`, so `debug.element_runtime` and the new `ui_element_runtime_*` counters are **supposed** to be absent in the default raw-memory baselines.

The repo now also carries dedicated retained-analysis entry points for the `nav none` card-floor sweep:

- `tools/diag-scripts/suites/ui-gallery-card-retained-analysis-navnone/suite.json`
  - Includes `simple_content`, `scaffold_only`, `preview_only`, and full-card retained-analysis scripts.
- `tools/diag-scripts/ui-gallery/memory/ui-gallery-simple-content-memory-retained-analysis.json`
- `tools/diag-scripts/ui-gallery/memory/ui-gallery-card-scaffold-only-memory-retained-analysis.json`
- `tools/diag-scripts/ui-gallery/memory/ui-gallery-card-preview-only-memory-retained-analysis.json`
- `tools/diag-scripts/ui-gallery/memory/ui-gallery-card-memory-retained-analysis.json`

These scripts bake in the analysis-only overrides directly:

- `FRET_DIAG_DEBUG_SNAPSHOT=1`
- `FRET_DIAG_SEMANTICS_TEST_IDS_ONLY=0`
- `FRET_DIAG_BUNDLE_DUMP_SEMANTICS_TEST_IDS_ONLY=1`
- `FRET_UI_GALLERY_NAV_QUERY=__none__`

Important: use them as a **fresh-launch script list** (for example, one `fretboard diag repeat ... --launch` invocation per script). Do **not** treat `diag suite` as a cold-memory baseline runner here, because suite mode intentionally reuses the same app process across scripts.

Example:

- `target/debug/fretboard diag repeat tools/diag-scripts/ui-gallery/memory/ui-gallery-card-preview-only-memory-retained-analysis.json --repeat 1 --dir target/diag/mem-ui-gallery-card-preview-only-retained-analysis --no-compare --launch -- target/release/fret-ui-gallery`
- Then read the counters either from `bundle.schema2.json` or from `fretboard diag memory-summary --json <run_dir>` / `evidence.index.json -> resources.bundle_last_frame_stats`.
- The `capture_bundle` path now backfills `script.result.json.last_bundle_dir` / `last_bundle_artifact`, and `diag repeat` mirrors that into `repeat.summary.json.runs[*].last_bundle_dir`, so a single-script `diag repeat --repeat 1 --no-compare --launch` run leaves a stable bundle anchor for later automation.

Release analysis reruns (N=1 each) used:

- `simple_content` (`target/diag/mem-ui-gallery-simple-content-navnone-r1-20260306-analysis-release/`)
  - allocated ≈ **63.7 MiB**, `MALLOC_SMALL` ≈ **68.1 MiB**
  - `ui_element_runtime_state_entries_total=89`, `nodes_count=103`, `bounds_entries_total=229`
  - scratch children vec pool ≈ **3.82 MiB** (`2048` pooled vecs, capacity total `3254`)
- `scaffold_only` (`target/diag/mem-ui-gallery-card-scaffold-only-navnone-r1-20260306-analysis-release/`)
  - allocated ≈ **69.0 MiB**, `MALLOC_SMALL` ≈ **72.3 MiB**
  - `state_entries_total=129`, `nodes_count=159`, `bounds_entries_total=341`
  - scratch pool ≈ **4.17 MiB**
- `preview_only` (`target/diag/mem-ui-gallery-card-preview-only-navnone-r1-20260306-analysis-release/`)
  - allocated ≈ **88.0 MiB**, `MALLOC_SMALL` ≈ **103.4 MiB**
  - `state_entries_total=204`, `nodes_count=640`, `bounds_entries_total=1303`
  - scratch pool ≈ **3.58 MiB**
- full card (`target/diag/mem-ui-gallery-card-full-navnone-r1-20260306-analysis-release/`)
  - allocated ≈ **98.1 MiB**, `MALLOC_SMALL` ≈ **119.6 MiB**
  - `state_entries_total=438`, `nodes_count=768`, `bounds_entries_total=1559`
  - scratch pool ≈ **1.14 MiB**

Delta summary:

- `simple -> scaffold`
  - memory: **+5.3 MiB allocated**, **+4.2 MiB `MALLOC_SMALL`**
  - retained state: **+40** state entries, **+56** nodes, **+112** bounds entries
  - interpretation: the remaining doc-shell floor does show up as modest retained-structure growth, so shell/node/bounds fanout is still a valid optimization target.
- `scaffold -> preview`
  - memory: **+19.0 MiB allocated**, **+31.1 MiB `MALLOC_SMALL`**
  - retained state: **+75** state entries, **+481** nodes, **+962** bounds entries
  - interpretation: this is the strongest retained-state correlation in the suite; the preview body is clearly node/bounds heavy.
- `preview -> full`
  - memory: **+10.1 MiB allocated**, **+16.2 MiB `MALLOC_SMALL`**
  - retained state: **+234** state entries, **+128** nodes, **+256** bounds entries
  - scratch pool goes **down** by about **2.4 MiB** and `ui_frame_arena_capacity_estimate_bytes` stays flat (`15616`)
  - interpretation: the last full-page delta is **not** explained by the scratch children vec pool or frame arena; it still looks more like allocator/table/vector growth (consistent with the `heap -s -H` callsite evidence) than a single obvious retained-runtime bucket.

Other analysis-mode invariants stayed stable:

- `ui_view_cache_roots_total=0` / `ui_view_cache_roots_reused=0` in all four runs, so view-cache reuse is not a factor in this `card` floor.
- Text/font buckets remained near-flat (`render_text_atlas_bytes_live_estimate_total ≈ 4.0 MiB`, shape cache ≈ `2.1–2.5 MiB`, font blobs = `0`), which again argues against fonts being the main residual.

This narrows the next bounded optimization candidate further:

1. **Doc-shell retained structure reduction** (`simple -> scaffold`): reduce shell-only node/bounds/state fanout and any table/vector growth attached to empty section scaffolding.
2. **Preview-body node/bounds reduction** (`scaffold -> preview`): reduce the large retained-node step-up in the preview body before chasing smaller sources.
3. Treat the remaining `preview -> full` gap as an **allocator-growth / table-vector / text-prep follow-up**, not a scratch-pool-only problem.

First implementation pass (landed, rerun checked):

- `apps/fret-ui-gallery/src/ui/content.rs`: removed one redundant wrapper around the preset row, moved the scroll area's `flex_1` onto the scroll element itself, and dropped one extra fill container under `ui-gallery-content-shell`.
- `apps/fret-ui-gallery/src/ui/doc_layout.rs`: collapsed the extra centering container inside `render_doc_page` so doc pages keep the same centering/max-width contract with one fewer wrapper layer.
- Branch-local release rerun (`target/diag/mem-ui-gallery-card-retained-rerun-release-20260306/`) shows the shell cut in the retained counters even though absolute heap bytes moved the wrong way:
  - `simple` stayed flat at **103** nodes / **229** bounds, which is expected because the wrapper cut lives above the scaffold path rather than in the simple floor.
  - `scaffold`, `preview`, and `full` each dropped by **5 nodes** and **10 bounds** versus the saved pre-pass release baseline.
  - `simple -> scaffold` moved from **+56 / +112** nodes/bounds to **+51 / +102** (and `next_state_entries` from **+40** to **+38**), so the wrapper reduction is real but modest.
  - `scaffold -> preview` remains the dominant retained step at **+481 nodes** / **+962 bounds**, so the preview body is still the next best optimization target.
  - The same rerun showed much larger `macos_malloc_zones_total_allocated_bytes` / frag totals than the saved earlier baseline, so small shell wins are currently masked by branch-local allocator drift; for this class of cuts, prefer retained counters first and raw heap bytes second.
- Second shared preview-body pass (`apps/fret-ui-gallery/src/ui/doc_layout.rs`): `no_shell()` sections now skip the redundant layout-only container, and single-line descriptions render directly instead of through a dedicated `v_flex` wrapper.
- Follow-up rerun (`target/diag/mem-ui-gallery-card-retained-rerun-release-20260306-previewcut/`) confirms this second pass lands where expected:
  - `preview_only` dropped from **635 / 1293** to **608 / 1239** nodes/bounds (**-27 / -54**), with allocated bytes only slightly lower (~**-0.8 MiB**).
  - full card dropped from **763 / 1549** to **736 / 1495** nodes/bounds (**-27 / -54**), with allocated bytes only slightly lower (~**-0.4 MiB**).
  - `scaffold -> preview` shrank from **+481 / +962** to **+454 / +908**, but it is still the dominant retained step by a wide margin.
  - `preview -> full` stayed at **+128 / +256** nodes/bounds, which suggests this pass hit shared preview-body wrappers rather than the remaining full-page delta.
- To avoid guessing inside that remaining `preview` floor, the repo now also carries retained bisect entry points for preview-only section analysis:
  - `tools/diag-scripts/suites/ui-gallery-card-preview-retained-bisect-navnone/suite.json`
  - `tools/diag-scripts/suites/ui-gallery-card-preview-retained-hotspots-navnone/suite.json`
  - `tools/diag-scripts/ui-gallery/memory/ui-gallery-card-no-heavy-sections-preview-only-memory-retained-analysis.json`
  - `tools/diag-scripts/ui-gallery/memory/ui-gallery-card-no-light-sections-preview-only-memory-retained-analysis.json`
  - `tools/diag-scripts/ui-gallery/memory/ui-gallery-card-no-image-preview-only-memory-retained-analysis.json`
  - `tools/diag-scripts/ui-gallery/memory/ui-gallery-card-no-compositions-preview-only-memory-retained-analysis.json`
  - `tools/diag-scripts/ui-gallery/memory/ui-gallery-card-no-meeting-notes-preview-only-memory-retained-analysis.json`
- Local release retained bisect reruns (`target/diag/mem-ui-gallery-card-preview-retained-bisect-release-20260306/`) split the remaining preview floor into two different stories:
  - Light-half (`Demo + Usage + Size + CardContent + RTL + Notes`) is more allocator-heavy: over scaffold it adds about **+60 state**, **+194 nodes**, **+388 bounds**, but also about **+25 MiB allocated** / **+13.5 MiB frag**.
  - Heavy-half (`Image + Compositions + Meeting Notes`) is more retained-structure-heavy: over scaffold it adds only **+17 state**, but about **+261 nodes** / **+522 bounds**.
  - In other words, `light_half` looks more like text/layout/allocation pressure, while `heavy_half` is the clearer node/bounds fanout target.
- Hotspot reruns (`target/diag/mem-ui-gallery-card-preview-retained-hotspots-release-20260306/`) then isolate the heavy half further via `preview_only - no_section` deltas:
  - `Compositions`: about **+119 nodes** / **+238 bounds** (largest retained hotspot in the preview body).
  - `Meeting Notes`: about **+88 nodes** / **+176 bounds**.
  - `Image`: about **+53 nodes** / **+106 bounds**.
- This makes the next code target much clearer: if we want the biggest retained-structure win per edit, start in `apps/fret-ui-gallery/src/ui/snippets/card/compositions.rs`; if we want the next follow-up, look at `apps/fret-ui-gallery/src/ui/snippets/card/meeting_notes.rs`.
- First `Compositions` pass (local 2026-03-06): `apps/fret-ui-gallery/src/ui/snippets/card/compositions.rs` now collapses the two standalone border cards into one combined bordered example and removes a few redundant header descriptions from repeated combinations.
  - Rerun pair: `target/diag/mem-ui-gallery-card-compositions-cut-release-20260306/preview/` and `target/diag/mem-ui-gallery-card-compositions-cut-release-20260306/no_compositions/`.
  - Result: preview-only total moved from **608 / 1239** to **596 / 1215** nodes/bounds (**-12 / -24**).
  - Isolated `Compositions` contribution moved from **119 / 238** to **107 / 214** nodes/bounds.
  - Heap bytes remained noisy (`MALLOC_SMALL` / frag moved in the wrong direction across runs again), so the retained counters remain the trustworthy signal for this cut.


### External baseline: Fret `hello_world_compare_demo` vs GPUI `hello_world` (local 2026-03-06)

To get closer to an apples-to-apples comparison, added a minimal Fret compare target plus a generic
external sampler:

- Fret compare target:
  - `apps/fret-examples/src/hello_world_compare_demo.rs`
  - `apps/fret-demo/src/bin/hello_world_compare_demo.rs`
  - Shape: `500x500` window, centered `"Hello, World!"` label, one row of six colored swatches.
  - Runtime posture: `fret::App::minimal_defaults()`, config files off, accessibility off.
  - Isolation knobs:
    - `FRET_HELLO_WORLD_COMPARE_NO_TEXT=1`
    - `FRET_HELLO_WORLD_COMPARE_NO_SWATCHES=1`
- External sampler:
  - `tools/sample_external_process_memory.py`
  - Launches any macOS GUI process, waits a fixed warmup, captures `footprint -j` + `vmmap -summary`, then writes `summary.json`.
- GPUI reference:
  - Source: `repo-ref/zed/crates/gpui/examples/hello_world.rs`
  - macOS backend evidence: `repo-ref/zed/crates/gpui/Cargo.toml` uses `blade-graphics`/Metal, not `wgpu`.
  - This is therefore a **behavior-class** baseline against another mature GPU-first UI stack, not a same-backend comparison.

Artifacts captured locally:

- Fret compare: `target/diag/external-fret-hello-world-compare-sample-20260306/summary.json`
- Fret compare (warm 12s): `target/diag/external-fret-hello-world-compare-sample-20260306-warm12/summary.json`
- Fret compare, no text: `target/diag/external-fret-hello-world-compare-no-text-sample-20260306/summary.json`
- Fret compare, empty: `target/diag/external-fret-hello-world-compare-empty-sample-20260306/summary.json`
- GPUI hello world: `target/diag/external-gpui-hello-world-sample-20260306-r2/summary.json`
- GPUI hello world (warm 12s): `target/diag/external-gpui-hello-world-sample-20260306-r3-warm12/summary.json`
- Earlier GPUI sample with a higher transient peak: `target/diag/external-gpui-hello-world-sample-20260306/summary.json`
- Fret compare timeline (same process, `2s/6s/12s`): `target/diag/external-fret-hello-world-compare-timeline-20260306/summary.json`
- GPUI timeline (same process, `2s/6s/12s`): `target/diag/external-gpui-hello-world-timeline-20260306/summary.json`
- GPUI timeline rerun (same binary, same sampler shape, local 2026-03-06): `target/diag/external-gpui-hello-world-timeline-20260306-r2/summary.json`
- Fret compare internal GPU timeline: `target/diag/external-fret-hello-world-compare-timeline-20260306/internal.gpu.json`
- Fret compare `NO_TEXT=1` internal GPU timeline: `target/diag/external-fret-hello-world-compare-no-text-timeline-20260306/internal.gpu.json`
- Fret compare `NO_SWATCHES=1` internal GPU timeline: `target/diag/external-fret-hello-world-compare-no-swatches-timeline-20260306/internal.gpu.json`
- Fret compare empty internal GPU timeline: `target/diag/external-fret-hello-world-compare-empty-timeline-20260306/internal.gpu.json`

| Sample | Physical footprint | `owned unmapped` dirty | `IOSurface` dirty | `IOAccelerator` dirty | `MALLOC_SMALL` dirty |
|---|---:|---:|---:|---:|---:|
| Fret compare (6s) | ~309.0 MiB | ~221.8 MiB | ~23.7 MiB | ~34.6 MiB | ~24.3 MiB |
| Fret compare (12s) | ~318.1 MiB | ~221.8 MiB | ~23.7 MiB | ~35.3 MiB | ~26.4 MiB |
| Fret compare, empty (6s) | ~303.6 MiB | ~208.5 MiB | ~23.7 MiB | ~40.7 MiB | ~26.3 MiB |
| Fret compare, no text (6s) | ~299.8 MiB | ~208.5 MiB | ~19.7 MiB | ~37.7 MiB | ~25.4 MiB |
| GPUI hello world (6s) | ~18.5 MiB | ~0 MiB (region absent) | ~0.1 MiB | ~0.2 MiB | ~10.4 MiB |
| GPUI hello world (12s) | ~18.6 MiB | ~0 MiB (region absent) | ~0.1 MiB | ~0.2 MiB | ~10.5 MiB |

Preferred same-process timeline evidence (less run-to-run variance):

| Stack | 2s physical / `owned unmapped` | 6s physical / `owned unmapped` | 12s physical / `owned unmapped` |
|---|---:|---:|---:|
| Fret compare timeline | ~263.1 / ~219.1 MiB | ~266.2 / ~221.8 MiB | ~269.3 / ~221.8 MiB |
| GPUI timeline (rerun) | ~29.8 / ~4.1 MiB | ~29.8 / ~4.1 MiB | ~27.8 / ~2.2 MiB |

Internal paired GPU evidence (same-process Fret timelines):

- Compare demo instrumentation is now built into `apps/fret-examples/src/hello_world_compare_demo.rs` and writes a JSON timeline when these env vars are set:
  - `FRET_HELLO_WORLD_COMPARE_INTERNAL_REPORT_PATH=/path/to/internal.gpu.json`
  - `FRET_HELLO_WORLD_COMPARE_INTERNAL_SAMPLE_AT_SECS=2,6,12`
- The external sampler also now supports `--post-sample-wait-secs` so the app has time to flush its last internal sample before termination.

| Fret variant | Time | External graphics visible to macOS (`owned unmapped` + `IOSurface` + `IOAccelerator`) | Internal `metal_current_allocated_size_bytes` | Gap |
|---|---:|---:|---:|---:|
| Full | 6s | ~237.9 MiB | ~42.5 MiB | ~195.5 MiB |
| `NO_TEXT=1` | 6s | ~227.4 MiB | ~38.3 MiB | ~189.0 MiB |
| Empty | 6s | ~224.6 MiB | ~38.3 MiB | ~186.3 MiB |

Additional internal observations:

- Full compare timeline stays at **13 textures / 10 texture views / 52 render pipelines** with a stable internal Metal current allocation of about **~42.5 MiB**.
- `NO_TEXT=1` drops to **11 textures / 10 texture views / 52 render pipelines** and about **~38.3 MiB** internal Metal current allocation.
- `NO_SWATCHES=1` keeps the same **~42.5 MiB** internal Metal current allocation and still uses **13 textures**, only reducing render pipelines from **52** to **50**.
- On this stack, `device.generate_allocator_report()` remains unavailable (`present=false` in the internal report), but raw Metal `current_allocated_size()` is available and already enough to prove that **most of the large macOS graphics bucket is outside the app-visible allocator numbers we currently expose**.

Key observations:

- **The Fret graphics bucket does not fall back after warmup.** In the same-process timeline it rises from about **~219.1 MiB** at `2s` to **~221.8 MiB** at `6s/12s`, so this is not just a launch transient.
- **Text/font payload is not the primary headline.** Removing the title (`no_text`) and even removing both the title and swatches (`empty`) still leaves Fret at about **~208.5 MiB** current `owned unmapped memory` dirty.
- **The dominant gap is already present in a nearly empty rendered Fret window.** The visible payload can change by a few MiB, but the large current bucket remains: `Owned physical footprint (unmapped) (graphics)` plus another **~44–64 MiB** across `IOAccelerator` / `IOSurface`.
- **GPUI still stabilizes far lower in the same class of scene, but this is not a same-backend control.** The rerun on the same external sampler shape sits at about **~29.8 MiB** at `2s/6s` and **~27.8 MiB** at `12s`, with only **~4.1 MiB -> ~2.2 MiB** current `owned unmapped memory` dirty. One earlier separate-run sample still showed a **~224 MiB peak** with only **~29.6 MiB** current footprint, which reinforces that GPUI startup/transient cost and steady-state cost are very different; however, this remains a **behavior-class** baseline because Zed GPUI is using Blade/Metal here, not `wgpu`.
- **Conclusion (current confidence):** the current high-memory story is no longer “maybe fonts”. The first-order gap is a large **steady-state graphics baseline** in Fret's minimal rendered window on macOS/Metal, and the paired internal GPU report now shows that only about **~38–42 MiB** of it is visible as current Metal allocation from inside the app.

This comparison closes two important ambiguities: Fret is not merely “a bit higher” than another mature GPU-first UI stack on a similar hello-world-class scene, and the gap is **not** explained by the small app-visible Metal allocation we can currently sample from inside the process. GPUI remains useful as a behavior-class baseline, but it still leaves one question open: how much of the headline gap is Fret-specific versus just the current `wgpu`/surface baseline on macOS/Metal?

Historical pre-fix same-backend `wgpu` control follow-up (local 2026-03-06):

- Added `apps/fret-demo/src/bin/wgpu_hello_world_control.rs`, a smallest `wgpu` window/surface control that reuses `fret_render::WgpuContext` + `SurfaceState` so adapter selection and surface configuration stay close to Fret's current `wgpu` backend rather than introducing a different GPU stack.
- Local artifacts:
  - `target/diag/external-wgpu-hello-world-control-timeline-20260306-r1/summary.json`
  - `target/diag/external-wgpu-hello-world-control-timeline-20260306-r1/internal.gpu.json`
  - `target/diag/external-fret-hello-world-compare-timeline-20260306-r4-samebackend/summary.json`
  - `target/diag/external-fret-hello-world-compare-timeline-20260306-r4-samebackend/internal.gpu.json`
  - `target/diag/external-fret-hello-world-compare-empty-timeline-20260306-r1-samebackend/summary.json`
  - `target/diag/external-fret-hello-world-compare-empty-timeline-20260306-r1-samebackend/internal.gpu.json`
  - `target/diag/wgpu-hello-world-control-vs-fret-20260306-r1/summary.json`
  - `target/diag/wgpu-hello-world-control-vs-fret-20260306-r1/summary.md`

| Case | 6s physical | 6s graphics visible to macOS | 6s internal Metal current allocation | 6s residual gap | 6s redraws / presents |
|---|---:|---:|---:|---:|---:|
| `wgpu` control | ~31.0 MiB | ~12.8 MiB | ~9.5 MiB | ~3.3 MiB | `2 / 2` |
| Fret compare full | ~266.2 MiB | ~238.0 MiB | ~42.5 MiB | ~195.6 MiB | `2 / n/a` |
| Fret compare empty | ~249.0 MiB | ~221.9 MiB | ~38.3 MiB | ~183.6 MiB | `2 / n/a` |

Key same-backend observations:

- **The plain same-backend control is small and stable.** By `6s` it sits at about **~31.0 MiB** physical / **~12.8 MiB** graphics visible to macOS / **~9.5 MiB** app-visible Metal current allocation, and it only redraws/presents twice.
- **Fret's large floor survives even after collapsing the scene to the empty compare case.** With both text and swatches removed, Fret still sits near **~249.0 MiB** physical / **~221.9 MiB** graphics / **~38.3 MiB** app-visible Metal, leaving a residual of about **~183.6 MiB**.
- **Relative to the same-backend control, the empty Fret scene is still about +218.0 MiB physical / +209.1 MiB graphics / +28.8 MiB app-visible Metal.** The full scene is higher again, but the headline gap is already present before content gets interesting.
- **This closes the backend-floor ambiguity.** The headline macOS graphics gap is **not** just the raw `wgpu` surface baseline. The remaining unknown is how Apple is charging the extra **~183–196 MiB** residual across `Owned physical footprint (unmapped) (graphics)`, `IOSurface`, `IOAccelerator`, Metal driver/private heaps, or other VM reservation that the current app-side counters do not expose.

Historical pre-fix raw `vmmap` / Apple-trace follow-up (local 2026-03-06):

- `tools/sample_external_process_memory.py` now accepts `--capture-vmmap-regions` and stores `resource.vmmap_regions_sorted.txt` next to each sampled `summary.json`.
- Local detailed captures:
  - `target/diag/external-fret-hello-world-compare-empty-vmmap-detail-20260306-r1/resource.vmmap_regions_sorted.txt`
  - `target/diag/external-wgpu-hello-world-control-vmmap-detail-20260306-r1/resource.vmmap_regions_sorted.txt`
  - `target/diag/external-fret-hello-world-compare-empty-vmmap-verbose-20260306-r1/resource.vmmap_verbose.txt`
- Detailed `vmmap` adds one useful surface-level clue but still does **not** close the main gap:
  - Fret `empty` shows **three** `1000x1000` `CAMetalLayer Display Drawable` `IOSurface` allocations (~`3 x 4001K`), while the same-backend `wgpu` control shows **two**. That explains about **~4 MiB** of the `IOSurface` delta, but only a small fraction of the total gap.
  - Even `vmmap -v` still collapses the headline `owned unmapped memory` bucket into a single aggregate line (~**232.8 MiB** virtual / **208.5 MiB** dirty in the empty Fret case), so raw `vmmap` alone still does not tell us which Metal driver/private heaps or VM reservations make up the remaining **~183–196 MiB** residual.
- `tools/summarize_hello_world_compare_xctrace.py` now also supports `--list-store-schemas`, which inspects a `.trace` bundle directly instead of relying on `xctrace export`.
- First `Game Memory` bundle inspection (`target/diag/hello-world-compare-game-memory-20260306-r1/empty/store-schemas.key.json`) confirms that the Apple trace already contains the most relevant candidate stores for the next attribution pass:
  - `metal-current-allocated-size`
  - `metal-resource-allocations`
  - `metal-io-surface-access`
  - `virtual-memory`
  - `metal-residency-set-interval`
  - `metal-residency-set-usage-event`
- First quick export summaries from that same trace are already informative:
  - `target/diag/hello-world-compare-game-memory-20260306-r1/empty/metal-current-allocated-size.quick-summary.json` stays at exactly **`38.33 MiB`** (`40189952` bytes) across all exported rows, which matches the app-side Metal counter rather than revealing a hidden extra Apple-only allocation.
  - `target/diag/hello-world-compare-game-memory-20260306-r1/empty/metal-io-surface-access.quick-summary.json` filters the app's own accesses down to exactly **three** `1000x1000` `AGB&` surfaces, which matches the `vmmap` drawable story.
  - `target/diag/hello-world-compare-game-memory-20260306-r1/empty/metal-resource-allocations.quick-summary.json` is dominated by repeated **`(wgpu internal) Staging`** buffer allocation/deallocation events with `resource_size_max=131072`, which suggests the current **attach-based** capture is likely missing the earlier large startup allocations that created the steady-state floor.
- Attach summarizer follow-up (local 2026-03-06):
  - `tools/summarize_hello_world_compare_xctrace.py` now supports `--preset game-memory-attach`, `--process-contains <substring>`, `--export-dir`, and `--export-timeout-secs`, so the useful `Game Memory` tables can be reduced into a bounded, repeatable JSON report without manual Instruments clicking.
  - Primary artifact: `target/diag/hello-world-compare-game-memory-20260306-r1/empty/game-memory-attach.summary.json` with cached XML exports in `target/diag/hello-world-compare-game-memory-20260306-r1/empty/exports/`.
  - What the scripted attach summary adds beyond the first ad-hoc quick exports:
    - `metal-io-surface-access`: the process-filtered view still lands on exactly **1115** app-owned accesses over **three** `1000x1000` `AGB&` surfaces, so the drawable count story is now machine-readable and repeatable.
    - `metal-resource-allocations`: all **6687** filtered rows still belong to `hello_world_compare_demo`, are all `Buffer` events, and the top label remains **`(wgpu internal) Staging`** at `128 KiB`, so attach mode still does not reveal a large steady-state private heap.
    - `virtual-memory`: only **70** process-filtered rows / **1.09 MiB** cumulative `size` show up, mostly `Zero Fill` / `Page Cache Hit`, which is far too small to explain the remaining **~183 MiB** residual.
    - `metal-residency-set-interval` exports empty on this trace and `metal-residency-set-{usage-event,resource-event}` currently export as zero-row tables, so the attach trace is not surfacing an obvious residency-set bucket either.
    - `metal-current-allocated-size` now has a direct indexed-store fallback when `xctrace export` times out: the updated summary reads `indexed-store-41/bulkstore` directly, detects a `4096`-byte data offset, parses fixed `56`-byte records, and recovers the same **`38.33 MiB`** value across all **`6690`** rows (`target/diag/hello-world-compare-game-memory-20260306-r1/empty/game-memory-attach.summary.json`).
    - The fallback also confirms the schema is effectively single-target on this trace (`spec.plist` marks `target-pid=SINGLE`) and that the `topology` block decodes cleanly (`direct_store_end_mismatch_count=0`), so this slow schema is now scriptable without depending on `xctrace export`.
- Interpretation: the next closure step is no longer “guess what Fret content is expensive”; it is to use the new attach summary path for repeatable evidence across both Fret and the same-backend control, while separately fixing launch-from-start capture for startup allocations, because attach mode still does not expose the missing Apple-side bucket behind the residual.
- Same-backend control follow-up (local 2026-03-06):
  - `apps/fret-demo/src/bin/wgpu_hello_world_control.rs` now also accepts `FRET_WGPU_HELLO_WORLD_CONTROL_EXIT_AFTER_SECS`, validated by `target/diag/test-wgpu-hello-world-control-auto-exit-20260306-r2.json` (`still_running_after_6s=false`).
  - New generic helper: `tools/capture_binary_xctrace.py` reproduces the compare-helper attach lifecycle for arbitrary binaries (launch target, attach after a delay, optionally leave the target running, bound xctrace finalization, and persist both target logs plus xctrace stdout/stderr).
  - The helper shows that `Game Memory` behavior is **flaky/template-specific**, not a simple “this binary cannot be attached” rule:
    - `Time Profiler` works through the helper on the same control binary: `target/diag/wgpu-hello-world-control-xctrace-helper-timeprofiler-20260306-r1/summary.json` recorded cleanly and produced a normal trace bundle with `corespace`.
    - `Game Memory` can still collapse into a partial `Trace1.run`-only bundle (`target/diag/wgpu-hello-world-control-xctrace-helper-20260306-r1/summary.json`, `target/diag/wgpu-hello-world-control-xctrace-helper-20260306-r2-leave-running/summary.json`, `target/diag/wgpu-hello-world-control-xctrace-helper-20260306-r3-self-exit-12/summary.json`), and changing whether the target is force-stopped / left running / self-exits after `12s` does **not** rescue finalization in those runs.
    - But `Game Memory` is **not** permanently broken either: `target/diag/wgpu-hello-world-control-xctrace-helper-20260306-r4/summary.json` produced a full bundle with `corespace`, even though xctrace still reported attach issues in `baseline.xctrace.stderr.log`.
    - The same instability now also reproduces on the Fret compare binary under the generic helper: `target/diag/hello-world-compare-xctrace-helper-game-memory-20260306-r1/summary.json` produced a full bundle, while `target/diag/hello-world-compare-xctrace-helper-game-memory-20260306-r2/summary.json` later timed out into another partial `Trace1.run`-only bundle.
  - New summarizer capability: when `xctrace export` hangs, `tools/summarize_hello_world_compare_xctrace.py` now falls back to direct indexed-store **metadata** so we still recover per-store row counts / record sizes / store sizes rather than only `timed_out`.
  - First full-bundle same-backend Apple-side control summary: `target/diag/wgpu-hello-world-control-xctrace-helper-20260306-r4/game-memory-attach.summary.json`
    - `metal-current-allocated-size`: header-only / **`0` rows** (`bulkstore=4096 B`) in this trace, unlike Fret's attach trace where the same schema stays at **`38.33 MiB`** over **`6690`** rows.
    - `metal-resource-allocations`: also header-only / **`0` rows** in this trace.
    - `virtual-memory`: direct-store metadata reports **`46`** rows (`bulkstore≈4.00 MiB`).
    - `metal-io-surface-access`: direct-store metadata reports **`6466`** rows (`bulkstore≈6.75 MiB`), but `xctrace export` still times out even at `45s`, so we do not yet have the per-surface breakdown to compare directly against Fret's `3 x 1000x1000` app-owned drawables.
  - Interpretation: we now have the first apples-to-apples Apple-side hint — this minimal control trace does **not** emit the same app-local `metal-current-allocated-size` / `metal-resource-allocations` stores that Fret emits — but `Game Memory` remains too unstable to treat a single run as definitive, so the next closure step is to stabilize capture or add more direct per-row parsers for the timed-out stores.

- Launch-mode follow-up (local 2026-03-06):
  - `tools/capture_hello_world_compare_xctrace.py` now supports `--record-mode launch` plus `--target-exit-after-secs`, and it preserves `recorded-with-issues` runs instead of dropping the trace entirely when `xctrace` finalization times out.
  - `apps/fret-examples/src/hello_world_compare_demo.rs` now also accepts `FRET_HELLO_WORLD_COMPARE_EXIT_AFTER_SECS`, which was sanity-checked in `target/diag/test-hello-world-compare-auto-exit-20260306-r1.json` (`returncode=0`, `elapsed_secs≈2.07`).
  - Helper validation artifacts:
    - `target/diag/test-hello-world-game-memory-launch-helper-20260306-r3/summary.json`
    - `target/diag/test-hello-world-game-memory-launch-helper-20260306-r4/summary.json`
  - What worked:
    - launch mode can now discover the spawned target pid and keep a structured run record even when `Game Memory` does not finalize cleanly.
    - the compare demo auto-exit knob works outside xctrace, so the launch-path blocker is not “the demo cannot stop itself”.
  - What still failed:
    - even with `FRET_HELLO_WORLD_COMPARE_EXIT_AFTER_SECS=6`, the local `Game Memory` launch capture still times out during finalization (`90s`) and leaves only a tiny ~`56 KiB` trace with no discoverable store schemas (`target/diag/test-hello-world-game-memory-launch-helper-20260306-r4/empty/store-schemas.json`).
    - `xctrace export --toc` reports `Document Missing Template Error` on those partial launch traces, so they are not yet useful for the startup-allocation attribution pass.
  - Updated interpretation:
    - the helper/runtime work is still worthwhile because it converts a previously manual failure into a reproducible artifact, but on this machine the current blocker has shifted from “how do we start from launch?” to **“why does `Game Memory` launch-mode finalization collapse into an empty partial bundle?”**.
    - Until that Apple-tooling issue is understood, the attach-based full trace remains the only currently useful source for store discovery / per-table export on this branch.

Direct-store row parser follow-up (local 2026-03-06):

- `tools/summarize_hello_world_compare_xctrace.py` now also has fixed-width direct-store row parsers for `metal-io-surface-access` and `virtual-memory`, plus `--pid-equals <pid>` so heavy `Game Memory` stores remain analyzable even when `xctrace export` times out.
- Validation against the known-good Fret attach trace succeeds:
  - Direct `metal-io-surface-access` with `--pid-equals 93488` reproduces the same app slice already seen via XML export: **`1115`** rows, **`3`** surfaces, and a single **`1000x1000`** extent (`target/diag/apple-direct-store-same-backend-20260306-r1/fret-metal-io-surface-access-pid-93488.json`).
  - Direct `virtual-memory` also reproduces the exported totals exactly for the same trace: **`70`** rows, **`1146880`** cumulative `size_bytes_sum`, and **`16384`** `size_bytes_max` (`target/diag/apple-direct-store-same-backend-20260306-r1/fret-virtual-memory.json`).
- Helper follow-up: `tools/capture_binary_xctrace.py` now records bounded `ps` snapshots and summary-level pid lineage (`spawned_target_pid`, `xctrace_attached_pid`, per-stage process snapshots), so attach ownership can be checked without manually grepping logs.
- Revised control interpretation after pid audit:
  - Idle control attach evidence (`target/diag/wgpu-control-pid-audit-20260306-r1/summary.json` + `analysis/summary.json`) shows `spawned_target_pid == xctrace_attached_pid == 4290`, with no descendants before/after attach, yet `metal-io-surface-access` still emits no rows for pid `4290`.
  - That no longer looks like a process-tree bug. The more likely explanation is attach timing: `wgpu_hello_world_control` only renders once by default, so by the time `Game Memory` attaches the app itself is mostly idle and the remaining `IOSurface` accesses belong to WindowServer / other GPU helper processes.
- To test that hypothesis, `apps/fret-demo/src/bin/wgpu_hello_world_control.rs` now also supports `FRET_WGPU_HELLO_WORLD_CONTROL_CONTINUOUS_REDRAW=1` (or `FRET_WGPU_HELLO_WORLD_CONTROL_CONTINUOUS_REDRAW_INTERVAL_MS=<ms>`).
  - Runtime validation: `target/diag/test-wgpu-control-continuous-redraw-runtime-20260306-r2.json` shows the control demo now keeps presenting after startup (`present_count=83` by `1.5s`).
- First usable continuous-control same-backend comparison artifact remains `target/diag/apple-direct-store-same-backend-20260306-r2/summary.md` / `summary.json`.
- New startup-inclusive same-backend control note: `target/diag/apple-direct-store-same-backend-20260306-r3/summary.md` / `summary.json`.
  - Control now uses **`pre-init sleep + continuous redraw`**, so startup allocations are visible on the control side: app-owned `metal-io-surface-access` is **`922`** rows / **`2`** `1000x1000` surfaces, `metal-current-allocated-size` is **`9437184`** bytes last (~`9.00 MiB`) / **`10092544`** bytes max (~`9.62 MiB`), and `metal-resource-allocations` is no longer empty (**`29`** rows, `resource_size_bytes_sum=12435456`).
  - Best-available Fret side (at that point still steady attach, empty compare case) remained **`1115`** app-owned `metal-io-surface-access` rows / **`3`** `1000x1000` surfaces and **`40189952`** bytes (~`38.33 MiB`) `metal-current-allocated-size`.
- New startup-inclusive same-backend note on **both** sides: `target/diag/apple-direct-store-same-backend-20260306-r4/summary.md` / `summary.json`.
  - `hello_world_compare_demo` now also supports `FRET_HELLO_WORLD_COMPARE_PRE_INIT_SLEEP_SECS`, and the helper supports `--pre-init-sleep-secs`, so the compare demo can be attached before GPU initialization too.
  - Startup-inclusive Fret empty capture (`target/diag/hello-world-compare-game-memory-startup-attach-20260306-r1/empty/game-memory-startup-attach.summary.json`) shows `metal-current-allocated-size` starting at **`31997952`** bytes (~`30.52 MiB`) and topping out at the same **`40189952`** bytes (~`38.33 MiB`) steady value already seen in steady attach.
  - The same startup-inclusive Fret trace also surfaces non-zero `metal-resource-allocations` (**`8100`** rows, `resource_size_bytes_sum=1069645824`, `resource_size_bytes_max=4112384`) and app-owned `metal-io-surface-access` of **`1351`** rows / **`3`** `1000x1000` surfaces.
  - Even with startup included on both sides, the same-backend steady delta is still only about **one extra app-owned drawable** plus **~`29.33 MiB`** of `metal-current-allocated-size` vs the startup-inclusive control. The extra startup coverage increases allocation-event visibility, but it still does **not** reveal any app-visible Metal current-allocation spike anywhere near the headline **~`183–196 MiB`** residual graphics gap.
- Updated interpretation:
  - The earlier control pid mismatch was primarily an **attach-after-idle artifact**, not strong evidence that Apple was inherently attributing the wrong process.
  - Same-backend Apple-side comparison is now good enough to say the app-visible Metal/drawable delta is real, but it still lives in the **tens of MiB**, not the **hundreds of MiB**.
  - The startup-inclusive Fret trace strengthens that conclusion further: startup inclusion reveals more allocation churn, but it still does not change the app-visible current-allocation ceiling beyond the familiar ~`38.33 MiB` plateau.
  - The next closure step is therefore no longer “make Fret startup-inclusive”, but **“continue decomposing the much larger Apple/macOS residual floor (`owned unmapped memory (graphics)` / driver reservation / private heaps / `IOAccelerator`) while optionally removing the remaining cadence asymmetry between Fret and control.”**.

Historical pre-fix surface / size sweep follow-up (local 2026-03-06):

- Compare demo now also accepts `FRET_HELLO_WORLD_COMPARE_WINDOW_WIDTH` / `FRET_HELLO_WORLD_COMPARE_WINDOW_HEIGHT`, and the internal JSON report records the requested scene/surface/renderer knobs used for the run.
- New helper: `tools/run_hello_world_compare_paired_sweep.py` drives the external sampler plus internal compare-demo report together and writes a per-case `summary.json` / `summary.md`.
- Full scene sweep: `target/diag/external-fret-hello-world-compare-surface-sweep-20260306/summary.json`
- Empty scene sweep: `target/diag/external-fret-hello-world-compare-empty-surface-sweep-20260306/summary.json`
- GPUI rerun: `target/diag/external-gpui-hello-world-timeline-20260306-r2/summary.json`

| Case | Scene | 6s physical | 6s graphics visible to macOS | 6s internal Metal current allocation | 6s residual gap |
|---|---|---:|---:|---:|---:|
| baseline | full | ~269.1 MiB | ~240.7 MiB | ~42.5 MiB | ~198.2 MiB |
| `latency=1` | full | ~262.0 MiB | ~234.0 MiB | ~38.5 MiB | ~195.5 MiB |
| `msaa=1` | full | ~264.8 MiB | ~237.9 MiB | ~22.3 MiB | ~215.6 MiB |
| `256x256` | full | ~254.7 MiB | ~226.5 MiB | ~18.6 MiB | ~207.9 MiB |
| `1000x1000` | full | ~307.4 MiB | ~278.9 MiB | ~131.8 MiB | ~147.1 MiB |
| baseline | empty | ~249.3 MiB | ~221.9 MiB | ~38.3 MiB | ~183.6 MiB |
| `latency=1` | empty | ~245.0 MiB | ~217.9 MiB | ~34.4 MiB | ~183.5 MiB |
| `msaa=1` | empty | ~253.4 MiB | ~227.4 MiB | ~18.2 MiB | ~209.2 MiB |
| `256x256` | empty | ~243.0 MiB | ~215.8 MiB | ~14.5 MiB | ~201.3 MiB |
| `1000x1000` | empty | ~290.0 MiB | ~262.8 MiB | ~127.7 MiB | ~135.2 MiB |

What changed, and what did not:

- **Frame latency mainly changes the visible presentation surfaces, not the large residual floor.** Moving from default latency to `desired_maximum_frame_latency=1` trims about **~4 MiB** of `IOSurface` / app-visible Metal current allocation at `500x500`, but the residual remains essentially unchanged (**~195.5 MiB** full, **~183.5 MiB** empty).
- **Path MSAA is visible inside the app, but it does not explain the headline macOS graphics bucket.** Dropping to `FRET_RENDER_WGPU_PATH_MSAA_SAMPLES=1` roughly halves internal Metal current allocation at `500x500` (**~42.5 -> ~22.3 MiB** full, **~38.3 -> ~18.2 MiB** empty), yet the residual gap persists and even grows.
- **Window size strongly scales the app-visible part, but a large content-light floor remains.** At `256x256`, the residual is still about **~207.9 MiB** full / **~201.3 MiB** empty. At `1000x1000`, app-visible Metal allocation jumps to about **~131.8 MiB** full / **~127.7 MiB** empty and `IOSurface` jumps to about **~45.2 MiB**, yet there is still a residual of about **~147.1 MiB** full / **~135.2 MiB** empty.
- **Upstream `wgpu` presentation context matches only part of the story.** `desired_maximum_frame_latency` is a documented surface knob, and the ongoing presentation API work (`gfx-rs/wgpu#2711`, `gfx-rs/wgpu#2869`) suggests drawable / swapchain policy can move some residency. Our local sweep agrees that it moves a small visible slice, but not the large residual floor. Typical leak-style issues such as `gfx-rs/wgpu#5394` describe continuing growth, which does not match our stable same-process timeline.
- **The current best explanation is now a two-part graphics story (inference).** There appears to be (1) a large macOS/Metal graphics floor outside the app-visible allocator counters we currently expose, plus (2) a size/MSAA-sensitive app-visible component that behaves like swapchain / drawable / intermediate render-target allocations. Surface frame latency alone does not explain the floor.

Fastest Apple-tooling path (local helper):

- New helper: `tools/capture_hello_world_compare_xctrace.py`
- Default behavior records the three most useful compare cases (`baseline`, `empty`, `size1000`) with **`Metal System Trace`** and writes `summary.json` / `summary.md` plus per-case `.trace` bundles.
- Recommended command:
  - `python3 tools/capture_hello_world_compare_xctrace.py --out-dir target/diag/hello-world-compare-xctrace-$(date +%Y%m%d-%H%M%S)`
- Useful overrides:
  - `--case empty` to focus on the smallest content-light repro.
  - `--template game-memory --finalize-timeout-secs 180` when we want Apple-side category attribution too.
  - `--time-limit 10s` / `--attach-delay-secs 2` for longer captures.
  - `--pre-init-sleep-secs 2` to inject `FRET_HELLO_WORLD_COMPARE_PRE_INIT_SLEEP_SECS=2` and give `Game Memory` a startup-inclusive attach window before the compare demo initializes GPU state.
- Local smoke success (validated): `target/diag/test-hello-world-xctrace-smoke-20260306-r4/summary.json`
- First full three-case capture (validated): `target/diag/hello-world-compare-xctrace-20260306-r1/summary.json`
- Practical note: on this machine, `Game Memory` finalization can take much longer than `Metal System Trace`; keep it opt-in instead of the default “one click” path so the helper stays predictable.

Interpretation update after app-side cadence sampling (local 2026-03-06):

- New compare-demo internal runtime sample: `target/diag/hello-world-compare-runtime-sample-20260306-r1.json`.
- New runner-side present samples:
  - steady/normal run: `target/diag/test-hello-world-compare-runner-present-runtime-normal-20260306-r1.json`
  - startup-inclusive run: `target/diag/test-hello-world-compare-runner-present-runtime-20260306-r1.json`
- In a normal run **without** any `FRET_DIAG*` environment, the view-level counters still stay at `render_count=2`, `last_frame_id=1`, and `last_render_since_launch_ms≈209`, so the minimal compare scene is **not** continuously re-running view render work in steady idle.
- But the new runner-side present counter shows the same run is still continuously presenting: `runner_present.total_present_count` reaches about **`98` / `216` / `335`** at `1s` / `2s` / `3s`, with `last_present_frame_id` tracking the same growth. In other words, the discrepancy is now **not** “Apple says frames, app says no frames”; it is “view rebuilds stay idle, but the runner still presents frames continuously”.
- The startup-inclusive sample closes the startup side too: before GPU init there are still **`0`** presents at the `1s` sample, then after startup the counter climbs to **`79`** by `3s`, which matches the intended `pre-init sleep` behavior.
- New xctrace summarizer: `tools/summarize_hello_world_compare_xctrace.py`. Local summaries: `target/diag/hello-world-compare-xctrace-20260306-r1/baseline/baseline.summary.json`, `target/diag/hello-world-compare-xctrace-20260306-r1/empty/empty.summary.json`, `target/diag/hello-world-compare-xctrace-20260306-r1/size1000/size1000.summary.json`, and `target/diag/gpui-hello-world-xctrace-20260306-r1/baseline/baseline.summary.json`.
- Important correction: `display-surface-queue` in the exported `Metal System Trace` data has **no process column**, so it should no longer be treated as an app-only frame-cadence proxy.
- The baseline trace still shows about **~118.9 Hz** `ca-client-present-request` rows and about **~237.9 Hz** `metal-application-encoders-list` rows for `hello_world_compare_demo`. With the new runner-side counter in place, the best current interpretation is that `ca-client-present-request` is much closer to a **real present-cadence signal** than to a view-rebuild signal: it aligns with runner presents far better than with app `render_count`.
- Diagnostics follow-up landed in-tree: `bundle.schema2.json` / repeat evidence / `diag memory-summary` now expose `ui_element_runtime_continuous_frame_lease_*` and `ui_element_runtime_animation_frame_request_roots_count`, and the runner now also exposes a present counter via `fret_runtime::RunnerPresentDiagnosticsStore`. Together they make it easier to separate “continuous presents” from “continuous declarative rerenders”. A minimal validation capture is available under `target/diag/test-hello-world-compare-repeat-20260306-r2/`.
- Apple's WWDC20 “Debug Metal app memory issues” session (`https://developer.apple.com/videos/play/wwdc2020/10632/`) is a useful interpretation aid here: it explicitly calls out `IOSurface` as the drawable/back-buffer side and `IOAccelerator` as the Metal-resource side, which makes `Game Memory` + `VM Tracker` the next easiest attribution path for the remaining residual rather than over-reading raw `Metal System Trace` cadence tables.

Closure update after runner/global-change attribution and command-palette gating fix (local 2026-03-06):

- New diagnostic artifacts now close the steady-idle cadence question end-to-end:
  - runner frame-drive: `target/diag/test-hello-world-compare-runner-frame-drive-runtime-normal-20260306-r4.json`
  - redraw request callsites: `target/diag/test-hello-world-compare-redraw-callsites-runtime-20260306-r1.json`
  - global changes (pre-fix): `target/diag/test-hello-world-compare-global-changes-runtime-20260306-r1.json`
  - post-fix verification: `target/diag/test-hello-world-compare-global-changes-runtime-20260306-r2.json`
- The runner-side evidence rules out `Effect::RequestAnimationFrame` as the steady-idle driver. The pre-fix loop was dominated by `Effect::Redraw`, and the dominant redraw-request callsite was `ecosystem/fret-bootstrap/src/ui_app_driver.rs:1707` inside `ui_app_handle_global_changes(...)`.
- The dominant pre-fix changed global was `fret_bootstrap::ui_app_driver::CommandPaletteService` (about `327 / 330` batches in the 3s sample), so the redraw loop was app-owned rather than an unexplained backend-only cadence.
- Root cause: `CommandPaletteService.gating_handle` bookkeeping used tracked global mutation even when steady-idle cleanup repeatedly observed `None`. That kept the service in `changed_globals`, `ui_app_handle_global_changes(...)` kept calling `app.request_redraw(window)`, and the runner kept presenting.
- Fix: command-palette gating bookkeeping now uses `with_global_mut_untracked(...)` for `set_gating_handle(...)` / `take_gating_handle(...)`, and `ui_app_driver::command_palette_cleanup_does_not_mark_service_changed_each_frame` locks the behavior with a focused regression test.
- Post-fix steady-idle verification now flattens immediately: the default compare-demo sample stays at `runner_present.total_present_count=5` across `1s/2s/3s`, `runner_frame_drive.reason_counts` stop at `about_to_wait_raf=2`, `effect_redraw=7`, `surface_bootstrap=1`, and `CommandPaletteService` disappears from the dominant-global list in favor of startup-only font/runtime globals.
- The same external steady-memory story changes completely on current head:
  - empty compare scene: `target/diag/hello-world-compare-post-fix-empty-20260306-r1/summary.json` now sits at about `48.9 MiB` physical / `22.1 MiB` macOS-visible graphics / `38.3 MiB` internal Metal current allocation at `6s`
  - full compare scene: `target/diag/hello-world-compare-post-fix-full-20260306-r1/summary.json` now sits at about `52.2 MiB` physical / `24.5 MiB` macOS-visible graphics / `42.5 MiB` internal Metal current allocation at `6s`
  - same-backend control: `target/diag/wgpu-hello-world-control-post-fix-20260306-r1/summary.json` sits at about `31.3 MiB` physical / `12.8 MiB` graphics / `9.5 MiB` internal Metal at `6s`
  - consolidated comparison: `target/diag/wgpu-hello-vs-fret-post-fix-20260306-r1/summary.json` / `summary.md`
- This materially changes the diagnosis: the earlier `~183–196 MiB` steady-state residual on the compare demo was not a durable framework floor on current head. It was largely an app-owned continuous-present bug. The remaining same-backend steady delta is now about `+17.6 MiB` (`empty`) to `+20.9 MiB` (`full`) physical and about `+9.2 MiB` (`empty`) to `+11.7 MiB` (`full`) in macOS-visible graphics versus the plain `wgpu` control.
- The minimal-scene content delta is now small (`full` vs `empty` is only about `+3.3 MiB` physical at `6s`), so fonts/content are currently a second-order factor for this hello-world-class scene rather than the first-order explanation.
- One caveat remains: both Fret and the same-backend control still show large startup `physical_footprint_peak` values (`~266–276 MiB` for Fret, `~236 MiB` for the control) that collapse before the first steady sample. Startup peak attribution is therefore now a separate closure item from steady-state floor attribution.


Active continuous-present follow-up (local 2026-03-07):

- The first deliberate-active compare run (`target/diag/wgpu-control-vs-fret-active-redraw-debug-20260307-r5/summary/summary.json`) proved that the compare demo can enter a real active-present mode on purpose, but it also exposed a runner-owned pacing issue: desktop `Effect::RequestAnimationFrame` both requested an immediate redraw in `effects.rs` and requested another redraw again in `about_to_wait`, which let light scenes self-spin into thousands of presents (`~5176` presents in `6s` for the empty compare scene).
- That pacing issue is now fixed on current head: desktop `Effect::RequestAnimationFrame` only records RAF intent into `raf_windows`, and `about_to_wait` remains the single pacing point for the next redraw. The post-fix validation artifact is `target/diag/wgpu-control-vs-fret-active-redraw-debug-20260307-r6/summary/summary.json`.
- The final active-mode split baseline now lives in `docs/workstreams/ui-memory-footprint-closure-v1/2026-03-07-active-mode-split-baseline.md` and is backed by:
  - `present-only`: `target/diag/wgpu-control-vs-fret-active-redraw-debug-20260307-r13-present-final/summary/summary.json`
  - `paint-model`: `target/diag/wgpu-control-vs-fret-active-redraw-debug-20260307-r14-paint-final/summary/summary.json`
  - `layout-model`: `target/diag/wgpu-control-vs-fret-active-redraw-debug-20260307-r15-layout-final/summary/summary.json`
- That split now answers the old caveat directly:
  - `present-only` really is a pure present loop (`render_count≈2`, `present_count≈526–570`), so it isolates continuous-present residency.
  - `paint-model` and `layout-model` now really rerender each frame (`render_count≈present_count≈560`), so they measure true per-frame declarative work rather than a static scene being presented repeatedly.
- The active plateau still exists even in `present-only`, which means most of the active hello-world gap versus the same-backend `wgpu` control is already explained before real per-frame content mutation is added:
  - `present-only` at `6s`: control `149.8 MiB` physical / `130.9 MiB` graphics / `13.4 MiB` internal Metal; Fret empty `255.6 MiB` / `227.4 MiB` / `38.3 MiB`; Fret full `263.4 MiB` / `235.1 MiB` / `42.5 MiB`.
- Real per-frame rerender/layout on the full scene adds only a single-digit increment above that plateau on current head:
  - `paint-model` vs `present-only` (full): about `+5.0 MiB` physical / `+2.9 MiB` graphics
  - `layout-model` vs `present-only` (full): about `+7.2 MiB` physical / `+5.0 MiB` graphics
- The compare-demo internal report now records effective surface config + renderer perf for active runs. Current hello-world steady samples consistently show `present_mode=Fifo`, `desired_maximum_frame_latency=2`, `configure_count=1`, and zero renderer-owned image / render-target / intermediate bytes. That makes it much less likely that the large active delta is coming from imported images or obvious Fret-owned offscreen buffers in this minimal scene.


## Open Questions

- What explains the remaining same-backend steady delta between `hello_world_compare_demo` and `wgpu_hello_world_control` on current head (about `+17.6 MiB` to `+20.9 MiB` physical and `+9.2 MiB` to `+11.7 MiB` macOS-visible graphics at `6s`)?
- How much of that remaining delta is:
  - extra drawables / surface configuration?
  - Fret-owned intermediate targets or MSAA surfaces?
  - persistent framework caches (text, images, theme/runtime services) that do not exist in the minimal control?
- Why do both Fret and the same-backend control still reach much higher startup `physical_footprint_peak` values (`~236–276 MiB`) before collapsing to their low steady-state numbers?
- Which cache(s) grow unbounded in real editor sessions, and how do we gate that without flakiness?

## Next experiments (recommended)

- **Use the GPUI same-scene matrix as the framework-level cross-check, not as a future TODO:** the compare path is now captured in both `docs/workstreams/ui-memory-footprint-closure-v1/2026-03-07-fret-vs-gpui-same-scene-active-matrix.md` (`debug/debug`) and `docs/workstreams/ui-memory-footprint-closure-v1/2026-03-07-fret-vs-gpui-same-scene-active-matrix-release.md` (`release/release`). The optimized rerun keeps the idle gap modest (about `+13–17 MiB` physical / `+5–8 MiB` graphics), but all cadence-aligned active rows still leave Fret about `~+94–118 MiB` physical / `~+86–107 MiB` graphics above GPUI on the same scene.

- **Treat the release/release GPUI matrix as the point where cadence chase ends:** the debug-only `paint-model full` caveat is gone (`GPUI≈583` renders, `Fret≈578`), so the next high-value step is no longer another cadence rerun. It is Apple-side attribution on the strongest rows (`rerender-only`, `paint-model`, `layout-model`) so the remaining graphics bucket can be split into swapchain / driver / OS-reservation contributions.

- **Separate startup peak from steady-state floor:** the pre-fix “huge steady floor” question is now closed enough to split the work. The next high-value step is a startup-focused capture path (`Game Memory`, `VM Tracker`, or Instruments allocations) that explains the transient `physical_footprint_peak` seen in both Fret and the same-backend control.

- **Prefer post-fix Apple category attribution over more cadence chase:** steady idle no longer needs a runner-path investigation. The fastest remaining closure path is `Game Memory` / `VM Tracker` on the post-fix head so `IOSurface`, `IOAccelerator`, driver/private heaps, and any remaining `owned unmapped` accounting can be separated with Apple's own categories.

- **Use instrument-only `xctrace` captures as the practical startup fallback:** `tools/capture_binary_xctrace.py` now supports `--record-mode launch`, repeatable `--instrument`, `--dry-run`, and trace-completeness markers. Local 2026-03-07 smoke runs show that `Virtual Memory Trace + Metal Application` produces a full launch trace for `wgpu_hello_world_control` (`target/diag/test-capture-binary-xctrace-launch-vm-plus-metal-20260307-r1/summary.json`) and exposes both `virtual-memory` and `metal-io-surface-access` in one capture (`target/diag/test-capture-binary-xctrace-launch-vm-plus-metal-20260307-r1/analysis/summary.json`). This is now a more dependable scripted path than `Game Memory` launch mode on this machine.

- **Treat launch-mode Metal attribution carefully on Fret itself:** the same `Virtual Memory Trace + Metal Application` launch path already records a full Fret empty trace (`target/diag/test-capture-binary-xctrace-launch-fret-empty-vm-plus-metal-20260307-r1/summary.json`), but its first `metal-io-surface-access` export still attributes rows primarily to WindowServer / GPU helpers rather than directly to `hello_world_compare_demo`. That means the next step is a same-backend paired run with stronger process/pid filtering, not immediate over-interpretation.

- **Idle-present gate is now closed for both `diag run` and `diag suite`:** `diag run` for `tools/diag-scripts/tooling/hello-world/hello-world-compare-idle-present-gate.json` and `diag suite hello-world-compare-idle-present` now both launch the compare demo through an external / no-diagnostics path, inject `FRET_HELLO_WORLD_COMPARE_INTERNAL_REPORT_PATH`, wait for the demo to self-exit, and then run `check.hello_world_compare_idle_present.json` without ever enabling the in-band diagnostics frame loop. The closure artifacts at `target/fret-diag-hello-world-compare-idle-gate-r2/` and `target/fret-diag-hello-world-compare-idle-suite-r3/` both show `diag_env_enabled_guess=false`, flat `runner_present.total_present_count` across the `2s/3s/4s` samples, `present_delta=0`, and passing tool-owned summaries (`script.result.json` / `suite.summary.json`).

- **Re-run the same-backend sweep only where it can move the residual:** now that the residual is tens of MiB rather than hundreds, focus on knobs like drawable count / surface latency / compositing / transparency and correlate them with render-target/intermediate byte estimates.

- **Use GPUI as a secondary behavior baseline, not the primary closure tool:** GPUI still matters for ecosystem context, but on macOS it remains Blade/Metal rather than `wgpu`, so post-fix closure work should stay anchored on the same-backend control first.

- **Correlate the compare sweep with renderer-side targets/intermediates:** the remaining post-fix delta is small enough that target/intermediate byte estimates should now be able to explain a meaningful fraction of it, rather than drowning under a hundreds-of-MiB residual.

- **Compositions first (still):** the first snippet pass already reduced `apps/fret-ui-gallery/src/ui/snippets/card/compositions.rs` from about **119 / 238** to **107 / 214** nodes/bounds, but it remains the single largest retained hotspot in the preview body.

- **Meeting Notes second:** `apps/fret-ui-gallery/src/ui/snippets/card/meeting_notes.rs` is still the next largest retained hotspot (about **88 nodes / 176 bounds**). After the next `Compositions` pass, this is the next best place to trim nested rows/wrappers.

- **Preview-body shared wrappers:** the second doc-layout pass removed another **27 nodes / 54 bounds** from both `preview_only` and full card, yet `scaffold -> preview` still adds about **454 nodes / 908 bounds**. The next optimization pass should stay inside per-section preview / tabs scaffolding before revisiting deeper allocator-only work.

- **Allocator-aware reporting:** the latest reruns moved `macos_malloc_zones_total_allocated_bytes` / `macos_malloc_zones_total_frag_bytes` much more than the retained counters, so `memory-summary` / docs should surface live alloc + frag deltas next to node/bounds deltas for small wrapper cuts.

- **Doc scaffold runtime state:** visible source/text payload is only about **31 KiB + <1 KiB**, and the follow-up lazy-build experiment did not remove the floor. The remaining scaffold work should still target retained node/runtime structure capacity (`UiTree::create_node`, `RawVec` growth, `hashbrown` tables, scratch children vec pooling) around `render_doc_page` / shared content-shell scaffolding, not strings.

- **Shared-shell floor sweep:** once the content side is reduced to the `simple_content + nav none` floor (~90.9 MiB `MALLOC_SMALL`), split the remaining frame chrome / status bar / overlay / command surfaces deterministically.

- **Allocator A/B (empty-idle + UI gallery baseline):** system vs `mimalloc` vs `jemalloc`.
  Early samples suggest allocator choice mostly impacts `MALLOC_SMALL` / heap dirty bytes, while the
  dominant macOS `owned unmapped memory` (graphics) bucket remains the headline in all cases.

- **Count sweep (image-heavy):** run `tools/diag-scripts/suites/tooling-image-heavy-memory-sweep-count/suite.json` and
  fit a simple slope:
  - `Owned physical footprint (unmapped) (graphics)` vs `renderer_gpu_images_bytes_estimate`
  - `wgpu_metal_current_allocated_size_bytes_max` vs `renderer_gpu_images_bytes_estimate`
  - This helps separate a “baseline intercept” (swapchain/driver/allocator) from per-image growth, and tells us where
    further instrumentation is worthwhile.
 - **Size sweep (image-heavy):** run `tools/diag-scripts/suites/tooling-image-heavy-memory-sweep-size/suite.json` and
  validate whether `wgpu_metal_current_allocated_size_bytes_max` and the macOS (graphics) unmapped footprint scale
  ~1:1 with `renderer_gpu_images_bytes_estimate` across texture sizes (detect tiling/alignment multipliers).

### Count sweep (results; local 2026-03-05)

Using `target/diag/mem-sweep-count-20260305/` (N=3 each; 1024×1024 RGBA8 images):

- p50 table:
  - count=6: images=24.0 MiB, `wgpu_metal_current_allocated_size_bytes_max`=122.9 MiB, `Owned physical footprint (unmapped) (graphics)`=240.9 MiB
  - count=12: images=48.0 MiB, `wgpu_metal_current_allocated_size_bytes_max`=147.0 MiB, `Owned physical footprint (unmapped) (graphics)`=267.2 MiB
  - count=24: images=96.0 MiB, `wgpu_metal_current_allocated_size_bytes_max`=195.4 MiB, `Owned physical footprint (unmapped) (graphics)`=315.6 MiB
  - count=48: images=192.0 MiB, `wgpu_metal_current_allocated_size_bytes_max`=292.2 MiB, `Owned physical footprint (unmapped) (graphics)`=412.3 MiB
- Linear fit (least squares; y = intercept + slope * images_bytes):
  - `Owned physical footprint (unmapped) (graphics)`: intercept ≈ 217.5 MiB, slope ≈ 1.02 bytes/byte
  - `wgpu_metal_current_allocated_size_bytes_max`: intercept ≈ 98.7 MiB, slope ≈ 1.01 bytes/byte

Interpretation:

- The headline bucket is dominated by **live texture pressure + a baseline intercept**, and scales ~1:1 with the estimated image bytes.
- This strongly suggests our “high memory” in image-heavy scenarios is not primarily font/text heap; it is expected GPU resource pressure plus a platform/driver baseline.

### Size sweep (results; local 2026-03-05)

Using `target/diag/mem-sweep-size-20260305/` (N=3 each; count=24; 512/1024/2048 RGBA8 images):

- p50 table:
  - size=512: images=24.0 MiB, `wgpu_metal_current_allocated_size_bytes_max`=123.0 MiB, `Owned physical footprint (unmapped) (graphics)`=241.1 MiB
  - size=1024: images=96.0 MiB, `wgpu_metal_current_allocated_size_bytes_max`=195.4 MiB, `Owned physical footprint (unmapped) (graphics)`=313.5 MiB
  - size=2048: images=384.0 MiB, `wgpu_metal_current_allocated_size_bytes_max`=485.7 MiB, `Owned physical footprint (unmapped) (graphics)`=605.8 MiB
- Linear fit (least squares; y = intercept + slope * images_bytes):
  - `Owned physical footprint (unmapped) (graphics)`: intercept ≈ 216.5 MiB, slope ≈ 1.01 bytes/byte
  - `wgpu_metal_current_allocated_size_bytes_max`: intercept ≈ 98.8 MiB, slope ≈ 1.01 bytes/byte

Interpretation:

- The ~1:1 bytes/byte scaling holds across texture sizes (no large tiling/alignment multiplier visible in this sweep).
- The intercepts closely match the count sweep, strengthening the “baseline + linear image pressure” model.
