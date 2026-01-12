---
title: "ADR 0073: Active Descendant for Composite Widgets (Command Palette / Listbox)"
---

# ADR 0073: Active Descendant for Composite Widgets (Command Palette / Listbox)

Status: Accepted

## Implementation Status (as of 2025-12-29)

Phase A (schema + bridge) is **implemented**.

What exists today:

- A semantics snapshot contract and platform bridge boundary (ADR 0033) with roles/flags/actions.
- `active_descendant: Option<NodeId>` in the semantics schema:
  - `crates/fret-core/src/semantics.rs`
  - `crates/fret-ui/src/tree/mod.rs` + `crates/fret-ui/src/widget.rs` (`SemanticsCx::set_active_descendant`)
  - `crates/fret-a11y-accesskit/src/lib.rs` (AccessKit mapping + unit tests)
  - `crates/fret-runner-winit/src/accessibility.rs` (winit adapter glue)
- Overlay + list semantics tests for several window-scoped overlay surfaces
  (see `docs/a11y-acceptance-checklist.md` and `ecosystem/fret-ui-kit/src/window_overlays/*`).
- Roving-focus navigation for composite lists is typically implemented by **moving focus** between
  rows (via `UiTree` focus primitives), which works without `active_descendant` but is not ideal
  for cmdk-style “focus stays in the text field” experiences.

What is missing (core gap):

- Component-layer cmdk-style policy wiring:
  - keep focus in the text input while navigating results,
  - update `active_descendant` on the focused input node to point at the highlighted result row,
  - ensure stable identity for result rows while present.

## Context

Fret is rebuilding shadcn-aligned UI surfaces as declarative-only components. One high-leverage
surface is the command palette (`command` / cmdk-style): a text input that filters a list of
results, with keyboard navigation (Up/Down) and activation (Enter).

In the web, cmdk commonly keeps **keyboard focus in the text field** and uses `aria-activedescendant`
to let assistive technology announce the currently highlighted result without moving focus away
from the input.

In Fret, we currently have:

- a semantics tree snapshot contract (ADR 0033),
- roles like `TextField`, `List`, and `ListItem` (see `crates/fret-core/src/semantics.rs`),
- selected/expanded/checked flags and a `value` string,
- no semantics relationship for "the focused node has an active descendant option".

Without an explicit relationship, a cmdk-style palette has two poor options:

1) Move focus from the input to the highlighted list item (better announcements, worse typing/IME UX).
2) Keep focus in the input but accept that AT may not announce highlight changes (worse accessibility).

This decision is hard to retrofit once command palette, combobox, listbox, menus, and typeahead
surfaces are widely used.

## Decision

### 1) Add an "active descendant" association to semantics nodes (P0)

Extend the semantics node schema with an optional association:

- `active_descendant: Option<NodeId>`

Semantics meaning:

- When a node is focused (typically `TextField`) and `active_descendant` is set, assistive
  technology should treat the referenced node as the *currently active option* for announcement
  and navigation context.
- The referenced node must be in the same window semantics snapshot and must be reachable under
  the current modal barrier/overlay gating rules (ADR 0011 / ADR 0033 / ADR 0066).

This association is purely semantic:

- It does not change focus.
- It does not grant pointer capture or change event routing.
- It is safe to update frequently as selection changes.

### 2) Component-layer policy: cmdk-style command palette uses active descendant (recommended)

For a cmdk-style palette:

- Keep focus in the text input while typing and while navigating results.
- Update `active_descendant` on the input semantics node to point at the highlighted result row.
- Mark the highlighted row as `selected = true` for redundancy.
- Activation (Enter) invokes the selected command; Escape dismisses the palette.

This preserves best-in-class typing/IME behavior while enabling AT announcements.

### 3) Virtualization constraint (P0)

Active descendant requires stable identity for result rows:

- The referenced `NodeId` must be stable for as long as the row is present.
- Virtualized lists that recycle row nodes may break announcements unless the semantics layer
  provides a stable, item-key-based identity mapping.

For MVP:

- Do not require virtualization in the command palette surface.
- If virtualization is used later, it must follow the stable-key contracts (ADR 0047 / ADR 0070)
  and define an accessibility strategy (either a stable semantics mirror or a virtualized AT
  contract).

## Alternatives Considered

### A) Move focus to the highlighted list item

Pros:
- Works with existing semantics schema; AT likely announces selection.

Cons:
- Typing and IME UX regress (focus leaves the input).
- Arrow keys and text editing semantics become ambiguous (caret movement vs selection movement).
- Harder to match cmdk/Radix behavior and user expectations.

### B) Encode selection into the input "value" and rely on announcements

Pros:
- No schema extension.

Cons:
- Conflates query text with selection state.
- Likely produces confusing announcements and breaks editing expectations.

### C) Defer accessibility and ship cmdk behavior without announcements

Pros:
- Faster short-term delivery.

Cons:
- Locks in a poor accessibility baseline for a highly used surface.

## Conformance Checklist (Target)

- Screen reader can keep focus in the text field and still announce highlight changes when using Up/Down.
- Highlight change does not disrupt IME composition or caret behavior.
- `active_descendant` never points to a node outside the current modal barrier scope.

## References

- Semantics tree + bridge boundary: `docs/adr/0033-semantics-tree-and-accessibility-bridge.md`
- Overlays and modal gating: `docs/adr/0011-overlays-and-multi-root.md`, `docs/adr/0066-fret-ui-runtime-contract-surface.md`
- Virtualization identity: `docs/adr/0047-virtual-list-data-source-and-stable-item-keys.md`, `docs/adr/0070-virtualization-contract.md`
- WAI-ARIA `aria-activedescendant` (conceptual reference): https://www.w3.org/TR/wai-aria-1.2/#aria-activedescendant

## Suggested Implementation Plan (incremental)

### Phase A — Schema + bridge (framework-level, minimal)

1) Extend the semantics data model with `active_descendant: Option<NodeId>`.
   - Location candidates:
     - `crates/fret-core/src/semantics.rs`: extend `SemanticsNode`.
     - Any snapshot/container types that need to carry the association.
2) Teach `crates/fret-ui` semantics snapshot production to populate it.
   - Mechanism: a widget/element must be able to set `active_descendant` on its semantics node
     during `Widget::semantics(...)` (or the declarative element equivalent).
3) Update the platform accessibility bridge (AccessKit) mapping so AT can announce the active row
   while focus remains on the input.

### Phase B — Component-layer policy (cmdk-style command palette)

1) Define a headless “command palette selection” policy in `ecosystem/fret-ui-kit`:
   - Keep focus in the text input (better IME/caret behavior).
   - Maintain an internal “active index / active row id”.
2) In `fret-ui-shadcn`, wire `Command`/`CommandList` recipes so:
   - the highlighted row is marked `selected=true`, and
   - the input semantics node sets `active_descendant` to the highlighted row id.

### Phase C — Virtualization constraints (only if needed)

If `CommandList` becomes virtualized:

- ensure stable item-key → stable semantics node identity (ADR 0047 / ADR 0070),
- ensure `active_descendant` never points to recycled/absent row nodes.
