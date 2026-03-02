---
title: Scroll Extents (DOM/GPUI Parity)
status: in progress
date: 2026-03-02
scope: fret-ui, scroll, layout, perf
---

## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- Zed: https://github.com/zed-industries/zed

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.

# Scroll Extents (DOM/GPUI Parity)

This workstream proposes a more scalable scroll extent strategy for `fret-ui` that avoids deep
`measure()` probes for large scrollable surfaces (notably UI Gallery page content, markdown/code
views, and other editor-grade panels).

For the current UI Gallery perf investigation context, see:

- `docs/workstreams/ui-gallery-perf-scroll-measure.md`

## Problem Statement

Today, `ScrollProps::probe_unbounded = true` drives a MaxContent-style probe on the scroll axis.
This often forces a deep subtree `measure()` walk to determine the scrollable content extent.

In debug/dev builds, this can cause noticeable stalls on page switches (e.g. UI Gallery nav click)
because the frame is blocked inside recursive measurement rather than only doing a single final
layout pass.

Short-term mitigation (experimental; evidence tracked in `docs/workstreams/ui-gallery-perf-scroll-measure.md`):

- defer the unbounded probe by one frame when the scroll content subtree is layout-invalidated, using last-frame
  `measured_size` as an estimate for the first post-click frame.

## Current Implementation (as of 2026-03-02)

This section is descriptive (not the target contract).

### Mechanism surfaces

- `ScrollProps::probe_unbounded` (default: `true`) controls whether the scroll content is measured
  using MaxContent available space on the scroll axis.
- `ScrollIntrinsicMeasureMode::Viewport` is an intrinsic-sizing-only escape hatch that avoids deep
  scroll subtree measurement during Min/MaxContent measurement passes, without changing final layout
  semantics.
- `ScrollHandle` stores `viewport_size`, `content_size`, and `offset`, and clamps `offset` based on
  `max_offset = max(content - viewport, 0)` (see `crates/fret-ui/src/scroll/mod.rs`).

Evidence anchors:

- Props/state: `crates/fret-ui/src/element.rs`
- Intrinsic measurement: `crates/fret-ui/src/declarative/host_widget/measure.rs`
- Layout + extent probing/caches: `crates/fret-ui/src/declarative/host_widget/layout/scrolling.rs`

### Layout algorithm (high-level)

In `layout_scroll_impl(...)` today:

1. Build child constraints:
   - along the scroll axis, use `AvailableSpace::MaxContent` when `probe_unbounded = true`;
   - otherwise use `AvailableSpace::Definite(viewport_axis_size)`.
2. Measure each child (`measure_in`) to compute a `max_child` size.
3. Compute `desired` (viewport) by clamping `max_child` to `ScrollProps.layout` and the available
   size.
4. Compute `content_size` from `max_child` (with scroll-axis rounding) and ensure it is at least
   the viewport size (DOM-like invariant).
5. During final layout passes, update the scroll handle:
   - `set_viewport_size_internal(desired)`
   - `set_content_size_internal(content_size)`
   - `set_offset_internal(prev_offset)` (re-clamps after size updates)
6. Layout children into `content_bounds = Rect(origin = cx.bounds.origin, size = content_size)`.

To mitigate stalls and correctness issues around caching/deferral, the implementation also:

- defers deep unbounded probes on resize or transient invalidation (runtime knobs in
  `crates/fret-ui/src/runtime_config.rs`);
- caches probe results within a frame and across frames;
- performs a post-layout “observed overflow” pass that can:
  - expand `content_size` when descendants overflow but the deep probe was deferred/cached, and
  - clamp `content_size` down after shrink in deferral flows.

## Target Contract (SE-100)

This is the normative contract for a DOM/GPUI-like scroll extent strategy. The goal is to define
what “scroll extents” mean in `fret-ui` independently of the current probing implementation.

### Definitions

- **Viewport rect**: the final scroll node bounds after applying `ScrollProps.layout` constraints.
- **Content space**: the coordinate space of child layout bounds prior to applying the runtime
  scroll render transform. Scroll offsets translate children in paint/hit-test, not in layout.
- **Content extent**: a size `(content_width, content_height)` such that:
  - it is derived from post-layout geometry (not from a pre-layout unbounded probe), and
  - it bounds the scrollable overflow region in content space.

### Coordinate spaces and transforms

1. Extents are computed from **layout bounds** (`UiTree::node_bounds`) in content space.
2. Extents must **not** depend on:
   - the current scroll offset, or
   - render-time transforms (e.g. visual transforms / effects), or
   - pixel-snapped paint geometry.

Rationale: scroll extents must be stable across frames and independent of transient paint-only
effects, matching DOM/GPUI expectations.

### Extent derivation (post-layout geometry)

After the final layout pass for the scroll subtree:

1. Compute `observed_extent` from post-layout bounds:
   - Consider the scroll content subtree rooted at the scroll node’s child roots.
   - Use the union of descendant **layout bounds** projected into content space to compute:
     - `observed_right = max(bounds.right - content_origin.x, 0)`
     - `observed_bottom = max(bounds.bottom - content_origin.y, 0)`
   - Then `observed_extent = Size(observed_right, observed_bottom)`.
2. Apply axis-specific rounding:
   - On the scroll axis, round **up** to the next whole pixel (`ceil`) to avoid under-reporting due
     to fractional layout rounding (DOM-like). Implementations should tolerate small floating point
     noise (e.g. subtract a tiny epsilon before `ceil`).
   - Cross axis uses the viewport size unless a dedicated cross-axis overflow mode is enabled.
3. Enforce invariants:
   - `content_size.width >= viewport_size.width`
   - `content_size.height >= viewport_size.height`
4. Update the scroll handle (final pass only):
   - set `viewport_size` and `content_size` using internal setters (do not bump revisions),
   - clamp the offset after updates.

### Chrome / padding / border policy

`fret-ui` scroll extents are defined in terms of **layout geometry** only. There is no implicit
padding/border contribution to `content_size` at the mechanism level.

If a component library wants visual padding to affect scroll extents (e.g. “scroll padding”), it
must do so explicitly by inserting a layout wrapper in the scroll content subtree.

### Negative origins policy

When projecting bounds into content space, negative origins must not make extents negative. Use
`max(..., 0)` for projected coordinates so the scrollable content box remains well-defined even if
some children are positioned above/left of the content origin.

### Interaction with overlays / anchoring

Scroll extent updates must not introduce additional layout passes.

The scroll content extent and `ScrollHandle` clamping must be derived from the same final layout
geometry that powers `bounds_for_element(...)` / overlay anchoring queries. This keeps overlay
placement stable and avoids “anchor uses old bounds while scroll uses new extents” mismatches.

#### SE-213 harness contract (scroll + overlay parity)

We treat “overlay anchoring parity” as a *cross-cutting* correctness requirement:

- When a scripted interaction causes scroll content height to change (e.g. switching a doc “Code”
  tab that expands a code block), the scroll extents update must be reflected in the same
  post-layout geometry source that overlay anchoring uses.
- When an anchored overlay is opened after such a change, it must solve against the updated anchor
  rect and remain clamped within the window.

Because this spans multiple systems (layout, scroll handle clamping, overlay solver), the minimal
verification path is currently a diagnostics harness (scripts + evidence queries), not unit tests.

Planned gate (SE-213):

- Script: a UI Gallery regression script that:
  1) opens an anchored overlay and captures `overlay_placement_trace`,
  2) expands a doc code tab (content growth) and proves scroll can still reach a lower section,
  3) re-opens the overlay and asserts `bounds_within_window` for the overlay panel.
- Tooling: `fretboard diag query overlay-placement-trace ...` prints/JSON-dumps the captured
  `script.result.json` evidence for offline triage (anchor rect, chosen side, final rect, shift
  delta, step/frame correlation).

Evidence anchors:

- Script: `tools/diag-scripts/ui-gallery/overlay/ui-gallery-tooltip-overlay-placement-after-code-tab-scroll-range.json`
- Query example:
  - `fretboard diag query overlay-placement-trace target/fret-diag/<run_id> --kind anchored_panel --anchor-test-id ui-gallery-tooltip-demo-trigger --json`

#### SE-213c evidence (baseline vs post-layout gate)

Status: recorded (2026-03-02, macOS aarch64, debug build).

Runs:

- Baseline (default extents): PASS (`run_id=1772436210204`)
  - Out dir: `target/fret-diag-se213c2-baseline`
- Gate on (`FRET_UI_SCROLL_EXTENTS_POST_LAYOUT=1`): PASS (`run_id=1772436308115`)
  - Out dir: `target/fret-diag-se213c2-post-layout`

Notes:

- `overlay_placement_trace` for tooltips may have `content_test_id=null` (the tooltip root element
  is used as the content identity). Filter by `--anchor-test-id` when querying.
- When using `--session-auto`, `fretboard diag query overlay-placement-trace <base_out_dir>`
  resolves to the nearest evidence-bearing `script.result.json` (session root), not bundle dump
  subdirectories.

Example (JSON):

- `fretboard diag query overlay-placement-trace target/fret-diag/<run_id> --kind anchored_panel --anchor-test-id ui-gallery-tooltip-demo-trigger --json`

### Inclusion / exclusion rules

These rules define which nodes can influence the scrollable extent:

- **Exclude** absolute-positioned nodes by default.
  - Motivation: absolute nodes often represent overlays, chrome, or hit-test scaffolding that
    should not silently change scroll ranges.
  - Note: current implementation inconsistently handles this (intrinsic probing skips absolute
    children, layout probing does not). Parity work should standardize this as part of SE-110.
- **Include** normal-flow descendants (including wrapper nodes) even if their own bounds are forced
  to match the viewport/content rect, as long as their descendants’ layout bounds overflow.

### Shrink behavior

When content shrinks, `content_size` is allowed to decrease in the same frame, and the scroll
offset must be clamped accordingly (matching `ScrollHandle` clamping semantics).

To avoid jarring oscillation on frames where probes/observation are partial, implementations may
apply small hysteresis (e.g. sub-pixel tolerances), but must not permanently “pin” content extents
to stale values.

## SE-110: Overflow Blockers Audit (as of 2026-03-02)

This section inventories current mechanism behaviors that can prevent post-layout extent derivation
from observing true overflow unless an explicit MaxContent/unbounded probe is performed.

The intent is to make SE-200 (prototype) realistic by identifying the smallest set of mechanism
changes required to support DOM/GPUI-like overflow.

### Root architectural constraint: layout uses definite `Rect` bounds

In `fret-ui` today, the layout phase passes **definite bounds** (`Rect`) from parent to child
(`layout_in(...)`). Many widgets treat `cx.available` (the bounds size) as a **hard maximum**
through `clamp_to_constraints(...)`.

Implication:

- Without an explicit “unbounded/MaxContent budget” concept in *layout*, the only way to allow a
  subtree to grow beyond its viewport-sized ancestor is to feed it a larger bounds rect (or to run
  a separate intrinsic measurement probe that computes such bounds).

This is the core reason the current scroll implementation uses `probe_unbounded` + deep
`measure_in(...)` to compute a `content_bounds` rect before laying out children.

### Budget-clamping wrappers (block overflow discovery)

Several common wrappers explicitly measure/probe children **within the parent’s constrained size**
to avoid “infinite viewport” outcomes during intrinsic sizing and to keep virtualized content from
seeing unbounded budgets.

These are correct for their original goals, but they also mean a post-layout extent strategy will
under-observe overflow unless the scroll mechanism provides a layout-time “scroll axis is
unbounded” budget that is visible to these wrappers.

Evidence anchors:

- Positioned containers:
  - `crates/fret-ui/src/declarative/host_widget/layout/positioned_container.rs`
  - uses `probe_available = clamp_to_constraints(cx.available, layout, cx.available)` and measures
    children with `probe_constraints_for_size(probe_bounds.size)` (definite).
- Generic container-like layouts:
  - `crates/fret-ui/src/declarative/host_widget/layout.rs`
  - “Probe within the available height budget” patterns appear in multiple element instances
    (container-ish shells, translated wrappers, etc.).

### Flex/Grid probe-pass behavior

`flex` and `grid` widgets treat `LayoutPassKind::Probe` as “run measure() under definite available
space”.

This preserves “probe pass does not see infinite viewport” invariants, but it also means that any
prototype that uses probe passes to derive extents must ensure probe constraints represent the
intended scroll-axis budget (or avoid probe passes entirely for extent derivation).

Evidence anchors:

- Flex: `crates/fret-ui/src/declarative/host_widget/layout/flex.rs`
- Grid: `crates/fret-ui/src/declarative/host_widget/layout/grid.rs`

### Absolute-positioned nodes inclusion is inconsistent

Intrinsic measurement probes often exclude absolute-positioned children:

- e.g. `max_non_absolute_children(...)` in `crates/fret-ui/src/declarative/host_widget/measure.rs`

However, post-layout observed bounds unions (used by scroll correctness guardrails today) can
include absolute descendants unless explicitly filtered.

Decision needed for parity:

- Default should likely **exclude** absolute nodes from scroll extents, matching the intuition that
  overlays/chrome should not silently change scroll range.
- If a component wants absolute nodes to affect extents, it should do so explicitly via normal-flow
  wrappers or a dedicated mechanism switch.

### Minimum change set for SE-200 to be viable

To make a “post-layout extents” prototype realistic (without relying on deep pre-layout measure
probes), we likely need at least one of these mechanism-level additions:

1. **Layout-time available-space budgets**:
   - Add a way for `layout_in(...)` / `LayoutCx` to carry “scroll axis budget is MaxContent” without
     requiring a giant `Rect` bounds.
   - Budget-clamping wrappers (positioned containers, container shells) must consult this budget

## Verification Plan (SE-210)

This section defines the minimal set of unit-testable invariants that lock down correctness while
SE-200 remains behind a gate.

### Unit-testable invariants (pure geometry)

- **Observed overflow is in content space**: extents are derived from `node_bounds` projected into
  `content_bounds.origin` coordinates (scroll offsets must not influence extents).
- **Wrapper peeling is safe**: “same-bounds” single-child wrapper chains do not hide overflow.
- **Bounded deep scan catches overflow**: when immediate children bounds do not reveal overflow
  (because wrappers are also forced to the content rect), a bounded DFS can still discover deeper
  overflow and grow extents.

Evidence anchors:

- Implementation: `crates/fret-ui/src/declarative/host_widget/layout/scrolling.rs`
- Unit tests: `crates/fret-ui/src/declarative/host_widget/layout/scrolling.rs`

### Non-unit-testable (for now)

- **Overlay anchoring parity** (reanchoring and scroll extent updates staying in sync) requires a
  higher-level harness (diag scripts or integration tests) and is tracked separately until the
  overlay/anchor query surface is directly assertable in unit tests.
     when probing/measuring children.
2. **Explicit overflow contexts**:
   - Install an inherited mechanism state at `Scroll` roots indicating which axis is allowed to
     overflow and how wrappers should treat `cx.available` during probes.
3. **Standardize absolute exclusion**:
   - Ensure the extent derivation path and the intrinsic measurement path agree on whether absolute
     nodes contribute to extents (default: exclude).

## Reference Direction (GPUI / DOM)

The DOM model (and GPUI’s implementation style) treats scroll extent as a property that can be
derived from **final layout geometry**, rather than requiring an additional “unbounded measure”
probe.

In particular, GPUI computes `content_size` from child layout bounds and then clamps the scroll
offset / computes `max_offset` after layout (see `repo-ref/zed/crates/gpui/src/elements/div.rs`).

This avoids performing a second deep subtree measurement solely to answer “how tall is the scroll
content?”

## Proposed Direction

Move `fret-ui` scroll extent computation toward “post-layout geometry”:

1. Layout children under the viewport-sized box (or otherwise well-defined container bounds).
2. Compute scroll extents from the resulting geometry (child bounds union, plus padding/border).
3. Clamp offsets and expose `max_offset` for scrollbars/automation.

Key constraints:

- Correctness first: no layout oscillation, stable anchor rects for overlays, deterministic scroll
  offset clamping across frames.
- Preserve an escape hatch for “true unbounded” width probing (e.g. code editor horizontal scroll)
  if needed, but avoid using it as the default for common vertical scrolling.

## Implementation Notes / Risks

- `probe_unbounded` currently couples two concerns:
  - how the subtree is measured (intrinsic sizing),
  - how scroll extent is derived.
  Untangling these is likely required.
- Some elements currently clamp measurement to the available size, which effectively forbids
  overflow unless a MaxContent probe is used. Moving toward a DOM-like model may require revisiting
  these constraints and/or introducing explicit “overflow allowed” semantics along the scroll axis.

## Next Steps

Track concrete tasks and “done criteria” in:

- `docs/workstreams/scroll-extents-dom-parity-todo.md`
