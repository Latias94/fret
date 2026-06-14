---
title: IMUI heavy-component perf progress log
type: progress-log
date: 2026-06-14
execution: code
related_plan: docs/plans/2026-06-14-001-imui-heavy-component-perf-architecture-audit-plan.md
---

# IMUI Heavy-Component Perf Progress Log

## Purpose

This document records execution-time findings, rejected experiments, and next decisions for the
active heavy-component performance goal. It complements the main plan rather than replacing it.

## Current Baseline

- The latest accepted code slice before this follow-up was
  `a2d54dbfe1 perf(gallery): avoid content cache on combobox page`.
- The current follow-up fixes the command/combobox virtual row viewport contract: virtualized
  `CommandPalette` rows now opt the internal `ScrollArea` out of unbounded viewport probing.
- `PAGE_COMBOBOX` opts out of whole-page content cache because the combobox page contains highly
  interactive query/list state.
- The view-cache investigation showed the wrong cache boundary clearly:
  - Whole-page content cache: `total=44825us`, `layout=40500us`,
    `layout_roots_apply=31049us`.
  - Shell-only view cache: `total=12973us`.
  - Combobox page content-cache opt-out: `total=12643us`, `layout_roots_apply=703us`.
  - No-view-cache baseline: `total=12950us`, with most remaining time in paint.
- The main bottleneck has shifted away from broad layout/root apply and toward paint, text
  preparation, renderer encode/finish, and occasional small layout bursts.
- A follow-up stats audit found that the newest `dev-fast-current` bundle's apparent worst frame was
  the scripted `capture_bundle` frame itself. The real top application frames after filtering are
  frames 145 and 146, not frame 148.
- The newest bounded-viewport run reports `total=9591us`, `layout=4436us`,
  `layout.engine_solve=829us`, `paint=4417us`, `roots.apply=516us`, and
  `script_capture_skipped=1`.
- Virtual-list telemetry now shows the long command list as `viewport=272px`, `window_range=0..8`,
  `overscan=8`, and `count=250`; the list model still describes all items, but layout only pays for
  the visible virtual window.

## Decisions

### D1. Do not cache the whole combobox page content root

Whole-page content caching is the wrong abstraction for pages with active overlays, query state,
virtual rows, and diagnostics selectors. It makes the cache boundary too broad and turns ordinary
interactive invalidation into large root-apply work.

The fix should stay at the gallery policy layer: pages with highly interactive long-list content
should opt out of whole-page content caching. This does not require a `fret-ui` mechanism rewrite.

### D2. Treat section-level doc cache as an unproven experiment

An exploratory edit wrapped each `DocSection` in `cx.cached_subtree(...)`. The code compiled with:

```text
cargo check -p fret-ui-gallery --profile dev-fast -j 1
```

The experiment was reverted before commit because it did not yet meet the evidence bar:

- The first perf command accidentally launched the existing release binary, so it did not measure
  the local `doc_layout.rs` edit.
- Default `CachedSubtreeProps` has no section-specific cache key. That is only safe for content
  whose render inputs are known to be stable through recorded model/global observations.
- The docs scaffold mixes static sections, interactive examples, code tabs, test-id decoration, and
  focus filtering. A broad section boundary could become another oversized cache boundary unless it
  is keyed and scoped deliberately.

Do not revive this approach unless the next slice adds explicit cache keys and a same-binary perf
comparison.

### D3. Exclude script capture frames from perf attribution

`capture_bundle` is diagnostic work, not application work. Counting the bundle dump frame in
`diag stats` can invert the next optimization decision by making a script artifact look like the
application's worst interaction frame.

The fix belongs in `fret-diag` stats attribution, not in shadcn components or runtime scheduling.
Stats now derive a capture-frame filter from the bundle-adjacent `script.result.json` sidecar and
apply it to both materialized schema2 bundles and `frames.index.json` stats-lite paths. The report
prints `script_capture_skipped` so future comparisons can tell when a diagnostic frame was excluded.

### D4. Keep the virtual-list viewport fix in the recipe layer

The bad behavior was not global `ScrollArea` semantics. It was the `CommandPalette` virtualized-row
branch composing an internal scroll container where an unbounded probe let the virtual list observe
the full content extent as its viewport.

The fix stays local: virtualized command rows set `viewport_probe_unbounded(false)` on the internal
`ScrollArea`. That keeps ordinary `ScrollArea` behavior unchanged and avoids pushing a policy
decision into `fret-ui`.

## Current Architecture Read

The current evidence argues against a single framework-level rewrite as the next move. The large
wins came from a sequence of narrower seams:

- Component policy: delayed combobox query clearing during close presence.
- Component rendering: virtualized long command/combobox rows.
- Component composition: bounded internal scroll probing for the virtualized command row branch.
- Shared mechanism: command availability interest caching.
- Declarative diff: stable single-line plain text content changes avoid layout invalidation.
- Gallery policy: combobox page opts out of whole-page content cache.

This supports a mixed strategy: optimize component recipes where their composition is wasteful, but
promote the fix into `fret-ui` only when repeated component evidence points at a shared mechanism.

## Next Work

1. Add a durable threshold/gate for the now-acceptable combobox long-list probe before broadening to
   another heavy component.
2. Use `diag stats --sort cpu_cycles --top 30` and `--sort time` on each newest bundle before
   changing code again.
3. Treat stats output without `script_capture_skipped` support as stale for scripted capture bundles.
4. If paint/text preparation dominates, inspect static text/code-block/icon preparation and paint
   cache key churn before changing layout code.
5. If renderer finish/encode dominates with low CPU signal, treat it as scheduling/renderer tail
   rather than a component tree problem until a trace proves otherwise.
6. Keep `CommandPalette`, `Combobox`, `DataTable` toolbar recipes, `Sidebar`, and carousel-heavy
   examples as the next heavy-component candidates. Avoid widening to every shadcn recipe until one
   candidate produces a reproducible tail.

## Verification Notes

- `cargo check -p fret-ui-gallery --profile dev-fast -j 1` passed after returning
  `apps/fret-ui-gallery/src/ui/doc_layout.rs` to the mainline shape.
- `cargo run -p fretboard-dev -- diag stats target/fret-diag/imui-heavy-perf-probes-combobox-devfast-current/1781414534335/bundle.schema2.json --sort time --top 5`
  now reports `script_capture_skipped=1`; top frame moved from the old capture frame 148 to frame
  145 (`total=21041us`) followed by frame 146 (`total=17686us`).
- `cargo run -p fretboard-dev -- diag stats target/fret-diag/imui-heavy-perf-probes-combobox-devfast-current/1781414534335/bundle.schema2.json --sort cpu_cycles --top 5`
  reports the same `script_capture_skipped=1` and the same top application frame 145.
- Focused Rust tests in this lane often time out during Windows test target compilation rather than
  failing assertions. Treat check/build plus perf bundles as the practical gate until the local test
  cache is warm or timeout budgets are raised.
- `cargo test -p fret-ui-shadcn --lib command_palette_virtualized_rows_use_bounded_scroll_viewport --profile dev-fast -j 1`
  passed and locks the bounded virtual-row viewport behavior.
- `cargo build -p fret-ui-gallery --profile dev-fast -j 1` passed before the latest perf run.
- `target\debug\fretboard-dev.exe diag perf tools\diag-scripts\ui-gallery\perf\ui-gallery-combobox-filter-select-steady.json --dir target\fret-diag\imui-heavy-perf-probes-combobox-devfast-bounded-viewport --repeat 1 --warmup-frames 2 --timeout-ms 240000 --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_UI_GALLERY_START_PAGE=combobox --launch -- target\dev-fast\fret-ui-gallery.exe`
  passed with worst frame `total=9591us`; evidence bundle
  `target/fret-diag/imui-heavy-perf-probes-combobox-devfast-bounded-viewport/1781424587044/bundle.schema2.json`.
