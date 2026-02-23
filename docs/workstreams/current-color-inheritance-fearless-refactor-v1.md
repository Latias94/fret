# Foreground inheritance (`currentColor`) (fearless refactor v1)

Status: In progress (workstream)

Last updated: 2026-02-23

Milestones: `docs/workstreams/current-color-inheritance-fearless-refactor-v1-milestones.md`

## Motivation

shadcn/ui + Radix recipes routinely rely on CSS `currentColor` (or equivalent) so that icons and other foreground
elements inherit the *semantic* foreground color of their container:

- A primary button can render a white icon without every callsite threading `primary-foreground`.
- Menu items can apply a single foreground to both text and icons.
- Disabled/hover/pressed states automatically update descendants when the host computes a new foreground.

In Fret today, many leaf elements (SVG icons, spinners, ad-hoc glyphs) require an explicit `ColorRef`, or they fallback
to a fixed theme token (commonly `muted-foreground`). This leads to:

- Copy/paste friction: docs and gallery examples must manually thread tokens.
- Real mismatches: a dark primary background + icon forced to `foreground` yields “black on black”.
- Refactor hazard: hosts compute stateful `fg`, but descendants do not automatically follow.

We want a renderer-agnostic, declarative-first, layering-correct equivalent of:

- Web: `currentColor`
- Flutter: `IconTheme` / `DefaultTextStyle`

## Scope note

This workstream is intentionally *ecosystem-layer* and *authoring-layer*:

- Mechanism/contract crates (`crates/fret-ui`, `crates/fret-core`) should not grow policy knobs.
- We implement an opt-in provider pattern using `ElementContext::inherited_state_*` (similar to Radix `DirectionProvider`)
  and adopt it in shadcn-aligned components in `ecosystem/`.

## Goals

- Provide an explicit “foreground inheritance” mechanism (`currentColor`) that is:
  - additive (no breaking API),
  - composable (nested providers restore correctly),
  - cheap (no heap allocations on the hot path beyond existing element creation).
- Make icons/spinners inherit foreground by default when a host provides it.
- Make key “foreground host” components provide `currentColor` (starting with shadcn `Button`).
- Reduce the need for manual token threading in gallery/docs examples.
- Lock the behavior with small tests and one or two targeted diag scripts.

## Non-goals (v1)

- A full text style inheritance stack (font, letter spacing, etc.). v1 focuses on foreground color.
- Changing the theme token set or renaming shadcn tokens.
- A global runtime context system beyond `inherited_state_*`.

## Ownership / layering

- `ecosystem/fret-ui-kit` (`declarative/`): authoring glue + provider surface + leaf defaults.
- `ecosystem/fret-ui-shadcn`: shadcn component policy decides where to provide `currentColor`.
- `apps/fret-ui-gallery`: demos should become simpler over time but should not define core behavior.

## Proposed surface

## Key constraint (Fret authoring model)

Fret’s declarative tree is *built in one pass* into concrete `AnyElement` nodes. That means leaf elements like
`SvgIconProps` currently resolve their final `Color` during `into_element(cx)`, not later during paint.

Practical implication:

- `currentColor` inheritance only applies when the leaf is *constructed under the provider scope*.
- If a component stores `AnyElement` slots that are already-built (e.g. `leading: AnyElement`), installing a provider
  inside the host **cannot retroactively recolor** that existing element.

To get Flutter/Web-like ergonomics, hosts should prefer *deferred construction* for common foreground children:

- icon-id slots (store `IconId`, build the icon inside the host under `currentColor`), or
- builder slots (store a closure to build the subtree inside the host), where feasible.

### Provider (authoring glue)

In `fret-ui-kit`, add a small provider module:

- `with_current_color_provider(cx, color, f)`
- `inherited_current_color(cx) -> Option<ColorRef>`

Evidence anchor:

- `ecosystem/fret-ui-kit/src/declarative/current_color.rs`

### Leaf adoption (consume `currentColor`)

Leaves should follow this resolution order:

1. Explicit color argument (if API supports it).
2. Inherited `currentColor` (if available).
3. Existing theme fallback (keep current behavior outside provider scopes).

Initial targets:

- `fret-ui-kit::declarative::icon_with(...)`
- `fret-ui-shadcn::Spinner`

Evidence anchors:

- `ecosystem/fret-ui-kit/src/declarative/icon.rs`
- `ecosystem/fret-ui-shadcn/src/spinner.rs`

### Host adoption (provide `currentColor`)

Hosts that compute a stateful foreground should provide it to their subtree. This is the decisive part that enables
“fearless refactor”: a single place computes `fg`, and all descendants follow.

Initial target:

- `fret-ui-shadcn::Button` provides its resolved `fg` when building content children.
- `fret-ui-shadcn::Button` offers deferred icon slots (`leading_icon` / `trailing_icon` / `icon`)
  so common icon usage is constructed *under* the provider (and therefore inherits `currentColor`).

Evidence anchor:

- `ecosystem/fret-ui-shadcn/src/button.rs`

## When to provide vs consume

### Provide (`with_current_color_provider`)

Use a provider when:

- The component is a “foreground host” (button, menu item, label-like control).
- The component computes `fg` as a function of state (disabled/hover/pressed/selected).
- The component accepts arbitrary `children` (slots) where the author may place icons/text/spinners.

### Consume (`inherited_current_color`)

Use inheritance when:

- The leaf renders a single foreground color and should “just work” inside a host (icon, spinner, glyph).
- The leaf already has an explicit color API, but we want a good default.

## Tracking table (fearless refactor closure)

Statuses:

- `Landed`: implemented + gated
- `In progress`: partially implemented or missing gates
- `Planned`: design agreed but not implemented

| Area | Layer | Status | Evidence anchor | Gate |
|---|---|---:|---|---|
| Provider surface (`currentColor`) | `fret-ui-kit` | Landed | `ecosystem/fret-ui-kit/src/declarative/current_color.rs` | unit test |
| Icon inheritance | `fret-ui-kit` | Landed | `ecosystem/fret-ui-kit/src/declarative/icon.rs` | unit test (`--features icons`) |
| Spinner inheritance | `fret-ui-shadcn` | Landed | `ecosystem/fret-ui-shadcn/src/spinner.rs` | existing shadcn tests (smoke) |
| Button provides `currentColor` | `fret-ui-shadcn` | Landed | `ecosystem/fret-ui-shadcn/src/button.rs` | web-vs-fret button tests |
| Button deferred icon slots (`leading_icon` / `trailing_icon` / `icon`) | `fret-ui-shadcn` | Landed | `ecosystem/fret-ui-shadcn/src/button.rs` | gallery + button goldens |
| Gallery ButtonGroup demo uses deferred icon slots | `fret-ui-gallery` | Landed | `apps/fret-ui-gallery/src/ui/previews/pages/components/basics/button_group.rs` | `tools/diag-scripts/ui-gallery-button-group-demo-screenshots.json` |
| DropdownMenuItem provides `currentColor` (and `leading_icon`) | `fret-ui-shadcn` | Landed | `ecosystem/fret-ui-shadcn/src/dropdown_menu.rs` | `tools/diag-scripts/ui-gallery-dropdown-menu-icons-screenshots.json` |
| CommandItem provides `currentColor` (and `leading_icon`) | `fret-ui-shadcn` | Landed | `ecosystem/fret-ui-shadcn/src/command.rs` | `tools/diag-scripts/ui-gallery-command-docs-demo-icons-screenshots.json` + `tools/diag-scripts/ui-gallery-command-docs-demo-icons-screenshots-zinc-dark.json` |
| Sidebar icon-only toggle uses deferred icon slot | `fret-ui-shadcn` | Landed | `ecosystem/fret-ui-shadcn/src/sidebar.rs` | (manual) |
| AI icon-only actions use deferred icon slots | `fret-ui-ai` | Landed | `ecosystem/fret-ui-ai/src/elements/message_actions.rs` | (manual) |
| Diag gate: primary button icon visibility (via ButtonGroup demo; zinc light/dark) | `fret-ui-gallery` | Landed | `tools/diag-scripts/ui-gallery-button-group-demo-icons-screenshots-zinc-light-dark.json` | `tools/diag-scripts/ui-gallery-button-group-demo-icons-screenshots-zinc-light-dark.json` |
| Badge provides `currentColor` | `fret-ui-shadcn` | Planned | `ecosystem/fret-ui-shadcn/src/badge.rs` | targeted test |
| Text defaults inherit `currentColor` | `fret-ui-kit` | Landed | `ecosystem/fret-ui-kit/src/ui.rs` | unit test |
| Gallery usage cleanup | `fret-ui-gallery` | Planned | (TBD: per-page) | diag screenshot gate(s) |

## Migration plan

1. Land provider + leaf adoption in `fret-ui-kit` (no behavior change unless a provider is present).
2. Adopt provider in the highest-ROI hosts (Button first, then menu items).
3. Add 1–2 diag scripts to lock a “primary button with icon” and “menu item with icon” outcome.
4. Optional cleanup: simplify gallery/docs examples by removing manual `*_fg` threading where inheritance is sufficient.

## Quality gates

- Formatting: `cargo fmt`
- Focused tests:
  - `cargo nextest run -p fret-ui-kit --features icons -E "test(icon_inherits_current_color_when_available)"`
  - `cargo nextest run -p fret-ui-kit -E "test(current_color_provider_inherits_and_restores)"`
- Targeted diag scripts:
  - One script that captures a primary button with an icon (ensures contrast and follow-foreground behavior).
