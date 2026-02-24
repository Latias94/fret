# ADR 0290: A11y Pressed Semantics (v1)

Status: Accepted

## Context

Many “toggle button” widgets represent state using an `aria-pressed`-like outcome:

- a boolean pressed state (`true` / `false`), and
- optionally a tri-state “mixed” value (`mixed`) for aggregated/partial selection.

In Fret, these outcomes were historically approximated by setting `SemanticsFlags.selected` on a button-like role. This
is not ideal:

- `selected` is semantically distinct (it typically describes selection in a set, not `aria-pressed`),
- adapters may map `selected` into different platform properties,
- and it makes it difficult to gate/automate toggle-button semantics without ambiguity.

We want a mechanism-level, portable representation that maps cleanly into AccessKit and can be adopted consistently by
ecosystem components (shadcn/Radix-aligned toggles first).

## Goals

1. Add a portable pressed semantics surface for toggle-button-like widgets.
2. Support the tri-state `mixed` outcome.
3. Keep the change additive and migration-friendly.
4. Map the pressed semantics into AccessKit properties consistently.

## Non-goals (v1)

- Introducing a dedicated `ToggleButton` role in `SemanticsRole` (ARIA expresses toggle buttons as `button` + pressed).
- Redesigning the action model beyond existing `invoke` handling for pressables.

## Decision

### D1 — Add `SemanticsPressedState` and `SemanticsFlags.pressed_state`

Add a portable tri-state enum:

- `SemanticsPressedState::{ False, True, Mixed }`

and a new optional flag field:

- `SemanticsFlags.pressed_state: Option<SemanticsPressedState>`

`None` means “not a toggle button / unknown”.

### D2 — AccessKit mapping

AccessKit does not have a dedicated toggle-button role; toggle buttons are represented as `Role::Button` with a toggled
state. We therefore map `pressed_state` into the AccessKit toggled property:

- `False -> Toggled::False`
- `True -> Toggled::True`
- `Mixed -> Toggled::Mixed`

Compatibility note:

- `SemanticsFlags.checked_state` / legacy `checked` already map to the same AccessKit toggled property.
- If a node provides both `checked*` and `pressed_state`, the adapter must choose one. In v1, **checked takes
  precedence** (pressed is used only when no checked state is present).

### D3 — Ecosystem adoption (shadcn first)

Ecosystem toggle primitives should publish `pressed_state` rather than overloading `selected`:

- `fret-ui-kit` `Toggle` publishes `pressed_state` on a button-like role.
- `ToggleGroup` multiple mode publishes `pressed_state` on button items.
- `ToggleGroup` single mode continues to use `role="radio" + checked` for parity with Radix.

## Evidence (implementation)

- Contract: `crates/fret-core/src/semantics.rs` (`SemanticsPressedState`, `SemanticsFlags.pressed_state`)
- UI plumbing: `crates/fret-ui/src/{widget.rs,element.rs,declarative/host_widget/semantics.rs}`
- AccessKit mapping + tests: `crates/fret-a11y-accesskit/src/{mapping.rs,tests.rs}`
- Ecosystem adoption:
  - `ecosystem/fret-ui-kit/src/primitives/{toggle.rs,toggle_group.rs}`
  - shadcn regression: `ecosystem/fret-ui-shadcn/tests/snapshots/toggle_pressed_semantics.json`
- Diagnostics snapshots include `pressed_state` for debugging:
  `ecosystem/fret-bootstrap/src/ui_diagnostics/semantics.rs`

## Alternatives considered

1. **Keep using `selected` for toggle buttons.**
   - Pros: no new contract field.
   - Cons: incorrect semantics; harder adapter mapping; ambiguous automation assertions.
2. **Add a new role `ToggleButton`.**
   - Pros: explicit role-level differentiation.
   - Cons: ARIA expresses toggles as `button` + pressed; role addition increases churn without clear cross-platform gains.

