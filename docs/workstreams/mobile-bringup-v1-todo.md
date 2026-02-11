---
title: Mobile Bring-up (v1) — TODO
status: draft
date: 2026-02-11
---

# Mobile Bring-up (v1) — TODO

Workstream entry:

- `docs/workstreams/mobile-bringup-v1.md`

## Docs

- [x] Add this workstream to `docs/README.md` and `docs/workstreams/README.md`.
- [ ] Add an explicit “Android-first MVP” note to `docs/roadmap.md` (if/when this becomes a priority).

## Touch scroll (baseline)

- [x] Implement touch pan-to-scroll for `Scroll` and `VirtualList` in `crates/fret-ui`.
  - Touch-only (no mouse drag scrolling).
  - Threshold-based arming (avoid scrolling on tiny jitter).
  - Preserve tap behavior when the gesture does not exceed click slop.
- [x] Add a unit test proving touch drag updates `ScrollHandle` offset.
- [ ] (Optional) Move richer policies (capture-steal, axis lock, inertia) into `ecosystem/fret-ui-kit`.

## Keyboard avoidance (policy)

- [ ] Add a keyboard avoidance policy helper in `ecosystem/fret-ui-kit` (environment query driven).
- [x] Apply a minimal policy in `apps/fret-ui-gallery` so focused inputs are not obscured by IME.
  - Start with bottom “scroll slack” based on `occlusion_insets.bottom`.

## Android plumbing (runner/backend)

- [ ] Add Android-specific environment commits:
  - safe-area insets (best-effort),
  - occlusion insets (IME / transient obstructions).
- [ ] Handle winit lifecycle events:
  - on `Suspended`: drop surfaces / pause rendering,
  - on `Resumed`: rebuild surfaces and request redraw.

Notes:

- Winit’s Android backend receives `InsetsChanged` internally but does not currently forward it as
  a public winit event (upstream TODO). Treat insets as “best-effort platform glue” and commit via
  `WindowMetricsService` when available.

## iOS plumbing (follow-up)

- [ ] iOS safe-area + keyboard occlusion commit (UIKit glue).
- [ ] iOS lifecycle surface rebuild policy.
