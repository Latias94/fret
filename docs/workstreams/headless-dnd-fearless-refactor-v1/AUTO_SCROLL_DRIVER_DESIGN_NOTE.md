# Auto-Scroll Driver Design Note

Date: 2026-03-13

## Status

Design note only.

This note does not extract a shared continuous auto-scroll driver yet. It records why the current
v1 boundary is still acceptable and what contract shape a later extraction should follow.

## Current state

The current stack is intentionally split in two:

- `ecosystem/fret-dnd` computes pure auto-scroll requests/deltas
- first-party product surfaces apply scrolling effects themselves

What is already shared:

- `AutoScrollConfig`
- `AutoScrollRequest`
- `compute_autoscroll(...)`
- `compute_autoscroll_x_clamped(...)`

What is not shared yet:

- the repeated driver loop that keeps applying scroll while drag remains near an edge,
- the lifecycle that starts/stops that loop,
- the follow-up recomputation that keeps drop targets/insertion indices in sync after scrolling.

## Decision summary

### 1) Keep the headless layer pure

`fret-dnd` must continue to stop at data-only output:

- compute request/delta
- clamp against explicit bounds
- return `None` / zero when scrolling should stop

It must not own:

- timers,
- redraw scheduling,
- host scroll mutation,
- window/frame lifecycle,
- post-scroll drop-target recomputation.

### 2) Any future continuous driver belongs in `ecosystem/fret-ui-kit::dnd`, not `fret-dnd`

If Fret extracts a reusable continuous driver later, it should live in the integration layer.

Reason:

- starting/stopping a repeated scroll loop depends on runtime host services,
- consumers need to mutate concrete scroll containers or model state,
- some surfaces depend on per-frame redraw cadence and per-window drag lifecycle.

Those are integration concerns, not headless policy concerns.

### 3) Do not extract a shared driver until at least two consumers converge on the same contract

Today there are two real first-party consumers of edge-driven drag auto-scroll:

- workspace tab strip
- docking tab bar

That proves the value of the shared **math**, but it does not yet prove that a single shared
continuous driver contract is ready:

- both surfaces still combine scrolling with product-local hover/insert-index recomputation,
- both surfaces scroll different state owners,
- `DndUpdate.autoscroll` is exposed truthfully but is not yet the contract those surfaces consume.

So the correct v1 move is to document the target semantics now and defer extraction until at least
two consumers can use the same driver interface.

## Evidence from current consumers

### Workspace tab strip

Workspace uses a local per-frame loop:

- compute `delta_x` with `compute_autoscroll_x_clamped(...)`,
- mutate the scroll handle,
- keep recomputing the drop target while dragging,
- request redraw while drag remains active.

This is a real driver, but it is tightly coupled to workspace tab insertion semantics.

### Docking tab bar

Docking uses the same shared clamped-x helper, but the driver loop is still local:

- it guards against duplicate application within the same frame,
- mutates tab-scroll state owned by docking,
- recomputes insert index after scrolling,
- verifies behavior through an end-to-end drag test.

This is close to workspace structurally, but not yet surfaced through the same reusable adapter
contract.

### `DndUpdate.autoscroll` is not yet the adopted product seam

`fret-ui-kit::dnd` now forwards truthful `autoscroll` output from the headless frame/engine, but no
first-party surface currently consumes that update field as the source of a shared continuous
driver.

That is the strongest reason not to force a premature abstraction.

## Required contract if we extract later

The future reusable driver should:

1. accept pure `AutoScrollRequest`/delta-style input from the headless DnD update stream,
2. apply scrolling only through a consumer-supplied integration callback,
3. request redraw or schedule the next tick only while a non-zero request remains active,
4. stop immediately on drag end/cancel or when the request becomes zero/`None`,
5. leave drop-target/insertion recomputation to either:
   - a consumer callback, or
   - a higher-level recipe layer,
   but not the headless crate.

## Explicit non-goals

- Do not merge this with text-selection auto-scroll in `crates/fret-ui`.
- Do not merge this with code-editor drag auto-scroll yet.
- Do not reintroduce scrolling side effects into `fret-dnd`.
- Do not require every DnD surface to use a timer-driven model; some surfaces may remain redraw- or
  event-driven.

## Extraction trigger

Revisit extraction once at least two consumers can use the same integration-layer driver interface,
for example:

- workspace tab strip and docking both consume `DndUpdate.autoscroll`,
- sortable virtualization needs the same repeated scroll lifecycle,
- keyboard DnD lands and needs the same driver contract as pointer DnD.

## Minimum gates before extraction

When extraction starts, require at least:

1. one unit/integration gate for the driver lifecycle (start, continue, stop),
2. one workspace or docking regression gate that proves insert targets stay aligned while scrolling,
3. one gate that proves zero/`None` requests stop the driver,
4. `python tools/check_layering.py`.

## Evidence anchors

- `docs/adr/0157-headless-dnd-v1-contract-surface.md`
- `ecosystem/fret-dnd/src/scroll.rs`
- `ecosystem/fret-dnd/src/frame.rs`
- `ecosystem/fret-ui-kit/src/dnd/types.rs`
- `ecosystem/fret-ui-kit/src/dnd/tests.rs`
- `ecosystem/fret-workspace/src/tab_strip/kernel.rs`
- `ecosystem/fret-workspace/src/tab_strip/mod.rs`
- `ecosystem/fret-docking/src/dock/space.rs`
- `ecosystem/fret-docking/src/dock/tests/drag.rs`
- `repo-ref/dnd-kit/packages/dom/src/core/plugins/scrolling/AutoScroller.ts`
