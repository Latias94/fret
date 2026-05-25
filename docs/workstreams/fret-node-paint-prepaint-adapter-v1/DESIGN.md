# Fret Node Paint Prepaint Adapter v1

Status: Active
Last updated: 2026-05-25

## Why This Lane Exists

`fret-node-low-level-adapter-v1` proved retained compatibility seams for low-level host operations
and command dispatch. Paint and prepaint need their own lane because the retained node graph paint
tree is broad: dozens of modules still take `PaintCx` directly, while prepaint owns cull-window
diagnostics and virtual-surface style frame preparation.

Trying to migrate all paint/prepaint code inside the low-level adapter lane would blur scope. This
lane starts with one small adapter proof and grows only through independently verifiable slices.

## Relevant Authority

- `docs/adr/0330-retained-runtime-internal-and-compat-surface.md`
- `docs/adr/0128-canvas-widgets-and-interactive-surfaces.md`
- `docs/adr/0175-prepaint-windowed-virtual-surfaces.md`
- `docs/workstreams/fret-node-low-level-adapter-v1/CLOSEOUT_AUDIT_2026-05-25.md`
- `ecosystem/fret-node/src/ui/canvas/widget/retained_widget_runtime_paint.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/retained_widget_cull_window.rs`

## Problem

Paint and prepaint are still lifecycle-shaped retained entrypoints. `paint_retained_widget` directly
uses `PaintCx` and calls into the retained paint module tree. `prepaint_cull_window` directly uses
`PrepaintCx` for bounds, view-state sync, and cull-window debug recording. The first adapter proof
should avoid a full renderer rewrite and instead isolate one lifecycle operation behind a named
node graph adapter.

## Target State

- Prepaint cull-window operations and/or paint root scene emission have named retained-agnostic
  adapter seams.
- Retained `PrepaintCx` and `PaintCx` bindings are isolated in explicit retained adapter modules.
- The broad paint module tree can migrate one operation family at a time.
- Source-policy tests prevent new policy helpers from directly taking retained paint/prepaint
  contexts unless they are retained adapter bindings or current compatibility oracles.

## In Scope

- Prepaint cull-window key/bounds/debug-record adapter proof.
- Paint root entrypoint adapter proof if it can be limited to one operation family.
- Source-policy tests that distinguish retained adapter modules from retained-agnostic helpers.

## Out Of Scope

- Rewriting the full paint tree.
- Changing renderer contracts or `fret-render`.
- Event routing and command dispatch.
- Public node graph style or skinning changes.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Prepaint cull-window is the smallest first proof. | Likely | `retained_widget_cull_window.rs` and `retained_widget_cull_window_shift.rs` are much narrower than the paint tree. | If cull-window is too coupled to `PrepaintCx`, first task should become a source audit and split a narrower prepaint debug adapter. |
| Paint root migration must be incremental. | Confident | `find ecosystem/fret-node/src/ui/canvas/widget -maxdepth 3 -type f -name '*paint*.rs'` shows 30+ paint files. | A broad paint adapter attempt would become a high-risk rewrite. |
| ADR 0175 matters for prepaint semantics. | Confident | Cull-window behavior is prepaint-driven and diagnostic-friendly. | The lane must update ADR alignment if it changes prepaint semantics. |

## Architecture Direction

Start with a narrow prepaint adapter around cull-window inputs and debug recording. Keep paint root
work as a separate milestone until the first prepaint seam proves the retained/agnostic split. Do
not move node graph paint policy into `crates/fret-ui`; this is a node graph compatibility adapter.

## Closeout Condition

This lane can close when one paint/prepaint lifecycle operation is behind a named adapter, retained
bindings are explicit, gates prove default and compat `fret-node`, and broader paint tree migration
is split into follow-ons.
