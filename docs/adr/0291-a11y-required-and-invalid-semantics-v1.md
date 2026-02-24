# ADR 0291: A11y Required and Invalid Semantics (v1)

Status: Accepted

## Context

Form controls commonly need two structured accessibility signals:

- whether the field is **required** (`aria-required` class), and
- whether the field is **invalid** (`aria-invalid` class).

Without a portable, mechanism-level representation, components tend to:

- encode these states into strings (e.g. “required”, “invalid”), or
- rely on purely-visual chrome (red borders) without exposing semantics to assistive technologies.

AccessKit supports both `required` and a structured `invalid` value, so Fret can provide a portable contract and map it
cleanly into platform accessibility APIs.

## Goals

1. Add portable `required` and `invalid` semantics for form controls.
2. Keep the change additive and low-policy (mechanism-only).
3. Map the fields into AccessKit consistently.
4. Provide at least one ecosystem adoption + regression gate (shadcn input first).

## Non-goals (v1)

- Modeling detailed validation messages or error summaries (policy layer concern).
- A full “form field” object model (labels, descriptions, constraints) beyond existing relations and label/value fields.

## Decision

### D1 — Extend `SemanticsFlags` with `required` and `invalid`

Add two fields to the portable semantics flags:

- `SemanticsFlags.required: bool` (default `false`)
- `SemanticsFlags.invalid: Option<SemanticsInvalid>`

and the enum:

- `SemanticsInvalid::{ True, Grammar, Spelling }`

`None` means “valid / not specified”.

### D2 — AccessKit mapping

When present, map the fields into AccessKit node properties:

- `required == true` → `Node::set_required()`
- `invalid == Some(..)` → `Node::set_invalid(Invalid::{True,Grammar,Spelling})`

### D3 — Ecosystem adoption (shadcn first)

Ecosystem inputs should publish these semantics:

- `aria_required` on shadcn `Input`/`Textarea` → `SemanticsFlags.required = true`
- `aria_invalid` on shadcn `Input`/`Textarea` → `SemanticsFlags.invalid = Some(SemanticsInvalid::True)`

## Evidence (implementation)

- Contract: `crates/fret-core/src/semantics.rs` (`SemanticsInvalid`, `SemanticsFlags.required/invalid`)
- UI writers:
  - `crates/fret-ui/src/widget.rs` (`SemanticsCx::set_required`, `set_invalid`)
  - `crates/fret-ui/src/declarative/host_widget/semantics.rs` (TextInput/TextArea/TextInputRegion write flags)
- AccessKit mapping + tests: `crates/fret-a11y-accesskit/src/{mapping.rs,tests.rs}`
- Ecosystem adoption + gates:
  - `ecosystem/fret-ui-shadcn/src/{input.rs,textarea.rs}`
  - shadcn snapshot gate: `ecosystem/fret-ui-shadcn/tests/snapshots/input_required_invalid_semantics.json`
- Diagnostics snapshot field + fingerprint:
  - `ecosystem/fret-bootstrap/src/ui_diagnostics/semantics.rs`
  - `ecosystem/fret-bootstrap/src/ui_diagnostics.rs`

## Alternatives considered

1. **Only expose invalid/required as strings in `label`/`value`.**
   - Pros: no contract change.
   - Cons: brittle; not mappable into platform-native properties; automation must parse strings.
2. **Expose `invalid` as a boolean only.**
   - Pros: simpler surface.
   - Cons: AccessKit supports richer values; the enum is still small and portable.

