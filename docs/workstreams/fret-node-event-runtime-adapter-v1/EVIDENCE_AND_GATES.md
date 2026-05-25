# Fret Node Event Runtime Adapter v1 - Evidence And Gates

Status: Active
Last updated: 2026-05-25

## Smallest Current Repro

```bash
cargo check -p fret-node --features compat-retained-canvas
```

## Gate Set

### Scope Gate

```bash
python3 -m json.tool docs/workstreams/fret-node-event-runtime-adapter-v1/WORKSTREAM.json
python3 tools/check_workstream_catalog.py
```

### Targeted Iteration Gate

```bash
cargo test -p fret-node --features compat-retained-canvas event_runtime_adapter
```

### Package Gates

```bash
cargo check -p fret-node
cargo check -p fret-node --features compat-retained-canvas
```

### Boundary Gate

```bash
python3 tools/check_layering.py
git diff --check
```

## Evidence Anchors

- `docs/workstreams/fret-node-event-runtime-adapter-v1/DESIGN.md`
- `ecosystem/fret-node/src/ui/canvas/widget/event_runtime_adapter.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/event_router.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/event_router_cx.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/retained_widget_runtime_event.rs`
- `ecosystem/fret-node/src/lib.rs`

## Initial Scope Evidence - 2026-05-25

Claim:

- Event routing is separate from the command dispatch and paint/prepaint adapter families.
- The first proof should target retained event runtime entrypoint preparation, because route
  internals already compose retained-agnostic route traits.

Fresh validation:

- `cargo fmt --package fret-node` passed.
- `cargo test -p fret-node --features compat-retained-canvas event_runtime_adapter` passed.
- `cargo check -p fret-node` passed.
- `cargo check -p fret-node --features compat-retained-canvas` passed.
- `python3 -m json.tool docs/workstreams/fret-node-event-runtime-adapter-v1/WORKSTREAM.json` passed.
- `python3 tools/check_workstream_catalog.py` passed.
- `python3 tools/check_layering.py` passed.
- `git diff --check` passed.

Notes:

- This lane intentionally starts after `fret-node-low-level-adapter-v1`; do not reopen that lane for
  event runtime adapter work.

## NEA-020 Adapter Proof - 2026-05-25

Claim:

- `event_runtime_adapter.rs` owns retained-agnostic event route preparation and dispatch through
  `CanvasEventRuntimeCx` and `dispatch_canvas_event`.
- `retained_widget_runtime_event.rs` now binds retained `EventCx` to that adapter and no longer owns
  runtime theme sync, view state sync, bounds update, or direct route dispatch.
- Command dispatch and paint/prepaint remain outside this lane.

Fresh validation:

- `cargo fmt --package fret-node` passed.
- `cargo test -p fret-node --features compat-retained-canvas event_runtime_adapter` passed.
- `cargo check -p fret-node` passed.
- `cargo check -p fret-node --features compat-retained-canvas` passed.
- `python3 -m json.tool docs/workstreams/fret-node-event-runtime-adapter-v1/WORKSTREAM.json` passed.
- `python3 tools/check_workstream_catalog.py` passed.
- `python3 tools/check_layering.py` passed.
- `git diff --check` passed.

Evidence:

- `ecosystem/fret-node/src/ui/canvas/widget/event_runtime_adapter.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/retained_widget_runtime_event.rs`
- `ecosystem/fret-node/src/lib.rs`
