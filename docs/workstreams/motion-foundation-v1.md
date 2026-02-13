# Motion Foundation (v1)

Status: Draft (notes only; ADRs remain the source of truth)

This workstream defines a reusable **motion/animation foundation** for Fret’s ecosystem layers
(shadcn/Radix-inspired, Material 3-inspired, MagicUI-inspired), while keeping `crates/fret-ui`
mechanism-only.

The goal is to make motion feel consistent across:

- variable refresh rates (60/120/144Hz),
- native + wasm runners,
- “web-like” duration/easing transitions and “iOS-like” spring/inertia interactions,
- deterministic diagnostics and scripted tests.

## Context and existing contracts

Scheduling is runner-owned and explicit (event-driven with continuous leases):

- `docs/adr/0034-timers-animation-and-redraw-scheduling.md`

Time and pointer motion are exposed as **non-reactive reads** (do not poison view-cache keys):

- `docs/adr/0240-frame-clock-and-reduced-motion-gates-v1.md`
- `docs/adr/0243-pointer-motion-snapshots-and-move-coalescing-v1.md`

Current state in ecosystem:

- `ecosystem/fret-ui-headless/src/transition.rs`: deterministic, tick-driven transitions.
- `ecosystem/fret-ui-headless/src/presence.rs`: tick-driven fade/scale-fade helpers.
- `ecosystem/fret-ui-kit/src/declarative/transition.rs`: driver that binds transitions to frame
  scheduling (`ContinuousFrames`) and invalidation.

## Problem statement (what “feels off” today)

Common UI libraries often encode motion as “duration + easing”, implicitly assuming wall-time.
If we represent duration as “N frames/ticks at 60fps”, then:

- high refresh (120Hz+) makes durations shorter in wall-time,
- background/foreground or frame drops can cause large `delta` jumps,
- tests become flaky if they implicitly depend on CPU-time deltas.

We want a foundation where:

1) **Wall-time** is the canonical duration unit.
2) Variable refresh rates do not change perceived duration.
3) Physics-based interactions use **velocity continuity** (no “restart stutter”).
4) Harness/diag can force deterministic time (fixed `delta`) for stable repros.

## Design principles (layering)

Mechanism vs policy:

- `crates/fret-ui`: expose frame clock snapshot reads and scheduling primitives (ADR 0034/0240),
  but do not define animation policies or component-default durations.
- `ecosystem/fret-ui-headless`: pure math/state machines/simulations (portable, deterministic,
  no theme, no scheduling).
- `ecosystem/fret-ui-kit`: drivers + ergonomic wrappers + token lookups (policy, recipes).
- component ecosystems (`fret-ui-shadcn`, future `fret-ui-material3`): consume `fret-ui-kit`
  surfaces and theme tokens; do not reinvent scheduling/time.

## References (local pinned sources)

- Flutter animation + physics:
  - `repo-ref/flutter/packages/flutter/lib/src/scheduler/ticker.dart`
  - `repo-ref/flutter/packages/flutter/lib/src/animation/animation_controller.dart`
  - `repo-ref/flutter/packages/flutter/lib/src/physics/{simulation.dart,spring_simulation.dart,friction_simulation.dart,clamped_simulation.dart}`
- motion.dev frameloop + generators:
  - `repo-ref/motion/packages/motion-dom/src/frameloop/batcher.ts`
  - `repo-ref/motion/packages/motion-dom/src/frameloop/sync-time.ts`
  - `repo-ref/motion/packages/motion-dom/src/animation/generators/{spring/*,inertia.ts}`
- Material 3 motion tokens (duration/easing/spring):
  - `repo-ref/material-web/tokens/versions/v30_0/sass/_md-sys-motion.scss`
- shadcn/ui v4 usage (durations + easings in recipes):
  - `repo-ref/ui/apps/v4/registry/new-york-v4/ui/*.tsx` (e.g. `sheet.tsx`, `sidebar.tsx`)

## Proposed foundation (v1)

### 1) Time model: `Duration`-based, driver-owned elapsed

In the UI kit driver, treat the frame clock snapshot as:

- `delta: Duration` (best-effort; can be fixed by harness/diag),
- `now_monotonic: Duration` (optional for diagnostics/tracing),
- per-window `frame_id` (for tracing / “same frame” coherence).

Driver rules:

- Clamp `delta` to a max (e.g. 40–50ms) to avoid huge leaps after stalls.
- Never read wall-clock time directly in view building.
- Hold a `ContinuousFrames` lease only while an animation is active.

### 2) Headless kernel: `Simulation` + `Timeline`

Add a new module: `ecosystem/fret-ui-headless/src/motion/`.

Core trait shapes (headless, no scheduling):

- `Simulation1D` (physics): provides `x(t)`, `dx(t)`, `is_done(t)` with tolerances, mirroring
  Flutter’s `Simulation`.
- `Timeline<T>` (tween-like): maps `t in [0,duration]` to `(value, velocity, done)` with easing.

Initial implementations (P0):

- `TweenTimeline<f32>`: cubic-bezier/linear easing for “web-like” transitions.
- `SpringSimulation`: analytic spring (stiffness/damping/mass or duration+bounce derivation).
- `FrictionSimulation` / `DecaySimulation`: inertial decay with optional bounds and spring-catch.

Why analytic/state-free shapes:

- Less sensitive to variable `delta` than explicit integrators.
- Enables deterministic stepping by simply advancing elapsed time.

### 3) Retargeting and cancellation (velocity continuity)

Introduce a retarget pattern in the driver:

- When a new target arrives mid-animation:
  - sample current `(x, dx)` at current elapsed,
  - start a new simulation/timeline from `(x, dx)` to the new target,
  - keep the same `MotionValue` identity (no pop/jump).

This is the key to “iOS-like” hand feel for draggable sheets/drawers and kinetic scrolling.

### 4) UI-kit driver API: `MotionValue` (element-owned state)

Provide a small reusable API in `ecosystem/fret-ui-kit` that fits the declarative element tree:

- `MotionValue<T>` stored in element state (`with_state`), sampled during build/layout/paint.
- `MotionValue::animate_to(target, spec)` triggers scheduling and advances until done.
- `MotionValue::set_immediate(value)` for reduced-motion and non-animated updates.

`spec` should be token-friendly:

- `MotionSpec::Tween { duration, easing }`
- `MotionSpec::Spring { stiffness, damping, mass, tolerance }`
- `MotionSpec::Inertia { velocity, bounds, bounce }`

### 5) Tokens: unify shadcn and Material 3 without coupling ecosystems

Use existing theme token surfaces:

- `Theme::duration_ms_by_key(...)`
- `Theme::easing_by_key(...)`

Add a first-class convention for motion keys (ecosystem policy):

- `md.sys.motion.duration.*` / `md.sys.motion.easing.*` / `md.sys.motion.spring.*` (M3-aligned)
- `shadcn.motion.duration.*` / `shadcn.motion.easing.*` (shadcn-aligned aliases)

Component ecosystems should request motion by **semantic key**, not hard-coded numbers.

### 6) Reduced motion policy (ecosystem-level)

Drivers in `fret-ui-kit` should offer helpers:

- “reduced motion” → snap or switch to minimal fades,
- preserve user-driven motion where needed (e.g. scroll position changes) but avoid continuous
  ambient animation loops.

This policy must live outside `crates/fret-ui`.

### 7) Diagnostics and determinism (non-negotiable)

Motion must be testable and reproducible:

- Support fixed `delta` in diag/harness (`--fixed-frame-delta-ms` and/or env) for stable scripted
  tests and screenshot captures.
- Add `fretboard diag` scripts for motion-critical components (sidebar toggle, sheet, drawer):
  validate invariants (open/close completes, geometry clamps, focus/dismiss behavior) under fixed
  delta.
- Record enough evidence in bundles (frame ids + any motion debug summaries) to explain failures.

## Migration plan (from tick-driven transitions)

We do not need to rewrite everything at once.

Suggested steps:

1) Add `Duration`-based drivers in `fret-ui-kit` (`TweenTimeline` + `MotionValue<f32>`).
2) Migrate the most “feel-sensitive” surfaces first:
   - `Sheet` / `Drawer` (drag + release spring + inertia),
   - `Sidebar` collapse/expand (simple tween but must be wall-time consistent),
   - `NavigationMenu` viewport motion.
3) Keep tick-driven `TransitionTimeline` for fully deterministic unit tests where it is useful,
   but stop encoding production UX durations as “60fps ticks”.

## Acceptance criteria (v1)

- A reusable headless `motion` module exists in `ecosystem/fret-ui-headless`.
- A small `MotionValue` + driver exists in `ecosystem/fret-ui-kit` and is used by at least:
  - one shadcn component (sidebar or sheet),
  - one “physics feel” interaction (drawer drag release or inertial scroll prototype).
- Motion tokens can be sourced from theme config (M3 keys and/or shadcn aliases).
- At least 2 diag scripts gate motion behavior under fixed `delta` (native runner).

## Implementation status (as of 2026-02-13)

Already landed (evidence anchors):

- Refresh-rate stable overlay transitions: `ecosystem/fret-ui-kit/src/declarative/transition.rs`
- Duration-based overlay transitions (Duration → 60Hz ticks → refresh-rate stable frames):
  - `ecosystem/fret-ui-kit/src/declarative/transition.rs` (`ticks_60hz_for_duration`, `drive_transition_*_duration`)
  - `ecosystem/fret-ui-kit/src/overlay_controller.rs` (`transition_with_durations*_duration`)
- Shadcn motion durations (wall-time constants): `ecosystem/fret-ui-kit/src/declarative/overlay_motion.rs` (`SHADCN_MOTION_DURATION_*`)
- Shadcn motion tokens sourced from theme (durations + cubic-bezier easing):
  - `ecosystem/fret-ui-kit/src/declarative/overlay_motion.rs` (`shadcn_motion_duration_*`, `shadcn_motion_ease_bezier`)
- Headless motion primitives: `ecosystem/fret-ui-headless/src/motion/`
  - `spring.rs`, `friction.rs`, `tween.rs`, `inertia.rs`
- UI-kit drivers: `ecosystem/fret-ui-kit/src/declarative/motion.rs`
  - `drive_tween_f32`, `drive_spring_f32`, `drive_inertia_f32`
- UI-kit `MotionValue` driver (unified snap/to/inertia update surface):
  - `ecosystem/fret-ui-kit/src/declarative/motion_value.rs` (`drive_motion_value_f32`)
- Pointer velocity snapshots exposed to component hooks (ADR 0243 alignment):
  - `crates/fret-ui/src/action.rs` (`PointerMoveCx.velocity_window`, `PointerUpCx.velocity_window`)
- Drawer release uses velocity projection to decide close/snap (starting point for Vaul-like feel):
  - `ecosystem/fret-ui-shadcn/src/drawer.rs`
- Drawer settle uses `MotionValue` (no manual priming fields like `settle_primed`):
  - `ecosystem/fret-ui-shadcn/src/drawer.rs`
- Sheet uses duration-based overlay transitions (no shadcn tick constants in component code):
  - `ecosystem/fret-ui-shadcn/src/sheet.rs`
- Presence supports duration + cubic-bezier (theme-friendly) drivers:
  - `ecosystem/fret-ui-kit/src/declarative/presence.rs`
  - `ecosystem/fret-ui-kit/src/primitives/presence.rs`
- Shadcn overlays that use Presence now read durations/easing from theme tokens (refresh-rate stable):
  - `ecosystem/fret-ui-shadcn/src/{context_menu,dropdown_menu,hover_card,menubar,popover,select,tooltip}.rs`
- Hover intent (tooltip/hover-card delays) scales 60Hz ticks to frame ticks for refresh-rate stability:
  - `ecosystem/fret-ui-kit/src/declarative/hover_intent.rs`

Diag gates:

- Sidebar toggle under fixed frame delta: `tools/diag-scripts/ui-gallery-sidebar-toggle-fixed-frame-delta.json`
- Dropdown menu open/close under fixed frame delta: `tools/diag-scripts/ui-gallery-dropdown-open-fixed-frame-delta.json`
- Drawer snap points drag + settle: `tools/diag-scripts/ui-gallery-drawer-snap-points-drag-settle.json`
