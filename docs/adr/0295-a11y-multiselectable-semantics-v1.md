# ADR 0295: A11y Multiselectable Semantics (v1)

Status: Accepted

## Context

Collections such as listboxes may support selecting **multiple** items. ARIA models this via the container attribute
`aria-multiselectable`. Assistive technologies can use this to announce the selection model and adjust expectations for
keyboard/interaction patterns.

AccessKit provides a portable `multiselectable` flag for this purpose.

## Goals

1. Add a portable mechanism-level “multi-select collection” signal.
2. Map it into AccessKit consistently.
3. Provide at least one ecosystem adoption + regression gate (shadcn multi-select combobox first).

## Non-goals (v1)

- Defining a global selection model or enforcing selection policy (policy-layer concern).
- Auto-inferring multiselectable from child selection state (policy-layer concern).

## Decision

### D1 — Extend `SemanticsFlags` with `multiselectable`

Add a new portable flag:

- `SemanticsFlags.multiselectable: bool` (default `false`)

This is intended to be set on collection/container roles such as `SemanticsRole::ListBox`.

### D2 — AccessKit mapping

When `multiselectable == true`, map into AccessKit:

- `multiselectable == true` → `Node::set_multiselectable()`

### D3 — Ecosystem adoption (shadcn combobox chips)

The shadcn multi-select combobox (`ComboboxChips`) should mark the underlying listbox as multiselectable when it uses a
checked-selection listbox pattern:

- `ComboboxChips` listbox → `SemanticsFlags.multiselectable = true`

## Evidence (implementation)

- Contract: `crates/fret-core/src/semantics.rs` (`SemanticsFlags.multiselectable`)
- UI writers:
  - `crates/fret-ui/src/widget.rs` (`SemanticsCx::set_multiselectable`)
  - `crates/fret-ui/src/{element.rs,declarative/host_widget/semantics.rs}` (decoration + writer plumbing)
- AccessKit mapping + test: `crates/fret-a11y-accesskit/src/{mapping.rs,tests.rs}`
- Ecosystem adoption + gates:
  - `ecosystem/fret-ui-shadcn/src/combobox_chips.rs` (sets listbox multiselectable)
  - shadcn snapshot gate: `ecosystem/fret-ui-shadcn/tests/snapshots/command_palette_multiselectable_semantics.json`
- Diagnostics snapshot field + fingerprint:
  - `ecosystem/fret-bootstrap/src/ui_diagnostics/semantics.rs`
  - `ecosystem/fret-bootstrap/src/ui_diagnostics.rs`

## Alternatives considered

1. **Infer multiselectable from multiple selected children.**
   - Pros: fewer authoring knobs.
   - Cons: ambiguous and policy-heavy; selection state may be virtualized or not represented as `selected` for all items.
2. **Keep multiselectable as ecosystem-only behavior.**
   - Pros: avoids contract expansion.
   - Cons: platform bridges and diagnostics cannot represent the container semantics consistently.

