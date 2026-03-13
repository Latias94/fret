# Remaining Manual DnD Forwarding Inventory

This note captures the remaining first-party call sites that still hand-wire pointer events into
`fret-ui-kit::dnd` instead of using `DndPointerForwarders`.

## Current status

- Preferred path in first-party recipe/teaching surfaces:
  - `ecosystem/fret-ui-kit/src/recipes/sortable_dnd.rs`
  - `ecosystem/fret-ui-shadcn/src/extras/kanban.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/carousel/demo.rs`
- Focused forwarder-backed gates now cover:
  - `ecosystem/fret-ui-kit/src/recipes/sortable_dnd.rs`
  - `ecosystem/fret-ui-shadcn/tests/kanban_dnd_forwarders.rs`
- Activation-only consumers now share a narrow seam instead of calling low-level controller helpers
  directly:
  - `ecosystem/fret-ui-kit/src/dnd/activation_probe.rs`
  - `ecosystem/fret-workspace/src/tab_strip/mod.rs`
  - `ecosystem/fret-node/src/ui/canvas/widget/insert_node_drag/pending.rs`
- Remaining manual forwarding is now concentrated in flows that still need bespoke lifecycle
  coupling after activation, especially cross-window hand-off and drag-state ownership.

## Remaining manual call sites

### 1) Workspace tab strip pre-drag detection

- Files:
  - `ecosystem/fret-workspace/src/tab_strip/mod.rs`
  - `ecosystem/fret-workspace/src/tab_strip/interaction.rs`
- Why still manual:
  - this path now uses `DndActivationProbe` as a pre-cross-window activation gate before entering
    workspace drag routing,
  - it is not a plain window-local reorder interaction and still needs direct control over
    pointer-capture lifecycle, drag-state updates, and cross-window hand-off into workspace models.
- Recommended next step:
  - decide whether a higher-level “activation then begin cross-window drag” helper is worth
    extracting, or whether workspace tabs should keep their specialized pre-drag path.

### 2) Node insert pre-cross-window drag activation

- File: `ecosystem/fret-node/src/ui/canvas/widget/insert_node_drag/pending.rs`
- Why still manual:
  - this path now uses `DndActivationProbe` only to determine when a local pending gesture should
    escalate into a cross-window/internal drag,
  - it is not a retained pointer-region recipe and still owns the promotion from pending insert
    intent into node-graph drag state.
- Recommended next step:
  - revisit together with the workspace-tab path, because both flows now share activation probing
    but still differ in how they hand off into app-specific drag state.
