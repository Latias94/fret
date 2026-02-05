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

## 2) Evidence: stable-frame paint is dominated by widget paint code

Commit: `c512be81` (`feat(diag): add paint node breakdown timers`)

Probe:

- Script: `tools/diag-scripts/ui-gallery-menubar-keyboard-nav-steady.json`
- Run dir: `target/fret-diag-perf/menubar-kbd-nav.after-c512be81.1770289882/`
- Command (repeat=7; `sort=time`):

```bash
target/debug/fretboard diag perf tools/diag-scripts/ui-gallery-menubar-keyboard-nav-steady.json \
  --dir target/fret-diag-perf/menubar-kbd-nav.after-c512be81.1770289882 \
  --reuse-launch --repeat 7 --timeout-ms 180000 --sort time --top 15 --json \
  --env FRET_UI_GALLERY_VIEW_CACHE=1 \
  --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 \
  --env FRET_DIAG_SEMANTICS=0 \
  --launch -- target/release/fret-ui-gallery
```

Worst bundle:

- `target/fret-diag-perf/menubar-kbd-nav.after-c512be81.1770289882/1770289882739-ui-gallery-menubar-file-escape-steady/bundle.json`

Worst-frame paint breakdown (from `fretboard diag stats --sort time --top 1`):

- `paint_time_us=2655`
- `paint_node.us(cache_key/hit_check/widget/obs_record)=3/0/2555/11`
- `paint_breakdown.us(input_ctx/scroll_inv/collect_roots/text_snapshot/collapse)=0/0/0/0/43`

Interpretation:

- For this stable workload, the paint pass is overwhelmingly dominated by **widget paint code**:
  - `paint_widget_time_us` is an **exclusive** timer that pauses when painting child subtrees (so it does not double
    count recursion through `PaintCx::paint()`).
- This strongly suggests that “view-cache reuse” does not currently provide a “skip most widget paint” fast path for
  stable frames; only a small number of paint-cache roots are replaying ops, while most nodes still run `Widget::paint()`.

### 2.1 Evidence: widget paint hotspots point at `ElementHostWidget`

Commit: `e1132c95` (`feat(diag): export paint widget hotspots`)

Probe:

- Script: `tools/diag-scripts/ui-gallery-menubar-keyboard-nav-steady.json`
- Run dir: `target/fret-diag-perf/menubar-kbd-nav.after-paint-widget-hotspots.1770292980/`
- Worst bundle:
  - `target/fret-diag-perf/menubar-kbd-nav.after-paint-widget-hotspots.1770292980/1770292982106-ui-gallery-menubar-file-escape-steady/bundle.json`

Worst-frame paint breakdown (from `fretboard diag stats --sort time --top 1`):

- `paint_time_us=2592`
- `paint_node.us(cache_key/hit_check/widget/obs_record)=3/0/2487/12`
- `paint_widget_hotspots` (top 3):
  - `us=1117 type=fret_ui::declarative::host_widget::ElementHostWidget ops(excl/incl)=1/1`
  - `us=942  type=fret_ui::declarative::host_widget::ElementHostWidget ops(excl/incl)=1/1`
  - `us=373  type=fret_ui::declarative::host_widget::ElementHostWidget ops(excl/incl)=1/1`

Interpretation:

- The stable-frame paint cost is not distributed across “many small widgets”; it is dominated by a few host-widget
  nodes.
- The ops deltas (`1/1`) suggest the cost is not scene construction, but CPU bookkeeping in the host-widget paint path.
- Initial hypothesis: element-runtime observation access was cloning per-element dependency vectors and/or paying
  hidden “touch” clone costs on cache-hit frames.
- Update: quick attempts to remove/avoid those clones (commits `424ca9fc`, `df5df0b7`) did not materially reduce the
  hotspots on this probe (see the perf log entries on 2026-02-05 20:28 and 20:37).
  - Next (done): add sub-timers inside `ElementHostWidget::paint_impl` (obs-models, obs-globals, instance lookup) so
    the remaining ~1ms slices are attributable before refactoring further.

Follow-up instrumentation:

- Commit: `188d7da1` (`feat(diag): add host-widget paint sub-timers`)
- Evidence bundle:
  - `target/fret-diag/1770297604582-ui-gallery-menubar-file-escape-steady/bundle.json`
- Worst-frame paint breakdown (from `fretboard diag stats --sort time --top 1`):
  - `paint_node.us(cache_key/hit_check/widget/obs_record)=3/0/2452/12`
  - `paint_host_widget.us(models/globals/instance)=16/10/16 items=14/1 calls=153`

Updated interpretation:

- Observed deps + instance lookup are **not** the cause of the ~1ms+ `ElementHostWidget` hotspots (they are O(10us)).
- Next: time the remaining “host-widget paint overhead” candidates:
  - child traversal overhead in `paint_children_clipped_if` (clip push/pop + `child_bounds` queries + `cx.paint` call
    overhead),
  - per-instance-variant overhead (e.g. `Container` vs `ViewCache`),
  - any per-frame “first paint” work hidden behind element instance properties (e.g. cache keys, transforms).

---

## 3) What we need to measure next (paint-phase micro timers)

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

## 4) Likely refactor directions (fearless)

Once the hotspot is confirmed, likely solutions fall into two buckets:

### 4.1 A/B: broaden paint-cache eligibility (experimental)

Commit: `f3078d25` adds an env-gated knob:

- `FRET_UI_PAINT_CACHE_RELAX_VIEW_CACHE_GATING=1`

Initial A/B on the menubar steady probe did not materially improve `paint_time_us` (see perf log entry), even though
it increased paint-cache hits and reduced `paint_nodes_performed`.

Implication:

- Stable-frame paint is likely dominated by a small set of “high-level” widgets that still run `paint()`.
- Next: add per-node paint hotspots (or cache-disabled reason counters) to identify why those widgets cannot be cached
  (render transforms, invalidation behavior, key instability, etc.).

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
