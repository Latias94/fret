# Action-First Authoring + View Runtime (Fearless Refactor v1) — Invalidation / Local-State Review

Status: draft, focused post-v1 review
Last updated: 2026-03-09

Related:

- TODO: `docs/workstreams/action-first-authoring-fearless-refactor-v1/TODO.md`
- Milestones: `docs/workstreams/action-first-authoring-fearless-refactor-v1/MILESTONES.md`
- Post-v1 shortlist: `docs/workstreams/action-first-authoring-fearless-refactor-v1/POST_V1_SURFACE_SHORTLIST.md`
- V2 golden path: `docs/workstreams/action-first-authoring-fearless-refactor-v1/V2_GOLDEN_PATH.md`
- Tracked-write inventory: `docs/workstreams/action-first-authoring-fearless-refactor-v1/TRACKED_WRITE_PATTERN_INVENTORY.md`
- Explicit-model collection inventory: `docs/workstreams/action-first-authoring-fearless-refactor-v1/EXPLICIT_MODEL_COLLECTION_SURFACE_INVENTORY.md`

---

## Purpose

This note performs a deliberately narrow post-v1 review:

> Pick a very small number of medium surfaces and decide whether the next ergonomics pressure is
> still really about invalidation/local-state API shape, or whether the remaining noise has moved
> elsewhere.

This pass is intentionally small.
It is meant to prevent another speculative helper expansion round.

---

## Review targets

Target surfaces:

- `apps/fret-cookbook/examples/simple_todo_v2_target.rs`
- `apps/fret-cookbook/examples/query_basics.rs`
- `apps/fret-cookbook/examples/commands_keymap_basics.rs`
- `apps/fret-cookbook/examples/form_basics.rs`

Why these surfaces:

- `simple_todo_v2_target` already uses `LocalState<Vec<TodoRow>>`, payload row actions, and
  snapshot checkbox bindings, so it is the best keyed-list evidence surface.
- `query_basics` is the clearest medium surface where local state only triggers a render-time
  invalidation/client effect instead of directly owning the final mutation.
- `commands_keymap_basics` is the clearest non-query medium surface where root action handling and
  availability are intentionally tied to command/keymap semantics.
- `form_basics` is the clearest non-query, non-command medium surface where root handlers still
  exist because validation and availability span multiple local fields.

Together they are the smallest useful set for answering whether the remaining authoring pressure is:

1. invalidation semantics,
2. local-state storage shape,
3. or event-handler placement.

---

## What the surfaces already prove

### Surface A — `simple_todo_v2_target`

### 1. Tracked writes already rerender without explicit `notify()`

Observed in the current example:

- `Add` uses `cx.on_action_notify_models::<act::Add>(...)`
- `ClearDone` uses `todos_state.update_in_if(...)`
- `Toggle` / `Remove` use `cx.on_payload_action_notify::<...>(...)` plus `update_in_if(...)`

There is no explicit `notify()` burden at the call site.

Interpretation:

- for this medium keyed-list surface, the current tracked-write boundary already behaves like the
  intended post-v1 default,
- so the next pressure is **not** “users still have to remember `notify()`”.

### 2. Store-side local-state reads are no longer the visible pain point

Observed in the current example:

- `draft_state.value_in_or_else(...)`
- `next_id_state.read_in(...)`
- `todos_state.update_in(...)` / `update_in_if(...)`

Interpretation:

- the previously noisy “reopen `local.model()` just to read/write in a transaction” problem is
  already sufficiently narrowed here,
- so another generic store-side local-state helper is not justified by this surface.

### 3. Small view-owned collections no longer require explicit collection models

Observed in the current example:

- the entire row list is stored in `LocalState<Vec<TodoRow>>`,
- per-row toggle/remove uses payload actions plus snapshot checkbox bindings,
- no nested row `Model<bool>` remains.

Interpretation:

- local-state collection viability is no longer hypothetical,
- this surface confirms the v2 local-state keyed-list path is already real.

---

### Surface B — `query_basics`

#### 1. The remaining redraw request is an intentional effect boundary

Observed in the current example:

- `Invalidate` / `InvalidateNamespace` only set local trigger flags through
  `on_action_notify_local_set`,
- the actual query invalidation happens in `render()`,
- the example then explicitly calls `cx.app.request_redraw(window)`.

Interpretation:

- this is not the same category as “tracked local writes should rerender automatically”,
- it is a render-time query/client escape hatch where the real effect lives outside the tracked
  local write itself.

#### 2. The current policy already fits this surface

Observed in the current example:

- `ToggleErrorMode` is already a straightforward local-state write,
- query invalidation stays explicit and render-time,
- there is still no explicit `notify()` burden at the action call site.

Interpretation:

- this surface reinforces the current policy that query/client/runtime invalidation should remain
  an explicit escape hatch,
- it does **not** justify adding another default invalidation helper.

---

### Surface C — `commands_keymap_basics`

#### 1. The invalidation story is already boring enough

Observed in the current example:

- `ToggleAllowCommand` uses `on_action_notify_toggle_local_bool`,
- `TogglePanel` uses `on_action_notify_models` plus `panel_open_state.update_in(...)`,
- there is no explicit `notify()` burden.

Interpretation:

- this surface does not point to another invalidation/local-state helper gap either.

#### 2. Root-scoped handlers are partly the point here

Observed in the current example:

- command metadata is registered globally,
- shortcut display comes from the keymap service,
- `TogglePanel` has both an action handler and an availability handler,
- the root also carries a key context.

Interpretation:

- unlike row-local todo mutations, this root-scoped wiring is not merely accidental ceremony,
- command/keymap surfaces need an explicit root-level ownership point for availability, key
  contexts, and dispatch semantics.

---

### Surface D — `form_basics`

#### 1. The invalidation story is already boring enough

Observed in the current example:

- `Submit` and `Reset` both use `on_action_notify_models`,
- field state and error state are all kept in `LocalState<_>`,
- there is no explicit `notify()` burden.

Interpretation:

- this surface also does not indicate an invalidation/local-state helper gap.

#### 2. Root handlers are still mostly real coordination here

Observed in the current example:

- `Submit` validates across `name`, `email`, and `error`,
- `Reset` clears three local state buckets together,
- `Submit` availability mirrors the same cross-field validation logic.

Interpretation:

- even without query/runtime effects or command/keymap semantics, this surface still needs a
  root-level ownership point because the write and availability logic spans multiple fields,
- so this is not strong evidence for a broader handler-sugar pass either.

---

## What still looks noisy

### 1. Root handler placement is still the main visible cost on keyed-list surfaces

The current example still registers four handlers at the root:

- one for `Add`,
- one for `ClearDone`,
- one for `Toggle(id)`,
- one for `Remove(id)`.

That cost is visible even though the underlying invalidation story is already working.

Interpretation:

- the remaining authoring density pressure here is mostly about **where handlers live**,
- not about whether tracked writes request redraw/notify correctly.

### 2. Payload row actions amplify the handler-placement cost

`Toggle(id)` and `Remove(id)` are structurally simple row-local mutations, but they still need
root-level payload handlers.

Interpretation:

- if this surface motivates any future additive API, it points more toward **narrow payload-row
  handler placement ergonomics**,
- not toward another invalidation helper.

### 3. Multi-state coordination remains real work

`Add` still coordinates:

- draft text read,
- next-id read/write,
- todos mutation,
- draft clear.

Interpretation:

- this is real coordination, not accidental syntax noise,
- so `cx.on_action_notify_models::<A>(|models| ...)` still looks like the right default ownership
  boundary for this category.

### 4. Query-trigger surfaces remain intentionally explicit

`query_basics` still uses:

- local trigger state,
- render-time query invalidation,
- explicit redraw.

Interpretation:

- that explicitness is still a feature, not a helper gap,
- because the real effect does not live inside the tracked local write boundary.

### 5. Command/keymap surfaces remain intentionally root-scoped

`commands_keymap_basics` still uses:

- a root-level action handler,
- a root-level availability callback,
- explicit command metadata/keymap registration.

Interpretation:

- this also is not pure “placement noise” in the same way as keyed row actions,
- so it should not be used as the main justification for keyed-list/payload-row sugar.

### 6. Cross-field form surfaces remain intentionally root-scoped

`form_basics` still uses:

- root-level submit/reset handlers,
- root-level submit availability,
- shared validation logic across multiple local fields.

Interpretation:

- this is closer to real orchestration than to accidental placement noise,
- so it also should not be used as the main justification for keyed-list/payload-row sugar.

---

## Decision

Current conclusion from this review:

1. **Do not add another default invalidation helper based on this surface.**
   - `notify()` is already off the call path here.
   - tracked writes already rerender through the existing action helpers.
2. **Do not add another generic local-state read/write helper based on this surface.**
   - the current `value_in*` / `read_in` / `update_in_if` surface is already sufficient here.
3. **Record a split result instead of collapsing everything into “invalidation ergonomics”.**
   - `simple_todo_v2_target` says keyed-list pressure has shifted to handler placement for payload
     row actions,
   - `query_basics` says query/client invalidation is still an intentional explicit escape hatch,
   - `commands_keymap_basics` says command/keymap root handling is often intentional runtime
     ownership rather than sugarable placement noise,
   - `form_basics` says non-query, non-command cross-field coordination still naturally lives at
     the root action boundary,
   - together they still rule out another `AFA-postv1-004` helper expansion, and they also make
     `AFA-postv1-003` look narrower than before: the only clearly remaining pressure is keyed-list
     payload-row handler placement, not medium surfaces in general.

---

## Recommended next step

Do **not** change runtime/API immediately.

Instead:

1. keep `AFA-postv1-004` on the current “no new helper yet” stance,
2. treat `simple_todo_v2_target` as explicit proof that keyed-list pressure has shifted to handler
   placement,
3. treat `query_basics` as explicit proof that query/client invalidation should remain an
   intentional render-time escape hatch,
4. treat `commands_keymap_basics` as explicit proof that command/keymap root handling is often a
   real ownership boundary, not the best sugar target,
5. treat `form_basics` as explicit proof that cross-field local coordination also remains a real
   root-level ownership boundary,
6. only re-open `AFA-postv1-003` if another medium surface shows the same keyed-list-style
   row-local handler-placement pressure as `simple_todo_v2_target`.

---

## Evidence anchors

- `apps/fret-cookbook/examples/simple_todo_v2_target.rs`
- `apps/fret-cookbook/examples/query_basics.rs`
- `apps/fret-cookbook/examples/commands_keymap_basics.rs`
- `apps/fret-cookbook/examples/form_basics.rs`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/TRACKED_WRITE_PATTERN_INVENTORY.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/EXPLICIT_MODEL_COLLECTION_SURFACE_INVENTORY.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/V2_GOLDEN_PATH.md`

---

## Provisional conclusion

For four different real medium surfaces, the post-v1 question is no longer:

> “How do we make tracked local writes invalidate with less ceremony?”

It is now split into two clearer answers:

- keyed-list surfaces ask:
  > “Do we want a narrower way to place keyed-row / payload action handlers without hiding action
  > identity or coordination boundaries?”
- query/runtime-trigger surfaces answer:
  > “Explicit render-time invalidation is still the right escape hatch when the real effect lives
  > outside the tracked local write.”
- command/keymap surfaces answer:
  > “Root-scoped action handling and availability are often the runtime contract, not accidental
  > syntax noise.”
- cross-field form surfaces answer:
  > “Root handlers still make sense when validation and availability span multiple local fields.”

That is an important difference.
It means the repo should resist inventing more invalidation/local-state helpers just because some
medium surfaces still look denser than the GPUI/Zed north-star, and it also means any future
handler-placement sugar still looks like a **keyed-list/payload-row-specific** question rather than
a general medium-surface solution.
