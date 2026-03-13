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

Primary contract references:

- `docs/adr/0149-headless-dnd-toolbox-and-ui-integration.md`
- `docs/adr/0157-headless-dnd-v1-contract-surface.md`
- `docs/adr/0155-docking-tab-dnd-contract.md`
- `docs/adr/0041-drag-and-drop-clipboard-and-cross-window-drag-sessions.md`

Primary upstream reference:

- `repo-ref/dnd-kit/`
