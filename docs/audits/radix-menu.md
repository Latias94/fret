# Radix Primitives Audit — Menu

This audit compares Fret's Radix-aligned menu substrate against the upstream Radix
`@radix-ui/react-menu` implementation pinned in `repo-ref/primitives`.

## Upstream references (source of truth)

- Implementation: `repo-ref/primitives/packages/react/menu/src/menu.tsx`
- Public exports: `repo-ref/primitives/packages/react/menu/src/index.ts`

Key upstream concepts:

- `Menu` is the shared primitive behind `DropdownMenu`, `ContextMenu`, and `Menubar`.
- `MenuContent` composes:
  - Popper placement (`PopperPrimitive.Content`),
  - roving focus group + typeahead (`RovingFocusGroup.Root`),
  - dismissal hooks (`DismissableLayer`),
  - focus scoping (`FocusScope`) and optional focus trap (modal),
  - optional outside pointer blocking (`disableOutsidePointerEvents`) when `modal` is enabled.
- Submenu ergonomics are part of the `Menu` primitive (not a standalone package):
  - pointer direction tracking (`pointerDirRef`),
  - "pointer grace intent" corridor when moving towards a submenu,
  - open/close delays and focus transfer policies.

## Fret mapping

Fret does not implement React context nor DOM event capture/bubble. It models the same outcomes by
composing:

- Overlay placement: `ecosystem/fret-ui-kit/src/primitives/popper.rs`
- Outside-press observation and pointer blocking (ADR 0069):
  - `OverlayRequest::dismissible_menu(...)` (consume outside presses + block underlay input),
  - `OverlayRequest::dismissible_popover(...)` (click-through outside presses).
- Dismissal routing and "preventDefault" outcome:
  - `OnDismissRequest(host, ActionCx, DismissReason)` installed via overlay requests.
- Menu policy helpers (Radix-named facade family):
  - `ecosystem/fret-ui-kit/src/primitives/menu/*`
  - `ecosystem/fret-ui-kit/src/primitives/dropdown_menu.rs`
  - `ecosystem/fret-ui-kit/src/primitives/context_menu.rs`
  - `ecosystem/fret-ui-kit/src/primitives/menubar.rs`

## Current parity notes

- Pass: Modal-vs-non-modal mapping is modeled via `dismissible_menu_request_with_modal(...)`:
  - `modal=true` -> `OverlayRequest::dismissible_menu(...)` (non-click-through + blocks underlay),
  - `modal=false` -> `OverlayRequest::dismissible_popover(...)` (click-through).
- Pass: Submenu pointer grace intent is available as a reusable pointer-move observer:
  `submenu_pointer_move_handler(...)`.
- Pass: Dismissals can be intercepted (Radix `DismissableLayer` "preventDefault" outcome) by
  providing an `OnDismissRequest` handler on the overlay request via
  `dismissible_menu_request_with_dismiss_handler(...)` (or
  `dismissible_menu_request_with_modal_and_dismiss_handler(...)`).
  - Focus-outside dismissal routes through the same handler with `DismissReason::FocusOutside`.
- Pass: Trigger `expanded`/`controls` relationships can be stamped via
  `menu::trigger::apply_menu_trigger_a11y(...)` (used by shadcn menu recipes).

## Recommended usage (Fret)

- Treat `ecosystem/fret-ui-kit/src/primitives/menu/*` as the "Radix menu boundary" for behavior and
  policy wiring.
- shadcn recipes should remain skin/layout only, and should request overlays via the Radix-named
  primitives facades.
- If a menu needs to keep itself open on outside interactions, provide an `OnDismissRequest`
  handler and decide whether to close the `open` model inside that handler (preventDefault
  analogue).

## Gaps / intentional differences

- Focus trap and outside scroll lock are modeled indirectly for `modal=true` menus via the overlay
  substrate: `OverlayRequest::dismissible_menu(...)` enables `disableOutsidePointerEvents`, which
  installs a modal barrier scope while the menu is open. This prevents focus traversal and wheel
  events from reaching underlay widgets (matching the observable outcome of Radix
  `FocusScope(trapped)` + scroll lock).
- ARIA hiding (`hideOthers`) is not currently modeled for menus; this is tracked alongside broader
  semantics-bridge work.

## Conformance gates

- `ecosystem/fret-ui-shadcn/tests/radix_web_overlay_geometry.rs` validates dropdown-menu placement
  (menu popper gap + cross-axis delta) against the Radix Vega web golden
  (`goldens/radix-web/v4/radix-vega/dropdown-menu-example.dropdown-menu.open-navigate-select.light.json`).
- Run (layout engine v2): `cargo nextest run -p fret-ui-shadcn -F fret-ui/layout-engine-v2 --test radix_web_overlay_geometry`
