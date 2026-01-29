# UI Gallery Performance: Scroll Measurement Hotspot (Native) — Tracker

Status: Draft (workstream note; ADRs remain the source of truth)

This document tracks a focused performance investigation for `apps/fret-ui-gallery` where worst frames are dominated
by layout engine solve time spent inside `Scroll` measurement (`measure_scroll`).

This is intentionally a **tracker** and **experiment log**, not a replacement for ADRs.

## 0) Context

Symptom:

- `fret-ui-gallery` feels stuttery in stress cases (notably virtualized list torture and other large scrollable
  surfaces).

Observed root cause (so far):

- worst frames are typically CPU-bound by layout, with `layout_engine_solve_time_us` dominated by `measure_time_us`.
- the hottest measure is usually `Scroll` (probe on scroll axis), and the time is mostly attributed to a single child
  subtree measured under `AvailableSpace::MaxContent`.

Primary code anchors:

- Scroll measurement: `crates/fret-ui/src/declarative/host_widget/measure.rs` (`measure_scroll`)
- Scroll probe helper: `crates/fret-ui/src/declarative/host_widget/measure.rs` (`max_non_absolute_children`)
- Scroll contract knob: `crates/fret-ui/src/element.rs` (`ScrollProps::probe_unbounded`)
- UI Gallery shell content scroll: `apps/fret-ui-gallery/src/ui.rs` (`content_view`)

Primary tool anchors:

- diagnostics/perf harness: `apps/fretboard` (`fretboard diag perf`, `fretboard diag stats`)
- scripted repros: `tools/diag-scripts/ui-gallery-virtual-list-torture.json`

## 1) Repro (deterministic)

Baseline workflow:

```powershell
cargo run -p fretboard -- diag perf tools/diag-scripts/ui-gallery-virtual-list-torture.json `
  --sort time --top 1 --json `
  --launch -- cargo run -p fret-ui-gallery --release
```

To isolate initial-mount costs for the target page, start directly on it:

```powershell
cargo run -p fretboard -- diag perf tools/diag-scripts/ui-gallery-virtual-list-torture.json `
  --env FRET_UI_GALLERY_START_PAGE=virtual_list_torture `
  --warmup-frames 0 --sort time --top 1 --json `
  --launch -- cargo run -p fret-ui-gallery --release
```

Then inspect the slow bundle:

```powershell
cargo run -p fretboard -- diag stats <bundle.json> --sort time --top 1 --json
```

## 2) Observability (what to turn on)

### 2.1 Bundle-based triage (default)

Use `fretboard diag perf` + `diag stats` to:

- locate the worst frames deterministically,
- see `debug.layout_engine_solves[].top_measures[]` (including nested `top_children` attribution),
- correlate hotspots to `SemanticsProps.test_id` when available.

### 2.2 Tracy timeline (when you need “who called this?”)

```powershell
$env:FRET_TRACY=1
$env:RUST_LOG="info,fret_ui=trace"
cargo run -p fret-ui-gallery --release --features fret-bootstrap/tracy
```

Then correlate `frame_id`/`tick_id` from the bundle to Tracy’s `fret.frame` spans.

### 2.3 Layout solved tree dumps (when bounds are wrong)

```powershell
$env:FRET_TAFFY_DUMP=1
$env:FRET_TAFFY_DUMP_DIR=".fret\\taffy-dumps"
$env:FRET_TAFFY_DUMP_MAX=30
```

Optionally filter by a root label/test id once the demo surface is tagged.

## 3) Current instrumentation (landed in this branch)

Goal: make the perf report “name the hot region” without requiring manual node-id archaeology.

### 3.1 Tag the UI Gallery content viewport

When the selected page is `virtual_list_torture`, the gallery now tags the content scroll viewport with:

- `ui-gallery-content-viewport-virtual_list_torture`

This is surfaced in `diag stats` nested measurement attribution as the hottest child subtree for the hot `Scroll`.

Code anchors:

- `apps/fret-ui-gallery/src/ui.rs` (conditional `viewport_test_id(...)`)
- `ecosystem/fret-ui-shadcn/src/scroll_area.rs` (`ScrollArea::viewport_test_id`)

### 3.2 Add a Scroll “intrinsic measurement” mode

Scroll measurement can now be configured to treat the scroll container as a viewport-sized barrier during intrinsic
measurement passes:

- `ScrollIntrinsicMeasureMode::Viewport`

This is intended for large “virtual surfaces” where measuring the full scrollable subtree under Min/MaxContent is both
expensive and semantically misleading (extent should come from metrics, not recursive subtree measurement).

Code anchors:

- `crates/fret-ui/src/element.rs` (`ScrollIntrinsicMeasureMode`, `ScrollProps::intrinsic_measure_mode`)
- `ecosystem/fret-ui-shadcn/src/scroll_area.rs` (`ScrollArea::viewport_intrinsic_measure_mode`)
- `apps/fret-ui-gallery/src/ui.rs` (enabled only for `virtual_list_torture`)

### 3.3 Add a trace span for `measure_scroll`

`measure_scroll` now emits a `fret_ui.measure_scroll` trace span with:

- node id, axis, probe knob, child_count, and available/known constraints

It also logs whether the intra-frame scroll probe cache hit and the probe duration when it misses.

Code anchors:

- `crates/fret-ui/src/declarative/host_widget/measure.rs` (`measure_scroll`)

## 4) Hypotheses (what we think is actually happening)

H1) The `Scroll` probe’s `AvailableSpace::MaxContent` contract is correct for “true unbounded content extent”, but it is
the wrong default for virtualized content where extent should come from a metrics model (virtualizer), not recursive
measurement of a large subtree.

H2) The hottest child subtree under `Scroll` is effectively a “container for a large surface” (VirtualList/table/code
view), and we should treat it as a barrier with an explicit extent contract rather than letting `Scroll` determine
extent by measuring the full subtree.

H3) Even when virtualization is used, we may still be forcing expensive measurement due to:

- scroll axis probing happening in measure (not only in final layout),
- missing/unstable cache keys (cache hits stay low),
- composing VirtualList under additional wrappers that accidentally trigger MaxContent measurement of an oversized tree.

## 5) Experiment matrix (toggle-driven)

### 5.1 UI Gallery toggles relevant to VirtualList torture

These env flags already exist in `apps/fret-ui-gallery/src/ui.rs`:

- `FRET_UI_GALLERY_VLIST_MINIMAL`
- `FRET_UI_GALLERY_VLIST_KNOWN_HEIGHTS`
- `FRET_UI_GALLERY_VLIST_RETAINED`
- `FRET_UI_GALLERY_VLIST_ROW_CACHE`

Use them to isolate whether the hot subtree is:

- row composition cost,
- row caching effectiveness,
- retained host overhead,
- or scroll/measure contract (extent probing).

### 5.2 View cache toggles

The gallery exposes:

- `FRET_UI_GALLERY_VIEW_CACHE`
- `FRET_UI_GALLERY_VIEW_CACHE_SHELL`
- `FRET_UI_GALLERY_VIEW_CACHE_INNER`

Use `fretboard diag perf ... --env ...` to build cached/uncached comparisons.

## 6) Plan (detailed, staged)

### Stage A — Identify the hot Scroll subtree precisely (Done)

- [x] Make `diag stats` report name the hot subtree via stable `test_id`.
- [x] Add `measure_scroll` tracing span to split cache hit vs miss + probe time.

Exit criteria:

- worst frame’s top `Scroll` has a nested child attribution with a stable `test_id` (not only `NodeId(...)`).

### Stage B — Decide the contract change (Proposed)

Candidate approaches:

1) Policy-side: for VirtualList-heavy pages, set `ScrollProps.probe_unbounded=false` (demo/ecosystem layer only).
   - Pros: cheap to try, no core contract change.
   - Cons: may change behavior for long unbreakable tokens and “true max content” cases; risk of masking deeper issues.

2) Mechanism-side: introduce an explicit “virtualized extent provider” contract so `Scroll` does not need to measure the
   full child subtree to compute max offset.
   - Pros: aligns with ADR virtualization intent; scalable.
   - Cons: needs careful boundary design to avoid policy leaks into `crates/fret-ui`.

Decision gate:

- If we can prove that the hot `Scroll` is a viewport around a virtual surface, prefer (2).
- If the hot `Scroll` is around text/code with long tokens, (1) may be required (but must be localized).

### Stage C — Implement the smallest behavior change that reduces the hot path (Planned)

Shortlist:

- Ensure VirtualList/table/code-view surfaces avoid MaxContent subtree measurement for extent (extent from metrics).
- Raise measure cache hit rate (stable cache keys; avoid clearing caches per frame unless required).
- Add a regression: scripted perf threshold on selected script(s) (opt-in / nightly).

Exit criteria:

- On `ui-gallery-virtual-list-torture`, worst `layout_engine_solve_time_us` drops meaningfully and remains stable across
  `--repeat N`.

## 7) References

- Debugging playbook: `docs/debugging-playbook.md`
- Tracy workflow: `docs/tracy.md`
- Layout engine roadmap: `docs/layout-engine-refactor-roadmap.md`
- Observability contract: `docs/adr/0036-observability-tracing-and-ui-inspector-hooks.md`
- AvailableSpace + measurement rules: `docs/adr/0115-available-space-and-non-reentrant-measurement.md`
- Virtualization boundary: `docs/adr/0042-virtualization-and-large-lists.md`
