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

### Mini suite: scroll + code-tab + overlay (recommended)

Suite id: `ui-gallery-scroll-extents-dom-parity`

This suite is designed to catch the UI Gallery “code tab expands but you can’t scroll further”
class of regressions, and to ensure anchored overlays remain clamped after scroll range changes.

Run (baseline):

- `cargo run -p fretboard -- diag suite ui-gallery-scroll-extents-dom-parity --dir target/fret-diag-se213-suite-baseline --session-auto --launch -- cargo run -p fret-ui-gallery`

Run (gate on):

- `cargo run -p fretboard -- diag suite ui-gallery-scroll-extents-dom-parity --dir target/fret-diag-se213-suite-post-layout --session-auto --env FRET_UI_SCROLL_EXTENTS_POST_LAYOUT=1 --launch -- cargo run -p fret-ui-gallery`

Evidence (2026-03-02, macOS aarch64, debug build):

- Baseline out dir: `target/fret-diag-se213-suite-baseline`
  - `ui-gallery-checkbox-invalid-code-tab-scroll-range`: `run_id=1772437569123`
  - `ui-gallery-tooltip-overlay-placement-after-code-tab-scroll-range`: `run_id=1772437659253`
  - `ui-gallery-typography-inline-code-tab-scroll-range`: `run_id=1772437747623`
  - Summary: `target/fret-diag-se213-suite-baseline/sessions/1772437474178-65965/suite.summary.json`
- Gate on out dir: `target/fret-diag-se213-suite-post-layout`
  - `ui-gallery-checkbox-invalid-code-tab-scroll-range`: `run_id=1772437848671`
  - `ui-gallery-tooltip-overlay-placement-after-code-tab-scroll-range`: `run_id=1772437937052`
  - `ui-gallery-typography-inline-code-tab-scroll-range`: `run_id=1772438027683`
  - Summary: `target/fret-diag-se213-suite-post-layout/sessions/1772437763689-21483/suite.summary.json`

Evidence (2026-03-02, macOS aarch64, debug build, after SE-112 overflow-context wiring):

- Baseline out dir: `target/fret-diag-se112-suite-baseline3`
  - `ui-gallery-checkbox-invalid-code-tab-scroll-range`: `run_id=1772442112463`
  - `ui-gallery-tooltip-overlay-placement-after-code-tab-scroll-range`: `run_id=1772442117838`
  - `ui-gallery-typography-inline-code-tab-scroll-range`: `run_id=1772442123499`
  - Summary: `target/fret-diag-se112-suite-baseline3/sessions/1772442109769-12830/suite.summary.json`
- Gate on out dir: `target/fret-diag-se112-suite-post-layout3`
  - `ui-gallery-checkbox-invalid-code-tab-scroll-range`: `run_id=1772442136322`
  - `ui-gallery-tooltip-overlay-placement-after-code-tab-scroll-range`: `run_id=1772442141765`
  - `ui-gallery-typography-inline-code-tab-scroll-range`: `run_id=1772442147987`
  - Summary: `target/fret-diag-se112-suite-post-layout3/sessions/1772442133993-13204/suite.summary.json`

Example (JSON):

- `fretboard diag query overlay-placement-trace target/fret-diag/<run_id> --kind anchored_panel --anchor-test-id ui-gallery-tooltip-demo-trigger --json`

### Inclusion / exclusion rules

These rules define which nodes can influence the scrollable extent:

- **Exclude** absolute-positioned nodes by default.
  - Motivation: absolute nodes often represent overlays, chrome, or hit-test scaffolding that
    should not silently change scroll ranges.
  - Status: standardized in SE-113 (post-layout observation now filters absolute nodes, matching
    the intrinsic measurement path).
  - Evidence: `crates/fret-ui/src/declarative/host_widget/layout/scrolling.rs`
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

### Blocker: `clamp_to_constraints(...)` treats `available` as a hard maximum (even for `Auto`)

`clamp_to_constraints(...)` always clamps the final `size.{width,height}` to `available`, even when
the corresponding style length is `Auto`.

This makes “auto-sized children can exceed the viewport and overflow” difficult to express in pure
post-layout terms: if parents consistently pass viewport-sized `Rect`s and children clamp to that
available size, then overflow never materializes in `node_bounds` for extents observation to union.

Evidence anchors:

- `crates/fret-ui/src/declarative/layout_helpers.rs` (`clamp_to_constraints`, final clamp to
  `available`)
- Callsite notes that depend on this behavior:
  - `crates/fret-ui/src/declarative/host_widget/layout/scrolling.rs`
  - `crates/fret-ui/src/declarative/host_widget/layout/positioned_container.rs`

### Budget-clamping wrappers (block overflow discovery)

Several common wrappers explicitly measure/probe children **within the parent’s constrained size**
to avoid “infinite viewport” outcomes during intrinsic sizing and to keep virtualized content from
seeing unbounded budgets.

These are correct for their original goals, but they also mean a post-layout extent strategy will
under-observe overflow unless the scroll mechanism provides a layout-time “scroll axis is
unbounded” budget that is visible to these wrappers.

Evidence anchors:

- Layout-local helpers that force **definite** probe budgets:
  - `probe_constraints_for_size(...)` in `crates/fret-ui/src/declarative/host_widget/layout.rs`
  - `probe_constraints_for_size(...)` in `crates/fret-ui/src/declarative/host_widget/layout/positioned_container.rs`
- Positioned containers:
  - `crates/fret-ui/src/declarative/host_widget/layout/positioned_container.rs`
  - uses `probe_available = clamp_to_constraints(cx.available, layout, cx.available)` and measures
    children with `probe_constraints_for_size(probe_bounds.size)` (definite).
- Generic container-like layouts:
  - `crates/fret-ui/src/declarative/host_widget/layout.rs`
  - “Probe within the available height budget” patterns appear in multiple element instances
    (container-ish shells, transform wrappers, anchored wrappers, etc.).
  - Examples in this file:
    - `ElementInstance::RenderTransform`
    - `ElementInstance::FractionalRenderTransform`
    - `ElementInstance::Anchored`

### Flex/Grid probe-pass behavior

`flex`, `grid`, and `text` widgets treat `LayoutPassKind::Probe` as “run measure() under a
viewport-sized budget”, but the budget is now constructed via `LayoutCx::probe_constraints_for_size(...)`
so overflow contexts can override the scroll axis to `MaxContent`.

This preserves “probe pass does not see infinite viewport” invariants on the cross axis, while
still allowing scroll surfaces to opt into DOM/GPUI-like overflow observability.

Evidence anchors:

- Flex: `crates/fret-ui/src/declarative/host_widget/layout/flex.rs`
- Grid: `crates/fret-ui/src/declarative/host_widget/layout/grid.rs`
- Text: `crates/fret-ui/src/declarative/host_widget/layout.rs`

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

1. **Layout-time available-space budgets / overflow contexts**:
   - Add a way for `layout_in(...)` / `LayoutCx` to carry “scroll axis budget is MaxContent” without
     requiring a giant `Rect` bounds.
   - Budget-clamping wrappers (positioned containers, container shells, flex/grid probe paths) must
     consult this budget when probing/measuring children.
2. **A mechanism-level clamp policy hook (scoped)**:
   - Keep `clamp_to_constraints(...)` as the default (it is widely relied on today),
   - but allow scroll roots (or any explicit “overflow context”) to request a different clamp
     policy on the scroll axis, so auto-sized descendants can exceed the viewport and still be
     clipped/observed for extents.
3. **Standardize absolute exclusion (SE-113 implemented)**:
   - Ensure the extent derivation path and the intrinsic measurement path agree on whether absolute
     nodes contribute to extents (default: exclude).
4. **Keep observation bounded and debuggable**:
   - Maintain a bounded scan budget (as we do today) and surface “budget hit” telemetry so we can
     detect cases where wrapper chains exceed peeling/scan limits (SE-114 implemented).

### SE-111 draft: overflow contexts + clamp policy (contract sketch)

Goal: make “fill vs fit vs overflow” an explicit mechanism contract so post-layout extents can be
derived from `node_bounds` without relying on deep pre-layout unbounded probes.

Proposed mechanism-level concepts:

- **Overflow context (inherited)**:
  - Installed at `Scroll` roots (and potentially other explicit overflow surfaces).
  - Carries axis-specific “budget” hints (e.g. `scroll_axis_budget = MaxContent`, `cross_axis_budget = Definite(viewport)`).
  - Intended to influence *measurement/probe constraints*, not to change paint-time clipping.
- **Scoped clamp policy**:
  - Default behavior remains: `clamp_to_constraints(...)` treats `available` as a hard maximum.
  - Under an overflow context, on the **scroll axis only**, auto-sized descendants may exceed the
    viewport-sized `Rect` budget so that overflow is visible in post-layout geometry.
  - Cross axis remains clamped to preserve “no infinite viewport” invariants.

Non-goals:

- Do not implicitly include padding/border/chrome in extents at the mechanism level (policy lives
  in component ecosystems via explicit wrappers).
- Do not make overlays/chrome affect scroll ranges by default (absolute descendants are excluded).

Open questions (to resolve before implementation):

- Should the overflow context be carried on `LayoutCx` (layout-time inherited state) or via
  `LayoutConstraints` (explicit constraints object)?
- What is the minimal set of element/layout implementations that must consult the context to make
  the behavior observable in real trees (docs pages + tab panels)?
- How do we expose “budget hit / under-observed overflow” telemetry without turning it into a
  perf hazard in release builds?

Implementation status:

- SE-112 (initial wiring) is in progress. A minimal `LayoutOverflowContext` exists and is
  propagated through `LayoutCx` / `UiTree::layout_in_with_pass_kind`. In the post-layout extents
  gate, scroll roots install a context that sets the scroll axis probe budget to `MaxContent` so
  wrapper-heavy subtrees can measure descendants without a hard scroll-axis clamp.
- SE-111 (clamp policy hook) is implemented. Under the scroll-installed overflow context, `Auto`
  sizing is allowed to exceed the viewport-sized `available` budget on the scroll axis, so
  overflow becomes visible in `node_bounds` for post-layout extent derivation.
  - Evidence:
    - Context field: `crates/fret-ui/src/layout/overflow.rs` (`allow_overflow_on_auto`)
    - Clamp helper: `crates/fret-ui/src/declarative/layout_helpers.rs` (`clamp_to_constraints_with_overflow_context`)
    - Clamp adoption: `crates/fret-ui/src/declarative/host_widget/layout.rs`
    - Scroll installs: `crates/fret-ui/src/declarative/host_widget/layout/scrolling.rs`
- SE-113 (absolute exclusion parity) is implemented. Post-layout overflow observation now excludes
  absolute-positioned nodes by default, matching the intrinsic measurement skip behavior.
- SE-114 (bounded-observation telemetry) is implemented. When wrapper peeling or bounded deep scan
  hits its budget, `UiDebugScrollNodeTelemetry` records an `overflow_observation` payload for the
  scroll node (and `FRET_DEBUG_SCROLL_EXTENT_PROBE=1` prints a budget-hit log line).
  - Tooling: `fretboard diag query scroll-extents-observation <base_out_dir|session_out_dir|bundle_dir|bundle.schema2.json> --json`
    - The JSON output includes a best-effort `test_id` field (nearest ancestor semantics decoration),
      to make “budget hit” reports easier to triage in UI Gallery pages.
    - Filter modes:
      - Default: only rows where observation hit its wrapper/deep-scan budget (actionable).
      - `--deep-scan`: include rows where a bounded deep scan ran (even if it stayed under budget).
      - `--all`: include all scroll nodes (may be large).
  - Perf note: the bounded deep scan is edge-gated (only allowed when the user is already at the
    current scroll-extent edge and the extent may be stale). This avoids spending scan budget on
    frames where a temporarily stale extent cannot cause “pinned scroll range” regressions.

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

- **Overlay anchoring parity** (reanchoring and scroll extent updates staying in sync) is covered
  via UI Gallery diag scripts while SE-200 remains behind a gate:
  - Tooltip (hover-triggered anchored panel): `tools/diag-scripts/ui-gallery/overlay/ui-gallery-tooltip-overlay-placement-after-code-tab-scroll-range.json`
  - Popover (click-triggered anchored panel): `tools/diag-scripts/ui-gallery/popover/ui-gallery-popover-overlay-placement-after-code-tab-scroll-range.json`
  - These scripts assert (1) scroll extents remain finite after code-tab content growth and (2)
    reopened overlays remain clamped within the window.

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
