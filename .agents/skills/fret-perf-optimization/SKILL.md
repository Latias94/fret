---
name: fret-perf-optimization
description: "This skill should be used when the user asks to \"optimize UI performance\", \"investigate jank\", \"create a perf baseline\", or \"attribute worst-frame hitches\". Provides a perf workflow (tail vs typical, suite normalization, worst-bundle attribution) to land reversible optimizations with evidence and gates."
---

# Fret performance optimization (contracts + attribution + reversible fixes)

## When to use

Use this skill when you need to **optimize performance** (not just measure it), especially for:

- Frame smoothness issues (tail spikes / stutter / resize/scroll jank).
- “Fast when run alone, slow in suite” behavior (cross-script state contamination).
- Establishing a durable **perf contract**: baselines + gates + explainable worst bundle.
- Windows-specific noise vs real CPU work (ETW/WPR + in-app CPU signals).

Use `fret-diag-workflow` when your main goal is simply “run a script, capture a bundle, and triage”.

## Quick start

1) Run a suite with normalization hooks (recommended defaults):

- `cargo run -p fretboard --release -- diag perf ui-gallery-steady --repeat 7 --warmup-frames 5 --reuse-launch --suite-prewarm tools/diag-scripts/tooling-suite-prewarm-fonts.json --suite-prelude tools/diag-scripts/tooling-suite-prelude-reset-diagnostics.json --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --launch -- cargo run -p fret-ui-gallery --release`

2) When a perf gate fails, go straight to per-failure evidence:

- Open `target/fret-diag/check.perf_thresholds.json`
- For each item in `failures[]`, run:
  - `cargo run -p fretboard --release -- diag stats <evidence_bundle> --sort cpu_cycles --top 30`

3) If you need node-level layout attribution:

- Re-run the single script with:
  - `--env FRET_LAYOUT_NODE_PROFILE=1`
  - `--env FRET_LAYOUT_NODE_PROFILE_TOP=20`
  - `--env FRET_LAYOUT_NODE_PROFILE_MIN_US=300`

## Workflow

1) Decide what you’re optimizing

- Tail vs typical:
  - Tail smoothness: optimize `max` / worst frames.
  - Typical perf: optimize p50/p95 and use `--perf-threshold-agg p95` with a percentile-seeded baseline.
- Pick a single “north star” metric per loop (usually `top_total_time_us`, then drill into `layout` vs `paint`).

2) Stabilize the measurement surface (reduce false regressions)

- Prefer **suite normalization hooks** over ad-hoc sleeps:
  - Prewarm once per process: fonts/catalogs/asset caches.
  - Prelude before each script (and optionally each run): reset diagnostics, dismiss overlays, return to a known state.
- If `--reuse-launch` makes a script slower, treat that as signal:
  - Either state contamination (scripts interfere), or real cache/invalidation behavior.
  - Use prelude to separate the two.

3) Attribute the slow frames (CPU work vs scheduling noise)

- Use `diag stats --sort cpu_cycles`:
  - High `cpu.cycles` + stable hotspots => real work regression (optimize code).
  - Low CPU signal + high `total_time_us` => likely scheduling/priority noise (confirm via ETW/WPR).
- Partition by phase:
  - Layout: `layout.engine_solve`, invalidations, roots/build_roots.
  - Paint: `paint.widget`, cache replay/misses, text prepare, scene encode.

4) Narrow to the smallest repro

- Prefer a single script JSON that triggers the hotspot deterministically.
- If needed, shrink the script (see `fret-diag-workflow`’s `diag script shrink` guidance).
- Keep evidence reproducible: same `--env`, same window size, same warmup, same suite hooks.

5) Land a reversible optimization

- Make the change small and measurable (one mechanism, one expected effect).
- Add a regression gate:
  - Tail contract: keep existing `max` baselines for P0.
  - Typical contract: create a local p95-seeded baseline and gate with `--perf-threshold-agg p95`.
- Preserve layering boundaries (pull `fret-boundary-checks` if a refactor crosses crates).

6) Leave evidence behind

- Update the active workstream doc with:
  - The repro command (including suite hooks).
  - The worst `evidence_bundle` path(s).
  - `diag stats` summary (top phases + one or two hotspots).
  - Rollback plan (which commit to revert).

## Evidence anchors

Common “where to look” anchors for smoothness work:

- Perf gate output: `target/fret-diag/check.perf_thresholds.json` (each failure includes `evidence_bundle`)
- Bundle triage: `cargo run -p fretboard --release -- diag stats <bundle.json> --sort cpu_cycles --top 30` (or `target/release/fretboard[.exe] ...` if already built)
- Layout mechanism:
  - `crates/fret-ui/src/tree/layout/mod.rs`
  - `crates/fret-ui/src/layout/engine.rs`
- Perf tooling:
  - `crates/fret-diag/src/lib.rs` (suite hooks, perf gates, baseline seeding)
  - `crates/fret-diag/src/stats.rs` (stats schema, cpu sort keys)
- Suite normalization scripts:
  - `tools/diag-scripts/tooling-suite-prewarm-fonts.json`
  - `tools/diag-scripts/tooling-suite-prelude-reset-diagnostics.json`

## Examples

- Example: convert "it feels janky" into a perf contract
  - User says: "Scrolling hitches sometimes—make it measurable and fixable."
  - Actions: run `diag perf`, attribute the worst bundle, land a reversible change, and gate it.
  - Result: a small optimization with a durable regression gate.

## Common pitfalls

- Using a `max`-seeded baseline with a `p95` gate (or vice versa): you’ll get systematic failures.
- Treating `--reuse-launch` slowdowns as “noise”: it’s often state contamination or a real cache/invalidation issue.
- Over-optimizing a demo script with unstable setup cost (fonts/catalog rescan): prewarm it instead.
- Chasing wall-clock spikes without checking CPU signal (`cpu_cycles`/`cpu_time`).

## Troubleshooting

- Symptom: perf gates fail after a machine change.
  - Fix: keep baselines environment-specific; re-seed baselines intentionally (do not silently loosen thresholds).
- Symptom: suite runs are slower than standalone.
  - Fix: normalize suite setup (prewarm/reset scripts) and watch for state contamination.

## Related skills

- `fret-diag-workflow`: scripts, bundles, perf gates, attribution tooling (the “how”)
- `fret-framework-maintainer-guide`: contract-first changes and evidence discipline (the “when it’s framework-level”)
- `fret-boundary-checks`: guardrails for refactors that might violate layering (the “don’t regress portability”)
