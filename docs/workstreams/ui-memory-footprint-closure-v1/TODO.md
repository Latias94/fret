# UI Memory Footprint Closure (v1) — TODO

## Diagnostics (tool-side)

- [ ] Parse `resource.vmmap_summary.txt` region table into structured JSON (top N by resident/dirty).
- [ ] Parse `MALLOC ZONE` allocated + frag into structured JSON when present.
- [ ] Add `vmmap` parsing fields to `resource.footprint.json` schema (best-effort; macOS-only).
- [ ] Add a `fretboard diag compare --footprint` view that prints deltas for the structured fields.

## Diagnostics (app-side)

- [ ] Add heap byte estimates for text caches (blob cache, shape cache, measure caches).
- [ ] Add cache byte estimates for images/assets where feasible (distinguish CPU decoded bytes vs GPU textures).
- [ ] Keep all new fields behind a “diagnostics” surface (non-contract; best-effort).

## Scripted repro matrix

- [ ] Add `tools/diag-scripts/tooling/empty/empty-idle-memory-steady.json` (schema v2).
- [ ] Add `tools/diag-scripts/tooling/text/text-heavy-memory-steady.json` (forces emoji/color glyphs).
- [ ] Add `tools/diag-scripts/tooling/images/image-heavy-memory-steady.json` (forces texture cache).

## Optimization candidates

- [ ] Run allocator A/B locally (mimalloc/jemalloc) and record impact on:
  - `owned_unmapped_memory_dirty_bytes`
  - `DefaultMallocZone` allocated/frag
- [ ] If allocator sensitivity is high, decide whether to expose a dev knob (env var) for repros.
- [ ] Identify top heap offenders via structured `vmmap` summary and pick one bounded optimization.

## Gates

- [ ] Calibrate a macOS footprint gate for `empty-idle` and `todo-memory-steady`.
- [ ] Calibrate a Metal allocated size gate for `empty-idle` and `todo-memory-steady`.
- [ ] Document acceptable drift policy (e.g. +X MiB allowed with justification).
