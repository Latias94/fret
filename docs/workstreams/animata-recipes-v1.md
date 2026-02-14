# Animata Recipes Alignment (v1)

Status: Active (semantic-first; v1 semantics frozen; outcomes gated with deterministic diag scripts)

See:

- Motion foundation (time model + primitives + drivers): `docs/workstreams/motion-foundation-v1.md`
- TODO tracker: `docs/workstreams/animata-recipes-v1-todo.md`

This workstream defines how we use Animata as a **UX spec source** for motion-heavy interaction
recipes in a GPU-first custom renderer (Fret).

Animata is DOM-first and frequently uses Framer Motion; we do **not** port its runtime model. We
align to *outcomes*:

- timing and sequencing,
- interruption and re-targeting rules,
- accessibility behavior (focus/dismiss),
- and “what moves, when”.

## Goals (v1)

1) Establish a **semantic motion vocabulary** (recipe-level intent) that component ecosystems can
   share.
2) Select a **small, common set** of Animata recipes that map cleanly to real app UIs.
3) For each selected recipe, define:
   - motion channels (opacity/scale/translate/clip/blur/layout),
   - which motion primitive(s) we expect to use (tween/spring/inertia/transition timeline),
   - optional theme token keys,
   - and at least one deterministic `fretboard diag` gate (fixed frame delta).

Non-goals:

- Implement every Animata component.
- Lock a mechanism-layer contract in `crates/fret-ui` for a specific animation.
- Copy DOM/CSS behavior 1:1 when it conflicts with a custom renderer’s semantics/hit-testing.

## Layering rules (repeatable “where does this live?”)

- `crates/fret-ui`: mechanisms/contracts (scheduling hooks, semantics, hit-testing, overlay roots).
- `ecosystem/fret-ui-headless`: portable motion math/state machines (tween/spring/inertia).
- `ecosystem/fret-ui-kit`: drivers + ergonomic wrappers + token lookups (policy).
- `ecosystem/fret-ui-shadcn`, `ecosystem/fret-ui-material3`: recipes/components (policy consumers).

## Semantic motion vocabulary (v1)

These are “recipe intents” (what the user perceives), not implementation details. The main value is
that multiple ecosystems can reuse the same intent keys and share diag gates.

Frozen list (v1):

- `presence.enter`, `presence.exit`
- `shared_indicator.move`
- `collapsible.toggle`
- `layout.expand`
- `hover_micro` (micro-interactions; short, non-layout motion)
- `drag_release_settle` (inertia continuity → settle)
- `stack.shift` (stack reflow + interruption rules)

| Intent (semantic) | Typical channels | Primitive(s) | Notes |
| --- | --- | --- | --- |
| `presence.enter/exit` | opacity + scale + optional blur | Duration tween or spring | Used by dialogs, popovers, toast stacks |
| `shared_indicator.move` | x/y + width/height | spring (Duration-based) | Tabs underline, nav active pill |
| `collapsible.toggle` | height/clip + opacity | Duration tween timeline | “height: auto” needs measure+animate choreography |
| `layout.expand` | width/flex-grow + blur/opacity | explicit choreography or FLIP-like (opt-in) | Prefer stable structure + overflow clip |
| `hover_micro` | scale/opacity/state-layer | short tween/spring | Avoid re-layout; keep hit-testing consistent |
| `drag_release_settle` | translate + scrim opacity | inertia → spring settle | Drawer/sheet; needs velocity continuity |
| `stack.shift` | translateY + opacity | tween timeline + interruption rules | Toast stack, list insert/remove |

## Token guidance (semantic-first; optional)

We already have shadcn and Material 3 token schemes. For Animata-style recipes, prefer **semantic**
keys and provide aliases rather than inventing a separate numeric scale.

Decision (v1):

- `duration.motion.*` / `easing.motion.*` / `number.motion.spring.*` are the **canonical** cross-ecosystem
  semantic keys for motion authoring.
- `duration.shadcn.motion.*` / `easing.shadcn.motion.*` remain supported as **ecosystem-scoped aliases**
  (useful when shadcn wants per-ecosystem tuning without polluting the global semantic namespace).
- Material 3 ecosystems keep `md.sys.motion.*` as their primary scheme; mapping to `*.motion.*` is optional
  and theme-driven (not a hard contract).

Recommended semantic keys (ecosystem-level; optional):

- Duration:
  - `duration.motion.presence.enter`, `duration.motion.presence.exit`
  - `duration.motion.collapsible.toggle`
  - `duration.motion.layout.expand`
  - `duration.motion.stack.shift`, `duration.motion.stack.shift.stagger`
  - `duration.motion.tooltip.enter`, `duration.motion.tooltip.exit`
- Easing:
  - `easing.motion.standard`, `easing.motion.emphasized`
  - `easing.motion.stack.shift`
  - `easing.motion.collapsible.toggle`
  - `easing.motion.layout.expand`
- Spring (authoring-friendly):
  - `duration.motion.spring.shared_indicator` + `number.motion.spring.shared_indicator.bounce`
  - `duration.motion.spring.drag_release_settle` + `number.motion.spring.drag_release_settle.bounce`

Mapping rule of thumb:

- When resolving a token for a shadcn recipe, prefer shadcn-scoped keys first, then fall back to the
  canonical semantic keys:
  - `duration.shadcn.motion.<...>` → `duration.motion.<...>`
  - `easing.shadcn.motion.<...>` → `easing.motion.<...>`
- Material 3 ecosystems should continue to use `md.sys.motion.*` keys; semantic keys can map to M3
  values in themes when desired, but must remain optional.

## Recipe alignment matrix (semantic-first)

This table is intentionally spec-first. “Status: Planned” means we have not committed to a specific
component API yet; we only commit to the semantic intent + gate.

| ID | Recipe | Animata source(s) | Fret target(s) | Semantic intent | Gate (deterministic) | Status |
| --- | --- | --- | --- | --- | --- | --- |
| AR-OVERLAY-001 | Modal presence (blur + spring) | `repo-ref/animata/animata/overlay/modal.tsx` | `ecosystem/fret-ui-shadcn/src/dialog.rs`, `ecosystem/fret-ui-shadcn/src/alert_dialog.rs` | `presence.enter/exit` (+ optional `hover_micro`) | `tools/diag-scripts/ui-gallery-overlay-dialog-open-close-fixed-frame-delta.json` + `tools/diag-scripts/ui-gallery-overlay-dialog-glass-backdrop-open-close-fixed-frame-delta.json` | Landed (baseline + glass backdrop; rotate variant TODO) |
| AR-ACCORDION-001 | FAQ accordion (height:auto + fade) | `repo-ref/animata/animata/accordion/faq.tsx` | UI Gallery shadcn accordion demo | `collapsible.toggle` | `tools/diag-scripts/ui-gallery-accordion-faq-toggle-fixed-frame-delta.json` (run with `--check-pixels-changed ui-gallery-accordion-demo-returns-item`) | Landed (token-driven duration/easing + gate) |
| AR-TABS-001 | Nav tabs shared indicator | `repo-ref/animata/animata/container/nav-tabs.tsx` | `ecosystem/fret-ui-material3/src/tabs.rs` + shadcn tabs recipes | `shared_indicator.move` | Landed (M3): `tools/diag-scripts/ui-gallery-material3-tabs-indicator-pixels-changed-fixed-frame-delta.json` | Landed (M3) / Planned (shadcn) |
| AR-TABS-002 | Fluid tabs (indicator + content switch) | `repo-ref/animata/animata/card/fluid-tabs.tsx` | shadcn tabs + content presence | `shared_indicator.move` + `presence.enter/exit` | Add a fixed-delta script after shadcn tabs MVP | Planned |
| AR-NAV-001 | Navigation active pill (bar/rail) | (Animata: reuse `shared_indicator.move` intent) | `ecosystem/fret-ui-material3/src/navigation_bar.rs`, `ecosystem/fret-ui-material3/src/navigation_rail.rs` | `shared_indicator.move` | `tools/diag-scripts/ui-gallery-material3-navigation-bar-indicator-pixels-changed-fixed-frame-delta.json` + `tools/diag-scripts/ui-gallery-material3-navigation-rail-indicator-pixels-changed-fixed-frame-delta.json` | Landed |
| AR-CAROUSEL-001 | Expandable carousel (flex-grow + blur) | `repo-ref/animata/animata/carousel/expandable.tsx` | UI Gallery carousel page (Animata: Expandable) | `layout.expand` (+ `hover_micro`) | `tools/diag-scripts/ui-gallery-carousel-expandable-fixed-frame-delta.json` | Landed (pilot; size interpolation baseline) |
| AR-TOAST-001 | Toast stack shift + interrupt | (Animata: use `stack.shift` intent) | `ecosystem/fret-ui-shadcn/src/sonner.rs` | `stack.shift` | `tools/diag-scripts/ui-gallery-sonner-interrupt-fixed-frame-delta.json` + `tools/diag-scripts/ui-gallery-sonner-stack-shift-stagger-fixed-frame-delta.json` + `tools/diag-scripts/ui-gallery-sonner-stack-shift-stagger-interrupt-fixed-frame-delta.json` | Landed (baseline + staggered reflow) |
| AR-TOAST-002 | Swipe-to-dismiss inertia | (Animata: use `drag_release_settle` intent) | `ecosystem/fret-ui-shadcn/src/sonner.rs` | `drag_release_settle` | `tools/diag-scripts/ui-gallery-sonner-swipe-dismiss-fixed-frame-delta.json` (run with `--check-pixels-changed ui-gallery-sonner-demo-toast-swipe`) | Landed (gate script) |
| AR-NAVMENU-001 | NavigationMenu viewport motion | (Animata: use `presence.enter/exit` + `layout.expand`) | `ecosystem/fret-ui-shadcn/src/navigation_menu.rs` | `layout.expand` + interruption | `tools/diag-scripts/ui-gallery-navigation-menu-viewport-fixed-frame-delta.json` (run with `--check-pixels-changed ui-gallery-navigation-menu-demo-viewport`) | Landed (gate script) |

## Definition of done (for a recipe row)

- A stable `test_id` surface exists for automation targets.
- A deterministic diag script exists under fixed `delta` (prefer `--fixed-frame-delta-ms 16`).
- The recipe uses `fret-ui-kit` drivers (Duration-first) rather than per-component frame math.
- Any DOM-specific assumptions are translated explicitly (hit-testing, overlay capture, focus).
