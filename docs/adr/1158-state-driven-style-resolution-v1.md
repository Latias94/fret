# ADR 1158: State-Driven Style Resolution v1 (WidgetStateProperty)

Status: Proposed

## Context

Fret’s styling contract is token-driven (ADR 0032 / ADR 0050 / ADR 0102). This works well for
static values, but **interactive widgets** also need a consistent way to resolve styles from widget
state:

- hover / active (pressed)
- focused vs focus-visible (ADR 0061)
- disabled
- selected (toggles, tabs, list rows, etc.)

Without a shared “state → style” primitive, component crates drift into ad-hoc patterns:

- each component re-invents state precedence rules (hover vs active vs focused),
- themes can’t reliably override state-specific visuals,
- building alternate component systems (e.g. Material 3) becomes expensive and inconsistent.

The design goal is to keep **mechanism** in the framework/kit layer, while leaving **policy**
(e.g. Material/Shadcn defaults) in ecosystem crates (ADR 0066, ADR 0169).

## Decision

Introduce a small, reusable “state → style” resolution primitive in `fret-ui-kit` that component
libraries can build on.

### 1) Widget state vocabulary

Define a `WidgetStates` bitset representing common interactive states:

- `disabled`
- `hovered`
- `active` (pressed)
- `focused`
- `focus_visible` (focused + keyboard-navigation intent; ADR 0061)
- `selected`

`focus_visible` is a separate state because **visual focus affordances must gate on focus-visible**
instead of focused-only. This matches both web semantics and Fret’s runtime policy (ADR 0061).

### 2) WidgetStateProperty<T>

Define `WidgetStateProperty<T>` as a value plus optional per-state overrides.

- Matching rule: an override applies when its required state-set is a subset of the current widget state.
- Precedence rule: **last matching override wins** (similar to override stacking).

This is intentionally minimal: it is a building block for higher-level `*Style` structs (e.g.
`ButtonStyle`) rather than a full CSS-like selector system.

### 3) Token-first resolution with derived fallbacks

Component libraries should prefer explicit state-specific tokens, but avoid “token explosion” by
providing consistent derived fallbacks.

Recommended token naming convention:

- `<base>.hover.<slot>` (e.g. `primary.hover.background`)
- `<base>.active.<slot>` (e.g. `primary.active.background`)
- `<base>.focus_visible.<slot>` (rare; prefer `ring`-class tokens)
- `<base>.disabled.<slot>` (optional; often handled via opacity)

Where `<slot>` is typically `background`, `foreground`, or `border`.

v1 provides a minimal derived fallback for colors:

- `ThemeTokenAlphaMul { key, mul }`: derive a color by multiplying the alpha of another token.

The derived fallback is used only when the state-specific token is missing, keeping token sets small
while still allowing explicit per-state overrides.

### 4) Layering / scope

- `fret-ui`: owns runtime “focus-visible” policy and low-level interaction substrate.
- `fret-ui-kit`: owns headless style authoring primitives (`WidgetStates`, `WidgetStateProperty<T>`,
  and small token fallback helpers).
- Component libraries (e.g. `fret-ui-shadcn`, future `fret-ui-material3`) define policy-heavy defaults
  using these primitives.

## Consequences

Pros:

- Component libraries can share a single resolution mechanism and consistent precedence rules.
- Themes can override state-specific visuals via predictable token keys.
- Focus-visible semantics become uniform across ecosystem components.

Cons / risks:

- A simplistic derived fallback (alpha multiply) is not sufficient for all design systems; future
  versions likely need richer color math (blend/lighten/darken/tonal palettes).
- `WidgetStateProperty<T>` is a primitive; without follow-up `*Style` structs, usage can remain ad-hoc.

## Implementation Notes (v1)

Evidence anchors:

- `ecosystem/fret-ui-kit/src/style/state.rs` (`WidgetStates`, `WidgetStateProperty<T>`)
- `ecosystem/fret-ui-kit/src/style/tokens.rs` (`ColorFallback::ThemeTokenAlphaMul`)
- `ecosystem/fret-ui-kit/src/style/tests.rs` (unit tests for state precedence + fallback behavior)
- `ecosystem/fret-ui-shadcn/src/button.rs` (pilot: button background uses per-state tokens with fallback; outline border respects focus-visible)

Known gaps:

- Only a pilot migration exists; most components still resolve state styling ad-hoc.
- No stable public `*Style` structs are defined yet (e.g. `ButtonStyle` with merge/override rules).
- No ecosystem-wide guidance for slot naming beyond v1 conventions above.

