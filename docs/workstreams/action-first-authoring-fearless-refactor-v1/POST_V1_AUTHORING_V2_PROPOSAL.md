# Action-First Authoring - Post-v1 / v2 Authoring Proposal

Status: Draft recommendation
Last updated: 2026-03-10

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
repo still exposes five recurring friction points in medium demos and the current teaching/product surfaces:

1. `LocalState<T>` is still model-backed, so view-owned state does not yet feel like plain-Rust
   fields or collections.
2. render code still relies heavily on generic tracked-store coordination once coordination crosses
   more than one field, even though the focused keyed-list/default template path is now closed.
3. `ui::children!` plus repeated `into_element(cx)` remain common composition patterns.
4. keyed-list / payload-row event wiring still relies on visible root-level `on_action_notify_*`
   ownership points, and the remaining question is whether that needs a narrower placement aid.
5. product-facing guidance still needs a sharper default/comparison/advanced taxonomy so users do
   not have to reverse-engineer the intended surface from scattered examples.

The text-value widget cliff is no longer the main blocker: `Input` / `Textarea` now accept the
narrow `IntoTextValueModel` bridge, so post-v1 code can pass `&LocalState<String>` directly.

The purpose of v2 is to address those friction points **without** undoing the v1 layering wins.

---

## 2) North-star for v2

The recommended v2 authoring surface is:

- **IR-first runtime, not DSL-first runtime**.
- **GPUI-style authoring defaults** for actions, key contexts, caching, and rebuild semantics.
- **Rust-first builder authoring** as the default path.
- **Optional future DSL/frontend** as an additive layer compiling into the same IR/runtime.

North-star clarification:

- the primary v2 target is **GPUI's authoring/runtime feel**,
- not gpui-component's component-library product shape,
- though selected productization/builder ideas can still be borrowed as secondary references.

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

Prototype status (as of 2026-03-06):

- an additive `LocalState<T>` wrapper now exists in `ecosystem/fret/src/view.rs`,
- `ViewCx::use_local*` / `watch_local(...)` are available as an experimental surface,
- `apps/fret-examples/src/hello_counter_demo.rs`, `apps/fret-examples/src/query_demo.rs`, and `apps/fret-examples/src/query_async_tokio_demo.rs` use the prototype to remove explicit local model-handle fields from the view struct,
- the query demos validate that `use_local` can coexist with `use_query` / `use_query_async` and transient invalidation without changing the default teaching-surface action path,
- the prototype is still model-backed and is **not yet** the final plain-Rust local-state answer.
- `apps/fret-cookbook/examples/simple_todo_v2_target.rs` now serves as the focused comparison target
  for small view-owned keyed collections: it keeps `Vec<TodoRow>` in `LocalState<Vec<_>>` and uses
  payload actions for row toggle/remove, and it now also uses a shadcn-aligned
  `Checkbox::from_checked(...).action_payload(...)` path so small dynamic collections no longer need
  per-row checkbox models. The remaining visible gap is now more about root-level action boilerplate
  than checkbox value binding itself.

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

Prototype status (as of 2026-03-06):

- `apps/fret-examples/src/query_demo.rs` and `apps/fret-examples/src/query_async_tokio_demo.rs` now demonstrate builder-first passes using `ui::h_row_build`, `ui::v_flex_build`, and `UiElementSinkExt` to remove `ui::children!` from their main layout sections,
- `ecosystem/fret-ui-kit/src/ui.rs` now bridges heterogeneous child pipelines through `UiChildIntoElement`, so `ui::children!` / `push_ui()` accept nested layout builders plus host-bound late builders without an extra `.into_element(cx)` cliff,
- `ecosystem/fret-ui-shadcn/src/card.rs` now provides `Card::build(...)` / `CardHeader::build(...)` / `CardContent::build(...)`, and that same card-builder path powers the `fretboard` todo/simple-todo templates plus cookbook `commands_keymap_basics`, `form_basics`, and `async_inbox_basics` through the generic `.ui()` patch path,
- `ecosystem/fret-ui-shadcn/src/layout.rs` now adds `container_vstack_build(...)` / `container_hstack_build(...)` / `container_hstack_centered_build(...)`, so older shadcn layout helpers can also stay on the same late-landing child pipeline,
- `ecosystem/fret-ui-shadcn/src/table.rs` now adds `Table::build(...)` / `TableHeader::build(...)` / `TableBody::build(...)` / `TableFooter::build(...)` / `TableRow::build(...)`, and `ecosystem/fret-genui-shadcn/src/resolver/data.rs` uses that same builder-first path for generated data tables,
- `ecosystem/fret-ui-shadcn/src/table.rs` also adds `TableCell::build(child)` as the first single-child late-landing sample, with `apps/fret-ui-gallery/src/ui/snippets/typography/table.rs` reflecting the intended teaching shape,
- `ecosystem/fret-ui-shadcn/src/dialog.rs`, `ecosystem/fret-ui-shadcn/src/sheet.rs`, and `ecosystem/fret-ui-shadcn/src/drawer.rs` now add `DialogTrigger::build(child)` / `SheetTrigger::build(child)` / `DrawerTrigger::build(child)` so the first overlay-trigger wrappers can late-land builder children too, and the `Dialog` / `Sheet` composition builders now accept those trigger-build values directly on `.trigger(...)`,
- `ecosystem/fret-ui-shadcn/src/popover.rs`, `ecosystem/fret-ui-shadcn/src/hover_card.rs`, and `ecosystem/fret-ui-shadcn/src/tooltip.rs` now extend that same builder-first path across the remaining overlay single-child wrappers (`PopoverTrigger::build(...)`, `PopoverAnchor::build(...)`, `HoverCardTrigger::build(...)`, `HoverCardAnchor::build(...)`, `TooltipTrigger::build(...)`, `TooltipAnchor::build(...)`), while keeping eager `new(...)` constructors for already-landed `AnyElement` and pre-landing anchor-id workflows,
- `ecosystem/fret-ui-shadcn/src/popover.rs`, `ecosystem/fret-ui-shadcn/src/hover_card.rs`, and `ecosystem/fret-ui-shadcn/src/tooltip.rs` now also add or extend host-bound root constructors (`Popover::build(...)` / `HoverCard::build(...)` / `HoverCard::build_controllable(...)` / `Tooltip::build(...)`) so trigger/content builders can cross the root API boundary without an early `into_element(cx)` step; `PopoverContent::test_id(...)` now survives that late-landing root path directly, and `Tooltip::new(...)` also accepts `TooltipContent` directly,
- `ecosystem/fret-ui-shadcn/src/dropdown_menu.rs` now brings the first composite menu root onto that same path through `DropdownMenuTrigger::build(...)` plus `DropdownMenu::build(...)` / `DropdownMenu::build_parts(...)`, letting trigger builders cross both the direct and part-based root APIs without an eager `into_element(cx)` step,
- `apps/fret-ui-gallery/src/ui/snippets/dropdown_menu/*.rs` now teach that dropdown-menu trigger/parts path consistently across the gallery, while the remaining visible landing points mostly come from broader composite wrappers plus the rest of the single-child API family that still collect `AnyElement` eagerly outside the modern child pipeline.

### 4.4 Invalidation and caching

The core rebuild contract should stay aligned with GPUI:

- dirty only what needs rerendering,
- reuse cached work unless the cache root is dirty,
- diagnostics can explain why a subtree rebuilt.

The difference in v2 is not the mechanism; it is the **default ergonomics** around that mechanism.

### 4.5 Gap to current Rust UI best-practice references

If we compare Fret's current post-v1 surface to the strongest ideas in contemporary Rust UI stacks,
our remaining gap is now fairly specific rather than architectural:

- **GPUI/Zed-style retained UI**: Fret already matches the action identity + cache/dirty + builder
  direction, but small app state and view-owned collections still require more explicit model-store
  coordination than GPUI-style `self` mutations.
- **Dioxus/Xilem-style local reactivity**: Fret now has a coherent local-state story, but it still
  splits into `LocalState<T>` vs `Model<T>` earlier than ideal for a simple todo-style view.
- **Iced-style narrow sugar**: a small amount of layout/event sugar may still help, but only after
  we finish the bigger productization/state-boundary gap; macros are not the main missing piece right now.
- **Slint-style alternate frontend**: optional DSL/visual tooling remains a valid future direction,
  but it is not required for v2 success as long as the Rust-first authoring path becomes dense and
  predictable.

This means the next design target is no longer small view-owned collection viability itself; that
default path now exists across cookbook, app-grade, and scaffold surfaces. The next design target is
**productizing that path** rather than introducing a new wave of helpers or macros.

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

---

## 8) “v1 best practice today” vs “v2 target” (side-by-side mental model)

This section is intentionally about what authors should do **today** (v1), and what we want the
default to feel like **after** v2 lands.

### v1 best practice today (in-tree teaching surfaces)

- State: use `cx.use_local*` for view-local fields, but keep truly shared state in explicit `Model<T>`; small keyed collections still often remain hybrid (`LocalState<String>` for draft text, `Model<Vec<_>>` for rows) in today's templates.
- Derivations/async: use `use_selector` / `use_query` as hooks.
- Actions: bind stable IDs (`.action(act::Save)`), register handlers via `cx.on_action_*` helpers.
- Composition: prefer late-landing child collection (`ui::children![cx; ...]` + `*_::build(...)` where available).
- Invalidation: `notify()` is available and remains correct; most rerenders should come from tracked writes and observed deps, not manual `notify()` calls.

Example (v1, late-landing card + children collection):

```rust,ignore
let shortcut = "Ctrl+S".to_string();
let row = ui::h_flex(|cx| {
    ui::children![cx;
        shadcn::Label::new("Shortcut:"),
        shadcn::Badge::new(shortcut)
            .variant(shadcn::BadgeVariant::Secondary),
    ]
})
.gap(Space::N2)
.items_center()
.into_element(cx);

let card = shadcn::Card::build(|cx, out| {
    out.push_ui(cx, shadcn::CardHeader::build(|cx, out| {
        out.push_ui(cx, shadcn::CardTitle::new("Title"));
    }));
    out.push_ui(cx, shadcn::CardContent::build(|_cx, out| out.push(row)));
})
.ui()
.w_full()
.into_element(cx);
```

### v2 target (best-practice authoring surface)

- State: local state should feel like plain Rust (`use_local`), and should not require `Model<T>` unless state is intentionally shared.
- Events: prefer `.on_click(cx.dispatch(act::Save))` / `cx.listener(...)` shapes where they reduce boilerplate, while keeping `ActionId` as the cross-frontend identity.
- Composition: builder-first `vflex().child(...)` style should become the default and remove most `ui::children!` usage from common demos/templates.
- Invalidation: state writes should request rerender implicitly; `notify()` remains a low-level escape hatch for imperative integrations and cache-oriented invalidation.

The purpose of v2 is to land these ergonomics wins while keeping the v1 layering and diagnostics
closure intact.
- `stack().child(...)`

### Escape hatches that remain valid

- `notify()`
- `ui::children!`
- raw `on_action(...)`
- explicit `Model<T>` authoring for shared state

---

## 8) Proposed migration order

### Phase 1 — productize the default path

- keep `hello` / `simple-todo` / `todo` as the explicit onboarding ladder,
- tighten docs/templates so the default/comparison/advanced split is visible without reading workstream notes,
- keep `use_local*` as the only default local-state teaching path and keep `notify()` documented as an escape hatch, not a first-contact step.

### Phase 2 — state and invalidation follow-through

- keep `use_local` / `use_local_keyed` as the additive default local-state path,
- keep `use_state` as compat for now,
- preserve the current rule that tracked local writes rerender implicitly,
- only revisit state APIs if real medium-surface evidence still shows a state-boundary cliff after the productization/doc pass.

### Phase 3 — keyed-list / payload-row handler ergonomics

- only investigate a narrower row-action / payload-handler placement aid,
- keep the root-level handler table as the underlying runtime mechanism,
- validate it only against keyed-list evidence surfaces, not command/query/form surfaces.

### Phase 4 — builder-first composition

- improve `.child(...)` / `.children(...)` ergonomics,
- keep `ui::children!` as compatibility-only/default-off teaching surface,
- migrate `hello_counter_demo` and `query_demo` to compare density and readability.

### Phase 5 — narrow macro decision

- evaluate whether a small macro surface still buys real value after builder-first improvements,
- if yes, add only narrowly-scoped macros,
- if no, keep macros limited to actions and diagnostics/test helpers.

### Phase 6 — cleanup

- remove old teaching-surface guidance,
- gate against deprecated patterns in demos/templates,
- decide which compat helpers can be hard-deleted in the next cleanup milestone.

---

## 9) Acceptance criteria for calling v2 successful

v2 should not be called successful until at least one medium demo proves all of the following:

- simple local state and simple view-owned collections do not require explicit `Model<T>` / `Model<Vec<_>>` handling,
- common button wiring does not require root-level helper boilerplate for every action,
- most layout authoring does not require `ui::children!` or repeated `into_element(cx)`,
- rerender behavior remains diagnosable and deterministic,
- no layering regression is introduced into `crates/fret-ui`.

Status update (as of 2026-03-08): the first criterion is now satisfied across the focused comparison
sample (`simple_todo_v2_target`), an app-grade demo (`todo_demo`), and the `fretboard` simple-todo
scaffold template. The remaining blockers are mainly authoring density, onboarding clarity, visual
defaults, and explicit advanced-surface positioning rather than keyed-list viability itself.

Recommended next comparison targets:

- `apps/fret-examples/src/hello_counter_demo.rs`
- `apps/fret-examples/src/query_demo.rs`
- onboarding/product docs (`docs/first-hour.md`, `docs/examples/todo-app-golden-path.md`)

---

## 10) Product goals after default-path convergence

With the keyed-list default path now landed, the next phase should optimize for **product clarity**
rather than additional mechanism churn.

Recommended goals:

1. **Onboarding clarity**
   - keep `hello` / `simple-todo` / `todo` as a stable ladder,
   - make the default/comparison/advanced split obvious in docs and templates,
   - avoid teaching both the old and new mental model as if they were equally recommended.

2. **Default-path density**
   - keep reducing the remaining `ui::children!` / `into_element(cx)` friction where evidence shows
     repeated medium-surface cost,
   - investigate keyed-list / payload-row handler ergonomics only if it improves keyed-list evidence
     surfaces without widening the helper surface indiscriminately.

3. **Visual productization**
   - deepen theme/recipe assets and make the default app output feel more polished out of the box,
   - treat this as a design-system / recipe problem, not a `crates/fret-ui` runtime problem.

4. **Surface taxonomy and cleanup**
   - keep explicit-model comparison examples on purpose,
   - label advanced/runtime-bound surfaces clearly,
   - plan `deprecated -> gated -> removed` cleanup only after the teaching surfaces and templates are
     fully aligned.

Explicit non-goals for this phase:

- do not add another generic tracked-write helper just because some advanced surfaces are still
  explicit,
- do not introduce macros as the primary answer before the remaining medium-surface friction is
  better measured,
- do not blur mechanism/policy boundaries to make recipes look shorter.

---

## 11) External reference notes

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
