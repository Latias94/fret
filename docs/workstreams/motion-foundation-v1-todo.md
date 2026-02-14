# Motion Foundation (v1) — TODO Tracker

Status: Active (foundation landed; recipe parity follow-ups planned)

This document tracks cross-cutting TODOs for:

- `docs/workstreams/motion-foundation-v1.md`
- `docs/workstreams/motion-foundation-v1-milestones.md`

Tracking format:

- Status: `[ ]` open, `[~]` in progress, `[x]` done, `[!]` blocked
- ID: `MF-MOTION-{area}-{nnn}`

When completing an item, prefer leaving 1–3 evidence anchors:

- file paths + key functions/tests
- and/or a `fretboard diag` script/suite name

## P0 — Landed foundation (Duration-first + overlay motion)

- [x] MF-MOTION-time-001 Use wall-time `Duration` as the canonical unit for production UX motion.
  - Evidence:
    - `ecosystem/fret-ui-kit/src/declarative/transition.rs`
- [x] MF-MOTION-time-002 Make overlay transitions refresh-rate stable (60Hz tick derivation + frame scaling).
  - Evidence:
    - `ecosystem/fret-ui-kit/src/declarative/transition.rs`
    - `ecosystem/fret-ui-kit/src/overlay_controller.rs`
- [x] MF-MOTION-theme-001 Source shadcn durations + easing from theme tokens (overrideable).
  - Evidence:
    - `ecosystem/fret-ui-kit/src/declarative/overlay_motion.rs`
- [x] MF-MOTION-diag-001 Add deterministic diag gates for motion-critical overlays under fixed `delta`.
  - Evidence:
    - `tools/diag-scripts/ui-gallery-sidebar-toggle-fixed-frame-delta.json`
    - `tools/diag-scripts/ui-gallery-dropdown-open-fixed-frame-delta.json`

## P0 — Landed foundation (headless motion + drivers)

- [x] MF-MOTION-headless-001 Add portable headless motion primitives (tween/spring/friction/inertia).
  - Evidence:
    - `ecosystem/fret-ui-headless/src/motion/`
- [x] MF-MOTION-kit-001 Add `MotionValue` and a unified driver surface in `fret-ui-kit`.
  - Evidence:
    - `ecosystem/fret-ui-kit/src/declarative/motion_value.rs`
- [x] MF-MOTION-feel-001 Wire one “physics feel” interaction (drag release → settle) using velocity projection.
  - Evidence:
    - `crates/fret-ui/src/action.rs`
    - `ecosystem/fret-ui-shadcn/src/drawer.rs`
    - `tools/diag-scripts/ui-gallery-drawer-snap-points-drag-settle.json`

## P1 — Animata-informed recipe parity pilot

- [x] MF-MOTION-recipes-001 Choose a 3-recipe pilot set and link the upstream references.
  - Recommended set:
    - Sidebar collapse/expand
    - Drawer or Sheet (drag + release)
    - Modal/Dialog (presence + focus/dismiss choreography)
  - Upstream references:
    - `repo-ref/animata/animata/overlay/`
    - `repo-ref/ui/apps/v4/content/docs/components/`
    - `repo-ref/ui/apps/v4/registry/new-york-v4/ui/*.tsx`
  - Evidence:
    - `docs/workstreams/motion-foundation-v1.md` (section "Recipe alignment matrix (draft)")

- [x] MF-MOTION-recipes-002 For each pilot recipe, document the intended motion channels and token keys.
  - Channels to be explicit about:
    - opacity, translate/scale, clip/size reveal, blur (if used), backdrop fade.
  - Evidence:
    - `docs/workstreams/motion-foundation-v1.md` (section "Recipe alignment matrix (draft)")

- [x] MF-MOTION-diag-002 Add a “motion pilot” diag suite (fixed `delta`) with 1 script per recipe.
  - Expected output:
    - deterministic bundles under `--fixed-frame-delta-ms 16`
  - Suite:
    - `fretboard diag suite ui-gallery-motion-pilot` (run with `--fixed-frame-delta-ms 16`)
  - Existing gates:
    - `tools/diag-scripts/ui-gallery-sidebar-toggle-fixed-frame-delta.json`
    - `tools/diag-scripts/ui-gallery-drawer-snap-points-drag-retarget-settle-fixed-frame-delta.json`
    - `tools/diag-scripts/ui-gallery-overlay-dialog-open-close-fixed-frame-delta.json`
    - `tools/diag-scripts/ui-gallery-sonner-interrupt-fixed-frame-delta.json`

- [x] MF-MOTION-diag-003 Add a high-refresh-rate sanity check run (optional): 60Hz vs 120Hz should match wall-time completion.
  - Evidence (local sanity runs; deterministic fixed delta):
    - `fretboard diag suite ui-gallery-motion-pilot --fixed-frame-delta-ms 16`
    - `fretboard diag suite ui-gallery-motion-pilot --fixed-frame-delta-ms 8` (120Hz-ish)
  - Note: keep this as a best-effort local gate if runner refresh rate is not easily forced.

Per-recipe tracking (pilot follow-ups):

- [x] MF-MOTION-pilot-010 Dialog fixed-delta open/close script + evidence anchors.
  - Evidence:
    - `tools/diag-scripts/ui-gallery-overlay-dialog-open-close-fixed-frame-delta.json`
- [x] MF-MOTION-pilot-030 Sidebar: confirm layout-affecting choreography expectations (do we need FLIP?).
  - Decision:
    - For Sidebar collapse/expand, keep the label subtree present and rely on width transition +
      overflow clipping (CSS-like), rather than FLIP.
  - Evidence:
    - `ecosystem/fret-ui-shadcn/src/sidebar.rs` (menu button label opacity; no branch pop)
- [x] MF-MOTION-pilot-035 Accordion/collapsible: token-driven measured-height toggle (duration + easing) with a fixed-delta gate.
  - Evidence:
    - `ecosystem/fret-ui-shadcn/src/accordion.rs`
    - `ecosystem/fret-ui-shadcn/src/collapsible.rs`
    - `ecosystem/fret-ui-kit/src/declarative/collapsible_motion.rs`
    - `tools/diag-scripts/ui-gallery-accordion-faq-toggle-fixed-frame-delta.json`
- [x] MF-MOTION-pilot-020 Toast fixed-delta interrupt coverage.
  - Evidence:
    - `tools/diag-scripts/ui-gallery-sonner-interrupt-fixed-frame-delta.json`
    - `crates/fret-diag/src/lib.rs` (builtin suite: `ui-gallery-motion-pilot`)
- [x] MF-MOTION-pilot-040 Drawer/sheet: add a mid-flight retarget scenario (drag, reverse, release).
  - Evidence:
    - `tools/diag-scripts/ui-gallery-drawer-snap-points-drag-retarget-settle-fixed-frame-delta.json`
    - `crates/fret-diag/src/lib.rs` (builtin suite: `ui-gallery-motion-pilot`)

## P1 — Spring feel calibration (native-like)

- [x] MF-MOTION-spring-001 Decide the public theme-facing spring token shape (ecosystem-level).
  - Options to document:
    - stiffness/damping/mass (physics params)
    - duration/bounce (authoring-friendly, derived to physics)
  - Decision (v1):
    - `duration_ms` + `number` bounce tokens (Flutter-style), derived to `SpringDescription`.
  - Evidence:
    - `docs/workstreams/motion-foundation-v1.md` (section "Spring tokens (v1)")
    - `ecosystem/fret-ui-kit/src/declarative/motion_springs.rs`

- [x] MF-MOTION-spring-002 Add a small “spring cookbook” section with recommended presets.
  - Presets to include (examples):
    - standard, emphasized, overshoot, critically-damped, “snappy”
  - Evidence:
    - `docs/workstreams/motion-foundation-v1.md` (section "Spring cookbook (starter presets; tune with diag gates)")

- [x] MF-MOTION-spring-003 Add at least one diag script that exercises re-targeting mid-flight (no restart stutter).
  - Evidence:
    - `tools/diag-scripts/ui-gallery-drawer-snap-points-spring-midflight-retarget-fixed-frame-delta.json`
    - `crates/fret-diag/src/lib.rs` (builtin suite: `ui-gallery-motion-pilot`)

## P1 — Token taxonomy (shadcn ↔ Material 3)

- [x] MF-MOTION-tokens-001 Document a stable semantic key scheme for duration/easing/spring.
  - Goal: component ecosystems request “semantic motion”, not raw numbers.
  - Evidence:
    - `docs/workstreams/motion-foundation-v1.md` (section "Token taxonomy (v1; conventions we can keep stable)")

- [x] MF-MOTION-tokens-002 Add shadcn aliases (already present) and define the M3 mapping table.
  - Evidence:
    - `docs/workstreams/motion-foundation-v1.md` (section "Pragmatic mapping table (shadcn numeric durations ↔ M3 duration tokens)")
    - `ecosystem/fret-ui-kit/src/declarative/overlay_motion.rs`
    - `ecosystem/fret-ui-material3/src/tokens/material_web_v30.rs`
    - `ecosystem/fret-ui-material3/src/foundation/motion_scheme.rs`

## P2 — Layout-affecting motion (optional follow-up)

- [x] MF-MOTION-layout-001 Choose and document the layout transition strategy (FLIP-like vs explicit choreography).
  - Decision (v1):
    - Explicit choreography + stable structure; avoid a generic FLIP layer.
  - Evidence:
    - `docs/workstreams/motion-foundation-v1.md` (section "Layout-affecting motion (v1)")

- [x] MF-MOTION-layout-002 Add one pilot (sidebar content reflow or tabs indicator) with a deterministic diag gate.
  - Pilot:
    - Material3 Tabs active indicator (measurement-driven + refresh-rate stable motion)
  - Evidence:
    - `ecosystem/fret-ui-material3/src/tabs.rs` (`primary_tab_list_indicator`)
    - `tools/diag-scripts/ui-gallery-material3-tabs-indicator-pixels-changed-fixed-frame-delta.json`
