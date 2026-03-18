# Tracked Write Pattern Inventory

Status: draft, post-v1 audit
Last updated: 2026-03-08

Related:

- TODO: `docs/workstreams/action-first-authoring-fearless-refactor-v1/TODO.md`
- Milestones: `docs/workstreams/action-first-authoring-fearless-refactor-v1/MILESTONES.md`
- V2 golden path: `docs/workstreams/action-first-authoring-fearless-refactor-v1/V2_GOLDEN_PATH.md`
- Teaching-surface local-state inventory: `docs/workstreams/action-first-authoring-fearless-refactor-v1/TEACHING_SURFACE_LOCAL_STATE_INVENTORY.md`

## Purpose

This note records the remaining post-v1 tracked-write patterns after the following additive ergonomics
steps landed:

- render-side `state.layout(cx).value_*` / `state.paint(cx).value_*`,
- store-side `LocalState::value_in*`,
- handled-aware `LocalState::update_in_if(...)` plus grouped `locals_with((...)).on::<A>(...)` /
  `payload_local_update_if::<A>(...)`.

The goal is to decide whether more API is actually justified, or whether the remaining complexity is
mostly *real coordination* that should stay on `cx.on_action_notify_models::<A>(|models| ...)`.

## Current pattern map

| Pattern | Typical example | Current default | Status | Notes |
| --- | --- | --- | --- | --- |
| Straight local set | counter reset, set step, set filter enum | `on_action_notify_local_set` | Covered | Low noise already. |
| Straight local toggle | simple bool flag | `on_action_notify_toggle_local_bool` | Covered | No new helper needed. |
| Store-side local read | todo draft/id reads inside multi-state transaction | `value_in_or*` | Covered | Matches render-side `value_*` naming now. |
| Collection mutation with handled decision | toggle/remove/retain by id in `LocalState<Vec<_>>` | `locals_with((...)).on::<A>(...)` + `update_in_if`, or `payload_local_update_if::<A>(...)` | Covered | Removes the common `mut handled = false` pattern without leaving a second direct rerender API on `LocalState<T>`. |
| Multi-state transaction touching local + explicit models | add todo, submit form, sync filters | `on_action_notify_models` | Keep as default | This is real coordination, not just syntax noise. |
| Shared explicit-model collections | legacy todo/template flows that still keep nested row models or shared list ownership explicit | `on_action_notify_models` + explicit `Model<T>` reads/writes | Intentional for now | Narrow enough now that surface-by-surface migration evidence is better than new helper design. |
| Runtime/effect transactions | query invalidation, host/runtime side effects, viewport interop | `on_action_notify_models` or render-time escape hatch | Intentional for now | Not a candidate for local-state sugar. |

## Evidence anchors

| Surface | What it demonstrates |
| --- | --- |
| `apps/fret-cookbook/examples/simple_todo.rs` | Default cookbook keyed list now on `LocalState<Vec<_>>` + payload row toggle |
| `apps/fret-cookbook/examples/simple_todo_v2_target.rs` | Denser `LocalState<Vec<_>>` comparison surface with payload-row/root-handler pressure still visible |
| `apps/fretboard/src/scaffold/templates.rs` | Template default path now using `LocalState<Vec<_>>` for the simple keyed-list scaffold while still relying on `on_action_notify_models` for coordinated writes |
| `apps/fret-cookbook/examples/text_input_basics.rs` | Multi-state local transaction that still fits `on_action_notify_models` cleanly |
| `apps/fret-examples/src/hello_counter_demo.rs` | Straight local writes that do not need more than current helpers |

## What no longer looks like the bottleneck

| Topic | Why it is no longer the main pressure |
| --- | --- |
| Store-side local reads | `value_in*` now covers the common path. |
| Handled-aware list writes | `update_in_if` together with `locals_with((...)).on::<A>(...)` / `payload_local_update_if::<A>(...)` covers the common local-collection mutation path. |
| Discrete widget parity | Checkbox/Switch/Toggle action-only parity is already closed elsewhere. |

## What still needs judgment

| Open question | Current recommendation |
| --- | --- |
| Should multi-state transactions get another default helper beyond `on_action_notify_models`? | Not yet. First gather more evidence from intentionally advanced explicit-model surfaces and richer templates. |
| Should shared explicit-model collection writes get a handled-aware helper too? | Probably not by default; the shared-model boundary is usually the reason the code is more explicit. |
| Should payload/local collection flows get dedicated sugar? | Not yet. `on_payload_action_notify` + `update_in_if` is currently readable enough. |

## Recommended next step

| Step | Why |
| --- | --- |
| Keep the helper surface stable after the scaffold migration | Default-surface collection migrations are no longer the main blocker. |
| Avoid adding another helper until two or more real surfaces need the same shape | Keeps the default path stable. |
| Keep documenting the difference between local-state ergonomics and real coordination cost | Prevents helper sprawl from hiding architectural boundaries. |

## Provisional conclusion

For the post-v1 default path, the next meaningful question is no longer "how do we read or toggle local
state with less syntax?" The local-state keyed-list path now covers cookbook, app-grade, and scaffold
default surfaces. The remaining question is:

> when a write spans multiple state buckets, should the framework offer a narrower orchestration
> helper, or is `on_action_notify_models::<A>(|models| ...)` already the right default because the
> underlying coordination is real?

Until more evidence says otherwise, stay on the current default:

- simple local writes -> local-state helpers,
- coordinated transactions -> `on_action_notify_models`,
- runtime/effect flows -> explicit escape hatches.
