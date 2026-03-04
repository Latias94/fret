# UI Memory Footprint Closure (v1) — TODO

## Diagnostics (tool-side)

- [x] Parse `resource.vmmap_summary.txt` region table into structured JSON (top N by resident/dirty).
- [x] Parse `MALLOC ZONE` allocated + frag into structured JSON when present.
- [x] Add `vmmap` parsing fields to `resource.footprint.json` schema (best-effort; macOS-only).
- [x] Add a `fretboard diag compare --footprint` view that prints deltas for the structured fields.

## Diagnostics (app-side)

- [ ] Add heap byte estimates for text caches (blob cache, shape cache, measure caches).
- [ ] Add cache byte estimates for images/assets where feasible (distinguish CPU decoded bytes vs GPU textures).
- [ ] Keep all new fields behind a “diagnostics” surface (non-contract; best-effort).

## Scripted repro matrix

- [x] Add `tools/diag-scripts/tooling/empty/empty-idle-memory-steady.json` (schema v2).
- [x] Add `tools/diag-scripts/tooling/text/text-heavy-memory-steady.json` (forces emoji/color glyphs).
- [ ] Add `tools/diag-scripts/tooling/images/image-heavy-memory-steady.json` (forces texture cache).

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

## Gates

- [x] Calibrate a macOS footprint gate for `empty-idle` and `text-heavy` (repeat samples captured under `target/fret-diag-mem-*-sample5/`).
- [x] Calibrate a Metal allocated size gate for `empty-idle` and `text-heavy` (requires `--env FRET_DIAG_WGPU_ALLOCATOR_REPORT=1`).
- [x] Add a text-atlas-focused gate (`--max-render-text-atlas-bytes-live-estimate-total`) for more stable attribution vs total Metal bytes.
- [ ] Document acceptable drift policy (e.g. +X MiB allowed with justification).
