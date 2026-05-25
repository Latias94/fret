# Fret Node Event Runtime Adapter v1 Closeout Audit - 2026-05-25

Status: Closed

## Verdict

This lane is closed.

It replaced the retained node graph event runtime entrypoint with a named retained-agnostic adapter
seam and proved that event route preparation can stay out of retained widget lifecycle code. The
lane intentionally stops at the runtime entrypoint boundary. Pointer, keyboard, menu, command, and
paint/prepaint policy work need separate lanes.

## What Shipped

### 1) Event runtime adapter seam

`event_runtime_adapter.rs` defines `CanvasEventRuntimeCx` and `dispatch_canvas_event`. The adapter
owns:

- runtime theme sync,
- view state sync,
- last bounds update,
- route dispatch into the existing event router.

### 2) Retained event binding isolation

`retained_widget_runtime_event.rs` now binds retained `EventCx` to `CanvasEventRuntimeCx` and
forwards to `dispatch_canvas_event`. It no longer directly owns route preparation or direct event
router dispatch.

### 3) Source-policy lock

`ecosystem/fret-node/src/lib.rs` locks the event runtime adapter and top-level event router away
from retained `EventCx`, `CommandCx`, `LayoutCx`, `PaintCx`, `PrepaintCx`, retained bridge, and
compat facade terms.

### 4) Retained edge audit

`NEA-030` found no additional old retained event runtime edge worth deleting inside this lane. The
remaining retained `EventCx` impl files are policy-specific bindings for existing route traits, not
owners of runtime event preparation.

## Evidence

- `ecosystem/fret-node/src/ui/canvas/widget/event_runtime_adapter.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/retained_widget_runtime_event.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/event_router.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/event_router_cx.rs`
- `ecosystem/fret-node/src/lib.rs`
- `docs/workstreams/fret-node-event-runtime-adapter-v1/EVIDENCE_AND_GATES.md`

## Gates

- `cargo fmt --check --package fret-node`
- `cargo test -p fret-node --features compat-retained-canvas event_runtime_adapter`
- `cargo check -p fret-node`
- `cargo check -p fret-node --features compat-retained-canvas`
- `python3 -m json.tool docs/workstreams/fret-node-event-runtime-adapter-v1/WORKSTREAM.json`
- `python3 tools/check_layering.py`
- `python3 tools/check_workstream_catalog.py`
- `git diff --check`

## Follow-On Policy

Do not reopen this lane for:

- command dispatch,
- paint/prepaint scene emission,
- broad pointer or keyboard route-policy rewrites,
- layout or semantics retained runtime work,
- public node graph authoring API changes.

Use follow-on lanes instead:

- `docs/workstreams/fret-node-paint-prepaint-adapter-v1/` for retained paint/prepaint runtime
  preparation and scene emission.
- A future narrow route-policy audit lane if pointer, keyboard, or context-menu route trait binding
  cleanup remains valuable after the runtime entrypoint work.
