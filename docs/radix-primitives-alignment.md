# Radix Primitives Alignment (Fret Mapping)

This document maps Radix UI Primitives concepts (upstream: <https://github.com/radix-ui/primitives>; pinned locally, see `docs/repo-ref.md`) to Fret’s
layered architecture:

- `crates/fret-ui` (mechanism-only runtime substrate)
- `crates/fret-components-ui` (headless primitives + policy helpers + overlay orchestration)
- `crates/fret-components-shadcn` (shadcn-aligned taxonomy + recipes)

This is **not** an API-compatibility goal. We port **behavior outcomes**, not React/DOM
implementations.

See also:

- Runtime contract surface: `docs/adr/0066-fret-ui-runtime-contract-surface.md`
- Policy split via action hooks: `docs/adr/0074-component-owned-interaction-policy-and-runtime-action-hooks.md`
- UI behavior reference stack: `docs/reference-stack-ui-behavior.md`

## Key principle

Radix “primitives” span both mechanism and policy in a web/React setting. In Fret, they split:

- Mechanism belongs to `fret-ui` (focus/capture/overlay roots, outside-press observation, placement).
- Policy composition belongs to `fret-components-ui` (headless state machines + action hook wiring).
- shadcn recipes belong to `fret-components-shadcn` (names + default styling + composition).

## Mapping table (concept → Fret)

| Radix concept | What it does (outcome) | Fret mechanism | Fret policy / headless | Notes / status |
| --- | --- | --- | --- | --- |
| Portal | Render outside normal tree; avoid clipping | Multi-root overlays substrate (ADR 0011) | `OverlayController` + per-window orchestration | Fret uses per-window overlay roots, not DOM portals |
| Presence | Animate mount/unmount | (n/a) | `fret-components-ui::primitives::presence` + `headless::presence::FadePresence` | Component-level helper (time-source agnostic) |
| VisuallyHidden | Keep content in a11y tree, not visually rendered | `fret-ui::element::Semantics` (paint transparent) | `fret-components-ui::primitives::visually_hidden` | Useful for screen-reader-only labels and hidden descriptions |
| DismissableLayer | Escape / outside press dismissal hooks | Outside-press observer pass (ADR 0069) + key routing | `fret-components-ui::primitives::dismissable_layer` + overlay policies | Policy decides what to do on dismiss request |
| FocusScope | Contain focus traversal | `fret-ui::element::FocusScope` (ADR 0068) | `fret-components-ui::primitives::focus_scope::focus_trap` | Trap semantics are policy-level composition |
| RovingFocus | Arrow-key item focus without tab stops | Runtime tracks focusability + pressable focus | `primitives::roving_focus_group` + `headless::{menu_nav, roving_focus, typeahead}` | Ensure items are not necessarily Tab stops |
| Menu | Submenus, roving focus, typeahead, pointer grace | Overlay roots + focus + outside-press observation | `fret-components-ui::primitives::menu` (WIP; includes `menu::pointer_grace_intent`) | DropdownMenu/Menubar/ContextMenu are wrappers |
| Popper / placement | Anchored placement, flip/shift/size/offset/arrow | `fret-ui::overlay_placement` (ADR 0064) | `fret-components-ui::overlay::*` helpers | Arrow positioning is supported; visuals live in components/recipes |
| Arrow | Render a positioned arrow pointing at an anchor | `fret-ui::overlay_placement::ArrowLayout` | `fret-components-ui::primitives::popper` helpers | Visual styling (diamond/border) lives in shadcn recipes |
| TooltipProvider | Shared open-delay + skip-delay window | (n/a) | `fret-components-ui::primitives::tooltip_provider` + `primitives::tooltip_delay_group` | Provider stack is driven per-frame (service) |
| Collection semantics | “Item X of Y”, roles, disabled skipping | Semantics snapshot (ADR 0033) | `fret-components-ui::primitives::collection` | Collection metadata is required for menus/lists |
| Active descendant | Highlight moves while focus stays in input | Semantics schema + snapshot + AccessKit mapping (ADR 0073) | `fret-components-ui::headless::cmdk_selection` + component wiring | Highest-leverage a11y closure item |

## Code entry points (practical)

- Overlay substrate + outside-press observation (mechanism): `crates/fret-ui/src/tree/mod.rs`
- Overlay placement solver (mechanism): `crates/fret-ui/src/overlay_placement/`
- Overlay anchoring helpers (policy helper): `crates/fret-components-ui/src/overlay.rs`
- Per-window overlay requests/presence (policy helper): `crates/fret-components-ui/src/overlay_controller.rs`
- Unstable overlay orchestration internals: `crates/fret-components-ui/src/window_overlays/`
- Headless primitives (policy/state machines): `crates/fret-components-ui/src/headless/`
- Semantics schema (portable contract): `crates/fret-core/src/semantics.rs`

## “Where should new work go?” (rules of thumb)

- If it changes **hit testing, routing, or semantics snapshot production**, it is likely `fret-ui`
  and must be justified by an ADR + tests (ADR 0066).
- If it is a **state machine** or **interaction policy composition**, it belongs in
  `fret-components-ui` (ADR 0074 / ADR 0090).
- If it is **shadcn naming or default styling**, it belongs in `fret-components-shadcn`.

## Current gaps worth tracking (from audits)

- (Done) Popper + Arrow primitive wiring: `fret-components-ui::primitives::popper` (layout + wrapper insets helpers).
- DropdownMenu submenu ergonomics: safe-hover close corridor now wired via `DismissibleLayer` pointer-move observers; remaining: intent heuristics + focus transfer closure.
- (Done) cmdk-style filtering/scoring headless primitive: `fret-components-ui::headless::cmdk_score`.
