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

Using `tools/diag-scripts/ui-gallery/memory/ui-gallery-code-editor-torture-memory-steady.json` on macOS/Metal (UI Gallery, editor-grade stress):

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

Interpretation:

- This workload's headline "high memory" is **not explained by GPU allocation** (stable ~113 MiB) nor by
  the measured code editor paint caches (tens of KiB). The dominant CPU-side contributors remain:
  - `owned unmapped memory` dirty (allocator retention / sticky reservations), and
  - `MALLOC_SMALL` dirty (heap allocations + fragmentation).
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
- `evidence.index.json.resources.bundle_last_frame_stats` (app-side last-frame stats)

Candidate gates:

- `--max-macos-physical-footprint-peak-bytes`
- `--max-macos-owned-unmapped-memory-dirty-bytes`
- `--max-macos-io-surface-dirty-bytes`
- `--max-macos-io-accelerator-dirty-bytes`
- `--max-macos-malloc-small-dirty-bytes`
- `--max-renderer-gpu-images-bytes-estimate`
- `--max-renderer-gpu-render-targets-bytes-estimate`
- `--max-renderer-intermediate-peak-in-use-bytes`
- `--max-wgpu-metal-current-allocated-size-bytes` (macOS/Metal; best-effort)
- `--max-render-text-atlas-bytes-live-estimate-total` (text-heavy attribution; stable, derived from `resource_caches.render_text`)
- `--max-render-text-registered-font-blobs-total-bytes` (guards memory-backed font injection growth; `resource_caches.render_text`)
- `--max-render-text-registered-font-blobs-count` (guards memory-backed font injection churn; `resource_caches.render_text`)
- `--max-render-text-shape-cache-entries` (guards unbounded text shaping cache growth; `resource_caches.render_text`)
- `--max-render-text-blob-cache-entries` (guards unbounded text blob cache growth; `resource_caches.render_text`)
- `--max-code-editor-buffer-len-bytes` (UI Gallery; `app_snapshot.code_editor.torture.memory`)
- `--max-code-editor-undo-text-bytes-estimate-total` (UI Gallery; `app_snapshot.code_editor.torture.memory`)
- `--max-code-editor-row-text-cache-entries` (UI Gallery; `app_snapshot.code_editor.torture.cache_sizes`)
- `--max-code-editor-row-rich-cache-entries` (UI Gallery; `app_snapshot.code_editor.torture.cache_sizes`)

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
- `ui-gallery-code-editor-torture-memory-steady`:
  - `--max-macos-physical-footprint-peak-bytes 419430400` (400 MiB)
  - `--max-macos-owned-unmapped-memory-dirty-bytes 268435456` (256 MiB)
  - `--max-macos-malloc-small-dirty-bytes 104857600` (100 MiB)
  - `--max-macos-io-surface-dirty-bytes 67108864` (64 MiB)
  - `--max-macos-io-accelerator-dirty-bytes 16777216` (16 MiB)
  - `--max-wgpu-metal-current-allocated-size-bytes 150994944` (144 MiB)

Note: these numbers are intentionally conservative and should be revisited when:

- the script payload changes (fonts/emoji coverage),
- the renderer backend changes (wgpu/wgpu-core bumps),
- or the measurement surface changes (new diagnostics fields enabled by default).

## Open Questions

- What does `owned unmapped memory` represent for our typical runs, and how sensitive is it to:
  - allocator choice?
  - thread count / worker pools?
  - Metal / wgpu internal allocators?
- Which cache(s) grow unbounded in real editor sessions, and how do we gate that without flakiness?
