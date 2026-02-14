# Motion Foundation (v1)

Status: Draft (notes only; ADRs remain the source of truth)

See:

- Milestones: `docs/workstreams/motion-foundation-v1-milestones.md`
- TODO tracker: `docs/workstreams/motion-foundation-v1-todo.md`

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
- Animata (recipe inspiration; Framer Motion / DOM-based, treat as “spec” not “runtime”):
  - Local mirror root (this workstation): `F:\SourceCodes\Rust\fret\repo-ref\animata`
  - `repo-ref/animata/README.md`
  - `repo-ref/animata/animata/overlay/*.tsx`

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
- `Theme::number_by_key(...)`

Add a first-class convention for motion keys (ecosystem policy):

- `md.sys.motion.duration.*` / `md.sys.motion.easing.*` / `md.sys.motion.spring.*` (M3-aligned)
- `duration.shadcn.motion.*` / `easing.shadcn.motion.*` / `number.shadcn.motion.*` (shadcn-aligned)
- `duration.motion.*` / `easing.motion.*` / `number.motion.spring.*` (cross-ecosystem semantic)

Component ecosystems should request motion by **semantic key**, not hard-coded numbers.

Token taxonomy (v1; conventions we can keep stable):

- Duration keys:
  - Prefer *semantic* keys for recipes/components (stable, future-proof):
    - `duration.shadcn.motion.overlay.open`
    - `duration.shadcn.motion.overlay.close`
    - `duration.shadcn.motion.toast.enter`
    - `duration.shadcn.motion.toast.exit`
    - `duration.shadcn.motion.sidebar.toggle`
    - `duration.shadcn.motion.collapsible.toggle`
  - Keep the existing numeric scale as leaf-level defaults and for quick authoring:
    - `duration.shadcn.motion.{100|200|300|500}`
- Easing keys:
  - Use semantic keys where a component needs a specific curve (don’t assume 1 curve fits all):
    - `easing.shadcn.motion.overlay`
    - `easing.shadcn.motion.toast`
    - `easing.shadcn.motion.collapsible.toggle`
  - Keep `easing.shadcn.motion` as a simple global default.
- Spring keys:
  - For duration+bounce (shadcn-style):
    - `duration.shadcn.motion.spring.<surface>.<intent>`
    - `number.shadcn.motion.spring.<surface>.<intent>.bounce`
  - For damping+stiffness (M3 scheme):
    - `md.sys.motion.spring.{default|fast|slow}.{spatial|effects}.{damping|stiffness}`

Seeded defaults (New York v4 presets):

- The built-in shadcn New York v4 theme presets seed the motion tokens above so recipes can
  request semantic keys without hard-coding numbers.
- Toast/Sonner timing is tuned slightly faster than the numeric `200ms` scale to better match the
  “web-like” feel while staying deterministic under fixed-delta diagnostics:
  - `duration.shadcn.motion.toast.enter = 160ms`
  - `duration.shadcn.motion.toast.exit = 120ms`

Implementation note:

- The semantic keys above are a *convention* we can migrate toward without requiring Theme-level
  aliasing. Pilot surfaces (Dialog / AlertDialog / Sidebar) already read semantic keys with numeric
  fallbacks; most other shadcn surfaces still read the numeric scale tokens directly. The goal is
  to gradually switch recipe code to semantic keys while keeping numeric scale keys as fallbacks.

Pragmatic mapping table (shadcn numeric durations ↔ M3 duration tokens):

| shadcn key | ms | closest M3 token | Notes |
| --- | ---: | --- | --- |
| `duration.shadcn.motion.100` | 100 | `md.sys.motion.duration.short2` | exact match in v30 |
| `duration.shadcn.motion.200` | 200 | `md.sys.motion.duration.short4` | exact match in v30 |
| `duration.shadcn.motion.300` | 300 | `md.sys.motion.duration.medium2` | exact match in v30 |
| `duration.shadcn.motion.500` | 500 | `md.sys.motion.duration.long2` | exact match in v30 |

Easing mapping note:

- `easing.shadcn.motion` (default: `cubic-bezier(0.22,1,0.36,1)`) does not have an exact match in
  Material Web v30’s sys easings. When bridging across ecosystems, treat easing as a per-recipe
  choice and pick the closest `md.sys.motion.easing.*` (or define an explicit shadcn curve key for
  that recipe).

Spring tokens (v1):

- Represent a spring using two theme tokens: `duration_ms` + `number` (bounce).
- Prefer authoring-friendly "duration + bounce" (Flutter-style) and derive physical params in
  headless math (`SpringDescription::with_duration_and_bounce`).
- Bounce semantics (mirrors Flutter):
  - `bounce = 0.0` is critically damped (no overshoot),
  - `bounce in (0, 1)` is underdamped (overshoot),
  - negative values are overdamped,
  - values must be `> -1.0` to keep damping derivation finite.

Current shadcn drawer spring keys (ecosystem policy; overrideable via theme JSON):

- `duration.shadcn.motion.spring.drawer.settle` (ms)
- `number.shadcn.motion.spring.drawer.settle.bounce`
- `duration.shadcn.motion.spring.drawer.inertia_bounce` (ms)
- `number.shadcn.motion.spring.drawer.inertia_bounce.bounce`

Theme JSON example:

```json
{
  "durations_ms": {
    "duration.shadcn.motion.spring.drawer.settle": 240,
    "duration.shadcn.motion.spring.drawer.inertia_bounce": 240
  },
  "numbers": {
    "number.shadcn.motion.spring.drawer.settle.bounce": 0.0,
    "number.shadcn.motion.spring.drawer.inertia_bounce.bounce": 0.25
  }
}
```

Spring cookbook (starter presets; tune with diag gates):

| Preset | duration_ms | bounce | Notes |
| --- | --- | --- | --- |
| `snappy` | 180 | 0.0 | fast, no overshoot |
| `standard` | 240 | 0.0 | baseline critical-damped |
| `emphasized` | 320 | 0.15 | mild overshoot for "hero" motions |
| `bouncy` | 300 | 0.35 | noticeable overshoot; use sparingly |
| `overdamped` | 260 | -0.20 | heavier feel; no overshoot |

Material 3 spring taxonomy (tokens already exist in `ecosystem/fret-ui-material3`):

- M3 expresses springs as **damping ratio + stiffness** (not duration+bounce):
  - `md.sys.motion.spring.{default|fast|slow}.{spatial|effects}.{damping|stiffness}`
- In Fret's M3 ecosystem today, these map to `SpringSpec { damping, stiffness }` and are advanced
  deterministically by `FrameId` (see `ecosystem/fret-ui-material3/src/motion.rs` and
  `ecosystem/fret-ui-material3/src/foundation/motion_scheme.rs`).

Bridging guidance (spec-only; no required unification yet):

- Prefer **duration+bounce** for shadcn-style "recipe motion" where authors think in wall-time.
- Prefer **damping ratio + stiffness** for M3 "motion scheme" where tokens are already published in
  that form.
- If/when we unify on a single headless spring kernel, both shapes should remain supported:
  - duration+bounce -> `SpringDescription::with_duration_and_bounce`
  - (ratio, stiffness) -> `SpringDescription::with_damping_ratio(mass=1.0, stiffness, ratio)`

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

## Animata alignment notes (how to use it safely)

Animata is a curated set of interaction recipes built for web (React/DOM) with Framer Motion. We
should not try to port its runtime model, but it is still valuable as a **source of “what moves,
when, and how it feels”**.

Recommended approach:

1) Treat Animata as a **UX spec** (timing curves, sequencing, variant breakdown), not an API to
   reproduce.
2) Translate each recipe into a small, portable “motion recipe” vocabulary (tween + spring +
   inertia + timeline/stagger).
3) Implement only the missing primitives in `fret-ui-headless` + `fret-ui-kit` (ecosystem policy),
   and keep `crates/fret-ui` mechanism-only.
4) Add deterministic diag gates under fixed `delta` for every feel-sensitive recipe.

High-value pilot recipes to align next (draft):

- Sidebar collapse/expand (layout-affecting + perceived snappiness)
- Drawer/sheet (drag → release inertia/spring settle)
- Modal/dialog (presence + focus/dismiss choreography)
- Tabs underline / navigation indicator (timeline + easing polish)
- Toast stack (stagger + interrupt/re-target behavior)

### Recipe alignment matrix (draft)

This table is intentionally spec-first. The goal is to make the intended feel explicit, then
land deterministic gates before polishing implementation details.

| Pilot | Recipe | Spec sources | Fret target | Motion channels & primitives | Tokens & gates | Status |
| --- | --- | --- | --- | --- | --- | --- |
| P1 | Sidebar collapse/expand | shadcn docs: `repo-ref/ui/apps/v4/content/docs/components/sidebar.mdx`<br>shadcn impl: `repo-ref/ui/apps/v4/registry/new-york-v4/ui/sidebar.tsx` | `ecosystem/fret-ui-shadcn/src/sidebar.rs` (`sidebar_collapse_motion`) | width / rail reveal (+ optional opacity)<br>tween timeline<br>optional: layout-aware choreography (future) | tokens (current): `duration.shadcn.motion.sidebar.toggle`, `easing.shadcn.motion.sidebar` (default: linear)<br>gate: `tools/diag-scripts/ui-gallery-sidebar-toggle-fixed-frame-delta.json` | Landed (baseline) |
| P1 | Accordion / Collapsible (height:auto) | Animata FAQ: `repo-ref/animata/animata/accordion/faq.tsx`<br>shadcn docs: `repo-ref/ui/apps/v4/content/docs/components/accordion.mdx`<br>shadcn impl: `repo-ref/ui/apps/v4/registry/new-york-v4/ui/accordion.tsx` | `ecosystem/fret-ui-shadcn/src/{accordion,collapsible}.rs`<br>`ecosystem/fret-ui-kit/src/declarative/collapsible_motion.rs` | measured-height reveal + opacity<br>Duration tween timeline + cubic-bezier easing | tokens: `duration.shadcn.motion.collapsible.toggle` → `duration.motion.collapsible.toggle` (fallback: `duration.shadcn.motion.200`)<br>`easing.shadcn.motion.collapsible.toggle` → `easing.motion.collapsible.toggle` (fallback: `easing.shadcn.motion`)<br>gate: `tools/diag-scripts/ui-gallery-accordion-faq-toggle-fixed-frame-delta.json` | Landed (token-driven) |
| P1 | Drawer / Sheet settle | shadcn docs: `repo-ref/ui/apps/v4/content/docs/components/drawer.mdx`, `repo-ref/ui/apps/v4/content/docs/components/sheet.mdx`<br>shadcn impl: `repo-ref/ui/apps/v4/registry/new-york-v4/ui/drawer.tsx`, `repo-ref/ui/apps/v4/registry/new-york-v4/ui/sheet.tsx` | `ecosystem/fret-ui-shadcn/src/drawer.rs`<br>`ecosystem/fret-ui-shadcn/src/sheet.rs` | translate + scrim opacity<br>drag velocity projection -> inertia -> spring settle<br>retarget mid-flight (no restart stutter) | tokens (current): `duration.shadcn.motion.spring.drawer.settle`, `number.shadcn.motion.spring.drawer.settle.bounce`<br>`duration.shadcn.motion.spring.drawer.inertia_bounce`, `number.shadcn.motion.spring.drawer.inertia_bounce.bounce`<br>gates: `tools/diag-scripts/ui-gallery-drawer-snap-points-drag-settle.json`, `tools/diag-scripts/ui-gallery-drawer-snap-points-drag-retarget-settle-fixed-frame-delta.json` | Landed (baseline) |
| P1 | Dialog / Modal presence | Animata modal: `repo-ref/animata/animata/overlay/modal.tsx`<br>shadcn docs: `repo-ref/ui/apps/v4/content/docs/components/dialog.mdx`, `repo-ref/ui/apps/v4/content/docs/components/alert-dialog.mdx`<br>shadcn impl: `repo-ref/ui/apps/v4/registry/new-york-v4/ui/dialog.tsx`, `repo-ref/ui/apps/v4/registry/new-york-v4/ui/alert-dialog.tsx` | `ecosystem/fret-ui-shadcn/src/dialog.rs`<br>`ecosystem/fret-ui-shadcn/src/alert_dialog.rs` | opacity + scale (and optional blur)<br>barrier fade + focus/dismiss choreography<br>optional: spring-based settle for native-like feel | tokens (current): shadcn overlay motion tokens (duration/easing)<br>gates: `tools/diag-scripts/ui-gallery-overlay-dialog-open-motion-snapshots.json`, `tools/diag-scripts/ui-gallery-overlay-dialog-open-close-fixed-frame-delta.json` | Landed (baseline) |
| P2 | Tabs shared indicator | Animata nav-tabs: `repo-ref/animata/animata/container/nav-tabs.tsx`<br>Material motion tokens: `repo-ref/material-web/tokens/versions/v30_0/sass/_md-sys-motion.scss` | `ecosystem/fret-ui-material3/src/tabs.rs` (`primary_tab_list_indicator`)<br>`ecosystem/fret-ui-shadcn/src/tabs.rs` (`tabs_shared_indicator`) | indicator x/y + width/height<br>measurement-driven target bounds<br>Duration-based spring (refresh-rate stable) | tokens: `duration.motion.spring.shared_indicator`, `number.motion.spring.shared_indicator.bounce`<br>gates: `tools/diag-scripts/ui-gallery-material3-tabs-indicator-pixels-changed-fixed-frame-delta.json` (M3) + `tools/diag-scripts/ui-gallery-alert-tabs-shared-indicator-pixels-changed-fixed-frame-delta.json` (shadcn) | Landed (pilot; M3 + shadcn) |
| P2 | Tabs content presence (fluid) | Animata fluid-tabs: `repo-ref/animata/animata/card/fluid-tabs.tsx` | `ecosystem/fret-ui-shadcn/src/tabs.rs` (`content_presence_motion`) | panel crossfade (presence.enter/exit) | tokens: `duration.motion.presence.enter`, `duration.motion.presence.exit`, `easing.motion.standard`<br>gate: `tools/diag-scripts/ui-gallery-motion-presets-fluid-tabs-pixels-changed-fixed-frame-delta.json` | Landed (MVP) |
| P2 | Toast stack (Sonner) | shadcn docs: `repo-ref/ui/apps/v4/content/docs/components/toast.mdx`<br>shadcn impl: `repo-ref/ui/apps/v4/registry/new-york-v4/ui/sonner.tsx` | `ecosystem/fret-ui-shadcn/src/toast.rs`<br>`ecosystem/fret-ui-shadcn/src/sonner.rs` | enter/exit presence + stack shift<br>stagger + interrupt/re-target behavior<br>swipe dismiss inertia (future) | tokens: `duration.shadcn.motion.toast.{enter|exit}`, `easing.shadcn.motion.toast`<br>gates: `tools/diag-scripts/ui-gallery-sonner-open-close-fixed-frame-delta.json`, `tools/diag-scripts/ui-gallery-sonner-interrupt-fixed-frame-delta.json` | Landed (baseline) |

## Layout-affecting motion (v1)

Not every “layout change” should be handled with a generic FLIP-style system. For a custom renderer,
the most reliable v1 strategy is **explicit choreography** + **stable structure**, with a small set
of reusable measurement-driven primitives.

Guidelines:

- Prefer keeping the element subtree **structurally stable** and animating via:
  - width/height + overflow clipping (CSS-like),
  - paint transforms (visual transforms) for non-interactive visuals (indicators),
  - render transforms only when hit-testing must follow visuals.
- If a layout needs measurement (e.g. a tab indicator tracks the selected slot’s bounds), treat
  measurement as an explicit input, then animate a small set of scalar channels (`x`, `w`, `alpha`)
  using `MotionValue` (Duration-based) rather than per-frame “60Hz ticks”.
- For overlays/panels that slide off-screen but should not steal layout space (off-canvas), separate:
  - the **layout wrapper** (width collapses to 0, overflow clipped),
  - the **visual surface** (relative offset within the wrapper).

Definition of done for a layout-motion pilot:

- the target uses Duration-based drivers (refresh-rate stable),
- retargeting mid-flight does not restart stutter,
- at least one deterministic diag gate exists under fixed `delta`.

## Acceptance criteria (v1)

- A reusable headless `motion` module exists in `ecosystem/fret-ui-headless`.
- A small `MotionValue` + driver exists in `ecosystem/fret-ui-kit` and is used by at least:
  - one shadcn component (sidebar or sheet),
  - one “physics feel” interaction (drawer drag release or inertial scroll prototype).
- Motion tokens can be sourced from theme config (M3 keys and/or shadcn aliases).
- At least 2 diag scripts gate motion behavior under fixed `delta` (native runner).

## Implementation status (as of 2026-02-14)

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
- Measured-height collapsible motion supports duration + cubic-bezier easing (height:auto-style choreography):
  - `ecosystem/fret-ui-kit/src/declarative/collapsible_motion.rs`
  - `ecosystem/fret-ui-kit/src/primitives/collapsible.rs`
- Shadcn overlays that use Presence now read durations/easing from theme tokens (refresh-rate stable):
  - `ecosystem/fret-ui-shadcn/src/{context_menu,dropdown_menu,hover_card,menubar,popover,select,tooltip}.rs`
- Shadcn accordion/collapsible now read duration/easing from theme tokens (semantic-first; refresh-rate stable):
  - `ecosystem/fret-ui-shadcn/src/accordion.rs`
  - `ecosystem/fret-ui-shadcn/src/collapsible.rs`
- Hover intent (tooltip/hover-card delays) scales 60Hz ticks to frame ticks for refresh-rate stability:
  - `ecosystem/fret-ui-kit/src/declarative/hover_intent.rs`

Diag gates:

- Sidebar toggle under fixed frame delta: `tools/diag-scripts/ui-gallery-sidebar-toggle-fixed-frame-delta.json`
- Dropdown menu open/close under fixed frame delta: `tools/diag-scripts/ui-gallery-dropdown-open-fixed-frame-delta.json`
- Accordion FAQ toggle under fixed frame delta: `tools/diag-scripts/ui-gallery-accordion-faq-toggle-fixed-frame-delta.json`
- Drawer snap points drag + settle: `tools/diag-scripts/ui-gallery-drawer-snap-points-drag-settle.json`
- Drawer snap points retarget + settle under fixed frame delta: `tools/diag-scripts/ui-gallery-drawer-snap-points-drag-retarget-settle-fixed-frame-delta.json`
- Overlay dialog open/close under fixed frame delta: `tools/diag-scripts/ui-gallery-overlay-dialog-open-close-fixed-frame-delta.json`
- Sonner open/close under fixed frame delta: `tools/diag-scripts/ui-gallery-sonner-open-close-fixed-frame-delta.json`
- Sonner interrupt under fixed frame delta: `tools/diag-scripts/ui-gallery-sonner-interrupt-fixed-frame-delta.json`
- shadcn Tabs shared indicator moves (pixels changed under fixed delta): `tools/diag-scripts/ui-gallery-alert-tabs-shared-indicator-pixels-changed-fixed-frame-delta.json`
- Docking split fraction re-target mid-flight (layout.expand) under fixed frame delta:
  - `tools/diag-scripts/docking-demo-split-toggle-retarget-fixed-frame-delta.json`
- Material3 tabs indicator moves (pixels changed under fixed delta): `tools/diag-scripts/ui-gallery-material3-tabs-indicator-pixels-changed-fixed-frame-delta.json`
- Material3 navigation bar active indicator moves (pixels changed under fixed delta): `tools/diag-scripts/ui-gallery-material3-navigation-bar-indicator-pixels-changed-fixed-frame-delta.json`
- Material3 navigation rail active indicator moves (pixels changed under fixed delta): `tools/diag-scripts/ui-gallery-material3-navigation-rail-indicator-pixels-changed-fixed-frame-delta.json`

Refresh-rate sanity (local):

- Run the same suite at 60Hz-ish and 120Hz-ish fixed deltas; wall-time completion should feel consistent:
  - `cargo run -p fretboard -- diag suite ui-gallery-motion-pilot --fixed-frame-delta-ms 16 --launch -- cargo run -p fret-ui-gallery --release`
  - `cargo run -p fretboard -- diag suite ui-gallery-motion-pilot --fixed-frame-delta-ms 8 --launch -- cargo run -p fret-ui-gallery --release`
