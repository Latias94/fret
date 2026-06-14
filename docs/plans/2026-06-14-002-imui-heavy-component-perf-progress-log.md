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

- The latest accepted code slice is `a2d54dbfe1 perf(gallery): avoid content cache on combobox page`.
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

## Current Architecture Read

The current evidence argues against a single framework-level rewrite as the next move. The large
wins came from a sequence of narrower seams:

- Component policy: delayed combobox query clearing during close presence.
- Component rendering: virtualized long command/combobox rows.
- Shared mechanism: command availability interest caching.
- Declarative diff: stable single-line plain text content changes avoid layout invalidation.
- Gallery policy: combobox page opts out of whole-page content cache.

This supports a mixed strategy: optimize component recipes where their composition is wasteful, but
promote the fix into `fret-ui` only when repeated component evidence points at a shared mechanism.

## Next Work

1. Re-run the combobox long-list perf probe against the intended binary and profile.
2. Use `diag stats --sort cpu_cycles --top 30` and `--sort time` on the newest bundle before
   changing code again.
3. If paint/text preparation dominates, inspect static text/code-block/icon preparation and paint
   cache key churn before changing layout code.
4. If renderer finish/encode dominates with low CPU signal, treat it as scheduling/renderer tail
   rather than a component tree problem until a trace proves otherwise.
5. Keep `CommandPalette`, `Combobox`, `DataTable` toolbar recipes, `Sidebar`, and carousel-heavy
   examples as the next heavy-component candidates. Avoid widening to every shadcn recipe until one
   candidate produces a reproducible tail.

## Verification Notes

- `cargo check -p fret-ui-gallery --profile dev-fast -j 1` passed after returning
  `apps/fret-ui-gallery/src/ui/doc_layout.rs` to the mainline shape.
- Focused Rust tests in this lane often time out during Windows test target compilation rather than
  failing assertions. Treat check/build plus perf bundles as the practical gate until the local test
  cache is warm or timeout budgets are raised.

