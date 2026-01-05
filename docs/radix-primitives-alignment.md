# Radix Primitives Alignment (Fret Mapping)

This document maps Radix UI Primitives concepts (upstream: <https://github.com/radix-ui/primitives>; pinned locally, see `docs/repo-ref.md`) to Fret’s
layered architecture:

- `crates/fret-ui` (mechanism-only runtime substrate)
- `ecosystem/fret-ui-kit` (headless primitives + policy helpers + overlay orchestration)
- `ecosystem/fret-ui-shadcn` (shadcn-aligned taxonomy + recipes)

This is **not** an API-compatibility goal. We port **behavior outcomes**, not React/DOM
implementations.

See also:

- Runtime contract surface: `docs/adr/0066-fret-ui-runtime-contract-surface.md`
- Policy split via action hooks: `docs/adr/0074-component-owned-interaction-policy-and-runtime-action-hooks.md`
- UI behavior reference stack: `docs/reference-stack-ui-behavior.md`

## Key principle

Radix “primitives” span both mechanism and policy in a web/React setting. In Fret, they split:

- Mechanism belongs to `fret-ui` (focus/capture/overlay roots, outside-press observation, placement).
- Policy composition belongs to `fret-ui-kit` (headless state machines + action hook wiring).
- shadcn recipes belong to `fret-ui-shadcn` (names + default styling + composition).

## Mapping table (concept → Fret)

| Radix concept | What it does (outcome) | Fret mechanism | Fret policy / headless | Notes / status |
| --- | --- | --- | --- | --- |
| Portal | Render outside normal tree; avoid clipping | Multi-root overlays substrate (ADR 0011) | `OverlayController` + per-window orchestration | Fret uses per-window overlay roots, not DOM portals |
| Presence | Animate mount/unmount | (n/a) | `fret-ui-kit::primitives::presence` + `headless::presence::FadePresence` | Component-level helper (time-source agnostic) |
| VisuallyHidden | Keep content in a11y tree, not visually rendered | `fret-ui::element::Semantics` (paint transparent) | `fret-ui-kit::primitives::visually_hidden` | Useful for screen-reader-only labels and hidden descriptions |
| DismissableLayer | Escape / outside press dismissal hooks | Outside-press observer pass (ADR 0069) + key routing | `fret-ui-kit::primitives::dismissable_layer` + overlay policies | Supports branches + focus-outside dismissal via overlay orchestration; click-through by default, with optional “consume outside pointer-down” (Radix `disableOutsidePointerEvents`) |
| FocusScope | Contain focus traversal | `fret-ui::element::FocusScope` (ADR 0068) | `fret-ui-kit::primitives::focus_scope::focus_trap` | Trap semantics are policy-level composition |
| RovingFocus | Arrow-key item focus without tab stops | Runtime tracks focusability + pressable focus | `primitives::roving_focus_group` + `headless::{menu_nav, roving_focus, typeahead}` | Ensure items are not necessarily Tab stops |
| Menu | Submenus, roving focus, typeahead, pointer grace | Overlay roots + focus + outside-press observation | `fret-ui-kit::primitives::menu` (WIP; includes `menu::pointer_grace_intent`) | DropdownMenu/Menubar/ContextMenu are wrappers |
| Popper / placement | Anchored placement, flip/shift/size/offset/arrow | `fret-ui::overlay_placement` (ADR 0064) | `fret-ui-kit::overlay::*` helpers | Arrow positioning is supported; visuals live in components/recipes |
| Arrow | Render a positioned arrow pointing at an anchor | `fret-ui::overlay_placement::ArrowLayout` | `fret-ui-kit::primitives::popper` helpers | Visual styling (diamond/border) lives in shadcn recipes |
| TooltipProvider | Shared open-delay + skip-delay window | (n/a) | `fret-ui-kit::primitives::tooltip_provider` + `primitives::tooltip_delay_group` | Provider stack is driven per-frame (service) |
| Collection semantics | “Item X of Y”, roles, disabled skipping | Semantics snapshot (ADR 0033) | `fret-ui-kit::primitives::collection` | Collection metadata is required for menus/lists |
| Active descendant | Highlight moves while focus stays in input | Semantics schema + snapshot + AccessKit mapping (ADR 0073) | `fret-ui-kit::headless::cmdk_selection` + component wiring | Highest-leverage a11y closure item |

## shadcn recipe mapping (component → primitives)

This table maps common shadcn-aligned components/recipes in this repo to the underlying Radix
concepts and Fret building blocks they depend on.

| shadcn surface | Underlying Radix concept(s) | Fret building blocks | Defaults (outside press / focus) | Code |
| --- | --- | --- | --- | --- |
| `Popover` | Popover = Portal + DismissableLayer + FocusScope | `OverlayController` + `OverlayRequest::dismissible_popover` + placement helpers | Outside press closes; click-through by default | `ecosystem/fret-ui-shadcn/src/popover.rs` |
| `Select` | Menu/Listbox-like overlay | `OverlayRequest::dismissible_menu` + roving + collection semantics | Outside press closes; non-click-through; roving highlight does not commit until activation | `ecosystem/fret-ui-shadcn/src/select.rs` |
| `Combobox` | Recipe: Popover + Command (cmdk-style) | `Popover` + `CommandPalette` + active-descendant semantics | Outside press closes; click-through; focus stays in input; highlight via `active_descendant` | `ecosystem/fret-ui-shadcn/src/combobox.rs` |
| `Command` / cmdk | (Not a Radix primitive; common shadcn recipe) | `headless::cmdk_score` + `headless::cmdk_selection` + active-descendant semantics | Focus stays in input; highlight via `active_descendant` | `ecosystem/fret-ui-shadcn/src/command.rs` |
| `DropdownMenu` | Menu | `OverlayRequest::dismissible_menu` + `primitives::menu`-style policy helpers | Outside press closes; non-click-through; modality-gated initial focus | `ecosystem/fret-ui-shadcn/src/dropdown_menu.rs` |
| `ContextMenu` | Menu | `OverlayRequest::dismissible_menu` + `primitives::menu`-style policy helpers | Outside press closes; non-click-through; pointer-anchored open | `ecosystem/fret-ui-shadcn/src/context_menu.rs` |
| `Menubar` | Menubar + Menu | `OverlayRequest::dismissible_menu` + roving + pointer grace corridor | Outside press closes; non-click-through; submenu safe-hover corridor | `ecosystem/fret-ui-shadcn/src/menubar.rs` |
| `Dialog` / `Sheet` | Dialog = Portal + FocusScope + modal barrier | Modal overlay roots + focus trap/restore policy | Underlay inert while present | `ecosystem/fret-ui-shadcn/src/dialog.rs`, `ecosystem/fret-ui-shadcn/src/sheet.rs` |
| `Tooltip` | TooltipProvider + Tooltip | Hover/pointer-move overlays + delay group | Click-through; delay/skip-delay policy | `ecosystem/fret-ui-shadcn/src/tooltip.rs` |
| `HoverCard` | Hover overlay | Hover intent + anchored overlay | Click-through; hover intent policy | `ecosystem/fret-ui-shadcn/src/hover_card.rs` |
| `Sonner` (toasts) | (Not a Radix primitive) | Per-window toast layer orchestration | Click-through; dismissal timers | `ecosystem/fret-ui-shadcn/src/sonner.rs` |

## Code entry points (practical)

- Overlay substrate + outside-press observation (mechanism): `crates/fret-ui/src/tree/mod.rs`
- Overlay placement solver (mechanism): `crates/fret-ui/src/overlay_placement/`
- Overlay anchoring helpers (policy helper): `ecosystem/fret-ui-kit/src/overlay.rs`
- Per-window overlay requests/presence (policy helper): `ecosystem/fret-ui-kit/src/overlay_controller.rs`
- Unstable overlay orchestration internals: `ecosystem/fret-ui-kit/src/window_overlays/`
- Headless primitives (policy/state machines): `ecosystem/fret-ui-kit/src/headless/`
- Semantics schema (portable contract): `crates/fret-core/src/semantics.rs`

## “Where should new work go?” (rules of thumb)

- If it changes **hit testing, routing, or semantics snapshot production**, it is likely `fret-ui`
  and must be justified by an ADR + tests (ADR 0066).
- If it is a **state machine** or **interaction policy composition**, it belongs in
  `fret-ui-kit` (ADR 0074 / ADR 0090).
- If it is **shadcn naming or default styling**, it belongs in `fret-ui-shadcn`.

## Current gaps worth tracking (from audits)

- (Done) Popper + Arrow primitive wiring: `fret-ui-kit::primitives::popper` (layout + wrapper insets helpers).
- DropdownMenu submenu ergonomics: safe-hover close corridor now wired via `DismissibleLayer` pointer-move observers; remaining: intent heuristics + focus transfer closure.
- (Done) cmdk-style filtering/scoring headless primitive: `fret-ui-kit::headless::cmdk_score`.
