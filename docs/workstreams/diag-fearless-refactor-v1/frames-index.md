---
title: Diagnostics Fearless Refactor v1 (Frames Index)
status: draft
date: 2026-02-22
scope: diagnostics, artifacts, indexing, agent
---

# frames.index.json

`frames.index.json` is a lightweight, regeneratable sidecar intended for **agent-friendly triage**
without reading or materializing large parts of the bundle artifact (`bundle.json` / `bundle.schema2.json`).

It is generated from bundle artifacts via:

- `fretboard diag frames-index <bundle_dir|bundle.json|bundle.schema2.json> --warmup-frames <n>`
- or indirectly via `fretboard diag doctor --fix-sidecars ...`
- and is included in `fretboard diag ai-packet ...` (subject to packet budgets).
- and is used by `fretboard diag triage --lite ...` to avoid loading full bundle artifacts into memory.
- and is used by `fretboard diag hotspots --lite ...` to provide a “slow frames” report without materializing bundle artifacts.

## What it contains

At a high level:

- per-window frame rows,
- a small set of per-frame selectors (`frame_id`, `window_snapshot_seq`, timestamps),
- a small set of per-frame timing/stat fields copied from `debug.stats`,
- semantics hints (`semantics_fingerprint`, `semantics_source_tag`).

It intentionally does **not** include large per-frame payloads like:

- full event logs,
- full semantics node arrays,
- full scene data.

## Schema (v1)

Top-level required fields (see sidecar policy for the full rules):

- `kind = "frames_index"`
- `schema_version = 1`
- `warmup_frames`
- `bundle` (best-effort label)
- `features[]` (optional; additive schema evolution flags)
- `columns[]`
- `windows[]`

### Columnar encoding

To keep size down, frames are stored as rows aligned to a `columns[]` list:

- `columns`: ordered list of column names
- `windows[i].rows[j]`: array where `rows[j][k]` corresponds to `columns[k]`

Missing values are encoded as `null`.

Current columns:

- `frame_id`
- `window_snapshot_seq`
- `timestamp_unix_ms`
- `total_time_us`
- `layout_time_us`
- `prepaint_time_us`
- `paint_time_us`
- `invalidation_walk_calls`
- `invalidation_walk_nodes`
- `semantics_fingerprint`
- `semantics_source_tag`

### Semantics source tag

`semantics_source_tag` is encoded as:

- `0`: none/unknown
- `1`: inline semantics present in the snapshot
- `2`: semantics expected to be resolvable from the semantics table (fingerprint present + bundle has a semantics table)

## Notes

- `warmup_frames` is interpreted as a **frame_id threshold** (consistent with existing warmup handling).
- To avoid unbounded memory usage, generators keep only a tail of frames per window and report clipping metadata.

## Optional window aggregates (additive, v1)

When `features[]` contains `window_aggregates.v1`, each `windows[i]` object may include:

- `aggregates.schema_version = 1`
- `aggregates.examined_snapshots_post_warmup`
- `aggregates.viewport_input_events_post_warmup`
- `aggregates.dock_drag_active_frames_post_warmup`
- `aggregates.viewport_capture_active_frames_post_warmup`
- `aggregates.view_cache_active_snapshots_post_warmup`
- `aggregates.view_cache_reuse_events_post_warmup`
- `aggregates.paint_cache_replayed_ops_post_warmup`
- `aggregates.overlay_synthesis_events_total_post_warmup`
- `aggregates.overlay_synthesis_events_synthesized_post_warmup`
- `aggregates.overlay_synthesis_events_suppressed_post_warmup`

These counters are computed over the full streamed snapshot sequence (post-warmup), even if `rows[]`
is tail-clipped.

Additional counters may be present as new `features[]` are added (e.g. overlay synthesis totals).
