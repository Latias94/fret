# Selector / Query Authoring Density (Fearless Refactor v1) — Target Interface State

Status: active
Last updated: 2026-03-20

Companion docs:

- `DESIGN.md`
- `MILESTONES.md`
- `TODO.md`

This file freezes the target posture for the new selector/query density lane.

## Target posture

| Need | Target posture | Must stay explicit | Must stop being repeated authoring noise |
| --- | --- | --- | --- |
| Query create-side | `QueryKey` + `QueryPolicy` + `cx.data().query*` remain explicit | key, policy, fetch ownership | helper families that hide the real query lifecycle |
| Query read-side semantics | `handle.read_layout(cx)` remains the default app-lane read | loading/error/success semantics | repeatedly rebuilding the same semantic status/refetch checks in app code |
| Query status projections | light semantic helpers on `QueryStatus` / `QueryState<T>` are allowed | actual lifecycle still visible | repeated string/status/presence boilerplate |
| Selector create-side | `cx.data().selector_layout(inputs, compute)` remains the default LocalState-first lane | invalidation phase and explicit inputs | LocalState-first churn that exists only because borrowed projection is unavailable |
| Shared-model selector lane | explicit `selector(...)` / shared-model grouped helpers remain available | ownership of shared graphs | leaking shared-model choreography into first-contact guidance |
| Router | adjacent only | route/history ownership | pulling router into this lane without new evidence |

## Concrete target properties

1. Query demos/templates should stop spelling the same status-to-label and refreshing checks over
   and over when those checks are purely semantic projections of the existing state machine.
2. The first reduction in this lane must not add shadcn-specific policy into `fret-query`.
3. Selector follow-on work, if any, must preserve explicit invalidation and stay on the app-facing
   layer.
4. No part of this lane should widen `fret::app::prelude::*`.
5. No new helper should land from Todo-only pressure.

## Current interpretation

This lane is not reopening the earlier dataflow closeout.

It is starting from that closed posture and asking a narrower question:

- now that the ownership boundaries are correct,
- what remaining selector/query authoring density is still accidental?
