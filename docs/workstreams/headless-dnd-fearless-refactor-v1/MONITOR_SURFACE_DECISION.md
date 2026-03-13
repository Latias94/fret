# Monitor Surface Decision

Date: 2026-03-13

## Decision

Defer a shared monitor/event surface from v1.

Do not widen `ecosystem/fret-dnd` or `ecosystem/fret-ui-kit::dnd` with a new observer contract yet.
Keep current observation product-local through `DndPointerForwardersConfig::on_update(...)` until at
least two real consumers need the same shared monitor surface.

## Why this is the right v1 boundary

### 1) Current observation is still local, not cross-cutting

The first-party call sites that currently observe drag updates all do so through local
`on_update(...)` handlers:

- `ecosystem/fret-ui-kit/src/recipes/sortable_dnd.rs`
- `ecosystem/fret-ui-shadcn/src/extras/kanban.rs`
- `apps/fret-ui-gallery/src/ui/snippets/carousel/demo.rs`
- `ecosystem/fret-ui-shadcn/tests/carousel_dnd_arbitration.rs`

Those are integration-local consumers. They do not yet prove a reusable monitor contract.

### 2) `sortable` is not a monitor-surface consumer

`sortable_dnd` consumes `DndUpdate.over` plus `SensorOutput::{DragStart, DragMove}` to maintain a
local insertion model for one homogeneous droppable set.

That is normal recipe logic, not evidence that the headless stack needs a separate event bus or
monitor API.

### 3) `kanban` is the strongest candidate, but still product-local

Kanban interprets `next_over`, `next_over_side`, and origin geometry inside its own authoring
layer, and it also relies on local semantic helpers such as:

- `kanban_card_dnd_id(...)`
- `kanban_column_dnd_id(...)`
- `is_column_id(...)`

This is still product policy, not a generic observer contract shared by multiple consumers.

### 4) Other first-party flows only need activation hand-off today

Workspace tab tear-out and node insert pending flows now share the activation-only seam, but after
activation they hand off to product-owned drag state:

- `ecosystem/fret-workspace/src/tab_strip/mod.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/insert_node_drag/pending.rs`

That means they are not monitor consumers either.

### 5) The `dnd-kit` monitor exists for plugin-style observers that Fret does not yet have

The relevant `dnd-kit` references show the monitor surface serving provider/plugin-style observers
such as shared sorting, diagnostics, accessibility, or scrolling integrations:

- `repo-ref/dnd-kit/apps/docs/concepts/drag-drop-manager.mdx`
- `repo-ref/dnd-kit/packages/react/src/core/hooks/useDragDropMonitor.ts`
- `repo-ref/dnd-kit/packages/dom/src/sortable/plugins/OptimisticSortingPlugin.ts`

Fret does not yet have at least two comparable cross-cutting consumers for the same monitor
contract.

## Revisit trigger

Re-open this decision only when at least two shared consumers appear, for example:

- a diagnostics/debug subscription surface,
- an optimistic sorting/shared sortable plugin,
- accessibility narration or keyboard-sensor observation,
- an auto-scroll integration plugin that should observe central operation transitions.

## Constraints if we revisit later

If a monitor surface is added later, it should remain:

- data-only,
- portable across native and wasm,
- free of DOM assumptions,
- free of product-specific Kanban/docking/node semantics.

It should be derived from the central drag operation/engine truth rather than becoming a second
state owner.
