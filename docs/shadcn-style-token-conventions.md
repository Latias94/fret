# Shadcn Style Token Conventions (v1)

This document defines **recommended** theme token key conventions for shadcn/ui-aligned component
libraries in Fret.

Contract references:

- Typed token theming + theme resolution: `docs/adr/0032-style-tokens-and-theme-resolution.md`
- Baseline tokens + semantic alias bridge: `docs/adr/0050-theme-config-schema-and-baseline-tokens.md`
- Focus-visible + ring primitive semantics: `docs/adr/0061-focus-rings-and-focus-visible.md`
- State-driven resolution (`WidgetStates`): `docs/adr/0219-state-driven-style-resolution-v1.md`
- Style override surface patterns: `docs/shadcn-style-override-patterns.md`

## Goals

- Make “state → style” authoring consistent across component ecosystems (shadcn now, Material 3 later).
- Provide a stable **key vocabulary** so themes can override behavior without per-widget hacks.
- Avoid “token explosion” by allowing cheap, deterministic fallbacks when state-specific keys are absent.

Non-goals:

- Defining a full design system or palette.
- Perfect CSS/Tailwind parity for every modifier.

## Vocabulary

### States

Map `WidgetStates` to state suffixes:

- `hover` → pointer hover
- `active` → pressed / active interaction
- `disabled` → disabled
- `selected` → selected/checked/current
- `open` → expanded/open (e.g. submenu open)
- `focus_visible` → focus-visible (keyboard modality; see ADR 0061)

Notes:

- `focus_visible` is distinct from `focused` by design (ADR 0061).
- If your theme does not need separate `focus_visible` colors, prefer reusing `ring` and keep border
  stable.

### Slots

Use these slot names for common control chrome:

- `background`
- `foreground`
- `border`
- `ring`

Slots should be kept small and composable. Prefer adding new slots only when multiple components
need them.

## Key Naming

### 1) Semantic base keys (preferred for palette)

Prefer shadcn-style semantic keys (resolved via `Theme::color_by_key` / alias bridge):

- `background`, `foreground`
- `card`, `popover`
- `border`, `input`, `ring`
- `muted`, `muted-foreground`
- `accent`, `accent-foreground`
- `primary`, `primary-foreground`
- `secondary`, `secondary-foreground`
- `destructive`, `destructive-foreground`

These should remain cross-component and design-system-friendly.

### 2) Derived state keys (optional)

When a theme wants explicit per-state colors, use:

`<base>.<state>.<slot>`

Examples:

- `primary.hover.background`
- `accent.active.background`
- `accent.selected.background`
- `destructive.hover.background`

If a derived key is missing, components should fall back deterministically (see “Fallbacks”).

### 3) Component namespace keys (preferred for metrics; allowed for colors)

Use `component.<control>.*` for per-control tuning (especially **metrics**):

- `component.input.*` (already used by input family recipes)
- `component.dropdown_menu.*` / `component.menubar.*` (panel sizing / arrow metrics)

When component-level colors are needed, prefer:

`component.<control>.<slot>` and `component.<control>.<state>.<slot>`

Examples:

- `component.tabs.trigger.selected.background`
- `component.menu.item.hover.background`

If a component does not need unique colors, prefer semantic keys instead of introducing component
keys.

## Fallbacks (avoid token explosion)

State-specific keys are optional. When they are absent, components should derive values with small
rules instead of requiring themes to define every combination.

Recommended fallback strategies (see `ColorFallback` in `fret-ui-kit`):

- `hover` / `active`: alpha-multiply the base token (e.g. `mul=0.9` / `0.8`) or derive from
  `accent` / `muted`.
- `disabled`: reduce alpha of the resolved foreground (or use `muted-foreground`).
- `focus_visible`: reuse `ring` for ring/border accent unless a design system needs a unique color.

Rule of thumb:

- Only introduce a new key if multiple components need it and fallbacks do not match the desired UX.

## Selected Mapping (v1 guidance)

`selected` is a persistent state (not a transient interaction):

- Tabs: the active tab trigger is `selected`.
- Toggle: the pressed/on state is `selected`.
- Menu radio/checkbox: selection is primarily expressed via the indicator, but row highlight may use
  `open`/`focused`/`hover`.

Default mapping guidance:

- Prefer `accent`/`accent-foreground` for selection highlights in navigation/list-like surfaces.
- Prefer `primary`/`primary-foreground` for primary action emphasis (buttons, main CTAs), not for
  “current selection” in lists.

If a component needs different semantics, expose a `*Style` override and/or allow component-level
theme keys under `component.<control>.*`.

## Implementation Guidance (component authors)

- Use `WidgetStateProperty<T>` as the primary “state → style” resolver.
- If you accept overrides, expose `*Style` structs with `OverrideSlot<T>` fields
  (`Option<WidgetStateProperty<Option<T>>>`) and `merged()` (right-biased) semantics.
- Prefer consuming semantic base keys via `Theme::color_by_key` and only add new theme keys when
  the fallback is insufficient.
