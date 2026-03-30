# imui shell transparent payload z-order v1 - design

Status: closed on 2026-03-30 (see `CLOSEOUT_AUDIT_2026-03-30.md`)

Last updated: 2026-03-30

Related:

- `docs/workstreams/imui-shell-ghost-choreography-v1/CLOSEOUT_AUDIT_2026-03-30.md`
- `docs/workstreams/docking-multiviewport-arbitration-v1/`
- `docs/workstreams/docking-hovered-window-contract-v1/`
- `docs/workstreams/imui-shell-transparent-payload-zorder-v1/CLOSEOUT_AUDIT_2026-03-30.md`

## Purpose

This workstream is the direct follow-on after the first docking-owned shell ghost choreography
slice closed.

That lane answered one narrow question:

- when a dock drag is still shell-local, show the payload ghost in `current_window`,
- and once the runner reports a real `moving_window`, suppress the in-window payload ghost.

It explicitly did **not** close the next question:

> how should transparent moving-window presentation, hit-test passthrough, and under-window
> routing compose when multiple real OS windows overlap during a dock drag?

This lane exists to lock that z-order / transparent-payload contract before more shell-specific
glue drifts into demos or runner effects.

## Current assessment

Fret already has most of the raw ingredients:

- runner diagnostics for `transparent_payload_applied`,
- runner diagnostics for hit-test passthrough application,
- `window_under_moving_window` and its source,
- docking-local payload ghost suppression once `moving_window` exists,
- and first-party arbitration scripts for transparent payload overlap cases.

What remains unresolved is not whether overlap routing exists.
What remains unresolved is the full shell-visible contract for:

- moving-window visual posture,
- z-order expectations while overlapping another window,
- preview continuity in the under-window target,
- and deterministic gates for those outcomes.

## Goals

### G1 - Freeze the owner split for transparent moving-window choreography

This lane must keep the ownership line explicit:

- runner / runtime crates own moving-window truth, passthrough application, and diagnostics,
- docking-aware layers own docking-specific preview continuity expectations,
- and generic recipe or `imui` layers do not absorb transparent-payload shell policy.

### G2 - Define one coherent overlap contract

The next stable contract must answer:

- what shell-visible posture the moving window should take under transparent payload,
- how under-window routing should behave while overlap persists,
- and which visual feedback remains allowed in the under-window target versus the moving window.

### G3 - Require first-party launched proof, not scene-only evidence

This lane must be proven through launched multi-window arbitration scripts.
Scene tests may support the work, but they are not sufficient closeout evidence by themselves.

## Non-goals

- reopening the already closed payload-ghost visibility rule,
- widening generic cross-window recipe ownership,
- native external drag images,
- aggregate multi-item preview in v1.

## Preferred proof surface

The first proof surface should stay in:

- `apps/fret-examples/src/docking_arbitration_demo.rs`

with the primary script gates:

- `tools/diag-scripts/docking/arbitration/docking-arbitration-demo-multiwindow-transparent-payload-zorder-switch.json`
- `tools/diag-scripts/docking/arbitration/docking-arbitration-demo-multiwindow-large-transparent-payload-zorder-switch.json`

## Exit criteria

This lane can close only when the repo can answer all of these clearly:

- which layer owns transparent moving-window posture and diagnostics,
- what overlap / z-order behavior is expected while transparent payload is active,
- how under-window routing and docking previews remain continuous under overlap,
- what launched regression artifacts prove those claims,
- and which wider shell preview questions remain deferred.
