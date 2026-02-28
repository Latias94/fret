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

## Focus watcher semantics (`focus=true`)

Embla option `focus` (default `true`) enables `SlideFocus`:

- when the user presses **Tab** and focus moves into a slide, Embla scrolls the focused slide into
  view instantly (`useDuration(0)` + `scrollTo.index(...)`).
- Embla uses a small time window (≈10ms) between the Tab keydown and the focus event to avoid
  reacting to non-keyboard focus changes.

Fret mapping (v2 MVP):

- `CarouselOptions.watch_focus` (default `true`) is the recipe-level equivalent of Embla `focus`.
- The shadcn recipe infers the focused slide by comparing the **focused element's visual bounds**
  against per-slide visual bounds from the previous frame.
- When focus changes and either:
  - a Tab press was observed recently, **or**
  - the focused element is outside the viewport bounds (programmatic focus support),
  then the recipe scrolls instantly to the snap that contains the focused slide.

Evidence + gates:

- Implementation: `ecosystem/fret-ui-shadcn/src/carousel.rs` (watch focus block in the geometry pass)
- Gate: `ecosystem/fret-ui-shadcn/tests/carousel_focus_watch_tab_scrolls.rs`

## Accessibility semantics (role/labels)

Recipe-level stamping (v2 MVP):

- Root: `SemanticsRole::Panel`, `label="Carousel"`, `orientation={horizontal|vertical}`.
- Slides: `SemanticsRole::Group`, `label="Slide N of M"`.

Known gap:

- Core semantics does not currently expose a portable `aria-roledescription` equivalent, so we
  cannot represent `aria-roledescription="carousel"` / `"slide"` yet.

Evidence + gates:

- Implementation: `ecosystem/fret-ui-shadcn/src/carousel.rs` (root + slide semantics decorations)
- Gate: `ecosystem/fret-ui-shadcn/tests/carousel_a11y_semantics.rs`

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

Implementation (MVP shipped):

- Headless engine wraps `location`/`target` by applying the loop distance without resetting motion.
- The shadcn recipe applies an additional per-slide `RenderTransform` translation (`±content_size`)
  so the viewport remains visually continuous when the scroll location wraps.

Evidence anchors:

- Scroll loop normalization: `ecosystem/fret-ui-headless/src/embla/engine.rs`
- Loop distance application: `ecosystem/fret-ui-headless/src/embla/scroll_body.rs`
- Slide translation helper: `ecosystem/fret-ui-headless/src/embla/slide_looper.rs`
- Recipe wiring: `ecosystem/fret-ui-shadcn/src/carousel.rs`

## Slides in view semantics

Embla options:

- `inViewThreshold` (default `0`)
- `inViewMargin` (default `"0px"`)

Contract:

- The engine exposes “slides currently in view” and “changed” signals.
- Threshold/margin influence inclusion in the in-view set.

Implementation (MVP shipped):

- Headless: a deterministic 1D viewport intersection tracker that approximates Embla’s
  `IntersectionObserver`-driven behavior.
- Recipe: exposes a policy-only snapshot model (`CarouselSlidesInViewSnapshot`) whose `generation`
  increments when the in-view set changes (and carries enter/leave indices for that generation).

Evidence anchors:

- Headless tracker: `ecosystem/fret-ui-headless/src/embla/slides_in_view.rs`
- Recipe wiring: `ecosystem/fret-ui-shadcn/src/carousel.rs`
- Gate: `ecosystem/fret-ui-shadcn/tests/carousel_slides_in_view_snapshot.rs`

## ReInit + resize contract

Embla emits a `reInit` event when it re-initializes due to geometry or option changes (e.g. resize,
breakpoints, slide list changes).

## Breakpoints / responsive options

Embla supports responsive option overrides via `breakpoints` (media queries). Fret v2 provides a
Rust-native equivalent that is evaluated based on the measured carousel viewport width.

Contract:

- Breakpoints are evaluated by selecting the last entry where `min_width_px <= viewport_width_px`.
- The recipe applies an options patch on top of the base `CarouselOptions`.
- `start_snap` is intentionally not overridden by breakpoint patches (it is an initial selection
  input, not a responsive option).
- When breakpoint selection changes, it is treated as a `reInit` trigger (observable via
  `reinit_generation` / `CarouselEvent::ReInit`).

Evidence anchors:

- Breakpoint evaluation + patching: `ecosystem/fret-ui-shadcn/src/carousel.rs`

### Contract: geometry-driven re-init is safe and preserves motion

When the measured geometry changes in a way that affects snaps/limits (`scrollSnaps`, `contentSize`,
`viewSize`):

- The engine rebuilds its derived state (`limit`, targeting helpers, bounds config).
- The engine preserves the scroll integrator state (velocity) but must ensure `location` and
  `target` remain valid under the updated limits.
- The selected index becomes the snap closest to the current scroll target vector after re-init.
- The operation is idempotent and safe to call multiple times during continuous resize.

### Event contract (MVP implemented; full API still pending)

- MVP: `reInit` and `select` are observable via monotonic generation counters published in
  `CarouselApiSnapshot` (`reinit_generation` / `select_generation`).
- Full parity: a `CarouselApi` handle should expose `on_select` / `on_reinit` (or a safe event queue)
  without requiring callers to store arbitrary closures inside models.
- If re-init changes the selected index, `select` must also fire. Order is not required to match
  Embla exactly, but it must be stable and documented.

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
