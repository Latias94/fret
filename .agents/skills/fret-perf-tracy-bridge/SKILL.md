---
name: fret-perf-tracy-bridge
description: "Bridge Fret's perf gates (diag perf + bundles) with Tracy timeline profiling: reproduce worst bundles, capture traces, correlate spans with bundle stats, and add low-overhead instrumentation safely."
---

# Fret perf + Tracy (end-to-end attribution workflow)

## When to use

Use this skill when you want to go beyond “numbers + worst bundle” and answer:

- *What exactly ran on the UI thread in the worst frame?*
- *Was the hitch real CPU work or scheduling noise?*
- *Which spans/roots/widgets explain the bundle’s top metrics?*

This is especially useful for:

- Tail spikes (max/p95) that are hard to reason about from aggregates alone.
- “Fast alone, slow in suite” behavior (state contamination).
- Renderer regressions where `paint` dominates and you need a timeline.

## Quick start

1) Run a perf suite with a baseline gate:

- `target/release/fretboard.exe diag perf ui-gallery-steady --repeat 3 --warmup-frames 5 --dir target/fret-diag --perf-baseline docs/workstreams/perf-baselines/ui-gallery-steady.windows-rtx4090.v1.json --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --launch -- target/release/fret-ui-gallery.exe`

2) For each failure, jump straight to the evidence bundle:

- Open `target/fret-diag/check.perf_thresholds.json`
- For each `failures[].evidence_bundle`, run:
  - `target/release/fretboard.exe diag stats <bundle.json> --sort cpu_cycles --top 30`

3) Reproduce the same script under Tracy:

- `target/release/fretboard.exe diag repro <script.json> --with tracy --dir target/fret-diag --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --launch -- target/release/fret-ui-gallery.exe`

Then open Tracy and look for the same span taxonomy you see in bundles (`fret.frame`, `fret.ui.layout.*`, `fret.ui.paint.*`, cache-root spans, renderer spans).

## Workflow

1) Decide what you’re optimizing

- Tail smoothness: use `diag perf` with `max` baselines and correlate the single worst bundle.
- Typical perf: seed and gate p95/p90 baselines; don’t overfit to one spike.

2) Attribute from bundle first (cheap)

- Use `diag stats --sort cpu_cycles` to separate “real work” vs “wall-clock noise”.
- Check phase split (`layout` vs `paint`), then the hot breakdown:
  - Layout: `layout.engine_solve`, invalidation walks, request/build roots, view-cache.
  - Paint: `paint.widget`, cache replay/misses, scene encoding, text prepare.

3) Move to Tracy only when you have a hypothesis

- Use Tracy to validate:
  - “Which span is actually the long pole?”
  - “Is it a single big region or many small regions?”
  - “Are we blocked on GPU submission or doing CPU work?”

4) Add instrumentation safely (low overhead by default)

- Prefer `tracing` spans that are:
  - Cheap when disabled (`tracing::enabled!` fast-path).
  - Stable names (so comparison across runs is meaningful).
  - Field-light (avoid formatting strings on hot paths).
- Prefer timing helpers that use `fret_core::time::Instant`:
  - `fret_perf::measure(...)`
  - `fret_perf::measure_span(...)`
- Keep “profiling mode” explicit:
  - Environment-gated instrumentation (e.g. `FRET_LAYOUT_PROFILE=1`) should be treated as *profiling-only* and is expected to change perf numbers.

5) Close the loop

- Re-run `diag perf` without profiling flags to confirm you improved the gate surface.
- Keep changes reversible and leave an evidence note in the active workstream doc.

## Evidence anchors

- Perf gate evidence: `target/fret-diag/check.perf_thresholds.json`
- Bundle attribution: `target/release/fretboard.exe diag stats <bundle.json>`
- Tracy usage + span taxonomy:
  - `docs/tracy.md`
  - `docs/adr/0036-observability-tracing-and-ui-inspector-hooks.md`
  - `docs/adr/0181-ui-automation-and-debug-recipes-v1.md`
- Layout and cache-root spans:
  - `crates/fret-ui/src/tree/layout.rs`
  - `crates/fret-ui/src/tree/paint.rs`

## Common pitfalls

- Running perf gates with profiling flags on (e.g. `FRET_LAYOUT_PROFILE=1`): you’ll “fail” due to instrumentation overhead.
- Capturing Tracy without a script repro: you’ll get a noisy timeline that’s hard to compare.
- Adding spans that allocate/format on hot paths: prefer IDs and small integers; avoid building debug paths unless diagnostics are on.

## Related skills

- `fret-diag-workflow`: scripts, bundles, triage, and perf baselines/gates.
- `fret-perf-optimization`: turning hitches into durable perf contracts and landing reversible fixes.
- `fret-framework-maintainer-guide`: contract-first changes and evidence discipline.

