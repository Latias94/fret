# ADR 0289: A11y Tri-state Checked Semantics (v1)

Status: Accepted

## Context

Fret’s portable semantics contract (`crates/fret-core/src/semantics.rs`) exposes a `checked: Option<bool>` flag for
checkable widgets. This is sufficient for binary states, but it cannot represent **indeterminate / mixed** states.

This gap shows up in common UI patterns:

- A checkbox that supports an indeterminate state (e.g. “some items selected”).
- Tree views where parent nodes can be “mixed” based on descendant selection.

Without a structured tri-state representation, components tend to encode the state as text (e.g. “Mixed”) or drop the
signal entirely, preventing platform adapters from mapping to native accessibility APIs (e.g. `aria-checked="mixed"`,
AccessKit’s `Toggled::Mixed`).

## Goals

1. Add a portable, structured tri-state contract for checkable widgets (mechanism-only).
2. Keep the change additive and migration-friendly for existing producers that only set `checked`.
3. Map the tri-state data into AccessKit’s toggled state where supported.
4. Enable ecosystem components (shadcn/Radix-aligned checkbox first) to publish indeterminate semantics without
   string conventions.

## Non-goals (v1)

- Redesigning the full “toggle” capability surface (actions, keyboard policies, focus/hover intent).
- Enforcing strict validation constraints on legacy producers that only set `checked`.

## Decision

### D1 — Add `SemanticsCheckedState` and `SemanticsFlags.checked_state`

Add a new tri-state enum to the portable contract:

- `SemanticsCheckedState::{ False, True, Mixed }`
- `SemanticsFlags.checked_state: Option<SemanticsCheckedState>`

This is additive and does not remove the existing `SemanticsFlags.checked: Option<bool>`.

### D2 — Compatibility behavior and precedence

- Producers MAY continue setting `checked` only (binary semantics).
- Producers that need indeterminate semantics SHOULD set `checked_state`.
- When `checked_state` is present, adapters treat it as the source of truth.
- For `checked_state == Mixed`, the legacy `checked` field SHOULD remain `None`.

Rationale:

- avoids breaking older code paths that only understand `Option<bool>`,
- avoids pretending the “mixed” state is “true” or “false”.

### D3 — AccessKit mapping

When `checked_state` is present, map it to AccessKit’s toggled state:

- `False -> Toggled::False`
- `True -> Toggled::True`
- `Mixed -> Toggled::Mixed`

When `checked_state` is absent, fall back to the legacy `checked` field.

## Evidence (implementation)

- Contract: `crates/fret-core/src/semantics.rs`
- UI plumbing:
  - `crates/fret-ui/src/widget.rs` (`SemanticsCx::set_checked_state`)
  - `crates/fret-ui/src/declarative/host_widget/semantics.rs` (`Pressable` emits tri-state)
- AccessKit mapping: `crates/fret-a11y-accesskit/src/mapping.rs` + unit test
  `crates/fret-a11y-accesskit/src/tests.rs`
- Ecosystem adoption:
  - headless state: `ecosystem/fret-ui-headless/src/checked_state.rs`
  - checkbox semantics: `ecosystem/fret-ui-kit/src/primitives/checkbox.rs`
  - shadcn gates: `ecosystem/fret-ui-shadcn/src/checkbox.rs` and
    `ecosystem/fret-ui-shadcn/tests/snapshots/checkbox_indeterminate_semantics.json`
- Diagnostics bundle snapshots include `checked_state` for debugging:
  `ecosystem/fret-bootstrap/src/ui_diagnostics/semantics.rs`

## Alternatives considered

1. **Keep tri-state as text only (encode in `value`).**
   - Pros: no contract change.
   - Cons: brittle; hard to map to platform-native checked semantics; automation must parse strings.
2. **Replace `checked: Option<bool>` with a tri-state enum everywhere.**
   - Pros: simpler model.
   - Cons: breaking contract change and a larger migration.

