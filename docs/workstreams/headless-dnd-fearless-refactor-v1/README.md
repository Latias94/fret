# Headless DnD (dnd-kit Alignment) — Fearless Refactor v1

This workstream defines the refactor plan for Fret's headless drag-and-drop stack:

- `ecosystem/fret-dnd`
- `ecosystem/fret-ui-kit::dnd`
- first-party adopters such as docking, workspace/tab strips, sortable recipes, Kanban, and node/canvas flows

The goal is not to copy `dnd-kit`'s framework adapters or DOM assumptions. The goal is to align the
**architectural outcomes** that make `dnd-kit` durable:

- a small headless core with stable vocabulary,
- a central data-only drag operation/engine model,
- thin UI integration adapters,
- extension seams for sensors/modifiers/auto-scroll/monitoring,
- reusable sortable and cross-surface recipes.

This directory contains:

- `DESIGN.md`: ownership, boundaries, current hazards, target architecture, parity plan
- `TODO.md`: execution checklist
- `MILESTONES.md`: staged acceptance criteria and gates
- `CRATE_AUDIT.md`: boundary snapshot, current parity gaps, and landable follow-on steps
- `DROPPABLE_METADATA_DECISION.md`: v1 decision note for `type` / `accept` / collision-hook
  metadata
- `MONITOR_SURFACE_DECISION.md`: v1 decision note for shared monitor/event-surface extraction
- `KEYBOARD_SENSOR_DESIGN_NOTE.md`: follow-on ownership and behavior note for keyboard-driven DnD
- `AUTO_SCROLL_DRIVER_DESIGN_NOTE.md`: follow-on ownership and extraction note for continuous
  drag auto-scroll drivers
- `MULTI_RECT_DROPPABLES_DECISION.md`: v1 decision note for one-droppable-many-rects support

Primary contract references:

- `docs/adr/0149-headless-dnd-toolbox-and-ui-integration.md`
- `docs/adr/0157-headless-dnd-v1-contract-surface.md`
- `docs/adr/0155-docking-tab-dnd-contract.md`
- `docs/adr/0041-drag-and-drop-clipboard-and-cross-window-drag-sessions.md`

Primary upstream reference:

- `repo-ref/dnd-kit/`

Current v1 scope decisions:

- droppable metadata widening is deferred until at least two real consumers need the same shared
  contract
- monitor/event-surface extraction is deferred until at least two cross-cutting observers need the
  same shared contract
- modifier and auto-scroll core stay pure/data-only; actual scrolling side effects remain in
  product/integration code
- multi-rect droppables are deferred until at least two registry-driven consumers need the same
  shared contract
