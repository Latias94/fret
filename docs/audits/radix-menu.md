# Radix Primitives Audit: Menu

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
  - `OverlayRequest::dismissible_menu(...)` (consume outside presses + disable underlay mouse input),
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
- Pass: Submenu close via keyboard uses the Radix key boundary (ArrowLeft / ArrowRight depending on
  `dir`) and explicitly restores focus to the submenu trigger (matching the upstream
  `MenuSubContent` behavior).
- Pass: Dismissals can be intercepted (Radix `DismissableLayer` "preventDefault" outcome) by
  providing an `OnDismissRequest` handler on the overlay request via
  `dismissible_menu_request_with_dismiss_handler(...)` (or
  `dismissible_menu_request_with_modal_and_dismiss_handler(...)`).
  - Focus-outside dismissal routes through the same handler with `DismissReason::FocusOutside`.
- Pass: Focus restoration on close honors Radix `onCloseAutoFocus` outcomes even when the overlay
  unmounts after motion:
  - `WindowOverlays` records whether `onCloseAutoFocus` ran and whether it prevented default while
    the overlay is closing.
  - The unmount/teardown path consults this recorded outcome (and runs the hook when needed) before
    applying the default “restore focus to trigger” policy.
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

- Fret models submenu "safe hover" with an explicit close-delay timer driven by pointer-move events
  (`pointer_grace_intent::drive_close_timer_on_pointer_move`). Radix does not use an explicit close
  timer in `@radix-ui/react-menu`; it prevents default on certain pointer transitions while a
  pointer-grace intent is active. The intended observable outcome should match, but this needs
  conformance coverage beyond "open + select".
- Radix behavior goldens treat submenu closure (e.g. ArrowLeft) as an immediate unmount. In Fret we
  currently align this by disabling close animation ticks for submenu content, even when the root
  menu content uses motion.
- Focus trap and outside scroll lock are modeled indirectly for `modal=true` menus via the overlay
  substrate: `OverlayRequest::dismissible_menu(...)` enables `disableOutsidePointerEvents`, which
  applies `PointerOcclusion::BlockMouseExceptScroll` while the menu is open.
  - Mouse interactions do not reach the underlay.
  - Wheel events may still route to underlay scrollables by default.
  - Focus trapping is *not* implied by `disableOutsidePointerEvents` in Fret; fully modal focus
    behavior is expressed via barrier-backed layers (`blocks_underlay_input=true`) and focus-scope
    policy.
- ARIA hiding (`hideOthers`) is not currently modeled for menus; this is tracked alongside broader
  semantics-bridge work.

## Conformance gates

- `ecosystem/fret-ui-shadcn/tests/radix_web_overlay_geometry.rs` validates dropdown-menu placement
  (menu popper gap + cross-axis delta) against the Radix Vega web golden
  (`goldens/radix-web/v4/radix-vega/dropdown-menu-example.dropdown-menu.open-navigate-select.light.json`).
- `ecosystem/fret-ui-shadcn/tests/radix_web_primitives_state.rs` validates dropdown-menu submenu
  hover-open + item select closes the root menu against the Radix Vega web golden
  (`goldens/radix-web/v4/radix-vega/dropdown-menu-example.dropdown-menu.submenu-hover-select.light.json`).
- `ecosystem/fret-ui-shadcn/tests/radix_web_primitives_state.rs` validates dropdown-menu submenu
  pointer grace corridor keeps the submenu open against the Radix Vega web golden
  (`goldens/radix-web/v4/radix-vega/dropdown-menu-example.dropdown-menu.submenu-grace-corridor.light.json`).
- `ecosystem/fret-ui-shadcn/tests/radix_web_primitives_state.rs` validates dropdown-menu submenu
  keyboard open/close (ArrowRight open, ArrowLeft close + focus restore) against the Radix Vega web
  golden
  (`goldens/radix-web/v4/radix-vega/dropdown-menu-example.dropdown-menu.submenu-keyboard-open-close.light.json`).
- `ecosystem/fret-ui-shadcn/tests/radix_web_primitives_state.rs` validates context-menu submenu
  keyboard open/close (ArrowRight open, ArrowLeft close + focus restore) against the Radix Vega web
  golden
  (`goldens/radix-web/v4/radix-vega/context-menu-example.context-menu.submenu-keyboard-open-close.light.json`).
- `ecosystem/fret-ui-shadcn/tests/radix_web_primitives_state.rs` validates context-menu submenu
  hover-open + item select closes the root menu against the Radix Vega web golden
  (`goldens/radix-web/v4/radix-vega/context-menu-example.context-menu.submenu-hover-select.light.json`).
- `ecosystem/fret-ui-shadcn/tests/radix_web_primitives_state.rs` validates context-menu submenu
  pointer grace corridor keeps the submenu open against the Radix Vega web golden
  (`goldens/radix-web/v4/radix-vega/context-menu-example.context-menu.submenu-grace-corridor.light.json`).
- `ecosystem/fret-ui-shadcn/tests/radix_web_overlay_geometry.rs` validates context-menu placement
  (menu popper gap + cross-axis delta, anchored to the right-click point) against the Radix Vega web
  golden
  (`goldens/radix-web/v4/radix-vega/context-menu-example.context-menu.context-open-close.light.json`).
- `ecosystem/fret-ui-shadcn/tests/radix_web_primitives_state.rs` validates menubar submenu
  hover-open + item select closes the root menu against the Radix Vega web golden
  (`goldens/radix-web/v4/radix-vega/menubar-example.menubar.submenu-hover-select.light.json`).
- `ecosystem/fret-ui-shadcn/tests/radix_web_primitives_state.rs` validates menubar submenu
  pointer grace corridor keeps the submenu open against the Radix Vega web golden
  (`goldens/radix-web/v4/radix-vega/menubar-example.menubar.submenu-grace-corridor.light.json`).
- `ecosystem/fret-ui-shadcn/tests/radix_web_overlay_geometry.rs` validates menubar menu placement
  (menu popper gap + cross-axis delta) against the Radix Vega web golden
  (`goldens/radix-web/v4/radix-vega/menubar-example.menubar.open-navigate-close.light.json`).
- `ecosystem/fret-ui-shadcn/tests/radix_web_primitives_state.rs` validates menubar submenu
  keyboard open/close (ArrowRight open, ArrowLeft close + focus restore) against the Radix Vega web
  golden
  (`goldens/radix-web/v4/radix-vega/menubar-example.menubar.submenu-keyboard-open-close.light.json`).
- Run: `cargo nextest run -p fret-ui-shadcn --test radix_web_overlay_geometry`
