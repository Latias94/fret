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
| DismissableLayer | Escape / outside press dismissal hooks | Outside-press observer pass (ADR 0069) + key routing | `fret-ui-kit::primitives::dismissable_layer` + overlay policies | Supports branches + focus-outside dismissal via overlay orchestration; click-through by default, with optional “consume outside pointer-down” for menu-like overlays |
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
| `Accordion` | Accordion + Collapsible + RovingFocus | `primitives::accordion` + `primitives::collapsible` (+ measured-height motion helpers) | `single` default non-collapsible; `multiple` toggles per item; arrow/Home/End roving | `ecosystem/fret-ui-shadcn/src/accordion.rs` |
| `Dialog` / `Sheet` | Dialog = Portal + FocusScope + modal barrier | Modal overlay roots + focus trap/restore policy | Underlay inert while present | `ecosystem/fret-ui-shadcn/src/dialog.rs`, `ecosystem/fret-ui-shadcn/src/sheet.rs` |
| `Tooltip` | TooltipProvider + Tooltip | Hover/pointer-move overlays + delay group | Click-through; delay/skip-delay policy | `ecosystem/fret-ui-shadcn/src/tooltip.rs` |
| `HoverCard` | Hover overlay | Hover intent + anchored overlay | Click-through; hover intent policy | `ecosystem/fret-ui-shadcn/src/hover_card.rs` |
| `Sonner` (toasts) | (Not a Radix primitive) | Per-window toast layer orchestration | Click-through; dismissal timers | `ecosystem/fret-ui-shadcn/src/sonner.rs` |

## Accordion (parity notes)

Upstream reference:

- Radix: `repo-ref/primitives/packages/react/accordion/src/accordion.tsx`
- shadcn (new-york-v4): `repo-ref/ui/apps/v4/registry/new-york-v4/ui/accordion.tsx`
- ai-elements (shadcn wrapper): `repo-ref/ai-elements/packages/shadcn-ui/components/ui/accordion.tsx`

Aligned outcomes (what we match):

- `type="single"` vs `type="multiple"` state models, including Radix-style controllable vs uncontrolled behavior.
- `collapsible` in single mode: when `false` (default), clicking the open item does not close it.
- Trigger semantics expose `expanded`, and trigger→content relationship is modeled via `controls`.
- Roving focus uses APG navigation (Arrow keys + Home/End) with a “tab stop” derived from open item(s) + enabled fallback.
- `loop` behavior is supported via `loop_navigation` (default `true`), matching Radix’s wrap-at-ends default.
- `orientation` and `dir` are supported at the policy boundary:
  - `AccordionOrientation` selects the navigation axis (vertical vs horizontal).
  - `dir` overrides RTL/LTR resolution for horizontal navigation (Left/Right arrow semantics flip in RTL).

Known gaps / deliberate non-1:1 mapping:

- DOM-only composition nodes are not modeled 1:1 (e.g. Radix `AccordionHeader` as `h3` wrapper).
- Radix `AccordionContent` sets `role="region"` and `aria-labelledby={triggerId}`; Fret currently has no dedicated `Region` semantics role and no “labelled-by” wiring for this surface, so we approximate via `SemanticsRole::List` (root) and pressable semantics on triggers.
- Radix sets `aria-disabled` on the trigger when the open item is not collapsible; Fret does not currently distinguish this from “disabled” at the accessibility contract level (we keep the trigger enabled but ignore the “close” action when not collapsible).

Evidence anchors:

- Policy/headless primitive: `ecosystem/fret-ui-kit/src/primitives/accordion.rs`
- shadcn skin + measured-height motion wiring: `ecosystem/fret-ui-shadcn/src/accordion.rs`
- Direction provider primitive: `ecosystem/fret-ui-kit/src/primitives/direction.rs`

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

## Related: ai-elements port (ecosystem)

The `ai-elements` component taxonomy is policy-heavy and sits above shadcn primitives. In Fret, that
maps naturally to a new ecosystem crate built on top of `fret-ui-shadcn`:

- `ecosystem/fret-ui-ai` (inspired by `repo-ref/ai-elements/packages/elements`)

## Current gaps worth tracking (from audits)

- (Done) Popper + Arrow primitive wiring: `fret-ui-kit::primitives::popper` (layout + wrapper insets helpers).
- DropdownMenu submenu ergonomics: safe-hover close corridor now wired via `DismissibleLayer` pointer-move observers; remaining: intent heuristics + focus transfer closure.
- (Done) cmdk-style filtering/scoring headless primitive: `fret-ui-kit::headless::cmdk_score`.
- Hover overlays (Tooltip/HoverCard): consolidate intent/timer state machines into `fret-ui-kit::headless` so shadcn recipes remain mostly wiring + styling, and add headless unit tests to prevent drift.
- Golden conformance expansion:
  - (Done) `radix-web` timeline scenarios now cover submenu keyboard open/close for
    dropdown-menu/context-menu/menubar (in addition to hover-open + grace corridor).
  - (Done) `shadcn-web` open snapshots now cover non-click input pages like `command-dialog` (key chord; see `docs/shadcn-web-goldens.md` `--openKeys`).
  - (Done) `shadcn-web` open snapshots now cover nested submenu states for dropdown-menu/context-menu/menubar (see `docs/shadcn-web-goldens.md` `--openSteps`).
  - (Done) Programmatic scroll handle updates now invalidate bound scroll nodes, so cmdk/select-style
    “scroll active option into view” works even when cached layout/paint skips subtrees. Evidence:
    `crates/fret-ui/src/scroll.rs`, `crates/fret-ui/src/declarative/mount.rs`.
  - (Done) `Select` item-aligned positioning now matches Radix’s layout + scroll sequencing:
    content inner-box offsets (`clientHeight` vs border box), scroll-button mount reposition, and the
    post-position “focus selected item” scroll-into-view behavior needed for `select-scrollable`
    option inset parity. Evidence: `ecosystem/fret-ui-headless/src/select_item_aligned.rs`,
    `ecosystem/fret-ui-kit/src/primitives/select.rs`, `ecosystem/fret-ui-shadcn/src/select.rs`, and
    `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_placement.rs` (`web_vs_fret_select_demo_open_option_metrics_match`,
    `web_vs_fret_select_scrollable_listbox_option_insets_match`).
  - (Done) `Button` shadcn variant styles are gated against `shadcn-web` computed styles (including `lab()` normalization to linear sRGB). Evidence: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_button.rs`.
  - (Done) `Combobox` (shadcn recipe) listbox height is gated against `shadcn-web` open snapshots. Evidence: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_placement.rs` (combobox-demo open variants).
  - (Done) `Select` listbox width parity is gated against shadcn-web: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_placement.rs`
    (`web_vs_fret_select_scrollable_listbox_width_matches`).
  - (Done) Overlay close focus restoration honors Radix `onCloseAutoFocus` outcomes even when the
    overlay unmounts after motion (teardown path respects `preventDefault`). Evidence:
    `ecosystem/fret-ui-kit/src/window_overlays/{state.rs,render.rs}` and the radix timeline parity
    gates in `ecosystem/fret-ui-shadcn/tests/radix_web_primitives_state.rs`.
  - (Done) `NavigationMenu` opt-in indicator `shadow-md` token is gated against shadcn-web. Evidence:
    `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_chrome.rs` (`web_vs_fret_navigation_menu_demo_indicator_shadow_matches_web*`)
    + `goldens/shadcn-web/v4/new-york-v4/navigation-menu-demo-indicator.open.json`.
  - (Done) `NavigationMenu` indicator geometry (track rect + diamond offset) is gated against shadcn-web. Evidence:
    `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_placement.rs` (`web_vs_fret_navigation_menu_demo_indicator_geometry_matches_web`)
    + `goldens/shadcn-web/v4/new-york-v4/navigation-menu-demo-indicator.open.json`.
  - (Done) `shadcn-web` golden extraction supports offline `next/font/google` builds via
    `goldens/shadcn-web/scripts/next-font-google-mock.cjs` (see `docs/shadcn-web-goldens.md`).
  - (Next) extend `shadcn-web` open snapshots for additional multi-step states (e.g. navigation-menu hover + content open, and other chained interactions).
