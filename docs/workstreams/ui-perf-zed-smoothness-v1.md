# UI Performance: Zed-level Smoothness (v1) — Workstream Plan

Status: Draft (workstream note; ADRs remain the source of truth)

This workstream focuses on closing the **performance smoothness** gap between Fret and the Zed/GPUI reference
substrate. The intended output is not “more widgets”, but a runtime that stays responsive and predictable under
editor-class workloads:

- large, scroll-heavy surfaces (lists/tables/logs/code views),
- hover/pointer-move heavy UIs (toolbars, inspectors, dense chrome),
- multi-root overlays (menus, dialogs, toasts, drag layers),
- window resize stress,
- 2D pan/zoom surfaces (canvas, node graphs, plots),
- multi-window/docking workflows.

This plan is deliberately **fearless**: it assumes we can refactor core data structures and hot paths as needed, as
long as we preserve the cross-crate contracts (or record contract changes via ADRs).

Related workstreams / anchors:

- GPUI parity (experience + performance): `docs/workstreams/gpui-parity-refactor.md`
- UI Gallery perf investigation (scroll/measure): `docs/workstreams/ui-gallery-perf-scroll-measure.md`
- Diagnostics + perf gates: `apps/fretboard` (`fretboard diag perf`, `fretboard diag repro`, `fretboard diag stats`)

Tracking:

- TODO tracker: `docs/workstreams/ui-perf-zed-smoothness-v1-todo.md`
- Perf log (commit-addressable results): `docs/workstreams/ui-perf-zed-smoothness-v1-log.md`

---

## 0) What “smoothness” means (so we can measure it)

We explicitly distinguish:

1) **Frame time** (CPU) stability: low p95/p99, few outliers, tight distribution.
2) **Redraw efficiency**: idle frames should not paint; paint-only chrome should not relayout/rerender.
3) **Responsiveness**: interaction latency (click/drag/scroll) should not block for 100ms+.
4) **Throughput**: large surfaces can scroll/pan at 60Hz/120Hz without “progressively getting worse”.

### 0.1 Target budgets (initial)

Budgets are expressed in the same terms as `fretboard diag perf` output:
`top.us(total/layout/solve/prepaint/paint)`.

These are *targets* to guide refactors; they must be calibrated per machine and per page. We keep two tiers:

- **Tier A (60Hz baseline)**: `p95 total <= 8ms`, `max total <= 16ms` for representative scripts.
- **Tier B (120Hz “Zed feel”)**: `p95 total <= 4ms`, `max total <= 8ms` for representative scripts on a high-end machine.

Notes:

- GPU can still be the limiting factor. These CPU budgets are necessary but not sufficient for end-to-end 120Hz.
- “Total” is the runtime’s internal CPU time (layout/prepaint/paint), not the OS present time.

### 0.2 Acceptance scripts (baseline suite)

We treat a small set of scripts as “editor-relevant perf probes”. The suite should cover:

- overlay + modal churn:
  - `tools/diag-scripts/ui-gallery-overlay-torture.json`
  - `tools/diag-scripts/ui-gallery-dropdown-open-select.json`
  - `tools/diag-scripts/ui-gallery-dialog-escape-focus-restore.json`
- scrolling + virtualization:
  - `tools/diag-scripts/ui-gallery-virtual-list-torture.json`
  - `tools/diag-scripts/ui-gallery-code-view-scroll-refresh-baseline.json`
- resize + layout stability:
  - `tools/diag-scripts/ui-gallery-window-resize-stress.json`
- 2D pan/zoom:
  - `tools/diag-scripts/ui-gallery-canvas-cull-torture-pan-zoom.json`
  - `tools/diag-scripts/ui-gallery-chart-torture-pan-zoom.json`

The suite is intentionally *small*; additional pages/scripts should exist, but these are the “must not regress” set.

---

## 1) Measurement protocol (to avoid noise)

### 1.1 Canonical commands

We track **two** perf gates:

1) **Cold-start gate**: measures “first mount + first interaction” (relaunch per run).
2) **Steady-state gate**: measures interaction costs after mount (reuse process + reset diagnostics in-script).

To keep runs reproducible and to avoid stale-trigger timeouts, prefer passing an explicit `--dir` for every run.
Treat the directory as part of the benchmark identity (it also makes bundle paths stable per run).

To turn “perf regressions” into a contract, generate a baseline file once per machine profile and keep it committed
under `docs/workstreams/perf-baselines/`. Then run future perf probes with `--perf-baseline` against that file.

Example (steady-state suite baseline):

```powershell
cargo run -p fretboard -- diag perf ui-gallery-steady ^
  --dir target/fret-diag-perf/ui-gallery-steady.<machine-tag> ^
  --reuse-launch --repeat 7 --sort time --top 15 --json ^
  --perf-baseline-out docs/workstreams/perf-baselines/ui-gallery-steady.<machine-tag>.v1.json ^
  --perf-baseline-headroom-pct 20 ^
  --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 ^
  --launch -- cargo run -p fret-ui-gallery --release
```

Run a single script and get the worst frame (**cold-start gate**):

```powershell
cargo run -p fretboard -- diag perf tools/diag-scripts/<script>.json ^
  --dir target/fret-diag-perf/<tag> ^
  --warmup-frames 5 --repeat 7 --sort time --top 15 --json ^
  --launch -- cargo run -p fret-ui-gallery --release
```

Run the steady-state suite (**steady-state gate**):

```powershell
cargo run -p fretboard -- diag perf ui-gallery-steady ^
  --dir target/fret-diag-perf/ui-gallery-steady ^
  --reuse-launch --repeat 7 --sort time --top 15 --json ^
  --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 ^
  --launch -- cargo run -p fret-ui-gallery --release
```

Notes:

- `ui-gallery-steady` scripts call `reset_diagnostics` just before the measured interaction(s).
  Prefer `--warmup-frames 0` (default) so the “first post-reset frames” are included.
- `--reuse-launch` keeps the launched demo alive across repeats/scripts so caches are actually warm.
- If you already have a running demo (or cannot use `--launch`), you can run the suite against it:

```powershell
cargo run -p fretboard -- diag perf ui-gallery-steady --dir target/fret-diag ^
  --repeat 7 --sort time --top 15 --json
```

Extract root cause from the worst bundle:

```powershell
cargo run -p fretboard -- diag stats <bundle.json> --sort time --top 30
```

View-cache on/off comparison should be routine:

```powershell
set FRET_UI_GALLERY_VIEW_CACHE=1
set FRET_UI_GALLERY_VIEW_CACHE_SHELL=1
```

### 1.2 What we treat as a regression

Regressions are assessed at the suite level, not from a single script. A regression is:

- any script’s `max top.total_us` exceeds its gate,
- or `p95` regresses by more than an agreed headroom percent,
- or redraw efficiency gates fail (idle paints / cache reuse stability / hitch logs).

Prefer `--perf-baseline` based gating for script-specific stability, rather than a single global threshold.

### 1.3 Current baseline + initial findings (2026-02-02)

Baseline suite results are recorded in:

- `docs/workstreams/ui-perf-zed-smoothness-v1-log.md`

Key observations from the current baseline run (repeat=7, `--launch`):

1) **`ui-gallery-window-resize-stress` is dominated by text measurement in the layout engine**

- Worst frame shows `layout.solve_us` dominating total CPU time.
- The top layout-engine solve is almost entirely `measure_us`, and a single `Text` node dominates the measure time
  (large label string; semantics label was redacted at len=645 in the bundle).
- Evidence bundle is referenced in the log under “Worst overall” for the baseline entry.

Implication:

- For resize-heavy workloads, we should treat **text measure/shaping** as a first-class perf gate and stabilize caches.
- Short-term wins likely come from improving cache keys / reusing computed metrics under intrinsic probes.
- Longer-term wins likely require splitting “shaping” from “wrapping/line breaking” so width changes do not force
  full re-shape work.

Update (2026-02-02):

- `perf(fret-render): fast-path wrapped text measure` (commit `9440648a`) implements “shape once, wrap by cluster stats”
  for `measure()` under `TextWrap::Word | TextWrap::Grapheme` when `max_width` is set.
- On `ui-gallery-window-resize-stress`, p95 total drops from ~30.9ms to ~15.5ms, and p95 `layout.solve_us` drops from
  ~17.6ms to ~1.7ms. The remaining dominant cost is now `paint_us` (see the latest log entry for exact numbers).

2) **Some scripts currently look “cold-start dominated” due to per-script process launches**

Scripts like `ui-gallery-dropdown-open-select` and `ui-gallery-dialog-escape-focus-restore` show their worst frames
at `tick=0 frame=0` in their bundles, which suggests we are measuring “startup + first mount” cost more than
steady-state interaction cost.

This is not necessarily wrong (cold-start matters), but we should explicitly decide:

- keep `--launch` as a “cold-start gate”, and add a second “steady-state gate” suite, or
- update `fretboard diag perf` to optionally reuse a single launched process across multiple scripts/steps,
  so interaction scripts measure the intended steady-state paths.

Next action:

- record the decision in this workstream and treat it as part of the performance contract (M0).

---

## 2) Hot path model (where the time goes)

Fret’s end-to-end frame pipeline is roughly:

1) input dispatch / effects draining
2) model/global change propagation (observation → invalidation)
3) declarative element render + mount reconciliation
4) layout engine solves + widget `measure()` walks
5) prepaint (derived interaction caches, windowed surfaces, hit-test caches)
6) paint (scene encoding, paint-cache replay, view-cache reuse, platform snapshots)
7) renderer (resource prep + GPU encoding)

For “Zed feel”, we want:

- pointer move / hover: typically **paint-only**, not layout; invalidation should be localized.
- wheel scroll: **no rerender** if the view can reuse ranges; visible windows update via prepaint when needed.
- resize: large layout changes should be amortized; avoid triggering deep `measure(MaxContent)` cascades.

---

## 3) Primary refactor themes (fearless but bounded)

This section lists “where we will be willing to rewrite structures”, and why.

### 3.1 Data structures + allocation model (largest leverage)

Goal: eliminate avoidable per-frame hashing and heap churn in the runtime substrate.

Candidate changes (likely needed):

- Replace `HashMap<NodeId, ...>` and `HashMap<GlobalElementId, ...>` hot stores with
  `slotmap::SecondaryMap` or dense `Vec`-backed tables where possible.
  - Primary target: `crates/fret-ui/src/declarative/frame.rs` (`WindowFrame.instances`, `WindowFrame.children`).
- Replace “clear and reallocate” scratch `HashMap`s with generation-stamped tables.
  - Primary target: invalidation `visited` and propagation caches in `crates/fret-ui/src/tree/mod.rs`.
- Introduce a per-frame **arena** for short-lived allocations (GPUI style).
  - Reference: `repo-ref/zed/crates/gpui/src/arena.rs`.
  - Candidates: mount scratch, GC reachability sets, semantics traversal stacks, path caches, temp vectors.

Success criteria:

- `diag perf` p95 improves under view-cache reuse workloads.
- memory allocations per frame reduce materially (instrumentation TBD).

### 3.2 Observation + invalidation discipline (keep recomputation local)

Goal: model/global changes should invalidate **minimum necessary** nodes, and reuse should win by default.

Candidate changes:

- Promote “paint-only chrome” patterns in ecosystem so hover/focus/pressed rarely triggers rerender.
- Improve observation uplift to cache roots and reduce redundant invalidation walks.
- Add targeted diagnostics to detect “layout observations on large subtrees” as lint-like warnings in bundles.

### 3.3 Hit testing: spatial index (pointer move must be cheap)

Goal: avoid full-tree hit tests on pointer move when UI scale grows (tens of thousands of nodes).

Candidate change:

- Implement a bounds tree (R-tree variant) per layer root during prepaint and use it for hit testing / ordering.
  - Reference: `repo-ref/zed/crates/gpui/src/bounds_tree.rs`.

Success criteria:

- “hover torture” scripts show stable, low invalidation counts and low CPU time even as node count grows.

### 3.4 Layout: avoid unbounded measurement cascades

Goal: large scroll surfaces should not trigger expensive `MaxContent` measurement walks in the steady state.

Candidate changes:

- Interim: debounce `probe_unbounded` scroll extent recomputation on viewport resize (avoid “live resize”
  `MaxContent` cascades; recompute once the viewport stabilizes).
- Adopt DOM/GPUI-like extent models for scroll (tracked separately), so “accurate extents” come from post-layout
  geometry rather than unbounded probes.
- Move window derivation for virtual surfaces into prepaint (ADR 0190 alignment) to avoid cache-root rerender on scroll.

### 3.5 Paint/scene replay: reduce CPU rewrite work

Goal: reuse should be cheap; replay should not require rewriting thousands of ops per frame.

Candidate change:

- Evaluate transform-wrapping replay (push a transform, replay ops unchanged) vs per-op translation.
  - Must be validated against renderer batching and stack semantics.

### 3.6 Text pipeline: stable caches for editor workloads

Goal: code/text views should scroll with stable cost; incremental edits should not cause a full re-shape of the world.

Candidate changes:

- lock a stable measurement/shaping cache keying strategy for:
  - font stack + size + style,
  - wrap width / constraints,
  - shaping options.
- ensure text atlas behavior has clear budgets and eviction is visible in perf snapshots.

---

## 4) Milestones (v1)

Milestones are defined to be *testable*. Each milestone must end with:

- a reproducible suite run,
- a baseline update or a perf gate update,
- a short write-up of what changed and why.

### M0: Baseline + perf gates (suite becomes a contract)

Deliverables:

- A named perf suite (documented; optionally a `fretboard` builtin later).
- A baseline JSON committed (or a stable generation recipe).
- Initial thresholds for Tier A and Tier B.

### M1: Frame-data structure refactor (hashing → dense)

Deliverables:

- Replace hottest `HashMap` stores in the per-frame pipeline with dense stores.
- Reduce per-frame allocations by reuse/generation patterns.

### M2: Allocation model: introduce per-frame arena for scratch

Deliverables:

- Arena-backed scratch for at least 2 hot paths (mount + semantics/GC).
- Evidence of reduced allocations and improved tail latency.

### M3: Hit-test bounds tree (spatial index)

Deliverables:

- Bounds tree built during prepaint (per layer root).
- Hit testing uses the index for pointer move/down routing.

### M4: Windowed surfaces (prepaint-driven visible windows)

Deliverables:

- At least one “editor-class” surface fully migrated:
  - VirtualList window changes, or
  - Code view visible lines window.
- Proof: scroll scripts no longer cause cache-root rerender in steady scroll.

### M5: Text pipeline stabilization (editor-ready)

Deliverables:

- A stable text cache strategy documented and instrumented.
- Scroll/edit scripts stay within Tier A budget; Tier B progress tracked.

### M6: CI-ready perf regression gates (optional but recommended)

Deliverables:

- A minimal CI job candidate that runs a reduced suite and fails on regressions.
- A mechanism to store/compare baselines per platform.

---

## 5) Future extensions (design for the long run)

- **Multi-window / docking**: perf suite should include tear-off and cross-window drag as it stabilizes.
- **Multiple viewports**: culling and surface embedding must remain cheap under pointer move and resizing.
- **WebGPU / wasm**: avoid designs that depend on OS threads or per-frame large allocations; favor reusable buffers.
- **GPU profiling**: extend `bundle.json` schema with optional GPU timings (behind feature flags) once CPU is stable.
