# Droppable Metadata Decision

Date: 2026-03-13

## Question

Should `fret-dnd` v1 add lightweight draggable/droppable metadata now, for example:

- `type`
- `accept`
- per-droppable collision-strategy hooks

## Decision

No. Defer this from the current v1 refactor.

The crate boundary is already correct, and the current first-party evidence is not strong enough to
justify widening the headless registry contract yet.

## Why this is deferred

### 1) `sortable_dnd` does not need typed droppables

The sortable recipe operates on a homogeneous droppable set:

- every registered droppable is a reorderable row,
- `update.over` is consumed directly as another item id,
- insertion semantics come from pointer-vs-rect splitting, not from droppable metadata.

Evidence anchors:

- `ecosystem/fret-ui-kit/src/recipes/sortable_dnd.rs`

### 2) Kanban is one real heterogeneous consumer, but only one

Kanban mixes at least two semantic droppable classes in the same scope:

- cards
- columns

Today it resolves that difference by mapping `DndItemId` back to board data:

- `kanban_card_dnd_id(...)`
- `kanban_column_dnd_id(...)`
- `is_column_id(...)`
- `apply_drop_reorder(...)`

That is evidence that metadata may eventually be useful, but by itself it is not enough reason to
lock a new headless registry contract.

Evidence anchors:

- `ecosystem/fret-ui-shadcn/src/extras/kanban.rs`

### 3) Other first-party DnD-adjacent flows are not second metadata consumers yet

Current additional adopters do not yet require shared droppable metadata:

- workspace tab strip uses the activation-only seam before handing off into workspace-owned
  cross-window drag state,
- node insert drag uses the same activation-only seam before promoting into node-graph drag state,
- docking currently uses activation thresholds and insertion-side helpers, but not a shared
  `fret-ui-kit` droppable registry with mixed semantic target classes.

Evidence anchors:

- `ecosystem/fret-workspace/src/tab_strip/mod.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/insert_node_drag/pending.rs`
- `ecosystem/fret-docking/src/dock/space.rs`

## What would justify revisiting this

Re-open this decision when at least two real consumers need the same shared semantics, for example:

1. heterogeneous droppable classes that should be filtered in the headless layer rather than by
   product-local id decoding;
2. a shared `accept` contract that can be expressed without app-specific model references;
3. a need for per-droppable collision-policy selection that cannot be solved cleanly with scopes or
   post-processing in the integration layer.

Likely candidates if they mature:

- Kanban card-vs-column target filtering
- node graph typed drop zones or typed edge/port drop targeting
- workspace/docking target filtering if they converge on the same registry-driven surface

## Interim rule

Until that threshold is met:

- keep `fret-dnd::Droppable` minimal (`id`, `rect`, `disabled`, `z_index`),
- prefer scopes and consumer-local id decoding over widening the headless contract,
- do not add metadata just to mirror `dnd-kit` terminology without a second consumer.

## Consequence for the current workstream

The M4 metadata item is considered decided for v1:

- decision: defer
- reason: only one first-party heterogeneous consumer currently demonstrates the need
- follow-up: revisit only when a second consumer appears
