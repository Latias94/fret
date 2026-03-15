# Authoring v2 Golden Path (Draft)

Status: draft, post-v1 guidance
Last updated: 2026-03-15

Related:

- Proposal: `docs/workstreams/action-first-authoring-fearless-refactor-v1/POST_V1_AUTHORING_V2_PROPOSAL.md`
- Current-vs-target gap note: `docs/workstreams/action-first-authoring-fearless-refactor-v1/V2_BEST_PRACTICE_GAP.md`
- `notify()` policy: `docs/workstreams/action-first-authoring-fearless-refactor-v1/NOTIFY_POLICY_DECISION_DRAFT.md`
- invalidation default rules: `docs/workstreams/action-first-authoring-fearless-refactor-v1/INVALIDATION_DEFAULT_RULES.md`
- Post-v1 shortlist: `docs/workstreams/action-first-authoring-fearless-refactor-v1/POST_V1_SURFACE_SHORTLIST.md`
- Invalidation/local-state review: `docs/workstreams/action-first-authoring-fearless-refactor-v1/INVALIDATION_LOCAL_STATE_REVIEW.md`
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
- widget wiring: prefer `.action(...)` / `.action_payload(...)` whenever the widget already exposes
  a stable action slot; for activation-only surfaces prefer `.dispatch::<A>(cx)`,
  `.dispatch_payload::<A>(cx, ...)`, and `.listen(cx, ...)` before reopening raw `Arc<dyn Fn...>`
  seams
- invalidation: tracked state writes should usually rerender implicitly; keep explicit `notify()` for
  imperative/cache-oriented escape hatches
- composition: stay on builder-first paths until the final runtime boundary

## The five recommended composition entry points

### 1. Root-owned composition entry points

Use root `build(...)` helpers or typed root constructors when the component owns the runtime boundary.

Examples:

- `shadcn::Card::build(...)`
- `shadcn::Alert::build(...)`
- `shadcn::ScrollArea::build(...)`
- `shadcn::FieldSet::build(...)`
- `shadcn::Dialog::build(cx, trigger, content)`
- `shadcn::AlertDialog::build(cx, trigger, content)`
- `shadcn::Popover::new(cx, trigger, content)`

Rule: if a component already has a host-bound root builder or typed root constructor, prefer it
over assembling early-landed `AnyElement` trees by hand. When a helper only feeds parent children,
prefer keeping it on the late-landing child pipeline (`UiChild` on app-facing surfaces; the
unified component conversion surface tracked by
`docs/workstreams/into-element-surface-fearless-refactor-v1/` elsewhere) over forcing an
intermediate `AnyElement`.

### 2. Section `build(...)`

Use section builders to keep nested content builder-first.

Examples:

- `CardHeader::build(...)`, `CardContent::build(...)`, `CardFooter::build(...)`
- `DialogContent::build(...)`, `DialogHeader::build(...)`, `DialogFooter::build(...)`
- `AlertDialogContent::build(...)`, `DrawerContent::build(...)`
- `FieldGroup::build(...)`, `Field::build(...)`
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
- in practice, the remaining default-path `on_action_notify_models::<A>(...)` surfaces should be
  read as one of three ownership classes: coordinated writes, command/keymap ownership, or
  cross-field form ownership
- `count.update_in(models, |value| { ... })` / `count.set_in(models, value)` are store-only transaction helpers; `count.update_in_if(models, |value| -> bool { ... })` is the handled-aware variant for collection-style writes. They become rerendering writes when used under `on_action_notify_models::<A>(...)`, while `on_action_notify_local_*` / `update_action(...)` / `update_action_if(...)` are the direct tracked-write path
- `count.value_in_or_default(models)` / `count.value_in_or(models, fallback)` for the common store-side read path, and `count.read_in(models, |value| ...)` / `count.revision_in(models)` when the closure needs a custom projection or revision check
- `let query_state = query_handle.layout(cx).value_or_else(QueryState::<T>::default);` for query results, instead of reopening `query_handle.model()` at the teaching surface
- `cx.on_payload_action_notify_local_update_if::<act::ToggleRow, Vec<Row>>(&rows, |rows, id| { ... })` as the narrow keyed-list / payload-row helper when the remaining noise is just local collection mutation boilerplate

Keep explicit `notify()` for:

- imperative integrations
- cache-boundary invalidation not represented by a tracked write
- host/runtime callbacks that mutate state outside the first-class authoring hooks
- render-time query/client invalidation flows where state is only a trigger and the real effect lives outside the tracked local write

Practical boundary: `use_local*` is the default local-state path for this phase, but it is still a
model-backed handle rather than a plain-Rust/self-owned state slot. Treat `clone_model()` and other
explicit widget/runtime bridges as intentional boundary markers, not as proof that the repo still
needs another layer of default local-state sugar.

## Current distance to the north-star

As of 2026-03-08, the biggest remaining ergonomics gap is no longer text inputs. The narrow text
bridge already lets `Input` / `Textarea` consume `&LocalState<String>` directly.

The clearest remaining gaps are now narrower:

- the biggest remaining "write UI" feel gap is now the adjacent conversion-surface split
  (`UiIntoElement`, `UiHostBoundIntoElement`, `UiChildIntoElement`, bridge traits) rather than the
  grouped app-facing surface,
- local state is now stable as a default teaching path, but `LocalState<T>` is still model-backed rather than the fully plain-Rust/self-owned north-star,
- keyed-list / payload-row flows now have one narrow helper for local collection mutation, but the
  canonical trio (`simple_todo_v2_target`, `todo_demo`, scaffold template) still keeps visible
  root action tables and build-sink friction,
- builder-first `.child(...)` composition is largely in maintenance mode now outside that
  canonical keyed/list slice; `ui::children!` remains common in some medium surfaces, but the
  remaining pressure is mostly adoption or advanced/runtime-owned seams rather than a missing
  generic builder family,
- product-facing docs/templates still need a sharper default/comparison/advanced taxonomy so users do
  not have to infer the intended path from scattered examples,
- `DataTable` remains a separate business-table/reference surface whose current pressure is more
  about state/output/toolbar recipe assembly than about missing primitive builder helpers,
- remaining explicit-model collection examples are now comparison-only or intentionally advanced rather
  than default-surface blockers.

`apps/fret-cookbook/examples/simple_todo_v2_target.rs`, `apps/fret-examples/src/todo_demo.rs`, and
`apps/fretboard/src/scaffold/templates.rs` now collectively show that the current v2 path already
covers cookbook, app-grade, and scaffold keyed-list surfaces via `LocalState<Vec<TodoRow>>`,
payload row actions, snapshot checkbox bindings, and the new narrow
`on_payload_action_notify_local_update_if::<...>(...)` helper. That shifts the remaining visible
pressure more toward default-path documentation and narrower widget ergonomics rather than macros or
another round of generic tracked-write helpers.

## Current recommendation order (2026-03-12)

The next post-v1 pass should stay disciplined:

1. **Collapse the conversion surface first**
   - use `docs/workstreams/into-element-surface-fearless-refactor-v1/` to remove the biggest
     remaining `into_element` taxonomy gap.
2. **Reopen canonical keyed/list/build-sink density second**
   - treat `simple_todo_v2_target`, `todo_demo`, and the scaffold template as the primary evidence set,
   - prefer one narrow list-authoring improvement over broad helper expansion.
3. **Productize the current default path around the same evidence**
   - keep `hello` → `simple-todo` → `todo` as the obvious onboarding ladder,
   - make default/comparison/advanced surfaces explicit in docs and templates,
   - keep the canonical trio aligned on one intended writing style.
4. **Keep broader builder-first seam work behind that**
   - the last clearly repeated medium families are already closed for this pass,
   - reopen only if a new cross-surface host/root seam still forces eager landing across multiple
     real default-facing surfaces outside the canonical keyed/list slice.
5. **Keep macros fifth and optional**
   - no new macro work is required for v2 success,
   - only revisit narrow composition macros if builder-first cleanup still leaves repeated structural noise.

## What is *not* default yet

These remain valid, but they are not the default golden path:

- raw `cx.on_action(...)`
- broader keyed-list / payload-row-specific handler sugar beyond
  `on_payload_action_notify_local_update_if::<...>(...)`
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
