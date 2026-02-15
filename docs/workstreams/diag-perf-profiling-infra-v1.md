# Diagnostics perf profiling infra v1

Status: Draft (proposal)

## Goal

Make Fret's perf profiling surfaces "review-grade" and low-friction:

- A failing perf gate links to explainable evidence (bundle + trace) with minimal tribal knowledge.
- Typical perf (p50/p95) and tail (max) are both first-class, with explicit aggregation policy.
- We can distinguish "real CPU work" from OS scheduling noise without requiring ETW/WPR.
- Instrumentation changes are additive, schema-versioned, and safe to refactor (fearless).

This workstream focuses on the **profiling/stats infrastructure** (schema + tooling), not on
optimizing any one hotspot.

## Non-goals

- Replacing external profilers (ETW/WPR, PIX/Nsight/RenderDoc). We want good in-app signals, not a
  full system profiler.
- Turning `debug.stats` into a general metrics system for long-running telemetry (that's a separate
  observability story).

## Current gaps (why this matters)

Symptoms we hit in real perf work:

- A perf threshold fails, but the reviewer has to guess which phase is responsible and which tool
  to run next.
- "Total time" in summaries is often phase-sum based; it misses uninstrumented work and can confuse
  diagnosis when CPU is high but phases look low.
- On Windows, tail spikes can be either real work or preemption; ETW/WPR may be blocked by policy.
- The schema grows organically; keys are "strings by convention", which makes refactors risky and
  makes consumers drift.

## Principles (contract-first)

1) **Perf keys are a contract**
   - Names, units, and aggregation semantics must be stable.
   - Changes should be additive; renames require a compatibility window.
2) **Three-lane profiling**
   - Always-on: cheap counters + coarse timings used by gates.
   - Opt-in: structured spans + top-N attribution used to explain failures.
   - External: sampling profilers for OS/GPU-level questions.
3) **Tail vs typical are explicit**
   - Every gated metric defines its preferred aggregate (`max`, `p95`, etc).
   - Tool output should always show both (so policy decisions are visible).
4) **Evidence loops are short**
   - From a failure: `check.perf_thresholds.json` → worst bundle → `diag stats` / `diag trace`.

## Proposed architecture

### A) Perf key registry (single source of truth)

Introduce a small registry that defines perf keys with metadata:

- key name (string)
- unit (`us`, `cycles`, `bytes`, `count`)
- kind (`timing`, `counter`, `gauge`)
- scope (`frame`, `window`, `process`)
- suggested aggregate(s) (`max`, `p95`)
- optional "drilldown mapping" (which sub-keys explain this key)

This registry should be consumable by:

- bundle exporters (writers)
- `diag stats` / `diag perf` (readers)
- chrome trace exporter (event mapping)
- docs (auto-generated field inventory / tables)

### B) Frame timeline contract (stable spans)

Treat trace event names as a contract (Chromium-style). A stable event namespace lets us build
muscle memory:

- `layout.request_build_roots`
- `layout.engine_solve`
- `layout.roots`
- `paint.cache_replay`
- `paint.widget`
- `dispatch.*`
- `hit_test.*`

The goal is that "worst bundle" always has enough timeline context to answer "which phase spiked".

### C) CPU-time vs schedule noise (Windows-friendly)

When ETW/WPR is unavailable:

- `ui_thread_cpu_time_us` is best-effort and can be coarse on some systems.
- Add and prefer `ui_thread_cpu_cycle_time_delta_cycles` (Windows-only) as a high-resolution signal.

Consumers should surface CPU deltas next to wall/phase time so "thread didn't run" vs "real work"
is visible.

In practice, this should be a one-command workflow:

- `diag stats --sort time` to find the worst wall-time frames
- `diag stats --sort cpu_cycles` (Windows) / `--sort cpu_time` (fallback) to find frames where the UI thread actually ran

## Work plan (incremental, fearless)

1) Land additive per-frame CPU signals (time + cycles) and surface them in:
   - bundles (`debug.stats`)
   - `diag stats` (human + `--json`)
   - `diag trace` (`fret.frame` args)
2) Add explicit "aggregation policy" wiring for gates and summaries:
   - `--perf-threshold-agg` becomes a first-class knob in runbooks.
3) Build a typed perf key registry:
   - move ad-hoc string handling behind one module
   - add contract tests (keys present + units stable)
4) Close the attribution loop for top gated metrics:
   - `top_layout_time_us` → sub-breakdown keys + optional node profiling hooks
5) Document the standard workflow:
   - "typical perf" review (p50/p95)
   - "tail regression" triage (max + worst bundle + trace)

## Comparative notes (how other stacks succeed)

- Chromium/Perfetto: trace event names are stable and treated as a contract; tooling builds on it.
- Flutter: engine timeline events + explicit "UI vs raster thread" separation; perf overlays focus
  on typical frame budget and jank attribution.
- egui/puffin: lightweight in-app profiler that makes typical perf review easy; external profilers
  still needed for tail spikes and OS noise.
- Game engines (Unity/Unreal): hierarchical CPU timeline + counters as the default view; sampling
  profilers are used when the timeline can't answer scheduling/driver questions.

## Evidence anchors (current code)

- Bundle exporter and per-frame stats snapshot: `ecosystem/fret-bootstrap/src/ui_diagnostics.rs`
- Stats aggregation and JSON: `crates/fret-diag/src/stats.rs`
- Chrome trace exporter: `crates/fret-diag/src/trace.rs`
- Layout phase timing sources: `crates/fret-ui/src/tree/layout.rs`
