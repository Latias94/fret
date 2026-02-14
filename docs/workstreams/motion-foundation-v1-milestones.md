# Motion Foundation (v1) — Milestones

This file tracks milestone gates for the motion foundation workstream.

See:

- Overview: `docs/workstreams/motion-foundation-v1.md`
- Task list: `docs/workstreams/motion-foundation-v1-todo.md`

## M0 — Refresh-rate stable time model (Duration-first)

Deliverables:

- Production UX durations are expressed as `Duration` (wall-time), not “60fps ticks”.
- Drivers convert `Duration -> 60Hz ticks -> frame ticks` (refresh-rate stable).
- Diagnostics can force fixed `delta` for deterministic scripted repros.

Status: Landed

Evidence:

- `ecosystem/fret-ui-kit/src/declarative/transition.rs`
- `ecosystem/fret-ui-kit/src/declarative/presence.rs`
- `ecosystem/fret-ui-kit/src/declarative/hover_intent.rs`
- `tools/diag-scripts/ui-gallery-sidebar-toggle-fixed-frame-delta.json`
- `tools/diag-scripts/ui-gallery-dropdown-open-fixed-frame-delta.json`

## M1 — shadcn motion tokens (duration + easing)

Deliverables:

- shadcn overlay motion uses theme tokens (overrideable) for durations + cubic-bezier easing.
- Components do not hard-code tick presets for production UX durations.

Status: Landed

Evidence:

- `ecosystem/fret-ui-kit/src/declarative/overlay_motion.rs`
- `ecosystem/fret-ui-shadcn/src/sheet.rs`
- `ecosystem/fret-ui-shadcn/src/{context_menu,dropdown_menu,hover_card,menubar,popover,select,tooltip}.rs`

## M2 — Headless motion primitives (portable)

Deliverables:

- A headless motion module exists in `fret-ui-headless` (no theme, no scheduling).
- At least: tween + spring + friction/inertia primitives with deterministic stepping.

Status: Landed

Evidence:

- `ecosystem/fret-ui-headless/src/motion/`
- `ecosystem/fret-ui-headless/src/motion/tween.rs`
- `ecosystem/fret-ui-headless/src/motion/spring.rs`
- `ecosystem/fret-ui-headless/src/motion/friction.rs`
- `ecosystem/fret-ui-headless/src/motion/inertia.rs`

## M3 — UI-kit motion drivers (MotionValue)

Deliverables:

- `fret-ui-kit` exposes a small driver surface that binds headless motion to the frame clock.
- At least one “physics feel” interaction uses `MotionValue` and pointer velocity projection.

Status: Landed

Evidence:

- `ecosystem/fret-ui-kit/src/declarative/motion.rs`
- `ecosystem/fret-ui-kit/src/declarative/motion_value.rs`
- `crates/fret-ui/src/action.rs` (pointer velocity snapshots)
- `ecosystem/fret-ui-shadcn/src/drawer.rs`
- `tools/diag-scripts/ui-gallery-drawer-snap-points-drag-settle.json`

## M4 — Recipe parity pilot (Animata-informed)

Deliverables:

- Pick 3 high-value recipes (recommended: sidebar, drawer/sheet, modal/dialog).
- For each recipe:
  - document the intended motion channels (opacity/transform/clip/blur),
  - define tokens (duration/easing and, if needed, spring params),
  - add a deterministic diag gate under fixed `delta`.

Status: Landed (baseline motion pilot suite + fixed-delta gates)

Exit criteria:

- A small “motion pilot suite” of scripts exists and runs deterministically under
  `--fixed-frame-delta-ms 16` (native runner).
- Evidence anchors exist in the workstream TODO tracker.

Evidence:

- `docs/workstreams/motion-foundation-v1.md` (section "Recipe alignment matrix (draft)")
- `tools/diag-scripts/ui-gallery-sidebar-toggle-fixed-frame-delta.json`
- `tools/diag-scripts/ui-gallery-overlay-dialog-open-close-fixed-frame-delta.json`
- `tools/diag-scripts/ui-gallery-sonner-open-close-fixed-frame-delta.json`
- `tools/diag-scripts/ui-gallery-sonner-interrupt-fixed-frame-delta.json`
- `tools/diag-scripts/ui-gallery-drawer-snap-points-drag-retarget-settle-fixed-frame-delta.json`
- `crates/fret-diag/src/lib.rs` (builtin suite: `ui-gallery-motion-pilot`)

## M5 — Motion token taxonomy (shadcn ↔ Material 3)

Deliverables:

- A stable semantic key scheme is documented for:
  - duration (short/medium/long + “component semantic” keys),
  - easing (standard/emphasized/decelerate/accelerate),
  - spring (stiffness/damping or duration+bounce).
- shadcn tokens remain as aliases (ecosystem-level), not mechanism-layer contracts.

Status: Landed (taxonomy doc + mapping table)

Evidence:

- `docs/workstreams/motion-foundation-v1.md` (section "Token taxonomy (v1; conventions we can keep stable)")
- `docs/workstreams/motion-foundation-v1.md` (section "Pragmatic mapping table (shadcn numeric durations ↔ M3 duration tokens)")
- `ecosystem/fret-ui-kit/src/declarative/overlay_motion.rs`
- `ecosystem/fret-ui-material3/src/tokens/material_web_v30.rs`
- `ecosystem/fret-ui-material3/src/foundation/motion_scheme.rs`

## M6 — Layout-affecting motion (optional v1 follow-up)

Deliverables:

- A recommended approach for layout transitions is chosen and documented:
  - FLIP-like approach for “measure -> animate delta -> settle to layout” (ecosystem policy),
  - or explicit layout + animation choreography rules (opt-in per component).
- Interrupt/re-target rules are tested (no “restart stutter”).

Status: Landed (pilot)

Evidence:

- Layout strategy doc (explicit choreography, no generic FLIP layer): `docs/workstreams/motion-foundation-v1.md` (section "Layout-affecting motion (v1)")
- Layout-expand motion token seeded in shadcn themes (cross-ecosystem semantic key):
  - `ecosystem/fret-ui-shadcn/src/shadcn_themes.rs`
- Layout-expand pilot (Animata: Expandable carousel) consumes semantic duration + easing:
  - `apps/fret-ui-gallery/src/ui/pages/carousel.rs`
- Deterministic retarget/interrupt gate (fixed delta; click item 3 then item 4 mid-flight):
  - `tools/diag-scripts/ui-gallery-carousel-expandable-fixed-frame-delta.json`
