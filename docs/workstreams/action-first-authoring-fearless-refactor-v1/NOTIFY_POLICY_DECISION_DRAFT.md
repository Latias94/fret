# Action-First Authoring + View Runtime (Fearless Refactor v1) — `notify()` Policy Decision Draft

Status: draft recommendation
Last updated: 2026-03-09

Related:

- Short default-rule card: `docs/workstreams/action-first-authoring-fearless-refactor-v1/INVALIDATION_DEFAULT_RULES.md`
- Invalidation/local-state review: `docs/workstreams/action-first-authoring-fearless-refactor-v1/INVALIDATION_LOCAL_STATE_REVIEW.md`
- Tracked-write inventory: `docs/workstreams/action-first-authoring-fearless-refactor-v1/TRACKED_WRITE_PATTERN_INVENTORY.md`
- V2 golden path: `docs/workstreams/action-first-authoring-fearless-refactor-v1/V2_GOLDEN_PATH.md`
- Current-vs-target gap note: `docs/workstreams/action-first-authoring-fearless-refactor-v1/V2_BEST_PRACTICE_GAP.md`
- TODO: `docs/workstreams/action-first-authoring-fearless-refactor-v1/TODO.md`
- Milestones: `docs/workstreams/action-first-authoring-fearless-refactor-v1/MILESTONES.md`

---

## Purpose

This note answers one narrow product/API question:

> After the post-v1 local-state and tracked-write work, should the repo keep investing in new
> invalidation helpers, or is `notify()` now correctly positioned as an explicit escape hatch?

Current recommendation:

- keep `notify()`,
- keep tracked writes as the boring default rerender path,
- do **not** add another generic invalidation helper right now,
- and treat the remaining pressure as a productization / handler-placement question rather than an
  invalidation-mechanism problem.
- lock the default teaching surfaces against accidental `notify()` reintroduction with a small gate.

For the short, execution-oriented rule set, use
`docs/workstreams/action-first-authoring-fearless-refactor-v1/INVALIDATION_DEFAULT_RULES.md`.

---

## Decision

### 1. Keep `notify()` as a public low-level escape hatch

`notify()` is still needed for cases such as:

- imperative integrations,
- cache-boundary invalidation,
- render-time runtime/query invalidation,
- host callbacks that mutate state outside the first-class tracked-write helpers.

This is still aligned with the retained-runtime model and should remain available.

### 2. Do not teach `notify()` as part of the default path

For ordinary authoring, the default expectation should now be:

- tracked local writes rerender through the action-aware helpers,
- observed models/selectors/queries invalidate through their own dependency contracts,
- users should not routinely think about `notify()` while writing a normal view.

### 3. Do not add another generic invalidation helper yet

The focused medium-surface review now shows:

- `simple_todo_v2_target` no longer has explicit `notify()` burden,
- `query_basics` uses explicit redraw/invalidation because the effect boundary is intentionally
  outside the tracked write,
- `commands_keymap_basics` and `form_basics` remain intentionally root-scoped because they express
  real coordination/ownership.

That means another general helper would mostly add surface area without removing the real cost.

---

## Why this is the current recommendation

### Evidence 1 — tracked writes already feel boring enough on real medium surfaces

Representative evidence:

- `apps/fret-cookbook/examples/simple_todo_v2_target.rs`
- `apps/fret-cookbook/examples/commands_keymap_basics.rs`
- `apps/fret-cookbook/examples/form_basics.rs`
- `ecosystem/fret/src/view.rs:1050`

Observed result:

- local tracked writes already request redraw + notify through the current action-aware helpers,
- ordinary medium surfaces no longer call `notify()` explicitly,
- the remaining visible cost is usually handler placement or multi-state coordination, not redraw
  triggering itself.

### Evidence 2 — explicit render-time invalidation is still correct for some surfaces

Representative evidence:

- `apps/fret-cookbook/examples/query_basics.rs`
- `apps/fret-examples/src/query_demo.rs`
- `apps/fret-examples/src/query_async_tokio_demo.rs`

Observed result:

- these surfaces intentionally invalidate runtime/query clients outside the tracked local write
  boundary,
- the explicit redraw path is part of the design, not a failure of local-state ergonomics.

### Evidence 3 — helper expansion pressure has moved elsewhere

Representative evidence:

- `docs/workstreams/action-first-authoring-fearless-refactor-v1/INVALIDATION_LOCAL_STATE_REVIEW.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/TRACKED_WRITE_PATTERN_INVENTORY.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/V2_BEST_PRACTICE_GAP.md`

Observed result:

- the remaining pressure is better described as:
  - keyed-list / payload-row handler placement,
  - product-facing default-path clarity,
  - a small number of medium-surface builder-density questions.

It is no longer best described as “users still need a better invalidation helper”.

---

## Practical default rule

Use this rule in docs/examples/templates:

### Default

- simple local writes: `on_action_notify_local_*`
- coordinated writes: `on_action_notify_models::<A>(...)`
- transient/runtime triggers: `on_action_notify_transient::<A>(...)`
- tracked reads: `value_*` / `value_in*`

### Escape hatch

Use explicit redraw / `notify()` only when:

- the effect lives outside the tracked write,
- the mutation happens in an imperative/host callback,
- the rerender reason is cache-oriented rather than a normal tracked state change.

---

## What this means for post-v1 planning

### What to do

1. Keep the current tracked-write helpers stable.
2. Keep `notify()` documented, but as advanced/runtime-facing guidance.
3. Shift the next work toward:
   - default-path productization,
   - and only then re-evaluate keyed-list / payload-row handler placement.
4. Keep a narrow source gate on the default ladder so `notify()` does not drift back into
   first-contact examples/templates.

### What not to do

1. Do not add another generic tracked-write helper just because some advanced surfaces are still
   dense.
2. Do not try to hide render-time runtime/query invalidation behind a fake “simple” helper.
3. Do not treat root-scoped command/form orchestration as if it were merely invalidation syntax
   noise.

---

## Provisional conclusion

For the current stage, the best framing is:

- `notify()` is still necessary,
- but it is no longer the default authoring problem,
- and the repo should stop spending near-term API budget on generic invalidation helper expansion.

That means the next high-value move is not:

> “invent one more invalidation API”

It is:

> “keep the current rerender rule stable, make the default path clearer, and only revisit the
> remaining keyed-list handler-placement pressure if it still matters after productization.”
