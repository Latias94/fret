---
title: UI Performance (Windows RTX 4090) - Smoothness Closure v1
status: draft
date: 2026-02-12
scope: performance, regression-gates, tail-latency, windows
---

# UI Performance (Windows RTX 4090) - Smoothness Closure (v1)

This workstream documents the **Windows** performance posture for Fret on a high-end NVIDIA GPU profile
(`windows-rtx4090`) and turns findings into **repeatable gates** and **landable TODOs**.

North star: **Zed/GPUI-level smoothness**, i.e. low tail latency under resize/scroll/pointer-move and predictable
steady-state interactions.

Related:

- Zed smoothness global plan: `docs/workstreams/ui-perf-zed-smoothness-v1.md`
- GPUI gap analysis: `docs/workstreams/ui-perf-gpui-gap-v1.md`
- Perf baselines: `docs/workstreams/perf-baselines/`
- TODO tracker: `docs/workstreams/ui-perf-windows-rtx4090-smoothness-v1-todo.md`
- Milestones: `docs/workstreams/ui-perf-windows-rtx4090-smoothness-v1-milestones.md`

---

## 0) Machine profile (baseline identity)

This workstream assumes a stable machine tag of `windows-rtx4090`.

Record (example from the initial run on 2026-02-12):

- CPU: Intel i9-13900KF (24C/32T)
- GPU: NVIDIA GeForce RTX 4090 (wgpu backend: Vulkan)
- NVIDIA driver: 591.86

If the machine profile changes (monitor refresh, scaling, driver version, BIOS settings, power limits),
refresh baselines rather than “chasing noise”.

---

## 1) Contract: what counts as “good enough”

We consider Windows perf “good enough to pause” when:

1) `ui-gallery-steady` passes its committed baseline on this machine profile, and
2) `ui-resize-probes` is stable under attempts=3 (strict majority pass), and
3) the worst bundles are explainable (one hitch class at a time), and
4) changes are reversible (commit-addressable evidence + worst bundles kept).

---

## 2) Canonical commands (PowerShell)

Prebuild (avoid compilation inside `diag perf` timeouts):

```powershell
cargo build -p fretboard -p fret-ui-gallery --release
```

Run the steady-state suite with the committed baseline:

```powershell
$tag = Get-Date -Format 'yyyyMMdd-HHmmss'
$outDir = "target/fret-diag-perf/ui-gallery-steady.windows-rtx4090.$tag"
New-Item -ItemType Directory -Force -Path $outDir | Out-Null

target/release/fretboard.exe diag perf ui-gallery-steady `
  --dir $outDir --timeout-ms 300000 `
  --reuse-launch --repeat 7 --warmup-frames 5 --sort time --top 15 --json `
  --perf-baseline docs/workstreams/perf-baselines/ui-gallery-steady.windows-rtx4090.v1.json `
  --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 `
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 `
  --launch -- target/release/fret-ui-gallery.exe `
  | Tee-Object -FilePath (Join-Path $outDir 'stdout.txt')
```

Run the P0 resize probes with the committed baseline:

```powershell
$tag = Get-Date -Format 'yyyyMMdd-HHmmss'
$outDir = "target/fret-diag-perf/ui-resize-probes.windows-rtx4090.$tag"
New-Item -ItemType Directory -Force -Path $outDir | Out-Null

target/release/fretboard.exe diag perf ui-resize-probes `
  --dir $outDir --timeout-ms 300000 `
  --reuse-launch --repeat 7 --warmup-frames 5 --sort time --top 15 --json `
  --perf-baseline docs/workstreams/perf-baselines/ui-resize-probes.windows-rtx4090.v1.json `
  --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 `
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 `
  --launch -- target/release/fret-ui-gallery.exe `
  | Tee-Object -FilePath (Join-Path $outDir 'stdout.txt')
```

When a gate fails, start with:

- `$outDir/check.perf_thresholds.json`
- worst bundle path in the perf JSON (`worst_overall.bundle`)
- `target/release/fretboard.exe diag stats <bundle.json> --sort time --top 30 --json`

---

## 3) Initial results (2026-02-12 snapshot)

This section is a **point-in-time** summary meant to anchor investigation.

Steady suite (`ui-gallery-steady`) against `ui-gallery-steady.windows-rtx4090.v1.json` failed with many threshold
violations (tail spikes). Evidence:

- Out dir: `target/fret-diag-perf/ui-gallery-steady.windows-rtx4090.20260212-131744`
- Threshold report: `target/fret-diag-perf/ui-gallery-steady.windows-rtx4090.20260212-131744/check.perf_thresholds.json`

Resize probes (`ui-resize-probes`) against `ui-resize-probes.windows-rtx4090.v1.json` failed (tail spikes). Evidence:

- Out dir: `target/fret-diag-perf/ui-resize-probes.windows-rtx4090.20260212-132727`
- Threshold report: `target/fret-diag-perf/ui-resize-probes.windows-rtx4090.20260212-132727/check.perf_thresholds.json`

Editor resize probes (`ui-code-editor-resize-probes`) passed its committed baseline on this machine profile:

- Out dir: `target/fret-diag-perf/ui-code-editor-resize-probes.windows-rtx4090.20260212-132923`

---

## 4) Hitch class notes (what the worst bundles suggest)

### 4.1 Resize stress worst frame is layout-dominated (CPU tail)

Worst overall bundle in the steady suite snapshot:

- `target/fret-diag-perf/ui-gallery-steady.windows-rtx4090.20260212-131744/1770873645814-ui-gallery-window-resize-stress-steady/bundle.json`

`diag stats` indicates a clear CPU-side tail, dominated by layout:

- max frame: `total_time_us ~= 106.8ms`
- max frame: `layout_time_us ~= 84.1ms`, `paint_time_us ~= 37.1ms`
- within layout (top frame breakdown):
  - `layout_request_build_roots_time_us ~= 29.5ms`
  - `layout_engine_solve_time_us ~= 8.1ms`

This aligns with the “GPUI gap” direction: reduce hashing-heavy request/build overhead, adopt dense tables,
and improve per-frame allocation discipline to avoid rare stalls.

### 4.2 Drag-jitter worst frame: large layout time, moderate request/build

Worst bundle in the resize probe snapshot:

- `target/fret-diag-perf/ui-resize-probes.windows-rtx4090.20260212-132727/1770874129552-ui-gallery-window-resize-drag-jitter-steady/bundle.json`

Top frame breakdown (indicative):

- `total_time_us ~= 78.1ms`
- `layout_time_us ~= 70.5ms`
- `layout_request_build_roots_time_us ~= 6.0ms`
- `layout_engine_solve_time_us ~= 4.9ms`

Interpretation: for drag-jitter, the “build roots” phase is not the only lever; we likely need broader
layout invalidation + caching/quantization strategies to keep per-frame work bounded.

---

## 5) Workstream strategy (lowest-risk first)

1) **Repro hygiene**: use a clean worktree / clean checkout when measuring baselines.
2) **Gate-first**: every change must preserve:
   - `ui-gallery-steady` (baseline pass) and
   - `ui-resize-probes` (attempts=3 majority pass) and
   - `ui-code-editor-resize-probes` (no regressions).
3) **One hitch class at a time**: pick the worst script, attribute with `diag stats`, make one change, rerun gates.
4) **Record evidence**: append results to a commit-addressable perf log (either the global log or a Windows section).

---

## 6) Proposed milestone outline

See:

- TODO tracker: `docs/workstreams/ui-perf-windows-rtx4090-smoothness-v1-todo.md`
- Milestones: `docs/workstreams/ui-perf-windows-rtx4090-smoothness-v1-milestones.md`

