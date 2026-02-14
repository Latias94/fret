# Animata Recipes Alignment (v1) — TODO Tracker

Status: Active (semantic-first; recipe selection in progress)

This document tracks TODOs for:

- `docs/workstreams/animata-recipes-v1.md`
- Motion foundation dependencies: `docs/workstreams/motion-foundation-v1.md`

Tracking format:

- Status: `[ ]` open, `[~]` in progress, `[x]` done, `[!]` blocked
- ID: `AR-{area}-{nnn}`

When completing an item, prefer leaving 1–3 evidence anchors:

- file paths + key functions/tests
- and/or a `tools/diag-scripts/*.json` gate

## P0 — Semantics and scope (document-first)

- [x] AR-SEM-001 Finalize the semantic motion vocabulary list (v1) and keep it stable across ecosystems.
  - Evidence:
    - `docs/workstreams/animata-recipes-v1.md` (section "Semantic motion vocabulary (v1)")
- [x] AR-SEM-002 Decide whether `duration.motion.*` becomes the canonical semantic namespace or stays as an alias layer over `duration.shadcn.motion.*`.
  - Decision notes:
    - Avoid mechanism coupling in `crates/fret-ui`.
    - Keep Material 3 `md.sys.motion.*` as the primary source for M3 ecosystems.
    - Decision (v1):
      - `duration.motion.*` / `easing.motion.*` / `number.motion.spring.*` are canonical semantic keys.
      - `duration.shadcn.motion.*` / `easing.shadcn.motion.*` are supported as ecosystem-scoped aliases.
  - Evidence:
    - `docs/workstreams/animata-recipes-v1.md` (section "Token guidance (semantic-first; optional)")

## P1 — Add deterministic gates for “missing rows”

- [x] AR-GATE-001 Add a fixed-delta diag gate for Animata FAQ accordion (height:auto + fade).
  - Animata source:
    - `repo-ref/animata/animata/accordion/faq.tsx`
  - Expected Fret target:
    - shadcn accordion/collapsible recipe surface
  - Output:
    - `tools/diag-scripts/ui-gallery-accordion-faq-toggle-fixed-frame-delta.json`
  - Evidence:
    - `tools/diag-scripts/ui-gallery-accordion-faq-toggle-fixed-frame-delta.json`

- [x] AR-GATE-002 Add a fixed-delta diag gate for NavigationMenu viewport motion (size interpolation + placement stability).
  - Upstream references:
    - shadcn: `repo-ref/ui/apps/v4/content/docs/components/navigation-menu.mdx`
    - Radix: navigation menu primitives
  - Fret target:
    - `ecosystem/fret-ui-shadcn/src/navigation_menu.rs`
  - Output:
    - `tools/diag-scripts/ui-gallery-navigation-menu-viewport-fixed-frame-delta.json`
  - Evidence:
    - `tools/diag-scripts/ui-gallery-navigation-menu-viewport-fixed-frame-delta.json`

- [x] AR-GATE-003 Add a fixed-delta diag gate for Sonner swipe-to-dismiss inertia + settle.
  - Fret target:
    - `ecosystem/fret-ui-shadcn/src/sonner.rs`
  - Output:
    - `tools/diag-scripts/ui-gallery-sonner-swipe-dismiss-fixed-frame-delta.json`
  - Evidence:
    - `tools/diag-scripts/ui-gallery-sonner-swipe-dismiss-fixed-frame-delta.json`

- [x] AR-GATE-004 Add a fixed-delta diag gate for Animata expandable carousel (layout.expand).
  - Animata source:
    - `repo-ref/animata/animata/carousel/expandable.tsx`
  - Output:
    - `tools/diag-scripts/ui-gallery-carousel-expandable-fixed-frame-delta.json`
  - Evidence:
    - `tools/diag-scripts/ui-gallery-carousel-expandable-fixed-frame-delta.json`
    - `apps/fret-ui-gallery/src/ui/pages/carousel.rs` (section "Animata: Expandable")

- [x] AR-GATE-005 Add a fixed-delta diag gate for Sonner `stack.shift` staggered reflow (non-expanded).
  - Goal:
    - When a new toast enters, existing stack items shift with a small stagger (closer to Sonner/web feel).
    - Scale changes should not “jump” when indices change.
  - Output:
    - `tools/diag-scripts/ui-gallery-sonner-stack-shift-stagger-fixed-frame-delta.json`
  - Evidence:
    - `ecosystem/fret-ui-kit/src/window_overlays/render.rs` (`toast_stack_shift_output`)
    - `tools/diag-scripts/ui-gallery-sonner-stack-shift-stagger-fixed-frame-delta.json`
    - `tools/diag-scripts/ui-gallery-sonner-stack-shift-stagger-interrupt-fixed-frame-delta.json`

## P1 — Optional: bring Animata “blurred backdrop” into a reusable recipe

- [x] AR-OVERLAY-010 Add a dialog variant or a separate recipe that uses backdrop blur (reduce-transparency aware).
  - Evidence anchors:
    - `ecosystem/fret-ui-kit/src/recipes/glass.rs`
    - `ecosystem/fret-ui-kit/src/declarative/glass.rs`
    - `ecosystem/fret-ui-shadcn/src/dialog.rs` (`DialogOverlayBackdrop::Glass`)
    - `apps/fret-ui-gallery/src/ui/previews/gallery/overlays/overlay/widgets.rs` (`dialog_glass`)
    - `tools/diag-scripts/ui-gallery-overlay-dialog-glass-backdrop-open-close-fixed-frame-delta.json`
  - Note:
    - Keep default shadcn dialog baseline conservative; make blur an explicit opt-in recipe.

## P2 — Missing primitives (only if needed by multiple recipes)

- [x] AR-PRIM-001 Add a small “stagger/sequence” helper surface in `fret-ui-headless` + `fret-ui-kit` if multiple recipes need it.
  - Goal:
    - Avoid re-implementing per-recipe ad-hoc stagger math.
  - Gate:
    - one deterministic script demonstrating staggered toast stack or list insert.
  - Evidence:
    - `ecosystem/fret-ui-headless/src/motion/stagger.rs`
    - `ecosystem/fret-ui-kit/src/headless/mod.rs`
    - `apps/fret-ui-gallery/src/ui/pages/motion_presets.rs` (stagger demo)
    - `tools/diag-scripts/ui-gallery-motion-presets-stagger-fixed-frame-delta.json`
