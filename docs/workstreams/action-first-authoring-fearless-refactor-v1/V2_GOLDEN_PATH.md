# Authoring v2 Golden Path (Draft)

Status: draft, post-v1 guidance
Last updated: 2026-03-09

Related:

- Proposal: `docs/workstreams/action-first-authoring-fearless-refactor-v1/POST_V1_AUTHORING_V2_PROPOSAL.md`
- Post-v1 shortlist: `docs/workstreams/action-first-authoring-fearless-refactor-v1/POST_V1_SURFACE_SHORTLIST.md`
- TODO: `docs/workstreams/action-first-authoring-fearless-refactor-v1/TODO.md`
- Migration notes: `docs/workstreams/action-first-authoring-fearless-refactor-v1/MIGRATION_GUIDE.md`

This note intentionally keeps the recommended surface **small**. The goal is to make default authoring
predictable while post-v1 ergonomics are still settling.

## Default rule

If the common path is unclear, prefer the following defaults:

- local state: `cx.use_local*` + `state.layout(cx).value_*` / `state.paint(cx).value_*`
- shared state: explicit `Model<T>` when state is intentionally shared or when a dynamic keyed collection still needs stable nested model handles; read both local and shared state through `state.layout(cx).value_*` / `state.paint(cx).value_*` by default
- actions: `cx.on_action_notify_models::<A>(|models| ...)` by default; use local-state-specific helpers
  only for straightforward local writes, and stay on the generic models path when command/keymap
  gating or broader coordination is involved
- invalidation: tracked state writes should usually rerender implicitly; keep explicit `notify()` for
  imperative/cache-oriented escape hatches
- composition: stay on builder-first paths until the final runtime boundary

## The five recommended composition entry points

### 1. Root `build(...)`

Use root `build(...)` helpers when the component owns the runtime boundary.

Examples:

- `shadcn::Card::build(...)`
- `shadcn::Dialog::build(cx, trigger, content)`
- `shadcn::AlertDialog::build(cx, trigger, content)`
- `shadcn::Popover::build(cx, trigger, content)`

Rule: if a component already has a host-bound root builder, prefer it over assembling early-landed
`AnyElement` trees by hand. When a helper only feeds parent children, prefer returning
`impl UiChildIntoElement<_>` over forcing an intermediate `AnyElement`.

### 2. Section `build(...)`

Use section builders to keep nested content builder-first.

Examples:

- `CardHeader::build(...)`, `CardContent::build(...)`, `CardFooter::build(...)`
- `DialogContent::build(...)`, `DialogHeader::build(...)`, `DialogFooter::build(...)`
- `AlertDialogContent::build(...)`, `DrawerContent::build(...)`
- `TableHeader::build(...)`, `TableBody::build(...)`, `TableRow::build(...)`, `TableCell::build(...)`

Rule: if content naturally comes from loops, conditionals, or `push_ui(...)`, prefer section builders.

### 3. Trigger / anchor `build(...)`

Use trigger/anchor builders when the wrapped child is still builder-first.

Examples:

- `DialogTrigger::build(child)`
- `AlertDialogTrigger::build(child)`
- `PopoverTrigger::build(child)` / `PopoverAnchor::build(child)`
- `TooltipTrigger::build(child)` / `TooltipAnchor::build(child)`

Rule: use `*_new(...)` only when the child is already an `AnyElement` and there is no builder-first
path to preserve.

### 4. `ui::keyed(...)`

Use `ui::keyed(...)` for repeated child identity that must survive reorder / replacement.

Examples:

- keyed todo rows
- virtualized or reordered list rows
- repeated controls inside generated cards/tables

Rule: if identity matters, key it at the authoring site instead of forcing a pre-collected `Vec<AnyElement>`
just to keep state stable.

### 5. `ui::container_props(...)`

Use `ui::container_props(...)` / `ui::container_props_build(...)` when you need a low-level container root
with explicit props but still want the modern child pipeline.

Use cases:

- custom row/container wrappers
- explicit hit-test/layout roots
- lower-level element shells that still need builder-first children

Rule: this is the preferred low-level root escape hatch before falling back to manual early landing.

## State and invalidation defaults

For post-v1 best-practice authoring, the intended direction is:

- `let count = cx.use_local_with(|| 0);`
- `let value = count.layout(cx).value_or_default();`
- `shadcn::Input::new(&draft)` / `shadcn::Textarea::new(&source)` for text widgets on the post-v1 local-state path
- `count.clone_model()` when an existing non-text widget still expects `Model<T>`
- `cx.on_action_notify_local_set::<act::Reset, i64>(&count, 0);`
- `cx.on_action_notify_toggle_local_bool::<act::ToggleEnabled>(&enabled);` for a straightforward local bool flag
- `cx.on_action_notify_models::<act::Inc>(|models| { ... })` when a write depends on multiple pieces of
  state, form validation/reset coordination, command availability gating, dynamic keyed-list coordination, or broader model-store coordination
- `count.update_in(models, |value| { ... })` / `count.set_in(models, value)` are store-only transaction helpers; `count.update_in_if(models, |value| -> bool { ... })` is the handled-aware variant for collection-style writes. They become rerendering writes when used under `on_action_notify_models::<A>(...)`, while `on_action_notify_local_*` / `update_action(...)` / `update_action_if(...)` are the direct tracked-write path
- `count.value_in_or_default(models)` / `count.value_in_or(models, fallback)` for the common store-side read path, and `count.read_in(models, |value| ...)` / `count.revision_in(models)` when the closure needs a custom projection or revision check
- `let query_state = query_handle.layout(cx).value_or_else(QueryState::<T>::default);` for query results, instead of reopening `query_handle.model()` at the teaching surface

Keep explicit `notify()` for:

- imperative integrations
- cache-boundary invalidation not represented by a tracked write
- host/runtime callbacks that mutate state outside the first-class authoring hooks
- render-time query/client invalidation flows where state is only a trigger and the real effect lives outside the tracked local write

## Current distance to the north-star

As of 2026-03-08, the biggest remaining ergonomics gap is no longer text inputs. The narrow text
bridge already lets `Input` / `Textarea` consume `&LocalState<String>` directly.

The clearest remaining gaps are now narrower:

- widget-local `listener` / `dispatch` sugar is still not the default path,
- builder-first `.child(...)` composition is improving but `ui::children!` remains common in medium surfaces,
- product-facing docs/templates still need a sharper default/comparison/advanced taxonomy so users do
  not have to infer the intended path from scattered examples,
- `DataTable` remains a separate business-table/reference surface whose current pressure is more
  about state/output/toolbar recipe assembly than about missing primitive builder helpers,
- remaining explicit-model collection examples are now comparison-only or intentionally advanced rather
  than default-surface blockers.

`apps/fret-cookbook/examples/simple_todo_v2_target.rs`, `apps/fret-examples/src/todo_demo.rs`, and
`apps/fretboard/src/scaffold/templates.rs` now collectively show that the current v2 path already
covers cookbook, app-grade, and scaffold keyed-list surfaces via `LocalState<Vec<TodoRow>>`,
payload row actions, and snapshot checkbox bindings. That shifts the remaining visible pressure more
toward handler placement, default-path documentation, and narrower widget ergonomics rather than
macros or another round of generic tracked-write helpers.

## Current recommendation order (2026-03-09)

The next post-v1 pass should stay disciplined:

1. **Productize the current default path first**
   - keep `hello` → `simple-todo` → `todo` as the obvious onboarding ladder,
   - make default/comparison/advanced surfaces explicit in docs and templates,
   - avoid promoting more helpers until that teaching surface is boring and consistent.
2. **Re-evaluate narrow widget-local action sugar second**
   - only if at least two real medium surfaces still look materially noisier than the root-handler path,
   - keep action identity and root handler table semantics visible.
3. **Keep macros third and optional**
   - no new macro work is required for v2 success,
   - only revisit narrow composition macros if builder-first cleanup still leaves repeated structural noise.

## What is *not* default yet

These remain valid, but they are not the default golden path:

- raw `cx.on_action(...)`
- widget-local `listener` / `dispatch` / `shortcut` sugar
- another default transaction helper beyond `on_action_notify_models::<A>(...)`
- macros beyond existing minimal helpers
- `DataTable` as a default first-contact teaching surface; until a curated recipe exists, treat it as
  an advanced business-table integration example
- broad `ui::children!`-heavy trees when a root/section/trigger builder already exists
- early `into_element(cx)` just to attach semantics or diagnostics hooks when the surrounding sink already accepts builders (for example numeric badges/text or other decorate-only patches); if a sink still requires concrete `AnyElement`s, land exactly at that boundary instead of inventing extra adapters

For the business-table tier specifically, use
`docs/workstreams/action-first-authoring-fearless-refactor-v1/DATA_TABLE_GOLDEN_PATH.md` as the
curated note instead of treating `DataTable` as part of the first-contact/default onboarding path.

## Promotion rule for new helpers

Do not promote a new helper into the default path unless:

1. at least two real demos/templates need the same shape,
2. the existing generic path is measurably noisier,
3. the helper does not hide action identity, key context, or cache-boundary semantics.

That rule is how v2 avoids turning every local annoyance into another permanent surface.
