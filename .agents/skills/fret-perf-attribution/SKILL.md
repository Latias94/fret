---
name: fret-perf-attribution
description: "Attribute and explain Fret performance hitches using diag bundles + perf gates, then choose the right next profiler (CPU stacks, allocations, GPU capture). Use when a perf gate fails or when UX feels janky despite passing averages."
---

# Fret perf attribution workflow

Use this skill when you already have a **perf gate result** (PASS/FAIL) and need to answer:

- *What made the worst frame slow?*
- *Is it CPU layout/paint work, renderer churn, GPU stalls, or allocator spikes?*
- *What is the next measurement that will reduce uncertainty?*

Companion skills:

- If you need **numbers/baselines/gates**, start with `fret-perf-workflow`.
- If you need a **minimal repro script / bundle packaging**, use `fret-diag-workflow`.

---

## Quick start (from a failing gate)

1) Summarize which attempt/script/metric failed and print worst bundles:

```bash
.agents/skills/fret-perf-workflow/scripts/triage_gate.sh <out-dir> --all --app-snapshot
```

2) Pick the worst bundle for the failing script, then inspect the top frames:

```bash
cargo run -p fretboard -- diag stats <bundle.json> --sort time --top 30
```

3) Decide the axis of work:

- If `layout.solve_us` dominates: focus on layout roots + text measure/shaping reuse.
- If `paint` dominates: focus on text prepare churn, scene-op rebuilds (Canvas), and cache replay hit rate.
- If CPU looks low but it still hitches: capture a trace (see sections below).

Record the result in the perf log:

- `docs/workstreams/ui-perf-zed-smoothness-v1-log.md`

---

## How to read `diag stats` (high-signal fields)

The `top` row includes:

- `time.us(total/layout/prepaint/paint)` — the CPU frame budget split.
- `layout.solve_us` — layout engine solve cost.
- `paint_text_prepare.us(time/calls)` and `paint_text_prepare.reasons(...)` — text prepare churn (often width/wrap).
- `paint_widget_hotspots` — “which element kind dominates paint”.
- `top_layout_engine_solves` — multi-root solves (viewport roots), and `measure.*` stats.

Heuristics:

- **High `layout.solve_us` + high `measure.calls`**: text measure/shaping is likely the driver.
- **High `paint_text_prepare` with `reasons=width`** during resize jitter: wrapped text width churn.
- **Canvas dominates `paint_widget_hotspots`**: scene-op rebuild; investigate replay boundaries.
- **Large `layout.nodes` / `paint.nodes` swings**: invalidation scope may be too broad.

---

## Find the frame that maxes a specific metric (bundle-local, deterministic)

Many perf thresholds are keyed to “max of a metric over all frames”, not necessarily the single “worst total” frame.
When you need the exact frame that triggered a max threshold (e.g. `layout_engine_solve_time_us`), use `jq` to scan
the bundle snapshots directly:

```bash
jq -c '
  .windows[0].snapshots
  | map({
      frame_id,
      tick_id,
      ts: .timestamp_unix_ms,
      layout: .debug.stats.layout_time_us,
      solve: .debug.stats.layout_engine_solve_time_us,
      paint: .debug.stats.paint_time_us,
      prepaint: .debug.stats.prepaint_time_us,
      total: ((.debug.stats.layout_time_us // 0) + (.debug.stats.prepaint_time_us // 0) + (.debug.stats.paint_time_us // 0))
    })
  | max_by(.solve)
' <bundle.json> | jq .
```

Then extract that exact snapshot (example uses `frame_id==1071`):

```bash
jq '
  .windows[0].snapshots[]
  | select(.frame_id == 1071)
  | {frame_id, tick_id, ts: .timestamp_unix_ms, stats: .debug.stats, layout_hotspots: .debug.layout_hotspots[0:10], paint_text_prepare_hotspots: .debug.paint_text_prepare_hotspots[0:10]}
' <bundle.json> | head -n 200
```

--- 

## Common hitch classes (and what to try next)

### A) Resize-drag jank

Symptoms:

- `ui-resize-probes` fails intermittently.
- `layout.solve_us` spikes, often correlated with wrapped text width changes.

Next steps:

- Verify “interactive resize shaping” knobs and caches (see the resize workstream notes).
- Check whether multiple viewport roots are being solved unexpectedly.
- Prefer “make work bounded and stable” rather than “skip layout”.

References:

- `docs/workstreams/ui-perf-resize-path-v1.md`
- `docs/workstreams/ui-perf-zed-smoothness-v1-todo.md`

### B) Editor/canvas paint dominates

Symptoms:

- `paint_widget_hotspots[0].element_kind=Canvas` dominates.
- `scene_ops_delta` grows, replay looks low.

Next steps:

- Add/enable internal attribution in `app_snapshot` (component-specific where needed).
- Define a replay/cache boundary that is correctness-safe (keyed, bounded).
- Add a gate probe that exercises the workload (resize jitter / autoscroll steady).

Reference:

- `docs/workstreams/ui-perf-zed-smoothness-v1-todo.md` (P1.5)

### C) “Numbers look fine” but UX still hitches

Possible causes:

- allocator spikes (rare tail allocations),
- GPU upload/eviction churn,
- OS scheduling / timer contention.

Next steps:

- Capture a CPU stack trace and allocations (see below).
- If CPU is truly low, attempt a GPU capture.

---

## External profiling recipes (choose one, keep it minimal)

### macOS: Instruments (recommended first)

CPU stacks:

- Use **Time Profiler** on the smallest repro (one script).
- Start recording, then trigger the hitch (resize drag / scroll).
- Correlate the hitch window with the `diag` bundle timestamps if possible.

Allocations:

- Use **Allocations** to confirm “per-frame hot allocations” during hitch windows.

### Linux: `perf`

CPU stacks:

```bash
perf record -g -- target/release/fret-ui-gallery
perf report
```

### GPU capture (best-effort)

- RenderDoc (when available): capture the worst-frame scenario and look for upload/resolve stalls.
- Tracy: use the repo docs if the integration is enabled on your target.

References:

- `docs/tracy.md`
- `docs/renderdoc-inspection.md`

---

## Evidence discipline (make it reversible)

When you finish attribution:

- Record the **commit hash**, **command**, **out-dir**, and **worst bundle** in the perf log.
- If you introduce a new probe/script, add a baseline and wire it into a gate before moving on.
