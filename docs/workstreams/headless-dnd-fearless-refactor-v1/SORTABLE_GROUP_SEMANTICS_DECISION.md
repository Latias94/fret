# Sortable Group Semantics Decision

Date: 2026-03-13

## Decision

Defer shared sortable group semantics from v1.

Keep `ecosystem/fret-dnd::sortable` limited to the current minimal insertion helper:

- collision chooses the over item
- pointer position resolves `InsertionSide::{Before, After}`

Do not add shared group/index transfer semantics to the headless core or UI-kit adapter yet.

## Why this is the right v1 boundary

### 1) The current shared sortable surface is intentionally minimal

`fret-dnd::sortable` currently provides:

- `insertion_side_for_pointer(...)`
- `sortable_insertion(...)`

That is a narrow, durable mechanism layer. It does not yet own:

- group membership,
- initial vs current group/index tracking,
- cross-list transfer rules,
- empty-container fallback semantics,
- optimistic sortable preview behavior.

### 2) Kanban is a real consumer, but still product-local

Kanban clearly needs sortable group semantics:

- cards move within a column and across columns,
- columns themselves are droppable,
- drop behavior differs for card-over-card vs card-over-column,
- optimistic preview and final reorder both depend on board-specific column/item rules.

But Kanban currently owns all of that itself:

- local `KanbanDragPlan`
- `apply_drop_reorder(...)`
- local `is_column_id(...)`
- local preview spacing and gap logic

That is evidence of one strong consumer, not evidence of a shared contract that is already stable.

### 3) Other first-party sortable-ish flows are not the same contract yet

The other first-party drag surfaces do not currently prove the same shared “sortable groups”
abstraction:

- `sortable_reorder_list` is single-list only
- workspace tab strip reordering is pane/tab-strip specific and coupled to cross-window hand-off
- docking tab insertion uses product-local geometry and internal drag routing

Those surfaces may become future consumers, but they do not currently share the same group/index
contract as Kanban.

### 4) Upstream `dnd-kit` places group semantics above the bare droppable core

`dnd-kit` multiple-list behavior is built through sortable-level state and plugins:

- `Sortable` instances carry `index` and optional `group`
- optimistic sorting tracks `initialIndex` / `initialGroup`
- multiple-list transfer uses sortable/group semantics plus additional column droppables

That means the parity lesson is not “widen `Droppable` more.” The lesson is that group semantics
belong in a sortable-layer abstraction once there are enough real consumers.

## Revisit trigger

Re-open this decision only when at least two consumers need the same shared sortable-group
contract, for example:

- Kanban plus another first-party multi-list surface,
- tree/list reparenting with stable group semantics,
- a reusable board/list recipe in `fret-ui-kit` that can no longer stay product-local.

## Constraints if revisited later

If shared sortable group semantics are added later, they should:

- build on top of the existing rect-based collision/insertion helpers,
- live in the sortable layer (`fret-dnd::sortable` and/or `fret-ui-kit` recipes), not in bare
  `Droppable`,
- preserve product ownership for final model mutation,
- keep empty-container and optimistic-preview policy configurable rather than hard-coded for Kanban.

## Evidence anchors

- `ecosystem/fret-dnd/src/sortable.rs`
- `ecosystem/fret-ui-kit/src/recipes/sortable_dnd.rs`
- `ecosystem/fret-ui-shadcn/src/extras/kanban.rs`
- `ecosystem/fret-workspace/src/tab_strip/mod.rs`
- `ecosystem/fret-docking/src/dock/space.rs`
- `repo-ref/dnd-kit/apps/docs/concepts/sortable.mdx`
- `repo-ref/dnd-kit/apps/docs/react/guides/multiple-sortable-lists.mdx`
- `repo-ref/dnd-kit/packages/dom/src/sortable/sortable.ts`
- `repo-ref/dnd-kit/packages/dom/src/sortable/plugins/OptimisticSortingPlugin.ts`
