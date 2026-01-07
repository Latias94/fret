# shadcn/ui v4 Audit - Navigation Menu (new-york)

This audit compares Fret's shadcn-aligned `NavigationMenu` surface against the upstream shadcn/ui
v4 documentation and the `new-york-v4` registry implementation in `repo-ref/ui`.

## Upstream references (source of truth)

- Registry implementation (new-york): `repo-ref/ui/apps/v4/registry/new-york-v4/ui/navigation-menu.tsx`
- Underlying primitive concept: Radix `@radix-ui/react-navigation-menu`

## What upstream exports (new-york)

Upstream shadcn/ui exports a thin wrapper around Radix:

- `NavigationMenu` (root)
  - Accepts an opt-in `viewport` boolean (default `true`).
  - When `viewport=true`, it renders `NavigationMenuViewport` as a sibling of the list.
- `NavigationMenuList`, `NavigationMenuItem`
- `NavigationMenuTrigger` (includes a chevron icon that rotates when open)
- `NavigationMenuContent`
  - When `viewport=true`, content is mounted into a shared viewport and animates between values
    using `data-motion`:
    - `from-start`/`from-end` + `slide-in-from-*-52`
    - `to-start`/`to-end` + `slide-out-to-*-52`
  - When `viewport=false`, content behaves like a popover-ish surface with open/close animations.
- `NavigationMenuViewport`
  - Uses CSS vars `--radix-navigation-menu-viewport-{width,height}` for sizing.
  - Uses zoom animations (`zoom-in-90` / `zoom-out-95`) keyed off `data-state`.
- `NavigationMenuIndicator`
  - Fades in/out and renders a rotated square "arrow" aligned to the active trigger.

## Fret implementation

- Component code: `ecosystem/fret-ui-shadcn/src/navigation_menu.rs`
- Key building blocks:
  - Radix-aligned timer/gating substrate: `ecosystem/fret-ui-kit/src/primitives/navigation_menu.rs`
    - Radix-shaped parts: `NavigationMenuRoot` / `NavigationMenuTrigger` (wiring) +
      trigger id registry helpers (anchoring).
  - Overlay roots: `ecosystem/fret-ui-kit/src/overlay_controller.rs`
  - Popper placement solver: `ecosystem/fret-ui-kit/src/primitives/popper.rs`
  - Presence/motion helpers: `ecosystem/fret-ui-kit/src/primitives/presence.rs`,
    `ecosystem/fret-ui-kit/src/declarative/overlay_motion.rs`

## Audit checklist

### Composition surface

- Pass: Fret provides a shadcn-friendly `NavigationMenu` builder with `NavigationMenuItem`
  (value + label + content + optional trigger children).
- Partial: Upstream exports a fully composable component family; Fret now shares the Radix-aligned
  trigger wiring in `fret-ui-kit::primitives::navigation_menu`, but the shadcn surface still models
  some pieces as "spec" structs rather than independent elements.
- Missing: `viewport=false` behavior. Fret currently always uses a shared viewport overlay.

### Open/close & hover behavior (Radix parity)

- Pass: Hover opens with delayed open semantics (Radix `delayDuration`).
- Pass: Close uses a delayed close timer (Radix `startCloseTimer`).
- Pass: Pointer-move gating after Escape close prevents immediate reopen (Radix behavior).

### Motion parity (new-york)

- Partial: Viewport overlay uses fade + zoom (shadcn taxonomy) with shadcn-aligned easing.
- Pass: Directional content switching matches shadcn's `data-motion` semantics via
  `navigation_menu_content_transition(...)`.

### Indicator parity (new-york)

- Pass: Indicator is rendered as a rotated square and shares the viewport's open/close motion.
- Note: Exact positioning and shadow/token fidelity may differ until the viewport sizing contract
  is fully downshifted (Radix uses an indicator track ref + layout measurement).

## Validation

- `cargo check -p fret-ui-shadcn`
- `cargo nextest run -p fret-ui-shadcn navigation_menu`

## Follow-ups (recommended)

- Add a headless "motion direction" helper so content transitions can match `data-motion`
  (`from-start`/`from-end`/`to-start`/`to-end`) without relying on DOM/CSS.
- Consider adding a `viewport` toggle to the shadcn wrapper if we want strict API parity.
