# Carousel Embla parity (v2) — Contracts

This document locks the semantics we intend to match when claiming “Embla parity” for Carousel in
Fret. It is intentionally **contract-first**: we define observable outcomes and the meaning of key
options before writing engine code.

Upstream references (local snapshots):

- Options + defaults: `repo-ref/embla-carousel/packages/embla-carousel/src/components/Options.ts`
- Drag semantics + preventClick: `repo-ref/embla-carousel/packages/embla-carousel/src/components/DragHandler.ts`
- ScrollBody integrator: `repo-ref/embla-carousel/packages/embla-carousel/src/components/ScrollBody.ts`
- Engine composition: `repo-ref/embla-carousel/packages/embla-carousel/src/components/Engine.ts`
- Scroll containment (trim/keep): `repo-ref/embla-carousel/packages/embla-carousel/src/components/ScrollContain.ts`
- Snap list grouping under containment: `repo-ref/embla-carousel/packages/embla-carousel/src/components/ScrollSnapList.ts`

## Layering (non-negotiable)

- `crates/fret-ui`: mechanisms only (routing/capture/cancel semantics/hit-testing).
- `ecosystem/fret-ui-headless`: headless engine math/state (no theme, no renderer types).
- `ecosystem/fret-ui-kit`: reusable interaction policies (optional).
- `ecosystem/fret-ui-shadcn`: composition + tokens + docs-aligned recipes.

## Terms

- **Location**: the current scroll location (track translate) in the main axis.
- **Target**: the desired location we are moving toward.
- **Seek step**: one integrator step, typically run once per rendered frame.
- **Settle**: the state where `abs(target - offset_location) < epsilon` (Embla uses `0.001`).

## Time model (critical)

Embla is designed around a `requestAnimationFrame` loop. Many internal values (e.g. `duration`)
behave like *per-frame integrator parameters*, not wall-clock milliseconds.

### Contract: “Embla duration” semantics

Embla option `duration` (default `25`) is a numeric parameter used by the scroll integrator
(`ScrollBody`): the displacement is divided by `scrollDuration` each seek step and then friction is
applied.

Therefore:

- `duration` is **not** a `std::time::Duration` in milliseconds.
- The observable outcome is “snappiness”/convergence speed, and it depends on how often `seek()`
  is called (frame rate).

### Fret mapping (v2 decision)

We will support two duration concepts:

1) `embla_duration: f32` (Embla semantics, default `25.0`), used by the headless engine.
2) `settle_duration: std::time::Duration` (Fret-friendly), used only by high-level recipes that
   want deterministic “animate-to” timelines (v1 already uses this).

The v2 engine will primarily use (1). Recipes may continue to expose (2) for shadcn docs alignment,
but “Embla parity” claims should be based on (1).

Reduced motion:

- When reduced motion is enabled, the engine should converge faster (or instantly) while remaining
  logically consistent (events still fire; indices update).

## Drag semantics

### Contract: arming + threshold + click prevention

Embla behavior:

- Drag does not “win” on pointer down.
- After `dragThreshold` px in the main axis, the drag wins and click is prevented (`preventClick`).
- On click prevention, the click is stopped and default prevented.

Fret translation:

- We keep the existing mechanism requirements:
  - parent can observe capture-phase moves and steal capture after threshold,
  - capture switch emits `PointerCancel` to the previous capture target.
- We consider “click prevented” satisfied if the descendant activation is suppressed.

### Contract: force shaping on release

Embla release computes:

- `rawForce = pointerUpDelta * forceBoost()`
- `force = allowedForce(direction(rawForce))`
- `forceFactor = factorAbs(rawForce, force)`
- `duration = baseDuration - 10 * forceFactor`
- `friction = baseFriction + forceFactor / 50`

The v2 engine will reproduce this shaping (best-effort) so that “fast swipe” and “slow drag”
produce observably different settle behavior.

## Contain scroll semantics

Embla option `containScroll` influences the **scroll snap list** and the effective **scroll limit**
when `loop=false`:

- `containScroll=false`: do not contain snaps (use aligned snaps).
- `containScroll='keepSnaps'`: clamp aligned snaps to scroll bounds but keep the full list.
- `containScroll='trimSnaps'` (default): clamp aligned snaps and trim the list using Embla’s
  `scrollContainLimit` rules (first/last snap groups expand to the edges via `ScrollSnapList`).

Edge constraints during interaction:

- The engine uses `ScrollBounds` to apply edge friction when the target and location are past the
  scroll limit.
- On pointer release, `ScrollBounds` may pull the target back to the limit when the displacement is
  small (Embla uses a 10% of view threshold).

Fret mapping (current baseline):

- `ecosystem/fret-ui-headless::carousel::snap_model_1d` already matches Embla’s `containScroll`
  outcomes for the shadcn Carousel recipe (v1 parity work).
- `ecosystem/fret-ui-headless::embla::scroll_contain` is a direct port of Embla `ScrollContain` and
  serves as a reference for v2 parity claims.
- `ecosystem/fret-ui-headless::embla::scroll_bounds` is used by the v2 engine ticks (and is applied
  while dragging in the shadcn recipe) to match Embla-style edge friction.

## Loop semantics

### Contract: loop=true means seamless loop

Embla `loop=true` is not “wrap index on prev/next”. It is a loop engine that wraps translations
continuously using loopers.

Fret v2 intent:

- If `loop=true` is in scope for v2, we implement:
  - scroll looper + slide looper translation wrapping,
  - without duplicating semantics/test ids in a way that breaks automation.

If we cannot implement seamless loop safely, we must explicitly keep `loop` as “selection wrap”
and **not** claim Embla parity for looping.

## Slides in view semantics

Embla options:

- `inViewThreshold` (default `0`)
- `inViewMargin` (default `"0px"`)

Contract:

- The engine exposes “slides currently in view” and “changed” signals.
- Threshold/margin influence inclusion in the in-view set.

## Events + API surface

Embla API expectations used by shadcn docs:

- `setApi(api)` to obtain the instance
- `api.scrollSnapList().length`
- `api.selectedScrollSnap()`
- `api.on('select', ...)`
- `api.on('reInit', ...)`

Fret v2 contract:

- Provide a Rust-native `CarouselApi` handle with:
  - `scroll_prev/scroll_next/scroll_to(index)`
  - `selected_scroll_snap`
  - `scroll_snap_list` (or `snap_count`)
  - `can_scroll_prev/can_scroll_next`
- Provide an event surface for:
  - `select`
  - `re_init`

We should avoid requiring callers to store arbitrary closures inside models; an event queue + model
versioning is acceptable if it is easy to use in typical UI code.
