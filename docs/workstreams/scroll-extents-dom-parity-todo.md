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
  - Gate on (`FRET_UI_SCROLL_EXTENTS_POST_LAYOUT=1`):
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

## Prototype (Behind a Gate)

- [x] SE-200 Add an opt-in implementation (env flag or compile-time flag) for “post-layout scroll extents”.
  - Must not change default behavior yet.
  - Env: `FRET_UI_SCROLL_EXTENTS_POST_LAYOUT=1`
- [~] SE-210 Add focused unit tests (incremental).
  - [x] SE-211 Pure-geometry overflow observation (wrapper peeling + bounded deep scan).
  - [x] SE-212 Offset clamping invariants (`ScrollHandle`).
  - [ ] SE-213 Scrollbar + overlay anchoring parity (needs harness).
    - [x] SE-213a Add `fretboard diag query overlay-placement-trace` (reads `script.result.json` evidence).
    - [x] SE-213b Add a UI Gallery script that (1) opens an anchored overlay, (2) expands a doc code tab
      (content growth), and (3) re-opens the overlay and asserts it is still clamped within the window.
    - [x] SE-213c Record evidence for baseline vs `FRET_UI_SCROLL_EXTENTS_POST_LAYOUT=1` (optional but recommended).

## Rollout

- [ ] SE-300 Turn the prototype into the default for vertical scrolling surfaces.
- [ ] SE-310 Keep an escape hatch for horizontal “true unbounded” cases if needed.
