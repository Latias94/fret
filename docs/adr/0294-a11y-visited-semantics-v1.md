# ADR 0294: A11y Visited Semantics (v1)

Status: Accepted

## Context

Links may have a meaningful "visited" state (e.g. documentation links, navigation history). Assistive technologies can
announce visited state for link roles on some platforms. AccessKit provides a portable `visited` flag that maps into the
native accessibility APIs where supported.

Without a portable, mechanism-level signal, components may:

- encode visited state into label text (brittle and localization-hostile), or
- expose no visited semantics at all, losing parity with common web behavior.

## Goals

1. Add a portable visited-link semantics signal.
2. Map it into AccessKit consistently.
3. Provide at least one ecosystem adoption + regression gate (shadcn badge-link first).

## Non-goals (v1)

- Defining a global visited-link persistence model (policy-layer concern).
- Inferring visited state from routing/navigation history automatically (policy-layer concern).

## Decision

### D1 — Extend `SemanticsFlags` with `visited`

Add a new portable flag:

- `SemanticsFlags.visited: bool` (default `false`)

This is intended to be used primarily with `SemanticsRole::Link`.

### D2 — AccessKit mapping

When `visited == true`, map into AccessKit:

- `visited == true` → `Node::set_visited()`

### D3 — Ecosystem adoption (shadcn badge link)

shadcn `Badge` links should be able to publish visited state via a simple authoring knob:

- `Badge::visited(true)` when rendered as a link → `SemanticsFlags.visited = true`

## Evidence (implementation)

- Contract: `crates/fret-core/src/semantics.rs` (`SemanticsFlags.visited`)
- UI writers:
  - `crates/fret-ui/src/widget.rs` (`SemanticsCx::set_visited`)
  - `crates/fret-ui/src/declarative/host_widget/semantics.rs` (Pressable mapping)
- AccessKit mapping + test: `crates/fret-a11y-accesskit/src/{mapping.rs,tests.rs}`
- Ecosystem adoption + gate:
  - `ecosystem/fret-ui-shadcn/src/badge.rs` (`Badge::visited`)
  - shadcn snapshot gate: `ecosystem/fret-ui-shadcn/tests/snapshots/badge_link_visited_semantics.json`
- Diagnostics snapshot field + fingerprint:
  - `ecosystem/fret-bootstrap/src/ui_diagnostics/semantics.rs`
  - `ecosystem/fret-bootstrap/src/ui_diagnostics.rs`

## Alternatives considered

1. **Encode visited state into the label/value text.**
   - Pros: no contract change.
   - Cons: brittle; localization-hostile; not mappable into native visited properties.
2. **Store visited state only in ecosystem (no `fret-core` contract).**
   - Pros: avoids a contract expansion.
   - Cons: platform bridges and diagnostics cannot reason about visited state consistently.

