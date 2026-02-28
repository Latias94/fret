# Carousel Embla fearless refactor (v1)

This workstream exists to make Embla parity work *fearless*: we lock **outcomes** with gates first,
then refactor the internals aggressively while keeping Fret’s layering contracts intact.

If you only need “make the demo look right”, stop here and use `apps/fret-ui-gallery` tweaks.
If you want long-term parity (variable slide sizes, contain scroll, align), follow this doc.

## Non-negotiable layering

- `crates/fret-ui`: mechanism/contracts (routing, pointer capture + cancel semantics, hit-testing).
- `ecosystem/fret-ui-headless` (or `ecosystem/fret-ui-kit`): headless state machines / engines.
- `ecosystem/fret-ui-shadcn`: shadcn composition + tokens + taxonomy.

Policy/physics/scroll math does **not** belong in `crates/fret-ui`.

## Upstream references (source of truth)

- Embla options defaults (e.g. `dragThreshold=10`, `containScroll=trimSnaps`, `duration=25`):
  - `repo-ref/embla-carousel/packages/embla-carousel/src/components/Options.ts`
- Embla drag + click prevention model (`preventClick`):
  - `repo-ref/embla-carousel/packages/embla-carousel/src/components/DragHandler.ts`
- Embla engine composition (snap list, contain scroll, limit):
  - `repo-ref/embla-carousel/packages/embla-carousel/src/components/Engine.ts`
- shadcn/ui composition (orientation + spacing conventions):
  - `repo-ref/ui/apps/v4/registry/new-york-v4/ui/carousel.tsx`
  - `repo-ref/ui/apps/v4/registry/new-york-v4/examples/carousel-orientation.tsx`

## Current in-tree anchors

- shadcn carousel composition/interaction:
  - `ecosystem/fret-ui-shadcn/src/carousel.rs`
- headless drag threshold state machine (already exists):
  - `ecosystem/fret-ui-headless/src/carousel.rs`
- UI gallery surface (parity showcase + stable `test_id`s):
  - `apps/fret-ui-gallery/src/ui/pages/carousel.rs`
- Existing geometry parity tests (web vs fret):
  - `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout/carousel.rs`
- Existing diag script (swipe/buttons):
  - `tools/diag-scripts/ui-gallery/carousel/ui-gallery-carousel-demo-swipe-and-buttons.json`

## Goal

Deliver Embla-aligned **interaction + geometry outcomes** for Fret’s shadcn-style `Carousel`, while:

1. Keeping the mechanism/policy/recipe layering clean.
2. Producing stable, CI-friendly **gates** so we can refactor without fear.
3. Ending with an engine that can grow to more Embla options without rewriting the component.

## Parity scope (prioritized)

We want “all of it” eventually, but to keep the work landable, we stage the parity.

### P0 — must-have (blocks the rest)

1. **Snap list from slide geometry** (variable slide sizes).
   - Slides have per-slide main-axis sizes + gaps.
   - Snaps are derived from measured slide rects, not a single “extent”.
2. **Align semantics** (`align: start|center|end`).
3. **Contain scroll: `trimSnaps`** (Embla default).
4. **Drag + click prevention outcomes**
   - Drag threshold arming (`10px` default).
   - If drag wins, descendant press/click does not activate (Embla `preventClick` outcome).

### P1 — important (after P0 gates are solid)

5. `slidesToScroll` (grouping snaps).
6. `skipSnaps` semantics.
7. Vertical axis parity (orientation affects axis only; keys remain left/right like shadcn).

### P2 — advanced

8. `loop` (slide looper semantics).
9. `dragFree` / scroll-body physics (momentum feel, friction/duration model).
10. Plugins/events/API parity (shadcn `setApi` surface).

## Deliverables (fearless checklist)

Every milestone must ship a “3-pack”:

1. **Repro**: smallest surface (UI gallery page + stable selectors).
2. **Gate**: at least one deterministic test or diag script check.
3. **Evidence**: upstream reference + in-tree anchors.

## Gates (what makes this fearless)

### A) Headless engine unit tests (fast, non-flaky)

Add tests that assert:

- Snap positions for a set of slide sizes + gaps match expected.
- `align=start|center|end` moves the snap list as Embla would.
- `containScroll=trimSnaps` clamps snaps correctly at edges.
- `slidesToScroll` groups slides by snap as expected.

### B) Web vs Fret geometry parity tests (layout outcomes)

Extend `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout/carousel.rs` to cover:

- Variable slide sizes (non-uniform).
- Orientation vertical with constrained viewport height.
- Spacing conventions (`-ml`/`pl` for horizontal, `-mt`/`pt` for vertical).

### C) Diag scripts (interaction semantics)

Add/extend scripts under `tools/diag-scripts/` to cover:

- Drag from interactive descendant cancels activation (press → drag → release).
- Touch cross-axis scroll lock (move mostly cross-axis should not start drag).
- Buttons + keyboard arrows (Left/Right) move selection.

Use fixed frame delta when motion is involved:

- CLI: `--fixed-frame-delta-ms 16` (when launching via `fretboard`)
- or env: `FRET_DIAG_FIXED_FRAME_DELTA_MS=16`

## Refactor strategy (how we migrate without breaking everything)

### Step 1 — introduce a headless “snap model” (no UI dependency)

Create an engine API (names illustrative):

- `CarouselGeometryInput { viewport_main_px, slide_main_px: Vec<Px>, gap_px, options... }`
- `CarouselSnapModel { snaps_px: Vec<Px>, slides_by_snap: Vec<Vec<usize>>, limit... }`
- `CarouselEngineState { selected_snap, offset_px, ... }`

This lives in `ecosystem/fret-ui-headless` (preferred) so it is reusable and testable.

### Step 2 — wire the engine into shadcn Carousel behind a private toggle

In `ecosystem/fret-ui-shadcn/src/carousel.rs`:

- Keep current implementation as “v0”.
- Add “v1 engine-backed” path.
- Choose path via an internal-only mechanism (no new public knobs in `crates/fret-ui`).

Land gates first; then switch UI gallery to v1; then switch default; then delete v0.

### Step 3 — move from uniform extent snapping to geometry-derived snapping

Replace:

- `extent * index` snap offsets

With:

- `snap_model.snaps_px[selected]` offsets derived from measured slide sizes + align/contain rules.

### Step 4 — deepen parity iteratively (slidesToScroll → loop → dragFree)

Each added option comes with:

- headless unit tests,
- at least one geometry parity assertion, and
- at least one interaction diag script (if behavior changes).

## Definition of done (v1)

V1 is “P0 done + at least one P1 item”:

- P0: geometry-derived snaps + align + containScroll(trimSnaps) + drag/click prevention outcomes are green.
- At least one additional behavior gate exists beyond demo swipes (e.g. interactive-descendant drag gate).
- UI gallery Carousel page is docs-aligned and demonstrates the parity surface.
