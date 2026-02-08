---
title: UI Performance: Resize Path (Fret vs GPUI) v1
status: draft
date: 2026-02-09
scope: performance, resize, layout, gpui-gap
---

# UI Performance: Resize Path (Fret vs GPUI) v1

This note exists to stop “resize feels janky” work from turning into untracked experiments.

Goals:

1. Explain what work Fret performs during interactive resize (what is paid per frame).
2. Cross-check the mechanism against GPUI/Zed (what is transferable vs not).
3. Turn the main cost centers into measurable, gated milestones.

Related:

- Workstream: `docs/workstreams/ui-perf-zed-smoothness-v1.md`
- TODO: `docs/workstreams/ui-perf-zed-smoothness-v1-todo.md`
- Perf log (commit-addressable evidence): `docs/workstreams/ui-perf-zed-smoothness-v1-log.md`
- GPUI gap map: `docs/workstreams/ui-perf-gpui-gap-v1.md`

## What happens on resize in Fret (today)

### 1) Runner coalescing (desktop)

Fret coalesces OS resize events and applies a pending surface resize at redraw time (once per frame).

This avoids “N resizes per frame”, but does not eliminate per-frame layout/paint work while the window size is
changing.

Reference:
- `crates/fret-launch/src/runner/desktop/app_handler.rs` (`pending_surface_resize` applied on `RedrawRequested`)

### 2) Layout engine solves are multi-root (window root + viewport roots)

In steady resize probes we observe:

- `top_layout_engine_solves_max` is typically `~4` for the resize drag-jitter probe.

This is expected given Fret’s “explicit layout barrier” design:

- the **window root** is solved once, and
- additional **viewport roots** (scroll/virtualization/docking/contained cache roots) are solved separately with
  their own bounds.

This is not necessarily worse than GPUI; it is a mechanism trade-off. The key is to keep the total work bounded:

- reduce the number of viewport roots solved on a typical editor page, and/or
- make each viewport solve cheaper (especially text measure/shaping).

## Why resize can still hitch even with coalescing

Interactive resize is a *stress multiplier*:

- Bounds change forces layout constraint recomputation.
- Many widgets request viewport roots (scroll content, lists, docking viewports), which increases solve count.
- Text wrap widths churn under small-step jitter, which amplifies measure/shaping costs.

To feel “Zed smooth”, we need:

1) stable per-frame work (low tail variance), and
2) a strict budget for *layout request/build* + *solve* + *paint* under resize.

## Observed hotspots (from recent resize probe bundles)

From the resize probe runs recorded in the perf log (see the entries around `2026-02-08`):

- `top_layout_request_build_roots_time_us` can be `~2.4ms` in worst frames for the drag-jitter probe.
  - This is “flow subtree request/build” overhead (walking nodes, setting styles/children, stable identity).
- `top_layout_engine_solve_time_us` can be `~2.2ms` (worst frames) for the drag-jitter probe.
  - This includes `TextService::measure` costs for wrapped text.

The next step is to reduce the sum of:

- request/build overhead + solve overhead (especially tail outliers),
- without regressing `ui-gallery-steady`.

## GPUI/Zed resize notes (transferable vs not)

GPUI is a strong reference for “Zed feel”, but it is not a complete template for Fret:

Transferable:

- strict “once per frame” work shaping + aggressive reuse contracts,
- allocation discipline (per-frame scratch arenas, bounded caches),
- text layout caching that amortizes width jitter (visible window aware).

Less transferable 1:1:

- Fret’s heavier mechanism set (docking + view cache + multiple viewports) means we may need more explicit barrier
  policies than GPUI.
- Renderer/effects architecture should also be cross-checked against engines like Flutter/Skia for pooled
  intermediates and GPU upload churn.

Local GPUI references:

- `repo-ref/zed/crates/gpui/src/platform/mac/window.rs`
- `repo-ref/zed/crates/gpui/src/platform/linux/wayland/window.rs` (interactive resize throttling)

## Milestone candidates (make the path measurable)

These are deliberately phrased as “mechanism-first” items that can be gated.

### M4.4.a Resize layout request/build budget

Goal:
- Reduce `top_layout_request_build_roots_time_us` tails during resize probes.

Probe:
- `ui-resize-probes` (drag-jitter + stress)

Likely levers:
- reduce per-frame tree-walk overhead (dense data structures, fewer HashMap lookups),
- avoid work that is invariant under resize (only constraints changed, not styles/structure),
- tighten viewport root batching and avoid redundant work in `flush_viewport_roots_after_root`.

### M4.4.b Viewport roots: reduce solve count or solve cost

Goal:
- Reduce typical `top_layout_engine_solves` and/or reduce `top_layout_engine_solve_time_us` for each solve.

Probe:
- `ui-resize-probes` + editor resize jitter suite.

Likely levers:
- reduce the number of simultaneously active viewport roots on common pages,
- ensure translation-only and cache-hit paths are common under jitter-class resizes,
- continue improving text measure/shaping reuse under width churn.

### M4.4.c Interactive resize policy ADR

Goal:
- Document what is allowed to be bucketed/deferred during live resize (and what must remain exact).

Deliverable:
- ADR update/new ADR + alignment entry.

This prevents “fearless refactors” from silently breaking UX expectations.

