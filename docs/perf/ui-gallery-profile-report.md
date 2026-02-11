---
title: UI Gallery Profiling Report (Native)
status: draft
date: 2026-01-17
scope: performance, diagnostics, triage
---

# UI Gallery Profiling Report (Native)

## Executive summary

The current “feels slow / stuttery” symptom in `fret-ui-gallery` is dominated by **CPU layout time**, specifically
**layout engine solve time** spent inside **measure callbacks**.

In the worst frames found by scripted perf triage, `layout_engine_solve_time_us` accounts for ~90%+ of total frame time,
and `measure_time_us` accounts for ~99% of solve time.

This report focuses on what to optimize next (and what not to spend time on).

### Update (post-scroll refactor)

After introducing a children-only scroll translation path and fine-grained scroll-handle invalidation (ADR 0217),
scroll offset changes no longer force layout-engine solves. The remaining worst frames in the scripted harness are
still dominated by initial-mount layout/measure work (not by translation-only scrolling).

## Repro (repeatable)

Use the scripted harness to find and pin the worst frames:

```powershell
cargo run -p fretboard -- diag perf tools/diag-scripts/ui-gallery-virtual-list-torture.json `
  --sort time --json --launch -- cargo run -p fret-ui-gallery --release
```

Then inspect the resulting bundle:

```powershell
cargo run -p fretboard -- diag stats target/fret-diag/<timestamp>/bundle.json --sort time --top 1 --json
```

## Evidence (what the tools currently show)

From the worst frames in `ui-gallery-virtual-list-torture`:

- `total_time_us`: ~785k
- `layout_time_us`: ~779k
- `layout_engine_solve_time_us`: ~695k
- `debug.layout_engine_solves[0].measure_time_us`: ~693k
- `debug.layout_engine_solves[0].top_measures[0]`:
  - `element_kind`: `Scroll`
  - `measure_time_us`: ~673k
  - `calls`: 9
  - `cache_hits`: 0
  - `top_children[0]`: the hottest child measured *from inside* this `Scroll` measurement (newer bundles only)

Interpretation:

- We are not “GPU bound” in these worst frames; CPU layout dominates.
- We are not primarily limited by paint cache replay or view-cache reuse in this repro.
- The hottest path is **measuring a `Scroll` node** (and its children) under `MaxContent` constraints.
  With nested attribution enabled, we can also see **which child subtree** dominates the `Scroll` measurement.

### Latest run (with nested child attribution)

In a newer bundle, the `Scroll` hotspot is now attributed to a single child:

- `top_measures[0].element_kind == "Scroll"`
- `top_measures[0].top_children[0].element_kind == "Flex"`
- `top_measures[0].top_children[0].measure_time_us` is ~equal to the parent `Scroll` measure time

This strongly suggests the `Scroll` measurement is spending almost all of its time recursively measuring a single
content container (currently a `Flex` subtree) rather than doing work in the scroll container itself.

## What to do next (recommended)

### 1) Optimize the hot path first (don’t refactor blindly)

Target: `Scroll` measurement / its subtree measurement.

Why: in the worst frames, this single measure hotspot can consume ~85% of the total frame budget.

Suggested next work items:

1. Add stable `SemanticsProps.test_id` around the virtual list torture region (in the demo/ecosystem layer) so
   perf reports can name the problematic region without relying on `NodeId(...)`.
2. Add focused tracing spans inside `measure_scroll` (and the relevant child measure functions) to break down
   the `Scroll` hotspot into its dominant sub-measure(s).
3. Evaluate whether `ScrollProps` needs a “don’t force MaxContent measurement” mode for virtualized content.
   - Today `Scroll` measure uses `MaxContent` on the scroll axis to measure full extent.
   - For `VirtualList` (and other virtualized containers), full extent should come from metrics, not from measuring
     full children trees.

### 2) Use Tracy call stacks only as a “who called this?” tool

Tracy zones can collect call stacks, but this adds overhead. Use it after `diag perf` has identified the exact slow
frame and hotspot, to answer “which code path triggered this measure/solve?”.

See: `docs/tracy.md`.

### 3) Keep development-time profiling cheap and automated

Current workflow is designed to be AI/CI friendly:

- `diag perf ... --json` finds the worst frames deterministically.
- `bundle.json` now exports `debug.layout_engine_solves[].top_measures[]` so tools can attribute solve time to
  specific nodes/elements.

The next step for “developer experience” is to add a small CI check that runs a selected script and fails if
`worst_overall.top_total_time_us` regresses beyond a threshold (opt-in, or nightly).

## What *not* to prioritize yet

### Large-scale redundant code cleanup

Unless cleanup directly reduces the hot path (layout/measure), it is unlikely to improve the stutter.

Broad refactors also risk breaking the diagnostics/trace contracts that make this issue debuggable (ADR 0036 / ADR 0216).

Recommendation: defer cleanup work until after the `Scroll` measure hotspot is understood and either fixed or
bounded by a clear contract.

## Notes / gotchas

- `diag perf --launch` produces many bundles. Ensure `FRET_DIAG_DIR` points to a disk with enough free space, or
  periodically prune `target/fret-diag/`.
