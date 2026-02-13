# Animata Recipes Alignment (v1)

Status: Draft (semantic-first; outcomes gated with deterministic diag scripts)

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

Recommended semantic keys (ecosystem-level; optional):

- Duration:
  - `duration.motion.presence.enter`, `duration.motion.presence.exit`
  - `duration.motion.collapsible.toggle`
  - `duration.motion.tooltip.enter`, `duration.motion.tooltip.exit`
- Easing:
  - `easing.motion.standard`, `easing.motion.emphasized`
- Spring (authoring-friendly):
  - `duration.motion.spring.shared_indicator` + `number.motion.spring.shared_indicator.bounce`
  - `duration.motion.spring.drag_release_settle` + `number.motion.spring.drag_release_settle.bounce`

Mapping rule of thumb:

- If a shadcn semantic key exists, keep it as the first lookup target and treat `*.motion.*` as an
  alias (or the other way around if we decide `duration.motion.*` becomes the “canonical” semantic
  namespace).
- Material 3 ecosystems should continue to use `md.sys.motion.*` keys; semantic keys can map to M3
  values in themes when desired, but must remain optional.

## Recipe alignment matrix (semantic-first)

This table is intentionally spec-first. “Status: Planned” means we have not committed to a specific
component API yet; we only commit to the semantic intent + gate.

| ID | Recipe | Animata source(s) | Fret target(s) | Semantic intent | Gate (deterministic) | Status |
| --- | --- | --- | --- | --- | --- | --- |
| AR-OVERLAY-001 | Modal presence (blur + spring) | `repo-ref/animata/animata/overlay/modal.tsx` | `ecosystem/fret-ui-shadcn/src/dialog.rs`, `ecosystem/fret-ui-shadcn/src/alert_dialog.rs` | `presence.enter/exit` (+ optional `hover_micro`) | `tools/diag-scripts/ui-gallery-overlay-dialog-open-close-fixed-frame-delta.json` | Landed (baseline; blur/rotate variant TODO) |
| AR-ACCORDION-001 | FAQ accordion (height:auto + fade) | `repo-ref/animata/animata/accordion/faq.tsx` | `ecosystem/fret-ui-shadcn/src/accordion.rs` (or `collapsible` primitives) | `collapsible.toggle` | Add: `tools/diag-scripts/ui-gallery-accordion-faq-toggle-fixed-frame-delta.json` | Planned |
| AR-TABS-001 | Nav tabs shared indicator | `repo-ref/animata/animata/container/nav-tabs.tsx` | `ecosystem/fret-ui-material3/src/tabs.rs` + shadcn tabs recipes | `shared_indicator.move` | Landed (M3): `tools/diag-scripts/ui-gallery-material3-tabs-indicator-pixels-changed-fixed-frame-delta.json` | Landed (M3) / Planned (shadcn) |
| AR-TABS-002 | Fluid tabs (indicator + content switch) | `repo-ref/animata/animata/card/fluid-tabs.tsx` | shadcn tabs + content presence | `shared_indicator.move` + `presence.enter/exit` | Add a fixed-delta script after shadcn tabs MVP | Planned |
| AR-NAV-001 | Navigation active pill (bar/rail) | (Animata: reuse `shared_indicator.move` intent) | `ecosystem/fret-ui-material3/src/navigation_bar.rs`, `ecosystem/fret-ui-material3/src/navigation_rail.rs` | `shared_indicator.move` | `tools/diag-scripts/ui-gallery-material3-navigation-bar-indicator-pixels-changed-fixed-frame-delta.json` + `tools/diag-scripts/ui-gallery-material3-navigation-rail-indicator-pixels-changed-fixed-frame-delta.json` | Landed |
| AR-CAROUSEL-001 | Expandable carousel (flex-grow + blur) | `repo-ref/animata/animata/carousel/expandable.tsx` | `ecosystem/fret-ui-*/carousel` (or a UI gallery recipe page) | `layout.expand` (+ `hover_micro`) | Add: `tools/diag-scripts/ui-gallery-carousel-expandable-fixed-frame-delta.json` | Planned |
| AR-TOAST-001 | Toast stack shift + interrupt | (Animata: use `stack.shift` intent) | `ecosystem/fret-ui-shadcn/src/sonner.rs` | `stack.shift` | `tools/diag-scripts/ui-gallery-sonner-interrupt-fixed-frame-delta.json` | Landed (baseline) |
| AR-TOAST-002 | Swipe-to-dismiss inertia | (Animata: use `drag_release_settle` intent) | `ecosystem/fret-ui-shadcn/src/sonner.rs` | `drag_release_settle` | Add: `tools/diag-scripts/ui-gallery-sonner-swipe-dismiss-fixed-frame-delta.json` | Planned |
| AR-NAVMENU-001 | NavigationMenu viewport motion | (Animata: use `presence.enter/exit` + `layout.expand`) | `ecosystem/fret-ui-shadcn/src/navigation_menu.rs` | `layout.expand` + interruption | Add: `tools/diag-scripts/ui-gallery-navigation-menu-viewport-fixed-frame-delta.json` | Planned |

## Definition of done (for a recipe row)

- A stable `test_id` surface exists for automation targets.
- A deterministic diag script exists under fixed `delta` (prefer `--fixed-frame-delta-ms 16`).
- The recipe uses `fret-ui-kit` drivers (Duration-first) rather than per-component frame math.
- Any DOM-specific assumptions are translated explicitly (hit-testing, overlay capture, focus).

