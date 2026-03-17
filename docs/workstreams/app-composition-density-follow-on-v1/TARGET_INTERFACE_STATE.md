# App Composition Density Follow-on v1 — Target Interface State

Status: target state for the narrow default app-lane density follow-on
Last updated: 2026-03-17

This document records the intended end state for the remaining default app-lane ceremony cleanup.

It freezes target properties first.
Exact method names may still change during the workstream, but the boundaries below should not.

## Target reading rule

Ordinary app authors should still be able to explain the default path with the same grouped nouns:

- `cx.state()`
- `cx.actions()`
- `cx.data()`
- `cx.effects()`

This lane must make that story shorter, not wider.

## Target posture by area

| Area | Target posture | Should remain explicit | Should stop being taught as first-contact ceremony |
| --- | --- | --- | --- |
| App-shell wrappers | one obvious default story for page/root/card-content composition on the app lane | wrapper ownership and layout intent remain explicit | wrapper chains whose only job is to transport one already-typed child through multiple closure layers |
| Single-child landing | keep `ui::single(cx, child)` as the default single-child landing rule | one-child vs many-children distinction | using `children![cx; child]` or extra closure forwarding for the same one-child case |
| Flex/stack transport | stacks/rows should read like layout intent, not transport glue | gap, alignment, sizing, and keyed identity stay explicit | wrapper-only `v_flex` / `h_flex` shells that exist only to host one already-composed child |
| Query reads | keep `cx.data().query(...)` + `handle.read_layout(cx)` as the default app-lane read posture | query lifecycle (`loading/error/success`) remains visible | reopening raw watch/plumbing chains as another default read dialect |
| Query invalidation | one grouped app-lane invalidation story that also owns redraw behavior | key vs namespace choice, cancellation, and cache policy remain explicit | raw `with_query_client(...)` + `request_redraw(...)` shell code as the first-contact app recipe |
| Router | keep router on its explicit `fret::router::*` seam | route codec/store/outlet/history ownership | pulling router onto generic app-lane composition or data helpers |

## Concrete target properties

1. A default app helper that only wraps one already-typed child should require at most one
   intentional landing seam per helper level.
2. Default app root/page helpers should not need both:
   - a wrapper closure that only forwards the child, and
   - a second explicit transport-only landing step
   unless the helper actually creates new nodes, reads tracked state, or owns diagnostics/a11y
   metadata.
3. Query invalidation on `AppUi` should be expressible in one grouped step rather than in repeated
   raw client plumbing.
4. The settled read-side and write-side default budgets should remain unchanged:
   - `cx.data().selector_layout(...)`
   - `cx.data().query(...)`
   - `handle.read_layout(cx)`
   - `cx.actions().locals(...)`
   - the one-slot write trio
   - `payload_local_update_if::<A>(...)`
5. Any new grouped helper must allow hard deletion of the displaced first-contact wording from
   docs/examples/templates once it lands.

## Explicit non-targets

This workstream does **not** aim to decide:

- a new selector/query read API family,
- a new `cx.actions()` family,
- router sugar for app shells,
- component/advanced authoring helper cleanup,
- a macro/DSL-first authoring model,
- or a prelude widening pass.

## Promotion rule for new shared API

Do not promote a new shared helper unless all of the following are true:

1. the same ceremony appears on Todo and at least one additional non-Todo default app surface,
2. stricter docs or existing helpers do not already solve it,
3. the helper stays on the app-facing grouped surface rather than leaking into lower layers,
4. router and advanced/component lanes do not need to adopt it,
5. the old first-contact wording can be removed afterward.

## Settled baseline this workstream reads from

These are already frozen inputs, not redesign questions:

- the app/component/advanced lane split
- the grouped app authoring surface
- `ui::single(cx, child)` as the single-child landing helper
- `ui::for_each_keyed(...)` as the keyed-list identity story
- `cx.data().selector_layout(...)` as the default selector story
- `cx.data().query(...)` + `handle.read_layout(cx)` as the default query read story
- the current `cx.actions()` default write budget
- router as an explicit adjacent seam

## Done-state summary

The done state is not "Fret now hides all runtime ownership".

It is:

- default app composition reads more like UI and less like transport glue,
- query invalidation no longer breaks the grouped app-lane dialect,
- and docs/examples/templates/gates all teach the same narrower story.
