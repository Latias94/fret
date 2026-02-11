# ADR 0084: Virtualized Accessibility for Collections (Composite Widgets + Large Lists)

Status: Accepted

## Context

Fret targets desktop-grade UI surfaces where **large lists** are common (editors, palettes, trees,
search results, asset browsers). We already have:

- A semantics snapshot contract and platform bridge boundary (ADR 0033).
- Multi-root overlays and modal barriers that gate both input and semantics visibility (ADR 0011 /
  ADR 0066 / ADR 0069).
- A virtualization vocabulary and stable item key requirement (ADR 0047 / ADR 0070).
- `active_descendant` in the semantics schema (ADR 0073) enabling cmdk-style “focus stays in the
  input” interaction patterns.

However, accessibility closure for **virtualized collections** is not locked. Today’s behavior is
implicitly “best effort” and can drift:

- Component-layer overlay lists may only expose a bounded number of items to assistive technology.
- Virtualized lists may recycle row nodes, which can invalidate `active_descendant` references and
  break announcements.
- Without collection metadata (e.g. “item 57 of 1200”), AT can only infer what is currently
  visible, which reduces usability for power users.

This is a high rewrite-risk area because:

- Accessibility integration is cross-cutting (runtime, components, platform bridge).
- Once many components rely on an implicit behavior, changing it becomes a breaking ecosystem
  migration.

## Goals

- Define a **portable, cross-platform contract** for “virtualized collection semantics”.
- Keep `crates/fret-ui` **mechanism-only** (ADR 0066): no per-component policy.
- Enable composite widget patterns (cmdk / combobox / listbox) to be accessible without moving
  focus away from the input (ADR 0073).
- Provide a staged path: a P0 contract we can ship early, and a P1 path to richer platform-native
  virtualization patterns.

## Non-goals (for this ADR)

- A complete, platform-perfect UIA/AX virtualization implementation in v1.
- New component APIs for every widget (those live in `fret-components-*`).

## Decision

### 1) Add collection metadata to `SemanticsNode` (P0, portable)

Extend the semantics schema with optional collection metadata so AT can understand list position
even when only a subset of items is present in the snapshot.

Add to `fret_core::SemanticsNode`:

- `pos_in_set: Option<u32>` — 1-based position in the logical collection.
- `set_size: Option<u32>` — total number of items in the logical collection.

Rules:

- These fields are only meaningful for collection items (e.g. `ListItem`, `MenuItem`, `TreeItem`),
  but are allowed on any node for forward compatibility.
- When present, `1 <= pos_in_set <= set_size` must hold.
- These fields are **semantic only** and must not affect hit testing, routing, or focus.

### 2) Define the “virtualized collection snapshot” contract (P0)

For large collections, the semantics snapshot is allowed to include only a **window** of items
(typically those visible on screen), provided that:

- Each exposed item has correct `pos_in_set` and `set_size` (when known).
- Item identity is stable for as long as that item remains exposed (stable keys in the UI tree:
  ADR 0047).
- Composite widgets using `active_descendant` (ADR 0073) only point to an item node that is present
  in the current snapshot and within the active modal barrier scope.

This makes the initial contract achievable and portable while still enabling useful AT output.

### 3) Active descendant + virtualization rule (P0)

If a composite widget keeps focus on an owner node (e.g. a `TextField`) and uses `active_descendant`
to represent the highlighted item:

- The highlighted item must be ensured visible (scroll into view) before setting
  `active_descendant`, or `active_descendant` must be cleared while it is not visible.
- The highlighted item must expose collection metadata (`pos_in_set`/`set_size`) when known, so AT
  announcements remain informative.

This avoids “dangling” active descendant references and prevents AT drift under recycling.

### 4) Ownership boundaries (enforced by layering)

Runtime (`crates/fret-ui`):

- Owns snapshot production and the ability for elements/widgets to set:
  - `role`, `label`, `value`, `flags`, `actions`,
  - `active_descendant`,
  - `pos_in_set` / `set_size` (new).
- Must remain policy-free (ADR 0066).

Component layer (`ecosystem/fret-ui-kit` / `ecosystem/fret-ui-shadcn`):

- Owns “which nodes are exposed” for a given surface (menus, lists, comboboxes, cmdk).
- Owns scroll/virtualization policy and ensures:
  - stable item keys (ADR 0047),
  - visible window selection,
  - active descendant correctness (ADR 0073),
  - correct collection metadata assignments.

Platform bridge (`crates/fret-runner-winit` + `crates/fret-a11y-accesskit`):

- Maps `pos_in_set` / `set_size` and `active_descendant` into AccessKit in the best possible way on
  each platform (mapping lives in `crates/fret-a11y-accesskit`, with backend glue in `crates/fret-runner-winit`).
- May adopt richer platform-native virtualization patterns later (P1), but must preserve the P0
  portable behavior.

### 5) Future direction: platform-native virtualization (P1, optional)

Once P0 is stable, we can optionally implement richer patterns (platform-specific) without changing
the portable contract:

- Windows UIA virtualization patterns (where supported by AccessKit).
- macOS AX table/outline patterns (where applicable).

These enhancements must be additive and must not require `fret-ui` to grow component policy.

## Consequences

Pros:

- Locks a portable baseline that prevents late-stage rewrites.
- Enables informative announcements like “Item 57 of 1200” even with windowed snapshots.
- Keeps the runtime clean and pushes policy to `fret-components-*` (ADR 0066).

Cons:

- P0 does not guarantee that AT can navigate to off-screen items without scrolling.
- Requires careful item key discipline and tests to prevent regressions.

## Alternatives Considered

### A) Expose all items in the semantics tree (no virtualization)

Pros:
- Simplest for AT; full navigation.

Cons:
- Not scalable for editor-grade lists; can explode snapshot size and platform bridge costs.

### B) Fixed-size semantics pool (truncate to N items)

Pros:
- Simple implementation.

Cons:
- Breaks accessibility for long lists; becomes a hidden “cap” that leaks into UX.

### C) Only implement platform-native virtualization, no portable contract

Pros:
- Potentially best user experience per platform.

Cons:
- High complexity and high drift risk; would delay shadcn surface scaling and likely cause forks.

## Conformance Checklist

- A virtualized list surface exposes correct `pos_in_set`/`set_size` for visible items.
- `active_descendant` never points to a node outside the current snapshot or modal barrier scope.
- When selection changes, AT announcements remain stable (no random node identity churn while the
  item is visible).

## Suggested Implementation Plan (incremental)

### Phase A — Schema + bridge (framework-level, minimal)

1) Extend `fret_core::SemanticsNode` with `pos_in_set` and `set_size`.
2) Extend `fret-ui` authoring surface (`SemanticsCx`) so elements can set these fields.
3) Map these fields into AccessKit in `crates/fret-a11y-accesskit/src/lib.rs` (best-effort on each platform),
   and wire the backend adapter in `crates/fret-runner-winit/src/accessibility.rs`.
4) Add a focused unit test in `crates/fret-a11y-accesskit` to ensure the AccessKit node receives the
   expected metadata for a `ListItem`.

### Phase B — Component policy wiring (virtualized list surfaces)

1) Add a small helper in `ecosystem/fret-ui-kit` for “collection semantics stamping”:
   - given `(index, count)` assign `pos_in_set`/`set_size` to item semantics,
   - enforce the 1-based invariant.
2) Update at least one dogfooding surface in `ecosystem/fret-ui-shadcn` (e.g. `Command` or a
   virtualized list demo) to populate the metadata.
3) Add regression tests in `ecosystem/fret-ui-kit` that validate:
   - metadata correctness for visible items,
   - `active_descendant` behavior when selection changes and when a modal barrier is present.

### Phase C — Improve AT navigation for very large lists (optional P1)

1) Decide whether to implement platform-native virtualization patterns behind `fret-platform` as
   an additive enhancement.
2) If we do, keep the P0 portable snapshot contract intact and add platform-specific tests.

## Implementation Status (Current Workspace)

P0 baseline is implemented end-to-end:

- Semantics schema: `crates/fret-core/src/semantics.rs` (`active_descendant`, `pos_in_set`, `set_size`).
- Runtime snapshot production: `crates/fret-ui/src/tree/mod.rs` (collects/stamps collection metadata and enforces basic invariants).
- AccessKit mapping: `crates/fret-a11y-accesskit/src/lib.rs` (maps to `active_descendant`, `position_in_set`, `size_of_set`).
- Component policy helpers and adoption:
  - `ecosystem/fret-ui-kit/src/declarative/collection_semantics.rs`
  - `ecosystem/fret-ui-kit/src/declarative/list.rs` (virtualized list stamps list item metadata)
  - shadcn surfaces (cmdk/select/context menu) assert metadata in tests

Follow-ups remain P1:

- richer platform-native virtualization patterns (if desired) without changing the P0 portable contract.

## References

- Semantics tree + bridge boundary: `docs/adr/0033-semantics-tree-and-accessibility-bridge.md`
- Runtime contract surface: `docs/adr/0066-fret-ui-runtime-contract-surface.md`
- Multi-root overlays + barriers: `docs/adr/0011-overlays-and-multi-root.md`
- Active descendant: `docs/adr/0073-active-descendant-and-composite-widget-semantics.md`
- Virtualization + stable keys: `docs/adr/0047-virtual-list-data-source-and-stable-item-keys.md`,
  `docs/adr/0070-virtualization-contract.md`
- A11y manual checklist: `docs/a11y-acceptance-checklist.md`
