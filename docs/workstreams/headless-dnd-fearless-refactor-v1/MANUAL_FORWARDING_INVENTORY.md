# Remaining Manual DnD Forwarding Inventory

This note captures the remaining first-party call sites that still hand-wire pointer events into
`fret-ui-kit::dnd` instead of using `DndPointerForwarders`.

## Current status

- Preferred path in first-party recipe/teaching surfaces:
  - `ecosystem/fret-ui-kit/src/recipes/sortable_dnd.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/carousel/demo.rs`
- Remaining manual forwarding is now concentrated in flows that still need bespoke lifecycle
  coupling beyond a simple pointer-region adapter.

## Remaining manual call sites

### 1) Kanban card drag recipe

- File: `ecosystem/fret-ui-shadcn/src/extras/kanban.rs`
- Why still manual:
  - card-local state still derives extra outputs from each DnD update (`translation`, `origin_rect`,
    `over_side`),
  - pointer capture intentionally starts on drag activation rather than on pointer-down,
  - drop semantics mix card-vs-column targets and recipe-owned reorder policy.
- Recommended next step:
  - migrate to `DndPointerForwarders` with `capture_pointer_on_down(false)` and keep the extra
    Kanban state updates in the `on_update` callback / wrapper closures.

### 2) Workspace tab strip pre-drag detection

- Files:
  - `ecosystem/fret-workspace/src/tab_strip/mod.rs`
  - `ecosystem/fret-workspace/src/tab_strip/interaction.rs`
- Why still manual:
  - this path uses `handle_sensor_move_or_init_in_scope(...)` as a pre-cross-window activation gate
    before entering internal drag routing,
  - it is not a plain window-local reorder interaction and still needs direct control over sensor
    lifecycle and hand-off into workspace drag state.
- Recommended next step:
  - keep manual until we decide whether `DndPointerForwarders` should grow a narrow
    sensor-only/activation-only wrapper, or whether workspace tabs should keep their specialized
    pre-drag path.

### 3) Node insert pre-cross-window drag activation

- File: `ecosystem/fret-node/src/ui/canvas/widget/insert_node_drag/pending.rs`
- Why still manual:
  - this path uses `handle_pointer_move_or_init_in_scope(...)` only to determine when a local
    pending gesture should escalate into a cross-window/internal drag,
  - it is not a retained pointer-region recipe and does not map cleanly to the current forwarder
    shape.
- Recommended next step:
  - revisit after the workspace-tab pre-drag path, because both flows want an activation-only seam
    rather than a full pointer-region adapter.
