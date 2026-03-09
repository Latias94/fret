# Action-First Authoring + View Runtime (Fearless Refactor v1) — Invalidation Default Rules

Status: active short policy note
Last updated: 2026-03-09

Related:

- `docs/workstreams/action-first-authoring-fearless-refactor-v1/NOTIFY_POLICY_DECISION_DRAFT.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/INVALIDATION_LOCAL_STATE_REVIEW.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/V2_GOLDEN_PATH.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/TODO.md`

---

## Purpose

This note is the short default-rule card for post-v1 invalidation authoring.

It answers one practical question:

> When should a surface stay on `on_action_notify_models::<A>(...)`, and when is explicit
> `notify()` / render-time invalidation still the correct boundary?

---

## Default rule

For normal teaching-surface authoring:

1. use `on_action_notify_local_*` for straightforward single-local tracked writes,
2. use `on_action_notify_models::<A>(...)` for coordinated writes or root-owned orchestration,
3. use `on_action_notify_transient::<A>(...)` for App/runtime effects that are already modeled as
   transient action handling,
4. keep explicit `notify()` out of the default path.

---

## Decision table

| Situation | Default path | Why |
| --- | --- | --- |
| Single local tracked write | `on_action_notify_local_*` | The write itself already owns redraw + notify semantics. |
| Coordinated write across multiple local/model values | `on_action_notify_models::<A>(...)` | The root action boundary is the real ownership boundary. |
| Command availability / command-root orchestration | `on_action_notify_models::<A>(...)` | This is runtime coordination, not invalidation noise. |
| Cross-field form validation/reset | `on_action_notify_models::<A>(...)` | Validation and availability span multiple fields. |
| View-owned keyed list mutation | existing tracked-write path; narrow payload-row helper only where already adopted | This is not a reason to widen generic invalidation helpers. |
| Query/client invalidation where the real effect is outside the tracked write | explicit render-time invalidation / redraw | The effect boundary is intentionally outside the tracked state write. |
| Imperative host callback or cache-boundary invalidation | explicit `notify()` / redraw | The rerender reason is not represented by a normal tracked write. |

---

## What counts as an escape hatch

Keep explicit `notify()` or render-time invalidation only when at least one of these is true:

- the effect lives outside the tracked write,
- the mutation happens in an imperative host/runtime callback,
- the rerender reason is cache/runtime oriented rather than an ordinary tracked state change,
- hiding the boundary behind a helper would make ownership less clear.

---

## What does **not** justify a new invalidation helper

These should not be treated as proof that the repo still needs broader invalidation sugar:

- root-scoped command/keymap handling,
- cross-field form orchestration,
- query/client invalidation that is intentionally render-time,
- keyed-list handler placement questions that are really about row-action ownership.

Those are different problems and should stay classified separately.

---

## Evidence split

Use the current medium-surface evidence this way:

- `simple_todo_v2_target` -> keyed-list / payload-row handler-placement pressure,
- `query_basics` -> explicit render-time invalidation remains correct,
- `commands_keymap_basics` -> root-scoped command ownership is intentional,
- `form_basics` -> root-scoped validation/reset ownership is intentional.

This is why `AFA-postv1-004` stays on “no new helper yet”.

---

## Practical review rule

Before proposing a new invalidation helper, verify all of the following:

1. the surface is not better explained as command/form/query/runtime ownership,
2. the existing tracked-write path still leaves repeated, review-visible invalidation boilerplate,
3. at least two real medium surfaces need the same simplification,
4. the helper would not hide action identity or ownership boundaries.

If any item fails, do not add the helper.
