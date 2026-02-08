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
- Pass: The shadcn surface exposes upstream-shaped parts:
  `NavigationMenuRoot` (alias), `NavigationMenuList`, `NavigationMenuItem`, `NavigationMenuTrigger`,
  `NavigationMenuContent`, `NavigationMenuLink`, plus `NavigationMenuViewport` and
  `NavigationMenuIndicator` configuration outcomes.
- Pass: `viewport=false` behavior is supported via `NavigationMenu::viewport(false)` /
  `NavigationMenu::viewport_component(NavigationMenuViewport::enabled(false))`.
- Pass: Indicator rendering can be disabled via `NavigationMenu::indicator(false)` /
  `NavigationMenu::indicator_component(NavigationMenuIndicator::enabled(false))`.
- Pass: Base UI timing props are mirrored by shadcn-friendly aliases:
  `delay_ms`/`delay_duration`, `close_delay_ms`/`close_delay_duration`,
  and `skip_delay_ms`/`skip_delay_duration`.

### Open/close & hover behavior (Radix parity)

- Pass: Hover opens with delayed open semantics (Radix `delayDuration`).
- Pass: Close uses a delayed close timer (Radix `startCloseTimer`).
- Pass: Pointer-move gating after Escape close prevents immediate reopen (Radix behavior).
- Pass: Link select semantics (modified clicks should not dismiss) are exposed via the shadcn
  `NavigationMenuLink` wrapper.

### Motion parity (new-york)

- Pass: Viewport overlay uses fade + zoom with shadcn-aligned easing, matching upstream's
  `zoom-in-90` on open and `zoom-out-95` on close (best-effort, tick driven).
- Pass: Directional content switching matches shadcn's `data-motion` semantics via
  `navigation_menu_content_transition(...)` + `navigation_menu_content_switch(...)`.
- Pass: Viewport placement uses logical start alignment under RTL when configured with
  `align=Start` (Radix/Floating parity).

### Indicator parity (new-york)

- Pass: Indicator support is available (rotated square). It is opt-in to match upstream
  composition (upstream exports `NavigationMenuIndicator` but does not mount it in `NavigationMenu`).
- Pass: Indicator placement is track-based (trigger width + gutter thickness) and uses the same
  `shadow-md` token as upstream for the diamond.
- Pass: Indicator geometry is gated against shadcn-web (track thickness + diamond placement + viewport panel size).
- Note: Exact positioning still differs in the details (Radix uses a DOM-measured indicator track);
  Fret anchors from trigger element ids + popper geometry.
  - Placement logic: `ecosystem/fret-ui-kit/src/primitives/navigation_menu.rs`
    (`navigation_menu_indicator_rect(...)`).

## Validation

- `cargo check -p fret-ui-shadcn`
- `cargo nextest run -p fret-ui-shadcn navigation_menu`
- `cargo nextest run -p fret-ui-shadcn navigation_menu_viewport_align_start_respects_direction_provider`
- Contract test: `navigation_menu_delay_aliases_update_config`
- Contract test: `navigation_menu_duration_aliases_update_config`
- Radix Web overlay geometry gate: `cargo nextest run -p fret-ui-shadcn --test radix_web_overlay_geometry`
  (`radix_web_navigation_menu_open_geometry_matches_fret`).
  - shadcn-web gates:
  - Chrome: `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_overlay_chrome`
    (`web_vs_fret_navigation_menu_demo_panel_chrome_matches`, `web_vs_fret_navigation_menu_demo_surface_colors_match_web`,
    `web_vs_fret_navigation_menu_demo_surface_colors_match_web_dark`, `web_vs_fret_navigation_menu_demo_shadow_matches_web`,
    `web_vs_fret_navigation_menu_demo_shadow_matches_web_dark`,
    `web_vs_fret_navigation_menu_demo_indicator_shadow_matches_web`,
    `web_vs_fret_navigation_menu_demo_indicator_shadow_matches_web_dark`,
    `web_vs_fret_navigation_menu_demo_home_mobile_viewport_shadow_matches_web`,
    `web_vs_fret_navigation_menu_demo_home_mobile_viewport_shadow_matches_web_dark`,
    `web_vs_fret_navigation_menu_demo_components_mobile_viewport_surface_colors_match_web`,
    `web_vs_fret_navigation_menu_demo_components_mobile_viewport_surface_colors_match_web_dark`,
    `web_vs_fret_navigation_menu_demo_components_mobile_viewport_shadow_matches_web`,
    `web_vs_fret_navigation_menu_demo_components_mobile_viewport_shadow_matches_web_dark`,
    `web_vs_fret_navigation_menu_demo_home_mobile_constrained_viewport_shadow_matches_web`,
    `web_vs_fret_navigation_menu_demo_home_mobile_constrained_viewport_shadow_matches_web_dark`,
    `web_vs_fret_navigation_menu_demo_components_mobile_constrained_viewport_surface_colors_match_web`,
    `web_vs_fret_navigation_menu_demo_components_mobile_constrained_viewport_surface_colors_match_web_dark`,
    `web_vs_fret_navigation_menu_demo_components_mobile_constrained_viewport_shadow_matches_web`,
    `web_vs_fret_navigation_menu_demo_components_mobile_constrained_viewport_shadow_matches_web_dark`;
    consumes:
    - `goldens/shadcn-web/v4/new-york-v4/navigation-menu-demo*.open.json`
    - `goldens/shadcn-web/v4/new-york-v4/navigation-menu-demo-indicator.open.json`).
  - Placement: `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_overlay_placement`
    (`web_vs_fret_navigation_menu_demo_overlay_placement_matches`,
    `web_vs_fret_navigation_menu_demo_indicator_geometry_matches_web`;
    consumes `goldens/shadcn-web/v4/new-york-v4/navigation-menu-demo*.open.json`).
    - Note: `web_vs_fret_navigation_menu_demo_overlay_placement_matches` also asserts the open
      content panel `w/h` for the `viewport=false` demo (`navigation-menu-demo`), so regressions in
      menu sizing (padding/border + wrapped content height) are caught.
    - Mobile viewport geometry is separately gated for the `Components` trigger:
      `web_vs_fret_navigation_menu_demo_components_mobile_viewport_height_matches` and
      `web_vs_fret_navigation_menu_demo_components_mobile_small_viewport_height_matches`.
      These gates assume shadcn/ui behavior where the viewport panel is placed relative to the
      root (not the active trigger) when `viewport=true`.

  - Underlay scroll anchor stability gate: `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_overlay_placement`
    (`fret_navigation_menu_tracks_trigger_when_underlay_scrolls`).
    - This exercises `viewport=false` anchoring when the trigger lives in a scrolling underlay.
    - Depends on last-frame visual-bounds tracking scroll-driven render transforms under paint-cache
      replay (see `crates/fret-ui/src/tree/paint.rs`).

## Follow-ups (recommended)

- Consider exposing an opt-in custom indicator renderer if parity-sensitive recipes need it (today
  the indicator visuals are not user-supplied, only toggled on/off).
