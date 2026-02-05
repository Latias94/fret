---
title: UI Performance: Paint Pass Breakdown (v1)
status: draft
date: 2026-02-05
scope: diagnostics, paint, view-cache, paint-cache, profiling
---

# UI Performance: Paint Pass Breakdown (v1)

Status: Draft (workstream note; ADRs remain the source of truth)

This workstream exists because several “steady-state” UI probes show that **paint** remains a sizable CPU slice
even when:

- view-cache roots are reused, and
- paint-cache hits replay prior ops.

The goal is to make the paint pass explainable (and therefore optimizable) using `fretboard diag perf` bundles.

Related:

- Zed smoothness workstream: `docs/workstreams/ui-perf-zed-smoothness-v1.md`
- GPUI gap analysis: `docs/workstreams/ui-perf-gpui-gap-v1.md`
- Perf log: `docs/workstreams/ui-perf-zed-smoothness-v1-log.md`

---

## 0) Evidence: “menubar steady” is paint-heavy but cache replay is not

Commit: `f2bee87a` (`feat(diag): add paint pass breakdown metrics`)

Probe:

- Script: `tools/diag-scripts/ui-gallery-menubar-keyboard-nav-steady.json`
- Command (steady; repeat=7):

```bash
target/debug/fretboard diag perf tools/diag-scripts/ui-gallery-menubar-keyboard-nav-steady.json \
  --dir target/fret-diag-perf/menubar-kbd-nav.after-f2bee87a \
  --reuse-launch --repeat 7 --timeout-ms 180000 --sort time --top 15 --json \
  --env FRET_UI_GALLERY_VIEW_CACHE=1 \
  --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 \
  --env FRET_DIAG_SEMANTICS=0 \
  --launch -- target/release/fret-ui-gallery
```

Worst bundle (example run dir used during investigation):

- `target/fret-diag-perf/menubar-kbd-nav.after-f2bee87a.1770300800/1770285619385-ui-gallery-menubar-file-escape-steady/bundle.json`

Key observation from `fretboard diag stats --sort time` on the worst frame:

- `paint_time_us` is still ~2.6ms.
- `paint_cache_replay_time_us` is single-digit microseconds (replaying ~450 ops).
- `paint_cache_bounds_translate_time_us` is ~0us (no subtree translation on this workload).
- `paint_record_visual_bounds_time_us` is low double-digit microseconds.

Implication:

- In this probe, the “expensive paint slice” is **not** caused by the paint-cache replay loop itself.
- We need finer-grained paint-phase attribution to locate the actual hotspot (per-node traversal overhead,
  observation bookkeeping, widget paint overhead, global snapshot plumbing, etc.).

---

## 1) Evidence: paint-all micro timers are not the hotspot (yet)

Commit: `b20a1280` (`feat(diag): add paint micro-breakdown timers`)

Probe:

- Script: `tools/diag-scripts/ui-gallery-menubar-keyboard-nav-steady.json`
- Run dir: `target/fret-diag-perf/menubar-kbd-nav.after-b20a1280.micro.1770287305/`
- Command (repeat=7; `sort=time`):

```bash
target/debug/fretboard diag perf tools/diag-scripts/ui-gallery-menubar-keyboard-nav-steady.json \
  --dir target/fret-diag-perf/menubar-kbd-nav.after-b20a1280.micro.1770287305 \
  --reuse-launch --repeat 7 --timeout-ms 180000 --sort time --top 15 --json \
  --env FRET_UI_GALLERY_VIEW_CACHE=1 \
  --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 \
  --env FRET_DIAG_SEMANTICS=0 \
  --launch -- target/release/fret-ui-gallery
```

Worst bundle:

- `target/fret-diag-perf/menubar-kbd-nav.after-b20a1280.micro.1770287305/1770287306932-ui-gallery-menubar-file-escape-steady/bundle.json`

Worst-frame paint breakdown (from `fretboard diag stats --sort time --top 1`):

- `paint_time_us=2693`
- `paint_cache_replay_time_us=6` (`paint_cache_replayed_ops=453`)
- `paint_record_visual_bounds_time_us=15` (`paint_record_visual_bounds_calls=155`)
- `paint_breakdown.us(input_ctx/scroll_inv/collect_roots/text_snapshot/collapse)=0/0/0/0/46`

Interpretation:

- The newly-attributed “paint-all plumbing” micro timers are **not** where the ~2.6ms paint time goes.
- Note: a `0us` timer means “< 1us” due to integer truncation in `as_micros()`; it still indicates the slice is
  not a meaningful contributor for this probe.
- The remaining paint slice is still likely dominated by one (or a combination) of:
  - per-node traversal overhead on stable frames (walking down to cache roots),
  - widget `paint()` costs on cache misses,
  - paint observation bookkeeping not yet timed.

---

## 2) What we need to measure next (paint-phase micro timers)

Add paint-phase sub-timers to diagnostics so the worst frame is explainable without a sampling profiler:

1) **Paint traversal** (per-node overhead on cache misses vs cache hits)
2) **Observation bookkeeping**
   - `observed_in_paint.record(...)`
   - `observed_globals_in_paint.record(...)`
3) **PaintCx + widget paint overhead**
   - `with_widget_mut(...)` and widget dispatch
4) **Window snapshot plumbing done during paint**
   - `WindowInputContextService` snapshot publish
   - `WindowTextInputSnapshotService` snapshot publish
5) **Layer root enumeration** (`visible_layers_in_paint_order` + root collection)
6) **Paint-cache bookkeeping**
   - key computation
   - cache hit bookkeeping (range checks, entry write-back)

Acceptance:

- `fretboard diag stats` top-frame print includes these sub-timers (microseconds).
- We can explain “paint ~2.5ms” by identifying 1–2 dominant sub-slices.

---

## 3) Likely refactor directions (fearless)

Once the hotspot is confirmed, likely solutions fall into two buckets:

### 2.1 Reduce per-node overhead on stable frames

If paint is dominated by “walking down to cache roots”, consider:

- keeping a per-layer list of cache roots to paint directly (avoid visiting non-cache-root nodes),
- tightening early-out conditions so stable frames short-circuit before setting up observation tracking,
- batching repeated global lookups (theme revision, capabilities, etc.) into per-frame snapshots.

### 2.2 Make “cached view replay” cheaper at a higher level (GPUI-aligned)

If stable frames still pay meaningful work even with view-cache reuse, consider moving toward a more GPUI-like model:

- cached views that can skip `Widget::paint()` entirely unless invalidated by `notify`,
- renderer-friendly replay primitives (pre-encoded chunks / display list segments), not just “replay ops into a scene”.

These changes likely require contract updates (ADRs) if they become part of the runtime surface.
