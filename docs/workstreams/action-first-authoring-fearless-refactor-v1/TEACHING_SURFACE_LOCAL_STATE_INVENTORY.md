# Teaching-Surface Local-State Inventory (Draft)

Status: draft, post-v1 audit
Last updated: 2026-03-09

Related:

- TODO: `docs/workstreams/action-first-authoring-fearless-refactor-v1/TODO.md`
- Milestones: `docs/workstreams/action-first-authoring-fearless-refactor-v1/MILESTONES.md`
- Migration notes: `docs/workstreams/action-first-authoring-fearless-refactor-v1/MIGRATION_GUIDE.md`
- Golden path: `docs/workstreams/action-first-authoring-fearless-refactor-v1/V2_GOLDEN_PATH.md`
- Widget-contract audit: `docs/workstreams/action-first-authoring-fearless-refactor-v1/MODEL_CENTERED_WIDGET_CONTRACT_AUDIT.md`

This note records which teaching surfaces still hold direct `Model<T>` fields and whether they
should migrate to `use_local*`, stay hybrid, or intentionally remain explicit-model examples.

## Already aligned with the post-v1 default path

These surfaces already serve as evidence for the intended default authoring model:

- `apps/fret-cookbook/examples/hello_counter.rs`
- `apps/fret-cookbook/examples/query_basics.rs`
- `apps/fret-cookbook/examples/commands_keymap_basics.rs`
- `apps/fret-cookbook/examples/text_input_basics.rs`
- `apps/fret-cookbook/examples/overlay_basics.rs`
- `apps/fret-cookbook/examples/imui_action_basics.rs`
- `apps/fret-cookbook/examples/date_picker_basics.rs`
- `apps/fret-cookbook/examples/form_basics.rs`
- `apps/fret-cookbook/examples/simple_todo.rs`
- `apps/fret-cookbook/examples/drop_shadow_basics.rs`
- `apps/fret-cookbook/examples/markdown_and_code_basics.rs`
- `apps/fret-cookbook/examples/assets_reload_epoch_basics.rs`
- `apps/fret-cookbook/examples/virtual_list_basics.rs`
- `apps/fret-cookbook/examples/theme_switching_basics.rs`
- `apps/fret-cookbook/examples/icons_and_assets_basics.rs`
- `apps/fret-cookbook/examples/customv1_basics.rs`
- `apps/fretboard/src/scaffold/templates.rs` (`simple_todo_template_main_rs`)

Those examples collectively cover:

- straightforward local writes,
- local overlay/interop examples that still bridge model-centered widget boundaries with `local.clone_model()`,
- command availability and widget interop,
- cross-frontend action convergence (declarative + IMUI + GenUI) while keeping the shared counter on `use_local*`,
- controlled widget bridges that still require `Model<T>` at the component boundary,
- pure toggle demos that still bridge into model-centered widgets,
- mixed editor/render-option demos that bridge multiple model-centered widgets on one page,
- local trigger demos that still keep host/runtime side effects in render-time escape hatches,
- local theme-selection demos that still keep render-time host effect application explicit,
- asset demos that keep reload synchronization explicit while localizing the trigger state,
- renderer/effect demos where control values become local state while capability/effect plumbing stays explicit,
- virtualization hybrids where collection identity + scroll coordination stay explicit but the surrounding controls move to local state,
- multi-field coordination that still stays on `on_action_notify_models`,
- keyed dynamic-list hybrids.

## Queue A: migrate next (mostly view-local state)

Queue A is currently cleared.

- `apps/fret-cookbook/examples/assets_reload_epoch_basics.rs` was the last pure local-trigger holder.
  It now uses `use_local*` / `state.layout(cx).value_*` for the bump counter while intentionally keeping the
  actual asset reload epoch bump, redraw request, and RAF scheduling as render-time host/runtime
  escape hatches.

## Queue B: hybrid candidates

Queue B is currently cleared.

- `apps/fret-cookbook/examples/customv1_basics.rs` was the last default-surface hybrid candidate.
  It now uses `use_local*` / `state.paint(cx).value_*` plus typed actions for `enabled` / `strength`, while
  effect registration, renderer capability checks, and effect-layer plumbing remain explicit
  render-time/runtime concerns.
- The remaining explicit-model cookbook cases are now advanced by design rather than pending
  local-state migrations.

## Keep explicit models for now

These surfaces currently look intentionally model-centric or interop-bound. They should not be the
first wave of `use_local*` migration.

### Runtime / async / service-bound

- `apps/fret-cookbook/examples/async_inbox_basics.rs`
- `apps/fret-cookbook/examples/canvas_pan_zoom_basics.rs`
- `apps/fret-cookbook/examples/embedded_viewport_basics.rs`
- `apps/fret-cookbook/examples/external_texture_import_basics.rs`

Reason: background execution, viewport coordination, diagnostics surfaces, or host interop are
part of the teaching goal.

### Component APIs that are still model-centered today

- `apps/fret-cookbook/examples/data_table_basics.rs`
- `apps/fret-cookbook/examples/date_picker_basics.rs` (the cookbook authoring side now uses `use_local*`
  + `local.clone_model()`, but the component contract still exposes `Model<T>` as the primary
  controlled surface)

Reason: the component contract itself still exposes `Model<T>` as the primary control surface.

### Intentional explicit-state examples

- `apps/fret-cookbook/examples/undo_basics.rs`
- `apps/fret-cookbook/examples/payload_actions_basics.rs`
- `apps/fret-cookbook/examples/router_basics.rs`
- `apps/fret-cookbook/examples/gizmo_basics.rs`

Reason: undo/history, payload routing, router synchronization, or tool/runtime shared state are part
of the example's point.

## Additional reference surfaces outside the cookbook/template default path

The inventory above closes the default cookbook/template migration queue. The following surfaces are
still useful in-tree references, but they should be treated as advanced/runtime-bound or
component-contract examples rather than blockers for the post-v1 default path.

### Keyed-list comparison / evidence targets

- `apps/fret-cookbook/examples/simple_todo_v2_target.rs`
- `apps/fret-examples/src/todo_demo.rs`

Reason: `simple_todo.rs`, `simple_todo_v2_target.rs`, `todo_demo.rs`, and the scaffold simple-todo
template now all live on the same local-state keyed-list baseline. The dedicated comparison pressure
has therefore narrowed to `simple_todo_v2_target.rs` (denser payload-row/root-handler shape) rather
than to an explicit-model-vs-local-state split. See
`EXPLICIT_MODEL_COLLECTION_SURFACE_INVENTORY.md` for the updated sequencing note.

### `apps/fret-examples` (advanced / runtime-bound)

- `apps/fret-examples/src/async_playground_demo.rs`
- `apps/fret-examples/src/embedded_viewport_demo.rs`
- `apps/fret-examples/src/window_hit_test_probe_demo.rs`

Reason: these demos intentionally exercise query/client wiring, shared runtime state, embedded
viewport interop, or multi-window host integration. `embedded_viewport_demo` now keeps its
view-local `size_preset` knob on the app-lane `LocalState` surface
(`cx.state().local_init(...)` + `layout_value(...)` + `cx.actions().local_set(...)`), but the
embedded viewport surfaces, forwarded input state, and host/window coordination remain
intentionally explicit/runtime-bound.

### `apps/fret-ui-gallery` snippets (reference composition / current controlled APIs)

- `apps/fret-ui-gallery/src/ui/snippets/card/demo.rs`
- `apps/fret-ui-gallery/src/ui/snippets/button_group/input.rs`
- `apps/fret-ui-gallery/src/ui/snippets/button_group/popover.rs`

Reason: these snippets currently mirror controlled/reference contracts that still speak in
`Model<T>` terms (`ensure_models(...)`, outward synchronization, or popover/text-value composition).
`Input` / `Textarea` now accept `&LocalState<String>` on the default path, so text widgets are no
longer a cookbook/template teaching-surface blocker; the remaining gallery cases stay here because
the snippet itself still chooses explicit synchronization semantics. The collapsible gallery snippet
is no longer in this bucket because `Collapsible` already exposes an uncontrolled `default_open(...)`
path and the snippet now uses it directly.

## Recommended migration order

No further default-surface local-state migrations are queued right now.

The next wave should focus on:

- tracked-state write ergonomics and the remaining explicit-model escape hatches after render-side `value_*` plus store-side `value_in*` reads landed,
- explicit-model collection documentation that keeps `simple_todo.rs` as the reference contrast while treating the migrated scaffold/app-grade keyed-list path as the default,
- explicit advanced docs for the remaining interop-bound/model-centered examples,
- only then consider another surgical advanced-demo cleanup if a remaining field is unambiguously view-local and does not blur the runtime/interop lesson,
- keeping new cookbook/template work on the post-v1 local-state default path.

## Deprecation implication

The remaining work before deprecating old teaching-surface `use_state` / direct view-held
`Model<T>` patterns is not "migrate every model in the repo". A safer sequence is:

1. keep templates/docs/tests aligned with the current local-state default path,
2. document the remaining explicit-model examples as advanced/interop-bound,
3. avoid reopening raw view-held `Model<T>` patterns in new default teaching surfaces,
4. only then start warning against raw view-held `Model<T>` in the default path.

Only after that should the repo start warning against raw view-held `Model<T>` in default teaching
surfaces.
