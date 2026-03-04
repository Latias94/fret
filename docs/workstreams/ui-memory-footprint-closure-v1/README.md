# UI Memory Footprint Closure (v1)

## Problem

Fret-based apps (even simple demos) can appear to have a high memory footprint on macOS. We need a
repeatable, diagnosable evidence chain that answers:

1) **CPU**: where does the resident/dirty memory come from and why does it not return to the OS?
2) **GPU**: how much memory is actually allocated on the GPU/device side, and which subsystems
   contribute?
3) Which optimizations are worth landing (high impact, low risk), with gates that prevent regressions.

This workstream is about **measurement → attribution → bounded optimizations**, not one-off profiling.

## Snapshot (2026-03-04)

Using `tools/diag-scripts/todo-memory-steady.json` on macOS/Metal:

- `vmmap -summary`:
  - `physical_footprint_peak_bytes`: ~347 MiB
  - `owned unmapped memory` dirty: ~224.6 MiB (stable across runs)
- `debug.stats.wgpu_metal_current_allocated_size_bytes`:
  - Before lazy color/subpixel text atlases: ~123 MiB
  - After lazy color/subpixel text atlases: ~58 MiB

Using `tools/diag-scripts/empty-idle-memory-steady.json` on macOS/Metal (baseline):

- Without UI diagnostics enabled (manual `vmmap -summary` on a plain run):
  - Physical footprint (peak): ~241 MiB
  - `owned unmapped memory` dirty: ~204 MiB
  - Default malloc zone: ~13.6 MiB allocated, ~4.0 MiB frag
- With `fretboard diag repro` (UI diagnostics enabled, plus tool-side `vmmap` capture):
  - Repeat sample (N=5):
    - `macos_vmmap.physical_footprint_peak_bytes`: 279,445,504 .. 285,946,675 (~266.6 .. 272.7 MiB)
    - `macos_owned_unmapped_memory_dirty_bytes`: 213,594,931 .. 216,321,229 (~203.7 .. 206.3 MiB)
    - `render_text_atlas_bytes_live_estimate_total`: `0` (after lazy mask atlas page allocation)
  - Default malloc zone: ~24.5 MB allocated, ~15.4 MB frag
  - `debug.stats.wgpu_metal_current_allocated_size_bytes`: 32,161,792 (~30.7 MiB; requires `--env FRET_DIAG_WGPU_ALLOCATOR_REPORT=1`)

Using `tools/diag-scripts/text-heavy-memory-steady.json` on macOS/Metal (fonts + emoji stress):

- Repeat sample (N=5):
  - `macos_vmmap.physical_footprint_peak_bytes`: 358,927,565 .. 368,364,749 (~342.4 .. 351.4 MiB)
  - `macos_owned_unmapped_memory_dirty_bytes`: 249,036,800 .. 254,699,110 (~237.5 .. 242.9 MiB)
  - `render_text_atlas_bytes_live_estimate_total`: ~20 MiB (after lazy mask atlas page allocation)
- Default malloc zone: ~26.6 MB allocated, ~20.9 MB frag (system allocator)
- `wgpu_metal_current_allocated_size_bytes`: 127,418,368 (~121.6 MiB; requires `--env FRET_DIAG_WGPU_ALLOCATOR_REPORT=1`)

Using `tools/diag-scripts/image-heavy-memory-steady.json` on macOS/Metal (texture upload stress):

- Repeat sample (N=5, defaults: `FRET_IMAGE_HEAVY_DEMO_COUNT=24`, `FRET_IMAGE_HEAVY_DEMO_SIZE_PX=1024`):
  - `macos_vmmap.physical_footprint_peak_bytes`: 483,917,824 .. 501,324,186 (~461.6 .. 478.2 MiB)
  - `macos_vmmap.regions.owned_unmapped_memory_dirty_bytes`: 331,874,304 .. 337,222,042 (~316.5 .. 321.6 MiB)
  - `macos_vmmap.regions.io_surface_dirty_bytes`: 34,393,293 (~32.8 MiB; stable)
  - `macos_vmmap.regions.io_accelerator_dirty_bytes`: 5,980,160 .. 7,372,800 (~5.7 .. 7.0 MiB)
  - `macos_vmmap.regions.malloc_small_dirty_bytes`: 41,104,179 .. 44,774,195 (~39.2 .. 42.7 MiB)
  - `wgpu_metal_current_allocated_size_bytes`: 204,914,688 (~195.4 MiB; stable; requires `--env FRET_DIAG_WGPU_ALLOCATOR_REPORT=1`)

Using `tools/diag-scripts/ui-gallery/memory/ui-gallery-code-editor-torture-memory-steady.json` on macOS/Metal (UI Gallery, editor-grade stress):

- Repeat sample (N=5; captured via `fretboard diag repro --launch`):
  - `macos_vmmap.physical_footprint_bytes`: 386,924,544 .. 389,545,984 (~369.1 .. 371.6 MiB)
  - `macos_vmmap.physical_footprint_peak_bytes`: 387,658,547 .. 390,279,987 (~369.8 .. 372.2 MiB)
  - `macos_vmmap.regions.owned_unmapped_memory_dirty_bytes`: 236,349,030 .. 236,978,176 (~225.4 .. 226.0 MiB)
  - `macos_vmmap.regions.malloc_small_dirty_bytes`: 80,628,941 .. 83,276,595 (~76.9 .. 79.4 MiB)
  - `macos_vmmap.regions.io_surface_dirty_bytes`: 37,748,736 (36.0 MiB; stable)
  - `macos_vmmap.regions.io_accelerator_dirty_bytes`: 5,324,800 (5.1 MiB; stable)
  - `wgpu_metal_current_allocated_size_bytes`: 118,308,864 (~112.8 MiB; stable)
- App-side attribution (`app_snapshot.code_editor.torture.cache_sizes`, last snapshot):
  - `row_text_cache_entries`: 429
  - `row_text_cache_text_bytes_estimate_total`: ~29–30 KiB
  - `row_rich_cache_entries`: 429
  - `row_rich_cache_line_bytes_estimate_total`: ~29–30 KiB

Interpretation:

- This workload's headline "high memory" is **not explained by GPU allocation** (stable ~113 MiB) nor by
  the measured code editor paint caches (tens of KiB). The dominant CPU-side contributors remain:
  - `owned unmapped memory` dirty (allocator retention / sticky reservations), and
  - `MALLOC_SMALL` dirty (heap allocations + fragmentation).
- Diagnostics stability note:
  - Full debug snapshot capture can make bundle dumps prohibitively expensive in editor torture scenarios.
    This script therefore sets env defaults (via `meta.env_defaults`) to record **stats-only** debug
    snapshots: `FRET_DIAG_DEBUG_SNAPSHOT=0`.

Allocator A/B (empty idle, `--release`, `fretboard diag repro`, same script):

- System allocator:
  - `macos_vmmap.physical_footprint_peak_bytes`: 284,164,096
  - `owned unmapped memory` dirty: 213,594,931
  - Default malloc zone: 23,907,533 allocated, 15,623,782 frag (~40%)
  - `wgpu_metal_current_allocated_size_bytes`: 32,161,792
- `mimalloc`:
  - `macos_vmmap.physical_footprint_peak_bytes`: 285,212,672 (Δ +1,048,576 vs system)
  - `owned unmapped memory` dirty: 213,594,931 (Δ 0 vs system)
  - Default malloc zone: 7,843,840 allocated, 5,574,656 frag (~42%)
  - `wgpu_metal_current_allocated_size_bytes`: 32,161,792
- `jemalloc`:
  - `macos_vmmap.physical_footprint_peak_bytes`: 280,494,080 (Δ -3,670,016 vs system)
  - `owned unmapped memory` dirty: 216,321,229 (Δ +2,726,298 vs system)
  - Default malloc zone: 7,814,144 allocated, 4,572,160 frag (~37%)
  - `wgpu_metal_current_allocated_size_bytes`: 32,161,792

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
    - `empty-idle`: `render_text_atlas_bytes_live_estimate_total` drops to `0` (no text draws).
    - `text-heavy`: `render_text_atlas_bytes_live_estimate_total` drops to `20 MiB` (mask pages `1`
      instead of `2`).
  - vmmap region attribution note:
    - `resource.footprint.json.macos_vmmap.regions` now also includes:
      - `io_surface_dirty_bytes` (Metal-backed surfaces/textures)
      - `io_accelerator_dirty_bytes` (GPU driver allocations)
      - `malloc_small_dirty_bytes` (CPU heap bucket)
    - These are intended to support more actionable macOS gates than “just physical footprint”.
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

### 1) Improve attribution (tool-side)

- Parse `resource.vmmap_summary.txt` into a structured summary:
  - Top region types by resident/dirty.
  - `MALLOC ZONE` allocated + fragmentation (where present).
- Persist these structured fields into `resource.footprint.json` so comparisons do not require manual
  text parsing.

### 2) Improve attribution (app-side)

- Add app-side stats for major caches (bytes + counts) where feasible:
  - Text shaping caches / blob caches (heap bytes, not just entry counts).
  - Image cache bytes and “live texture” estimates (already partially present).
  - Code editor: buffer/undo/syntax memory estimates (rope chunks, undo history, parse tree / spans),
    so `MALLOC_SMALL` vs `owned unmapped` can be explained with app-level counters.
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
