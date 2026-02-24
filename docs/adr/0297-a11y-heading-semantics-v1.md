# ADR 0297: A11y Heading Semantics (v1)

Status: Accepted

## Context

Headings provide a high-value navigation primitive for assistive technologies (e.g. “jump between headings”). In Fret,
many “title” and document surfaces currently render as plain text nodes, which prevents platforms from recognizing a
structured heading outline.

AccessKit supports a dedicated `Heading` role and a `level` property.

## Goals

1. Add a portable `Heading` role to the semantics contract.
2. Reuse the existing portable hierarchy `level` field for heading levels (1-based).
3. Adopt in at least one ecosystem surface and gate the outcome.

## Non-goals (v1)

- Defining a global policy for which visual text styles are headings (policy-layer concern).
- Enforcing a strict `h1..h6` range in the contract (platform policies vary; v1 stays permissive).

## Decision

### D1 — Extend `SemanticsRole` with `Heading`

Add a new role:

- `SemanticsRole::Heading`

### D2 — Heading level uses `SemanticsNodeExtra.level`

Heading levels are represented via the existing portable field:

- `SemanticsNodeExtra.level: Option<u32>` (1-based)

The existing invariant remains: `level == 0` is invalid and must be rejected.

### D3 — AccessKit mapping

Map the role into AccessKit:

- `SemanticsRole::Heading` → `Role::Heading`

When `extra.level` is present, it is already mapped into AccessKit via `Node::set_level(...)`.

### D4 — Ecosystem adoption (modal titles)

Shadcn modal titles should publish heading semantics to improve navigation and consistent naming:

- `DialogTitle` → `role=Heading`, `level=2`
- `AlertDialogTitle` → `role=Heading`, `level=2`

Markdown renderers should publish heading semantics directly from the document structure:

- `fret-markdown` pulldown headings → `role=Heading`, `level = h1..h6`

## Evidence (implementation)

- Contract: `crates/fret-core/src/semantics.rs` (`SemanticsRole::Heading`)
- AccessKit role mapping: `crates/fret-a11y-accesskit/src/roles.rs`
- Ecosystem adoption:
  - `ecosystem/fret-ui-shadcn/src/dialog.rs` (`DialogTitle`)
  - `ecosystem/fret-ui-shadcn/src/alert_dialog.rs` (`AlertDialogTitle`)
- Markdown adoption: `ecosystem/fret-markdown/src/pulldown_render.rs`
- Regression gates:
  - shadcn snapshot: `ecosystem/fret-ui-shadcn/tests/snapshots/heading_level_semantics.json`
  - markdown unit tests: `ecosystem/fret-markdown/src/semantics_tests.rs`

## Alternatives considered

1. **Infer headings from text size/weight.**
   - Pros: zero authoring.
   - Cons: policy-heavy and theme-dependent; hard to keep stable across apps.
2. **Keep headings as text-only conventions.**
   - Pros: avoids contract change.
   - Cons: platforms cannot provide heading navigation; diagnostics cannot assert structure.
