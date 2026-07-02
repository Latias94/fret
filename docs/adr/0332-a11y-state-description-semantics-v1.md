# ADR 0332: A11y State Description Semantics (v1)

Status: Accepted

Date: 2026-05-29

## Context

Some component states need a short assistive-technology announcement that is distinct from role,
label, value, and structured boolean flags. Material 3 SearchBar is one such case: Compose
Material3 publishes a default search content description and, while expanded, a state description
for available suggestions.

Fret already has portable fields for role descriptions, placeholders, relations, expanded state,
and structured checked/numeric metadata, but it did not have a state-description channel. Recipes
could set `expanded=true`, but they could not express the human-readable state phrase without
overloading label or value.

## Decision

Add a portable optional state-description field:

- `SemanticsNodeExtra::state_description: Option<String>`

Expose it through `crates/fret-ui` authoring surfaces:

- `SemanticsCx::set_state_description(...)`
- `SemanticsDecoration::state_description(...)`
- `PressableA11y::state_description`
- `TextInputProps::a11y_state_description`
- `TextAreaProps::a11y_state_description`

Map it into AccessKit:

- `NodeBuilder::set_state_description(...)`

## Consequences

- Component ecosystems can publish state text without corrupting accessible names or values.
- The field remains mechanism-only: design-system recipes choose when a state description is
  useful and what localized string to publish.
- Diagnostics snapshot export can remain additive future work; direct semantics snapshots and
  AccessKit mapping already cover the runtime contract.

## Evidence

- Contract: `crates/fret-core/src/semantics.rs`
- UI authoring and writers:
  - `crates/fret-ui/src/widget.rs`
  - `crates/fret-ui/src/element.rs`
  - `crates/fret-ui/src/declarative/host_widget/semantics.rs`
- AccessKit mapping: `crates/fret-a11y-accesskit/src/mapping.rs`
- Ecosystem adoption: `ecosystem/fret-ui-material3/src/search_bar.rs`
- Gates:
  - `cargo nextest run -p fret-ui --lib declarative_text_input_respects_a11y_role_override_and_expanded declarative_attach_semantics_can_override_state_and_relations`
  - `cargo nextest run -p fret-a11y-accesskit --lib maps_state_description maps_role_description`
  - `cargo nextest run -p fret-ui-material3 --features diagnostics --test search_bar_accessibility`
