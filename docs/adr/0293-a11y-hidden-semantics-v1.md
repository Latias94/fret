# ADR 0293: A11y Hidden Semantics (v1)

Status: Accepted

## Context

Some UI elements are intentionally **not part of the accessibility tree** even when they are present for pointer users:

- decorative affordances (e.g. scroll arrows in a popup),
- redundant chrome,
- implementation details that would distract assistive technology users.

ARIA typically models this via `aria-hidden` (or `hidden` on the DOM), and AccessKit provides a portable `hidden` flag to
exclude nodes from the filtered tree presented to assistive technologies.

Historically, Fret approximated this by coercing roles/actions (e.g. mapping to `Generic` and disabling actions). This is
not robust and can still leak nodes into platform accessibility APIs.

## Goals

1. Add a portable, mechanism-level “exclude from AT tree” signal.
2. Map it into AccessKit using the native `hidden` property.
3. Preserve deterministic diagnostics visibility (the semantics snapshot can still include the node for debugging).
4. Provide at least one regression gate (snapshot + unit test).

## Non-goals (v1)

- Defining “visual hidden” vs “a11y hidden” policy across the entire ecosystem.
- Auto-deriving hidden from layout (e.g. zero-size, opacity 0): policy-layer concern.

## Decision

### D1 — Extend `SemanticsFlags` with `hidden`

Add a new portable flag:

- `SemanticsFlags.hidden: bool` (default `false`)

When true, this node should be excluded from the platform accessibility tree.

### D2 — AccessKit mapping

When `hidden == true`, map into AccessKit:

- `hidden == true` → `Node::set_hidden()`

### D3 — Declarative `PressableA11y.hidden` uses the portable flag

When a declarative pressable is marked as hidden (`PressableA11y.hidden == true`), the UI runtime should set
`SemanticsFlags.hidden = true` instead of relying on role/action coercion.

## Evidence (implementation)

- Contract: `crates/fret-core/src/semantics.rs` (`SemanticsFlags.hidden`)
- UI writers:
  - `crates/fret-ui/src/widget.rs` (`SemanticsCx::set_hidden`)
  - `crates/fret-ui/src/declarative/host_widget/semantics.rs` (Pressable mapping)
- AccessKit mapping + test: `crates/fret-a11y-accesskit/src/{mapping.rs,tests.rs}`
- Ecosystem usage + gate:
  - shadcn snapshot gate: `ecosystem/fret-ui-shadcn/tests/snapshots/pressable_hidden_semantics.json`
  - real usage example: `ecosystem/fret-ui-shadcn/src/select.rs` (scroll arrows use `PressableA11y.hidden`)
- Diagnostics snapshot field + fingerprint:
  - `ecosystem/fret-bootstrap/src/ui_diagnostics/semantics.rs`
  - `ecosystem/fret-bootstrap/src/ui_diagnostics.rs`

## Alternatives considered

1. **Keep coercing hidden nodes to `Generic` and disabling actions.**
   - Pros: no contract change.
   - Cons: does not guarantee platform exclusion; can leak nodes and confuse AT users.
2. **Remove hidden nodes from the semantics snapshot entirely.**
   - Pros: simpler mapping.
   - Cons: loses useful diagnostics visibility; complicates authoring/debugging and determinism.

