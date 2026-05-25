# Fret Node Event Runtime Adapter v1

Status: Closed
Last updated: 2026-05-25

Closed on 2026-05-25 after the event runtime adapter seam shipped, the retained event edge audit
found no additional event-runtime edge to delete, and follow-ons were split by behavior family.

## Why This Lane Exists

`fret-node-low-level-adapter-v1` proved the first retained compatibility seams for low-level host
operations and command dispatch. Event routing is a separate behavior family: the route graph is
already mostly expressed through retained-agnostic traits, but the retained widget runtime entrypoint
still binds `EventCx` directly to route orchestration, theme/runtime sync, bounds updates, and view
state sync.

This lane owns that entrypoint split.

## Relevant Authority

- `docs/adr/0330-retained-runtime-internal-and-compat-surface.md`
- `docs/adr/0128-canvas-widgets-and-interactive-surfaces.md`
- `docs/workstreams/fret-node-low-level-adapter-v1/CLOSEOUT_AUDIT_2026-05-25.md`
- `docs/workstreams/fret-node-event-runtime-adapter-v1/CLOSEOUT_AUDIT_2026-05-25.md`
- `ecosystem/fret-node/src/ui/canvas/widget/event_runtime_adapter.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/event_router.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/event_router_cx.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/retained_widget_runtime_event.rs`

## Problem

The current event route tree has good internal decomposition (`EventRouteCx`, `SystemRouteCx`,
`PointerEventRouteCx`, `KeyboardRouteCx`), but the top-level retained runtime path still makes the
retained `EventCx` entrypoint the owner of event preparation. That keeps the compatibility island
alive as a widget lifecycle detail instead of a named node graph runtime adapter.

## Target State

- Event route orchestration is callable through a named retained-agnostic runtime adapter seam.
- Retained `EventCx` binding is isolated in an explicit retained adapter module.
- Source-policy tests prevent new route modules from depending directly on retained `EventCx`,
  `CommandCx`, `LayoutCx`, `PaintCx`, or `PrepaintCx`.
- Default `fret-node` remains declarative, and `compat-retained-canvas` remains the explicit
  compatibility oracle while this lane is active.

## In Scope

- `event_router.rs`, `event_router_cx.rs`, and top-level retained event runtime wiring.
- Route preparation operations: runtime theme sync, view state sync, last bounds update, and route
  dispatch.
- Narrow source-policy tests that lock the retained/agnostic split.

## Out Of Scope

- Command dispatch migration; that belongs to `fret-node-low-level-adapter-v1` follow-ons.
- Paint/prepaint scene emission; that belongs to `fret-node-paint-prepaint-adapter-v1`.
- Rewriting pointer, keyboard, menu, or drag policy.
- Changing public `fret-node` authoring API.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Event routing already has a useful retained-agnostic route trait stack. | Confident | `event_router_cx.rs`, `event_keyboard_route.rs`, `event_pointer_down_route_cx.rs` | The first task may need to split lower-level route traits before the runtime adapter lands. |
| The retained-specific part is the runtime entrypoint preparation, not every route handler. | Likely | `retained_widget_runtime_event.rs` syncs theme, view state, bounds, then calls `handle_event`. | The lane may need a deeper audit task before implementation. |
| A first slice can be proven with source-policy tests and `fret-node` compat checks. | Confident | Prior low-level adapter lane gates in `docs/workstreams/fret-node-low-level-adapter-v1/EVIDENCE_AND_GATES.md` | If source-policy is too weak, add a targeted unit test around the adapter seam. |

## Architecture Direction

Introduce the smallest event runtime adapter that names the preparation contract without moving
event policy into `fret-ui`. Retained `EventCx` should only implement or feed that adapter; routing
modules should remain expressed in node graph terms.

## Closeout Condition

This lane can close when one retained event runtime entrypoint is replaced or quarantined behind a
named adapter, source-policy gates lock the retained split, and follow-on event route migrations are
either completed or split into narrower lanes.

## Closeout Outcome

- The retained event runtime entrypoint is behind `event_runtime_adapter.rs`.
- Source-policy tests lock the adapter and top-level router away from retained Cx terms.
- `NEA-030` found no further old event runtime edge to delete without crossing into other behavior
  families.
- Remaining work is split to `fret-node-paint-prepaint-adapter-v1` or a future route-policy audit
  lane.
