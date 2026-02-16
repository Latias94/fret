# Radix Primitives Audit бк Navigation Menu


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- Radix UI Primitives: https://github.com/radix-ui/primitives

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret's Radix-aligned navigation-menu substrate against the upstream Radix
`@radix-ui/react-navigation-menu` implementation pinned in `repo-ref/primitives`.

## Upstream references (source of truth)

- Implementation: `repo-ref/primitives/packages/react/navigation-menu/src/navigation-menu.tsx`
- Public exports: `repo-ref/primitives/packages/react/navigation-menu/src/index.ts`

Key upstream concepts:

- A controlled/uncontrolled selected value (`value` / `defaultValue`) where an empty string means
  "closed". Fret represents this as `Option<Arc<str>>` (`None` means closed).
- Delayed open and close timers:
  - `delayDuration` (default 200ms)
  - `skipDelayDuration` (default 300ms)
  - close delay (default 150ms)
- Pointer-move gating after an Escape close ("do not immediately reopen on pointer move").
- Viewport/indicator coordination:
  - Root tracks a "viewport content map" keyed by value.
  - Content can animate between values via `data-motion` (`from-start`/`from-end`/`to-start`/`to-end`).
  - Viewport size is exposed via CSS vars `--radix-navigation-menu-viewport-{width,height}`.
  - Indicator is a separate element that tracks the trigger row.
  - Link selection dismisses content unless the click is modified (e.g. `metaKey`).

## Fret mapping

Fret does not use React context nor CSS variables. Outcomes are composed via:

- Mechanism layer (runtime): `crates/fret-ui` (pointer regions, timers, focus, overlay roots).
- Radix-named primitive facade:
  - `ecosystem/fret-ui-kit/src/primitives/navigation_menu.rs`
    - `NavigationMenuRootState` (delay/skipDelay/closeDelay state machine + timer wiring)
    - `NavigationMenuTriggerState` (per-trigger gate: pointer-move reopen suppression after Escape)
    - `NavigationMenuRoot` / `NavigationMenuContext` (Radix-shaped composition surface)
    - `NavigationMenuTrigger` / `NavigationMenuContent` / `NavigationMenuLink` (Radix-named parts)
    - `navigation_menu_use_value_model(...)` (controlled/uncontrolled model helper)
    - `navigation_menu_register_trigger_id(...)` / `navigation_menu_trigger_id(...)` (indicator/viewport anchoring)
    - `navigation_menu_register_viewport_size(...)` / `navigation_menu_viewport_size_for_transition(...)`
      (viewport sizing contract; Radix CSS var equivalent)
    - `navigation_menu_indicator_rect(...)` (indicator placement helper)
    - `navigation_menu_register_viewport_content_id(...)` / `navigation_menu_viewport_content_id(...)`
      (viewport content id registry; Radix "viewport content map" analogue)
    - `navigation_menu_viewport_content_semantics_id(...)` /
      `navigation_menu_viewport_content_pressable_with_id_props(...)`
      (stable, precomputable content ids for trigger `aria-controls`, keyed by `value`)
    - `navigation_menu_viewport_selected_value(...)` (stable selection while present)
    - `navigation_menu_content_switch(...)` (selected+previous content switch helper)
- shadcn recipe layer:
  - `ecosystem/fret-ui-shadcn/src/navigation_menu.rs` (styling + overlay composition)

## Current parity notes

- Pass: Delay/skipDelay/closeDelay timers match Radix defaults and intent.
- Pass: Trigger pointer-move gating after Escape close matches Radix's "do not reopen immediately".
- Pass: Value model supports controlled/uncontrolled selection (Radix `useControllableState`).
- Pass: Dismissal (Escape/outside) clears both the `open` model and the selected value, matching
  Radix's "closed = empty value" behavior.
- Pass: Viewport/indicator overlay wiring is available as a reusable primitives helper
  (`navigation_menu_request_viewport_overlay(...)`); recipes remain responsible for skin/layout.
- Pass: Trigger `aria-controls` relationships can be derived deterministically from the overlay root
  name + `value`, matching Radix's `makeContentId(baseId, value)` approach.
- Pass: `data-motion` direction semantics are exposed via `NavigationMenuContentMotion` and
  `navigation_menu_content_transition(...)`.
- Pass: Viewport-measured width/height can be registered and interpolated via the primitive facade,
  providing a portable replacement for Radix CSS vars.

## Follow-ups (recommended)

- Consider evolving the viewport sizing contract to cover RTL and "content registry" semantics
  (Radix tracks per-item content refs and sizes) if/when non-shadcn recipes need it.

## Conformance gate

- Radix Web overlay geometry parity: `ecosystem/fret-ui-shadcn/tests/radix_web_overlay_geometry.rs`
  (`radix_web_navigation_menu_open_geometry_matches_fret`).
- Radix Web state parity: `ecosystem/fret-ui-shadcn/tests/radix_web_primitives_state.rs`
  (`radix_web_navigation_menu_open_close_matches_fret`).
