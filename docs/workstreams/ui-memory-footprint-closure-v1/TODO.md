# UI Memory Footprint Closure (v1) — TODO

## Diagnostics (tool-side)

- [x] Parse `resource.vmmap_summary.txt` region table into structured JSON (top N by resident/dirty).
- [x] Parse `MALLOC ZONE` allocated + frag into structured JSON when present.
- [x] Capture a bounded `vmmap -sortBySize -wide -interleaved -noCoalesce` region list to break down large buckets like `owned unmapped memory`.
- [x] Add `vmmap` parsing fields to `resource.footprint.json` schema (best-effort; macOS-only).
- [x] Add a `fretboard diag compare --footprint` view that prints deltas for the structured fields.
- [x] Add `fretboard diag memory-summary` to summarize distributions across multiple `--session-auto` samples.
- [x] Capture Apple `/usr/bin/footprint --json` output in bundles (macOS-only) and surface a summary under `macos_footprint_tool_steady`.
- [x] Add `fretboard diag memory-summary --footprint-categories-agg` to aggregate `footprint` category dirty bytes across samples.
- [x] Surface renderer attribution fields in `memory-summary` (`renderer_gpu_images_bytes_estimate`, `renderer_gpu_render_targets_bytes_estimate`, `renderer_intermediate_peak_in_use_bytes`).

## Diagnostics (app-side)

- [ ] Add heap byte estimates for text caches (blob cache, shape cache, measure caches).
- [ ] Add cache byte estimates for images/assets where feasible (distinguish CPU decoded bytes vs GPU textures).
- [ ] Keep all new fields behind a “diagnostics” surface (non-contract; best-effort).

## Scripted repro matrix

- [x] Add `tools/diag-scripts/tooling/empty/empty-idle-memory-steady.json` (schema v2).
- [x] Add `tools/diag-scripts/tooling/text/text-heavy-memory-steady.json` (forces emoji/color glyphs).
- [x] Add `tools/diag-scripts/tooling/images/image-heavy-memory-steady.json` (forces texture cache).
- [x] Add `tools/diag-scripts/tooling/images/image-heavy-memory-steady-after-drop.json` (drops registered images + idle).

## Attribution experiments (macOS / Metal)

- [x] Sweep `FRET_DIAG_WGPU_ALLOCATOR_REPORT_EVERY_N_FRAMES` and `FRET_DIAG_WGPU_REPORT_EVERY_N_FRAMES`
  (60 vs 600) and measure:
  - outlier frequency for `wgpu_metal_current_allocated_size_bytes_{min,max}`
  - stability of `macos_vmmap_steady.regions.io_surface_dirty_bytes` / `io_accelerator_dirty_bytes`
  - overhead (bundle size + tooling time)
- [x] Confirmed: cadence 60 produces outliers; cadence 600 is stable (see workstream README snapshot).
- [x] Default memory scripts to cadence 600 and keep a separate “high-frequency attribution” script for deep dives:
  - Baseline: `tools/diag-scripts/tooling/todo/todo-memory-steady.json` (cadence 600)
  - Deep dive: `tools/diag-scripts/tooling/todo/todo-memory-steady-wgpu-highfreq.json` (cadence 60)
  - Baseline scripts also include `empty-idle`, `text-heavy`, `image-heavy` (cadence 600)

- [ ] Correlate the `vmmap` headline bucket (`owned unmapped memory` dirty) with `footprint` categories:
  - Determine which `footprint` categories rise/fall with `owned unmapped memory` across scenarios.
  - If one category dominates, add a dedicated gate for it (monitor-only at first).

- [x] Attribution: sweep `FRET_RENDER_WGPU_SURFACE_DESIRED_MAX_FRAME_LATENCY` (1/2/3) on `empty-idle` and record the impact.
- [x] Attribution: sweep `FRET_WGPU_MEMORY_HINTS` (`performance` vs `memory`) on `text-heavy` and record the impact.
- [x] Attribution: release images + idle (image-heavy) and confirm `Owned physical footprint (unmapped) (graphics)` returns close to baseline after `renderer.unregister_image`.
- [x] Attribution: A/B `FRET_IMAGE_HEAVY_DEMO_POLL_AFTER_DROP` (1 vs 0; idle 1200 frames) and confirm no material delta in post-drop steady state.

### Evidence (captured)

- `empty-idle-memory-steady` (macOS native)
  - Script: `tools/diag-scripts/tooling/empty/empty-idle-memory-steady.json`
  - Demo: `target/debug/empty_idle_demo` (from `apps/fret-demo`)
  - Sample run output (local): `target/fret-diag-mem-empty-idle-steady/`
  - GPU sampling (optional):
    - Run with `--env FRET_DIAG_WGPU_ALLOCATOR_REPORT=1`
    - Sample run output (local): `target/fret-diag-mem-empty-idle-steady-wgpu/`
  - Observed (sample):
    - `resources.process_footprint.macos_physical_footprint_bytes`: ~288 MB
    - `resources.process_footprint.macos_vmmap_top_dirty_region_type`: `owned unmapped memory` (~216 MB dirty)
    - `macos_vmmap.tables.malloc_zones.top_allocated[0]`: Default malloc zone ~24.5 MB allocated, ~15.4 MB frag
    - `resources.bundle_last_frame_stats.wgpu_metal_current_allocated_size_bytes`: ~30.7 MiB (with GPU sampling enabled)

## Optimization candidates

- [x] Run allocator A/B locally (mimalloc/jemalloc) and record impact on:
  - `resources.process_footprint.macos_owned_unmapped_memory_dirty_bytes`
  - `macos_vmmap.tables.malloc_zones.top_allocated[0]` (`allocated_bytes`, `frag_bytes`, `frag_percent`)
- Observed (empty idle, sample):
  - System vs `mimalloc`: default malloc zone allocated drops ~23.9 MB → ~7.8 MB; `owned unmapped memory` dirty unchanged (~213.6 MB).
  - System vs `jemalloc`: default malloc zone allocated drops ~23.9 MB → ~7.8 MB; `owned unmapped memory` dirty remains the headline (~216.3 MB).
- [ ] Decide whether to keep allocator selection as a dev-only feature (A/B), and whether to surface it in `fretboard dev` presets.
- [ ] Identify top heap offenders via structured `vmmap` summary and pick one bounded optimization.
- [x] Reduce baseline text atlas allocations by lazily allocating the mask atlas pages (avoid preallocating `TEXT_ATLAS_MAX_PAGES`).

## Gates

- [x] Calibrate a macOS footprint gate for `empty-idle` and `text-heavy` (repeat samples captured under `target/fret-diag-mem-*-sample5/`).
- [x] Calibrate a Metal allocated size gate for `empty-idle` and `text-heavy` (requires `--env FRET_DIAG_WGPU_ALLOCATOR_REPORT=1`).
- [x] Add a wgpu hub counts gate (`check.wgpu_hub_counts.json`; requires `--env FRET_DIAG_WGPU_REPORT=1`).
- [x] Add a text-atlas-focused gate (`--max-render-text-atlas-bytes-live-estimate-total`) for more stable attribution vs total Metal bytes.
- [x] Calibrate a post-drop release gate for `image-heavy-memory-steady-after-drop` (avoid peak-based gates; prefer `owned_unmapped_memory` and `wgpu_metal_current_allocated_size_bytes` thresholds).
- [ ] Document acceptable drift policy (e.g. +X MiB allowed with justification).
