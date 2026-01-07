# Radix Primitives Audit бк Navigation Menu

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
- shadcn recipe layer:
  - `ecosystem/fret-ui-shadcn/src/navigation_menu.rs` (styling + overlay composition)

## Current parity notes

- Pass: Delay/skipDelay/closeDelay timers match Radix defaults and intent.
- Pass: Trigger pointer-move gating after Escape close matches Radix's "do not reopen immediately".
- Pass: Value model supports controlled/uncontrolled selection (Radix `useControllableState`).
- Partial: Viewport/indicator are modeled as recipe-layer overlay elements rather than a reusable
  Radix-named primitives facade surface.
- Pass: `data-motion` direction semantics are exposed via `NavigationMenuContentMotion` and
  `navigation_menu_content_transition(...)`.
- Missing: Viewport-measured width/height exposure as stable variables (Radix CSS vars).

## Follow-ups (recommended)

- Consider downshifting the "viewport content registry" into `fret-ui-kit::primitives::navigation_menu`
  so non-shadcn consumers can reuse it (similar to how `menu` is shared and facaded).
- Consider exposing a "viewport size contract" surface (Radix CSS vars
  `--radix-navigation-menu-viewport-{width,height}`) so recipes can converge on a shared sizing
  policy without relying on DOM/CSS.
