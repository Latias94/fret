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
  - `macos_vmmap.physical_footprint_peak_bytes`: ~288 MB (tool JSON; `vmmap` prints ~275 MiB)
  - `owned unmapped memory` dirty: ~216 MB
  - Default malloc zone: ~24.5 MB allocated, ~15.4 MB frag
  - `debug.stats.wgpu_metal_current_allocated_size_bytes`: ~30.7 MiB (requires `--env FRET_DIAG_WGPU_ALLOCATOR_REPORT=1`)

Interpretation:

- GPU memory can be substantial but may not be reflected by `physical footprint` in a stable way.
- The largest CPU-side “mystery” is `owned unmapped memory` dirty, which likely reflects allocator
  behavior, caching, or sticky runtime allocations.

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
- Keep all fields “best effort” and clearly labeled (estimate vs exact).

### 3) Build a minimal baseline matrix

Add scripted repros that isolate hypotheses:

- **Empty idle**: minimal window, no text, no images (baseline CPU + GPU).
- **Text heavy**: many font faces, emoji, and diverse glyphs (forces atlas growth).
- **Image heavy**: representative image decode + texture upload path (forces texture cache).

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
- (new) `--max-wgpu-metal-current-allocated-size-bytes` (if stable enough)

## Open Questions

- What does `owned unmapped memory` represent for our typical runs, and how sensitive is it to:
  - allocator choice?
  - thread count / worker pools?
  - Metal / wgpu internal allocators?
- Which cache(s) grow unbounded in real editor sessions, and how do we gate that without flakiness?
