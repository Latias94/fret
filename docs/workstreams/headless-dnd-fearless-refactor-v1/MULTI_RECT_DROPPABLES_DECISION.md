# Multi-Rect Droppables Decision

Date: 2026-03-13

## Decision

Defer multi-rect droppables from v1.

Do not widen `ecosystem/fret-dnd::Droppable` or the UI-kit registry surface to support one
semantic droppable ID owning multiple rects yet.

Keep the v1 core snapshot rect-based and single-rect-per-droppable, and let current domain
surfaces continue using product-local geometry when they need richer hit regions.

## Why this is the right v1 boundary

### 1) The current contract is explicitly rect-only

ADR 0157 already locks the v1 registry snapshot as rect-based:

- `Droppable { id, rect, disabled, z_index }`
- deterministic ordering over rect-based candidates

That is not an implementation accident. It is the current contract.

### 2) The current registry shape assumes one rect per `DndItemId`

The UI-kit registry stores:

- snapshot entries as `Vec<Droppable>`
- per-id lookup as `HashMap<DndItemId, Rect>`

That makes single-rect lookup a first-class assumption today, including helper access such as
`droppable_rect_in_scope(...)`.

### 3) Current first-party DnD consumers do not demonstrate a shared multi-rect contract

The first-party registry-driven consumers all still model one semantic entity with one rect:

- sortable recipe: one item, one row rect
- Kanban cards: one card, one rect
- Kanban columns: one column, one rect

That is enough to prove the single-rect contract remains truthful for the current shared surface.

### 4) The strongest “multi-region” candidates still use product-local geometry

Workspace tab strip and docking tab bar do not currently need one `DndItemId` with many rects in
the shared registry.

Instead, they keep richer geometry locally:

- workspace tab strip tracks tab rects, pinned-boundary rect, end-drop rect, and scroll controls
- docking tab insertion/hover uses product-local geometry and internal drag routing

Those are domain hit models, not evidence that `fret-dnd` needs a generalized multi-rect snapshot.

### 5) Node/canvas flows are also still domain-local

The node graph’s richer hit targets (edge anchors, edge hits, insert-node previews) currently live
inside its own canvas interaction model and internal drag handling.

That makes node/canvas a possible future trigger, but not a current registry-driven consumer of a
shared multi-rect droppable contract.

### 6) Upstream `dnd-kit` does not force a core multi-rect outcome

`dnd-kit`’s abstract/dom droppable model is richer in metadata and shape handling, but its core
entity model is still a singular `shape`, not an explicit “one ID owns many rects” public contract.

Its multiple-sortable-lists guidance solves empty columns by adding more droppable entities and
group semantics, not by requiring a multi-rect droppable snapshot.

That means Fret should not widen its core snapshot speculatively just in the name of parity.

## Revisit trigger

Re-open this decision only when at least two shared registry-driven consumers need the same
contract for one semantic droppable spanning multiple rects, for example:

- node graph ports/anchors and another first-party surface both want one semantic drop target over
  multiple disjoint hit regions,
- virtualized sortable/grouped lists need a shared “logical item across multiple visible rects”
  contract,
- docking/workspace converge on the same registry-driven multi-zone drop target model instead of
  product-local geometry.

## Constraints if revisited later

If multi-rect support is added later, it should:

- remain data-only in `fret-dnd`,
- preserve deterministic collision ordering,
- avoid breaking single-rect helpers for existing consumers,
- keep per-window/per-scope registry ownership in `fret-ui-kit`,
- not encode workspace/docking/node-specific semantics into the core droppable type.

## Evidence anchors

- `docs/adr/0157-headless-dnd-v1-contract-surface.md`
- `ecosystem/fret-dnd/src/registry.rs`
- `ecosystem/fret-ui-kit/src/dnd/registry.rs`
- `ecosystem/fret-ui-kit/src/recipes/sortable_dnd.rs`
- `ecosystem/fret-ui-shadcn/src/extras/kanban.rs`
- `ecosystem/fret-workspace/src/tab_strip/geometry.rs`
- `ecosystem/fret-workspace/src/tab_strip/mod.rs`
- `ecosystem/fret-docking/src/dock/space.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/insert_node_drag/pending.rs`
- `repo-ref/dnd-kit/packages/abstract/src/core/entities/droppable/droppable.ts`
- `repo-ref/dnd-kit/apps/docs/react/guides/multiple-sortable-lists.mdx`
