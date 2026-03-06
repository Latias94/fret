# Action-First Authoring - Post-v1 / v2 Authoring Proposal

Status: Draft recommendation
Last updated: 2026-03-06

Related:

- v1 design: `docs/workstreams/action-first-authoring-fearless-refactor-v1/DESIGN.md`
- v1 TODO: `docs/workstreams/action-first-authoring-fearless-refactor-v1/TODO.md`
- v1 milestones: `docs/workstreams/action-first-authoring-fearless-refactor-v1/MILESTONES.md`
- migration guide: `docs/workstreams/action-first-authoring-fearless-refactor-v1/MIGRATION_GUIDE.md`
- ADR 0307: `docs/adr/0307-action-registry-and-typed-action-dispatch-v1.md`
- ADR 0308: `docs/adr/0308-view-authoring-runtime-and-hooks-v1.md`

---

## 1) Why a post-v1 / v2 phase exists

v1 succeeded as an architectural reset:

- action-first dispatch exists,
- `View` + `ViewCx` landed,
- selector/query hooks are in tree,
- in-tree MVU was removed,
- default teaching surfaces converged on a smaller helper set.

However, v1 did **not** fully reach the original GPUI/Zed-style authoring density goal. The current
repo still exposes four recurring friction points in medium demos:

1. `ViewCx::use_state` is model-backed and returns `Model<T>`.
2. render code still relies heavily on `watch_model(...)` / `models.update(...)`.
3. `ui::children!` plus repeated `into_element(cx)` remain common composition patterns.
4. widget-local event wiring still prefers root-level `on_action_notify_*` helpers instead of a
   lighter `listener` / `dispatch` mental model.

The purpose of v2 is to address those friction points **without** undoing the v1 layering wins.

---

## 2) North-star for v2

The recommended v2 authoring surface is:

- **IR-first runtime, not DSL-first runtime**.
- **GPUI-style authoring defaults** for actions, key contexts, caching, and rebuild semantics.
- **Rust-first builder authoring** as the default path.
- **Optional future DSL/frontend** as an additive layer compiling into the same IR/runtime.

In other words:

- keep the current Fret runtime contracts,
- make the user-facing authoring API feel substantially closer to GPUI/Zed,
- do not introduce a second mandatory authoring language.

---

## 3) What v2 should keep from v1

v2 should preserve these decisions:

- `crates/fret-ui` remains mechanism/contract-only.
- `ActionId` remains the cross-frontend trigger identity.
- selectors/queries remain first-class hooks.
- cache reuse continues to be explicit and diagnosable.
- IMUI remains a frontend over the same runtime, not a separate UI system.
- future DSL work remains optional and additive.

---

## 4) Recommended default mental model

### 4.1 State

Split state authoring into two explicit buckets:

1. **Local state**
   - recommended surface: `use_local(...)` / `use_local_keyed(...)`, or state stored directly on the
     `View` object when the lifecycle is obvious,
   - expected feel: plain-Rust local state with built-in rerender semantics.

2. **Shared/app state**
   - recommended surface: explicit `Model<T>` / `use_model(...)` / existing shared-model helpers,
   - expected feel: opt into the model store only when state is intentionally shared or host-visible.

Selectors and queries stay as hooks:

- `use_selector(...)` for derived values,
- `use_query(...)` for async resources.

### 4.2 Actions and event wiring

The default action story should become:

- widgets declare action identity (`.action(act::Save)`),
- local UI events can dispatch the same action directly (`.on_click(cx.dispatch(act::Save))`),
- local widget glue uses a scoped closure helper (`cx.listener(...)`),
- page/root views can still register action handlers (`.on_action(cx.handle(...))`).

This keeps one action pipeline while reducing authoring noise for simple controls.

### 4.3 Composition

The default composition surface should become builder-first:

- `vflex().child(...).child(...)`
- `hflex().children(iter.map(...))`
- `stack().child(...)`

`ui::children!` should remain available for compatibility and heterogeneous escape-hatch cases, but it
should no longer be the primary authoring pattern in demos/templates.

### 4.4 Invalidation and caching

The core rebuild contract should stay aligned with GPUI:

- dirty only what needs rerendering,
- reuse cached work unless the cache root is dirty,
- diagnostics can explain why a subtree rebuilt.

The difference in v2 is not the mechanism; it is the **default ergonomics** around that mechanism.

---

## 5) `notify` evaluation

## Recommendation

Keep `notify`, but demote it from a default authoring burden to a lower-level runtime escape hatch.

### Why `notify` should stay

Explicit dirty requests remain valuable for:

- cache-boundary invalidation that is not expressed by a tracked state write,
- imperative integrations (IMUI-local ephemeral state, viewport internals, host callbacks),
- manual rerender requests after non-model / non-hook mutations,
- diagnostics and replay explainability.

This matches the spirit of GPUI, where explicit `notify` remains part of the retained runtime model.

### Why `notify` should become less visible to users

For best-practice authoring, most rerenders should happen implicitly when the user performs a tracked
state write:

- `use_local(...).set(...)` should request rerender automatically,
- shared-model updates should continue to invalidate via the model system,
- selector/query changes should continue to invalidate from dependency observation.

That means v2 should aim for:

- **explicit `notify` in the mechanism layer**,
- **implicit rerender for common state writes in the authoring layer**.

### Practical v2 rule

- If state changed through a first-class state API, users should usually **not** call `notify()`.
- If the rerender reason is imperative / cache-oriented / external to tracked state, users **should**
  still have `notify()` available.

---

## 6) Macro evaluation

## Recommendation

Macros can help, but v2 should avoid turning macros into a second primary UI language.

### Decision (recommended)

v2 does **not** require a new general-purpose UI macro layer to be considered successful.

Recommended stance:

- keep existing action-oriented macros (`actions!`, `payload_actions!`),
- do **not** block v2 on a new `rsx!` / DSL-like UI macro,
- treat any new UI macro as optional and evidence-driven,
- only add a narrow macro if builder-first authoring still leaves repeated structural boilerplate
  after the local-state + event-wiring improvements land.

### Good macro categories for v2

1. **Identity / action macros**
   - keep `actions!` and `payload_actions!`.
   - these are high-value, low-ambiguity, and already aligned with the action-first model.

2. **Narrow collection/composition macros**
   - consider narrowly-scoped macros only where Rust expression syntax is demonstrably noisy,
   - examples worth evaluating:
     - keyed child list helpers,
     - repeated layout child collection helpers,
     - optional short-form layout collection sugar similar in spirit to Iced's `column![]`.

3. **Testing / diagnostics macros**
   - if useful, prefer macros that reduce repetitive gate/script scaffolding rather than macros that
     define a whole new UI DSL.

### Macro categories v2 should avoid as the default

1. **Full `rsx!` / JSX-like DSL as the primary path**
   - Dioxus proves this can be ergonomic, but it creates a macro-language that becomes the main UI
     authoring surface.
   - That is a bigger strategic commitment than the repo currently needs.

2. **Full UI DSL replacement**
   - Slint demonstrates the power of a DSL, especially for tooling/live preview, but that should stay
     a future optional frontend for Fret, not the v2 default for framework users.

### Macro policy recommendation

- Prefer methods/builders first.
- Add macros only when they remove repeated structural boilerplate without hiding runtime semantics.
- Do not add macros that obscure action identity, key context, cache boundaries, or diagnostics.

---

## 7) Proposed v2 API shape (illustrative)

### State

- `cx.use_local(|| T::default())`
- `cx.use_local_keyed(key, || T::default())`
- `cx.use_model(model_handle)` or equivalent explicit shared-model hook
- `cx.use_selector(...)`
- `cx.use_query(...)`

### Events

- `button("Save").action(act::Save)`
- `button("Save").on_click(cx.dispatch(act::Save))`
- `button("Close").on_click(cx.listener(|this, cx| { ... }))`
- `root.on_action(cx.handle(|this, _: &act::Save, cx| { ... }))`
- `cx.shortcut::<act::Save>()`

### Composition

- `vflex().gap(8).child(...).child(...)`
- `hflex().children(...)`
- `stack().child(...)`

### Escape hatches that remain valid

- `notify()`
- `ui::children!`
- raw `on_action(...)`
- explicit `Model<T>` authoring for shared state

---

## 8) Proposed migration order

### Phase 1 ? state and invalidation semantics

- add `use_local` / `use_local_keyed`,
- keep `use_state` as compat for now,
- decide whether `use_state` becomes a deprecated alias or is repointed in a later major version,
- make local-state writes request rerender automatically.

### Phase 2 ? widget-local event ergonomics

- add `listener`, `dispatch`, and `shortcut` helpers,
- keep the root-level handler table as the underlying runtime mechanism,
- migrate one medium demo to validate the story.

### Phase 3 ? builder-first composition

- improve `.child(...)` / `.children(...)` ergonomics,
- keep `ui::children!` as compatibility-only/default-off teaching surface,
- migrate `hello_counter_demo` and `query_demo` to compare density and readability.

### Phase 4 ? narrow macro decision

- evaluate whether a small macro surface still buys real value after builder-first improvements,
- if yes, add only narrowly-scoped macros,
- if no, keep macros limited to actions and diagnostics/test helpers.

### Phase 5 ? cleanup

- remove old teaching-surface guidance,
- gate against deprecated patterns in demos/templates,
- decide which compat helpers can be hard-deleted in the next cleanup milestone.

---

## 9) Acceptance criteria for calling v2 successful

v2 should not be called successful until at least one medium demo proves all of the following:

- simple local state does not require explicit `Model<T>` handling,
- common button wiring does not require root-level helper boilerplate for every action,
- most layout authoring does not require `ui::children!` or repeated `into_element(cx)`,
- rerender behavior remains diagnosable and deterministic,
- no layering regression is introduced into `crates/fret-ui`.

Recommended comparison targets:

- `apps/fret-examples/src/hello_counter_demo.rs`
- `apps/fret-examples/src/query_demo.rs`

---

## 10) External reference notes

These references inform the recommendation, but they do not override Fret's layering rules:

- GPUI/Zed local reference:
  - `F:/SourceCodes/Rust/fret/repo-ref/zed/crates/gpui/src/action.rs`
  - `F:/SourceCodes/Rust/fret/repo-ref/zed/crates/gpui/src/key_dispatch.rs`
  - `F:/SourceCodes/Rust/fret/repo-ref/zed/crates/gpui/src/view.rs`
- Dioxus README (signals + `rsx!`):
  - https://github.com/DioxusLabs/dioxus
- Iced macro docs (`column![]` as narrow layout sugar):
  - https://docs.iced.rs/iced/widget/macro.column.html
- Xilem README (reactive view tree over a retained widget/runtime architecture):
  - https://github.com/linebender/xilem
- Slint docs (DSL + callbacks/properties as an optional frontend model):
  - https://docs.slint.dev/latest/docs/slint/guide/language/concepts/slint-language/
  - https://docs.slint.dev/latest/docs/slint/guide/language/coding/properties/
  - https://docs.slint.dev/latest/docs/slint/guide/language/coding/functions-and-callbacks/
