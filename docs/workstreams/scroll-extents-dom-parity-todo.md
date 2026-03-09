# Scroll Extents (DOM/GPUI Parity) — TODO Tracker

Status: In progress

Tracking format:

- ID: `SE-{nnn}`
- Status: `[ ]` open, `[~]` in progress, `[x]` done, `[!]` blocked

---

## Evidence & Baseline

- [x] SE-010 Repro and evidence for “scroll probe stall” in UI Gallery.
  - Script: `tools/diag-scripts/ui-gallery-nav-card-click-latency.json`
  - Tracker: `docs/workstreams/ui-gallery-perf-scroll-measure.md`
- [x] SE-020 Record a stable before/after table for dt_ms + layout_time_us for the repro script.
  - Goal: p95/max dt_ms < 33ms in dev builds for nav clicks on common pages.
  - Note: tool-launched runs (`--launch`/`--reuse-launch`) require `schema_version=2`, so use the
    resolved script path: `tools/diag-scripts/ui-gallery/navigation/ui-gallery-nav-card-click-latency.json`.
  - Method (2026-03-02, macOS aarch64, debug build):
    - `dt_ms` is derived from `bundle.index.json` snapshot timestamp deltas.
    - `layout_time_us` is taken from `fretboard diag stats ... --json` (`max.layout_time_us`) for the captured
      `...-second` bundle.
  - Baseline (no post-layout extents):
    - Command: `fretboard diag perf tools/diag-scripts/ui-gallery/navigation/ui-gallery-nav-card-click-latency.json --repeat 3 --warmup-frames 0 --env RUST_LOG=error ...`
    - `dt_ms` min/p50/p95/max = `61/62/62/62`
    - `layout_time_us` min/p50/p95/max = `32093/32651/33184/33184`
    - Bundles: `target/fret-diag-se020-baseline/*ui-gallery-nav-card-click-latency-second/bundle.json`
  - Authoritative post-layout path (historically gate-on before 2026-03-09):
    - `dt_ms` min/p50/p95/max = `59/61/63/63`
    - `layout_time_us` min/p50/p95/max = `30913/31956/32330/32330`
    - Bundles: `target/fret-diag-se020-post-layout/*ui-gallery-nav-card-click-latency-second/bundle.json`

## Design (Contract)

- [x] SE-100 Define the scroll-extent contract: how extents are derived from layout geometry.
  - Include: padding/border, negative origins policy, rounding policy.
  - Include: interaction with overlays and anchoring.
  - Draft: `docs/workstreams/scroll-extents-dom-parity.md` (Target Contract / SE-100)
- [~] SE-110 Identify current “available size clamping” behaviors that prevent overflow.
  - List the minimum set of element types that must change to allow DOM-like overflow.
  - Initial inventory: `docs/workstreams/scroll-extents-dom-parity.md` (SE-110)
  - Blockers called out so far:
    - `clamp_to_constraints(...)` treats `available` as a hard maximum (even for `Auto`).
    - Layout probe paths that always use definite `Rect` budgets (container-ish wrappers, flex/grid).
    - Absolute-positioned nodes inclusion/exclusion inconsistencies.
    - Observation boundedness (wrapper peeling depth + DFS budget) needs telemetry.

- [x] SE-111 Decide and implement the mechanism contract for “fill vs fit” along the scroll axis.
  - Goal: make “auto can overflow” vs “fill must clamp” an explicit, testable contract.
  - Evidence: `docs/workstreams/scroll-extents-dom-parity.md` (SE-110, clamp policy blockers).
- [~] SE-112 Add a layout-time “overflow context” / available-space budget carrier.
  - Goal: let layout paths express `MaxContent` on the scroll axis without requiring a huge `Rect`.
  - Targets: `LayoutCx` + the key budget-clamping wrappers (container-ish layouts, positioned
    containers, flex/grid probe paths).
  - Audit targets (evidence anchors in SE-110):
    - `probe_constraints_for_size(...)` helpers that currently force definite budgets.
    - `ElementInstance::RenderTransform` / `FractionalRenderTransform` / `Anchored` probe paths.
    - `flex` / `grid` / `text` probe-pass behavior (definite `available`).
  - Implementation evidence (initial wiring, 2026-03-02):
    - `crates/fret-ui/src/layout/overflow.rs` (`LayoutOverflowContext`)
    - `crates/fret-ui/src/widget.rs` (`LayoutCx::probe_constraints_for_size`, context propagation)
    - `crates/fret-ui/src/declarative/host_widget/layout/scrolling.rs` (scroll installs context under gate)
    - `crates/fret-ui/src/declarative/host_widget/layout.rs` and
      `crates/fret-ui/src/declarative/host_widget/layout/positioned_container.rs` (probe helpers consult context)
- [x] SE-113 Standardize absolute-positioned node exclusion for extents.
  - Goal: ensure post-layout extents observation and intrinsic sizing agree (default: exclude).
  - Implementation evidence:
    - `crates/fret-ui/src/declarative/host_widget/layout/scrolling.rs` (`observe_scroll_overflow_extents`, filters absolute nodes)
- [x] SE-114 Surface bounded-observation telemetry for extents (budget hits).
  - Goal: detect when wrapper peeling/DFS budgets under-observe overflow in real UIs.
  - Implementation evidence:
    - `crates/fret-ui/src/tree/debug/scroll.rs` (`UiDebugScrollOverflowObservationTelemetry`)
    - `crates/fret-ui/src/declarative/host_widget/layout/scrolling.rs` (`observe_scroll_overflow_extents`, budget-hit recording)

## Implementation Rollout

- [x] SE-200 Land the post-layout scroll extents implementation and keep historical gate evidence.
  - Status: post-layout extents is now the default authoritative path; the temporary env toggle was retired on 2026-03-09.
- [~] SE-210 Add focused unit tests (incremental).
  - [x] SE-211 Pure-geometry overflow observation (wrapper peeling + bounded deep scan).
  - [x] SE-212 Offset clamping invariants (`ScrollHandle`).
- [x] SE-213 Scrollbar + overlay anchoring parity (needs harness).
    - [x] SE-213a Add `fretboard diag query overlay-placement-trace` (reads `script.result.json` evidence).
    - [x] SE-213b Add a UI Gallery script that (1) opens an anchored overlay, (2) expands a doc code tab
      (content growth), and (3) re-opens the overlay and asserts it is still clamped within the window.
    - [x] SE-213c Record historical evidence for baseline vs the former gate-on path (optional but recommended).
    - [x] SE-213d Add Popover coverage (click-triggered anchored panel) to reduce false confidence from hover-only overlays.
      - Scripts:
        - `tools/diag-scripts/ui-gallery/overlay/ui-gallery-tooltip-overlay-placement-after-code-tab-scroll-range.json`
        - `tools/diag-scripts/ui-gallery/popover/ui-gallery-popover-overlay-placement-after-code-tab-scroll-range.json`

## Rollout

- [x] SE-300 Turn the post-layout path into the default for vertical scrolling surfaces.
- [x] SE-310 Retire the temporary env escape hatch; keep scope-based fallback logic inside the mechanism for unsupported cases.
