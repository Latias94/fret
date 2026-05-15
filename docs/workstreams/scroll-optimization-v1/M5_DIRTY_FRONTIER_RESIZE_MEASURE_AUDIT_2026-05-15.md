# M5 Dirty Frontier / Resize-Measure Audit

Date: 2026-05-15
Status: Slice closed; workstream remains active.

## Objective

Close the current `layout resize-measure dirty frontier` slice for scroll/view-cache resize stress:

- reduce or rule out broad `direct-child-invalidated` / `resize-measure` request-build and layout
  solve breadth as the next hot path,
- preserve conservative correctness for `Scroll` and `VirtualList` side-effect nodes,
- leave repeatable correctness gates, `diag perf` evidence, layout attribution, and follow-on
  guidance.

## Changes

- `crates/fret-ui/src/declarative/host_widget/layout/scrolling.rs`
  - Scroll post-layout overflow observation now records whether the observed root is the synthetic
    scroll content extent on each axis.
  - A stale, pinned synthetic content root now trusts the observed child frontier instead of keeping
    a contracted extent pinned to the old content box.
- `crates/fret-ui/src/declarative/host_widget/event/scroll.rs`
  - Non-retained `VirtualList` visible-range escapes now notify the cache root for rerender.
  - Retained virtual lists keep using the retained reconcile marker and do not notify the cache
    root.

## Correctness Gates

- `cargo nextest run -p fret-ui scroll_post_layout_mixed_child_invalidation_keeps_descendant_only_shrink_authoritative virtual_list_triggers_visible_range_rerender_on_wheel_scroll_when_cached retained_virtual_list_updates_visible_range_on_wheel_scroll_without_notifying_view_cache --no-fail-fast`
  - Result: passed (`4` tests).
- `cargo nextest run -p fret-ui scroll --no-fail-fast`
  - Result: passed (`151` tests).

These gates cover:

- mixed direct-child invalidation plus descendant-only shrink at the scroll edge,
- the non-edge shrink variant,
- non-retained virtual-list cache-root rerender on visible-range escape,
- retained virtual-list reconcile without view-cache rerender notification,
- the broader scroll and virtual-list mechanism surface.

## Perf Attribution

Command:

```bash
target/release/fretboard-dev diag perf ui-resize-probes \
  --repeat 3 \
  --warmup-frames 5 \
  --reuse-launch \
  --prewarm-script tools/diag-scripts/_prelude/tooling-suite-prewarm-fonts.json \
  --prelude-script tools/diag-scripts/_prelude/tooling-suite-prelude-reset-diagnostics.json \
  --env FRET_UI_GALLERY_BOOTSTRAP_FONTS=1 \
  --env FRET_UI_GALLERY_VIEW_CACHE=1 \
  --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 \
  --env FRET_SCROLL_LAYOUT_PROFILE=1 \
  --env FRET_SCROLL_LAYOUT_PROFILE_MIN_US=300 \
  --env FRET_LAYOUT_NODE_PROFILE=1 \
  --env FRET_LAYOUT_NODE_PROFILE_TOP=20 \
  --env FRET_LAYOUT_NODE_PROFILE_MIN_US=300 \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 \
  --env FRET_DIAG_SEMANTICS=0 \
  --sort time \
  --top 15 \
  --json \
  --dir target/fret-diag/scroll-optimization-v1-ui-resize-probes-20260515 \
  --launch -- target/release/fret-ui-gallery
```

Artifacts:

- Worst bundle:
  `target/fret-diag/scroll-optimization-v1-ui-resize-probes-20260515/1778819328814/bundle.schema2.json`
- Layout attribution:
  `target/fret-diag/scroll-optimization-v1-ui-resize-probes-20260515/layout.perf.summary.v1.json`
- Regression summary:
  `target/fret-diag/scroll-optimization-v1-ui-resize-probes-20260515/regression.summary.json`

Suite scripts:

- `ui-gallery-window-resize-drag-jitter-steady`:
  - total p50/p95/max: `989/1002/1002us`
  - layout p50/p95/max: `599/616/616us`
  - paint p50/p95/max: `343/347/347us`
  - layout engine solve p50/p95/max: `329/344/344us`
  - barrier relayouts: `0`
  - contained relayouts: `0`
  - reused cache roots: `2`
  - visible-range refresh p95: `1`
- `ui-gallery-window-resize-stress-steady`:
  - total p50/p95/max: `2041/2220/2220us`
  - layout p50/p95/max: `732/812/812us`
  - paint p50/p95/max: `1210/1270/1270us`
  - layout engine solve p50/p95/max: `330/377/377us`
  - barrier relayouts: `0`
  - contained relayouts: `0`
  - reused cache roots: `2`
  - visible-range refresh p95: `0`

Worst-bundle `diag stats --sort time --top 15`:

- considered frames: `10`
- total p50/p95: `236/2220us`
- layout p50/p95: `91/812us`
- paint p50/p95: `107/1270us`
- hot p50/p95: `layout.engine_solve=0/377us`, `paint.widget=27/649us`,
  `paint.text_prepare=0/0us`
- top frames: `inv.calls=0`, `barrier(set_children/scheduled/performed)=0/0/0`,
  `contained_relayouts=0`, `cache.reused=2`

Layout summary:

- worst frame layout: `812us`
- engine solve: `377us` across `2` solves
- top solves are `new_frame_key_changed` on bounded subtrees:
  - `subtree_nodes=100`, `solve_time_us=273`
  - `subtree_nodes=107`, `solve_time_us=103`
- widget measure time in the top solves is `0us`.

## Verdict

The normalized view-cache resize-stress proof surface no longer supports another
`direct-child-invalidated` / `resize-measure` narrowing as the next high-value change:

- invalidation-walk breadth is absent from the considered steady frames,
- barrier and contained relayout churn are absent from the top frames,
- layout solve is bounded below `0.4ms`,
- the tail is paint-dominant on this sample.

The correct next step is not another broad scroll apply-skip branch. If resize performance still
needs improvement, switch to a hotter proof surface or target paint/cache replay attribution first.

## Residual Risk

- This audit covers the normalized view-cache `ui-gallery-window-resize-stress-steady` proof
  surface, not every resize script in the workspace.
- The initial launch/warmup path can still log larger scroll layout work; that is not the steady
  p95 proof surface used here.
- CPU cycle counters were unavailable in this bundle (`cpu.cycles=0`), so the attribution relies
  on wall-clock phase timing and layout summaries.
