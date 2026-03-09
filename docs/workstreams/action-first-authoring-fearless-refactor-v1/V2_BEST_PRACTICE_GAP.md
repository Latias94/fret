# Action-First Authoring + View Runtime (Fearless Refactor v1) — Current Best Practice vs v2 Target

Status: draft, post-v1 guidance
Last updated: 2026-03-09

Related:

- V2 golden path: `docs/workstreams/action-first-authoring-fearless-refactor-v1/V2_GOLDEN_PATH.md`
- Post-v1 proposal: `docs/workstreams/action-first-authoring-fearless-refactor-v1/POST_V1_AUTHORING_V2_PROPOSAL.md`
- Post-v1 shortlist: `docs/workstreams/action-first-authoring-fearless-refactor-v1/POST_V1_SURFACE_SHORTLIST.md`
- TODO: `docs/workstreams/action-first-authoring-fearless-refactor-v1/TODO.md`
- Milestones: `docs/workstreams/action-first-authoring-fearless-refactor-v1/MILESTONES.md`

---

## Purpose

This note answers one practical question:

> What is the repo’s **current recommended writing style** today, and how far is that from the
> original GPUI/Zed-like v2 north-star?

The goal is not to invent another API pass.
The goal is to keep productization work honest by separating:

- what is already good enough to teach as the default path,
- what is still a real ergonomics gap,
- and what is intentionally advanced rather than unfinished.

---

## Executive summary

Current conclusion:

1. The repo has **already reached the v1 architectural target**:
   - action-first routing,
   - `View` + `ViewCx`,
   - local/shared state split,
   - selector/query hooks,
   - builder-first direction,
   - MVU removed in-tree.
2. The repo has **not fully reached the v2 density target** yet.
3. The remaining distance is now mostly:
   - local-state write/invalidation ergonomics,
   - builder-first medium-surface composition density,
   - default/comparison/advanced productization clarity,
   - and a small amount of event-handler placement noise.
4. The remaining distance is **not** mainly:
   - menu-family action aliases,
   - command-first residue chasing,
   - broad macro work,
   - or another runtime architecture rewrite.

---

## Current best practice (today)

### 1. App entry

Recommended default:

- `fret::view::<V>()`

Not default:

- `App::ui*`
- compat driver entrypoints unless the app is intentionally low-level/advanced

### 2. State

Recommended default:

- local state: `cx.use_local*`
- shared/app-visible state: explicit `Model<T>`
- common reads: `state.layout(cx).value_*` / `state.paint(cx).value_*`

Not default:

- `use_state::<T>()` as the first local-state teaching path
- reopening raw model handles for ordinary reads

### 3. Actions

Recommended default:

- typed actions
- `cx.on_action_notify_models::<A>(...)` as the boring coordinated-write path
- narrow local-state action helpers only for very straightforward local writes
- widget builder aliases such as `.action(...)`

Not default:

- raw command-shaped builder naming on default-facing surfaces
- broad helper proliferation

### 4. Composition

Recommended default:

- root/section/trigger `build(...)`
- `ui::keyed(...)`
- `ui::container_props(...)`
- stay builder-first until the final runtime boundary

Not default:

- early `into_element(cx)` just to decorate
- reopening broad legacy layout helper families

### 5. Product-facing teaching path

Recommended default ladder:

- `hello`
- `simple_todo`
- `todo`

Not default:

- `DataTable`
- compat/conformance examples
- host/runtime-heavy demos

---

## v2 target (north-star)

The intended v2 feel is still:

- **View object**
- **typed actions**
- **builder-first composition**
- **hooks for derived/query/local state**
- **tracked writes that usually rerender without explicit burden**

In short:

> the runtime stays Fret,
> the user-facing feel moves closer to GPUI/Zed.

---

## Gap matrix

| Area | Current best practice | v2 target | Gap level | Recommended next move |
| --- | --- | --- | --- | --- |
| App entry | `view::<V>()` is already the only default path | same | Low | Hold docs/gates; wait out deprecation window for old entries |
| Action naming | default-facing widget families already expose `action(...)` and teaching surfaces prefer it | same | Low | Maintenance only; reopen only if a new default-facing leak appears |
| Local state reads | `use_local*` + `value_*` path is coherent | same | Low | Keep stable |
| Local state writes / invalidation | coordinated writes still often route through `on_action_notify_models::<A>(...)`; tracked writes are better but not yet boring enough everywhere | tracked writes should usually feel implicit and lighter | Medium | Productize write guidance before adding more helpers |
| Medium-surface composition density | builder-first exists, but `ui::children!` and explicit landing are still visible in medium surfaces | more fluent builder-first composition on common surfaces | Medium | Keep closing real builder cliffs only where evidence repeats |
| Handler placement for keyed collections / payload rows | a narrow local payload-row helper now covers the current todo-like evidence slice, while root handler tables remain visible by design | slightly narrower, still explicit action identity | Low | Keep the current helper; reopen only if another medium surface shows the same row-local pressure |
| Default/comparison/advanced taxonomy | much better than before, but still spread across docs/templates/gallery/workstreams | obvious and boring for new users | Medium | This is now one of the highest-value productization tasks |
| `DataTable` | intentionally advanced/reference-only | still likely advanced, but with a clearer curated recipe | Medium | Treat as product recipe work, not generic authoring cleanup |
| Macros | existing action macros are enough | maybe small optional sugar later | Low | Do not prioritize now |

---

## What is already “good enough”

These areas should be treated as effectively settled for the current phase:

- app entry default path
- action-first builder naming on default-facing menus/widgets
- `use_local*` as the default local-state teaching path
- command-first residue moved to maintenance mode
- DSL not being a prerequisite for success

This matters because it means the repo should stop pretending it is “still migrating” in those
areas.

---

## What is still genuinely missing

If the repo wants to feel closer to the original north-star, the most defensible gaps are now:

### 1. Tracked-write / invalidation ergonomics

The remaining friction is no longer “how do I read local state”.
It is more often:

- where should multi-state coordinated writes live,
- when should the default path stay on `on_action_notify_models::<A>(...)`,
- and how much explicit invalidation burden still leaks into medium demos.

### 2. Productized onboarding taxonomy

The code surface is now narrower than the docs surface.
Users can still over-read advanced/reference examples as if they were the default path.

That is a documentation/product problem more than an API problem.

### 3. Medium-surface builder density

The repo has already closed many hard composition cliffs.
The remaining question is narrower:

- which medium, real demos still feel too heavy even after the current builder-first path,
- and whether those cases justify another very small helper pass.

The first disciplined post-v1 answer is now in place:

- `Alert::build(...)` and `AlertAction::build(...)` close one repeated alert-family seam without adding a new helper family,
- cookbook `form_basics` / `assets_reload_epoch_basics` plus the main ui-gallery alert snippets now serve as the evidence slice,
- `ScrollArea::build(...)` now closes the next repeated runtime-root seam while preserving the existing scroll config surface,
- cookbook `markdown_and_code_basics`, `async_playground_demo`, and the main ui-gallery scroll-area demo now serve as the second evidence slice,
- `FieldSet::build(...)` / `FieldGroup::build(...)` / `Field::build(...)` now close the next dense form-layout seam without reopening broad helper expansion,
- ui-gallery field `input`, field `fieldset`, and form `demo` now serve as the third evidence slice,
- so the remaining medium-surface pressure has shifted away from alert/scroll-area/field composition and toward other still-repeated families only if they show the same cross-surface pattern.

---

## What should not be mistaken for missing v2

The following items are **not** the main blocker now:

- continuing broad command-shaped residue cleanup
- rewriting command palette/catalog to stop looking command-centric
- treating `DataTable` as if it were a primitive table API problem
- introducing a broad new UI macro system
- reopening runtime/mechanism boundaries

Those moves would create churn without materially improving the default path.

---

## Recommended next order

1. **Productize the current default path**
   - make the onboarding ladder and default/comparison/advanced labels boring and explicit,
   - keep templates, README-level docs, and workstream guidance aligned.
2. **Review tracked-write / invalidation ergonomics**
   - use real medium demos as evidence,
   - prefer guidance and one narrow improvement over another helper family.
3. **Only then revisit medium-surface builder density**
   - and only where repeated evidence still shows a real cliff.
4. **Keep macros last**
   - optional polish only.

---

## Decision

For the current stage, the repo should describe itself this way:

- **v1 migration is effectively complete**
- **v2 is now a productization + ergonomics pass**
- **the highest-value work is no longer broad API churn**

That means the practical question is no longer:

> “What else do we still need to migrate?”

It is:

> “Which remaining gaps still harm the default authoring experience enough to justify additive,
> disciplined changes?”
