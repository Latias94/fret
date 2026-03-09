# Action-First Authoring + View Runtime (Fearless Refactor v1) — Milestones

Last updated: 2026-03-09

Related:

- Design: `docs/workstreams/action-first-authoring-fearless-refactor-v1/DESIGN.md`
- TODO: `docs/workstreams/action-first-authoring-fearless-refactor-v1/TODO.md`
- Post-v1 proposal: `docs/workstreams/action-first-authoring-fearless-refactor-v1/POST_V1_AUTHORING_V2_PROPOSAL.md`
- Post-v1 shortlist: `docs/workstreams/action-first-authoring-fearless-refactor-v1/POST_V1_SURFACE_SHORTLIST.md`
- Default-path productization: `docs/workstreams/action-first-authoring-fearless-refactor-v1/DEFAULT_PATH_PRODUCTIZATION.md`
- Invalidation/local-state review: `docs/workstreams/action-first-authoring-fearless-refactor-v1/INVALIDATION_LOCAL_STATE_REVIEW.md`
- V2 golden path: `docs/workstreams/action-first-authoring-fearless-refactor-v1/V2_GOLDEN_PATH.md`
- DataTable audit: `docs/workstreams/action-first-authoring-fearless-refactor-v1/DATA_TABLE_AUTHORING_AUDIT.md`
- DataTable golden path: `docs/workstreams/action-first-authoring-fearless-refactor-v1/DATA_TABLE_GOLDEN_PATH.md`
- Teaching-surface inventory: `docs/workstreams/action-first-authoring-fearless-refactor-v1/TEACHING_SURFACE_LOCAL_STATE_INVENTORY.md`
- Widget-contract audit: `docs/workstreams/action-first-authoring-fearless-refactor-v1/MODEL_CENTERED_WIDGET_CONTRACT_AUDIT.md`
- Hard-delete execution checklist: `docs/workstreams/action-first-authoring-fearless-refactor-v1/HARD_DELETE_EXECUTION_CHECKLIST.md`
- Compat-driver inventory: `docs/workstreams/action-first-authoring-fearless-refactor-v1/COMPAT_DRIVER_CALLER_INVENTORY.md`
- Compat-driver policy: `docs/workstreams/action-first-authoring-fearless-refactor-v1/COMPAT_DRIVER_POLICY_DECISION_DRAFT.md`
- `use_state` inventory: `docs/workstreams/action-first-authoring-fearless-refactor-v1/USE_STATE_CALLER_INVENTORY.md`
- `use_state` policy: `docs/workstreams/action-first-authoring-fearless-refactor-v1/USE_STATE_POLICY_DECISION_DRAFT.md`
- Command-first widget audit: `docs/workstreams/action-first-authoring-fearless-refactor-v1/COMMAND_FIRST_WIDGET_CONTRACT_AUDIT.md`

---

## Current status snapshot (as of 2026-03-09)

This snapshot is intentionally evidence-based: only mark a milestone as “Met” when the in-tree code,
teaching surfaces, and gates line up.

- **M0**: Met (workstream docs + ADRs exist; indices are updated).
- **M1**: Met (typed unit actions exist; keymap/palette/menu/pointer triggers converge on the same dispatch pipeline, with diagnostics traces explaining availability/dispatch outcomes).
- **M2**: In progress (View runtime v1 exists; `ViewCx` action helpers landed; default onboarding has narrowed to three entrypoints; adoption in templates + cookbook/examples is ongoing).
- **M3**: Planned (multi-frontend convergence: declarative + imui + GenUI).
- **M4**: In progress (cookbook/examples + ui-gallery now share the same default `value_*` read suffix, default teaching/reference surfaces have moved off `use_state`, and broader builder-first cleanup continues).
- **M4 note**: primitive `Table` builder-first cleanup is now close to saturated; the remaining
  `DataTable` pressure is tracked separately as a post-v1 business-table authoring/productization
  audit rather than as unfinished primitive-table migration work.
- **M4 note**: a docs-first `DataTable` golden-path note now exists, so future work should only
  widen helpers if a smaller curated recipe still looks materially too noisy in practice.
- **M5**: Planned (editor-grade proof points: docking/workspace integration).
- **M6**: Met (MVU long-term stance is decided; in-tree MVU is removed and only archival migration notes remain).
- **M7-M9**: Met (payload actions v2 landed; MVU hard delete and reintroduction gates are in place).
- **Overall assessment**: v1 is successful as an architectural reset and teaching-surface convergence
  pass; the remaining gap to the original GPUI/Zed-style density target is treated as post-v1
  ergonomics work rather than unfinished migration closure.

Adoption note (as of 2026-03-07):

- A follow-up polish pass is active to reduce “early element landing” (`into_element(cx)` cliffs) in
  cookbook demos by preferring late-landing child composition (`ui::children!`, `*_::build(...)`).
  Recent closure steps also kept child-only helpers on `impl UiChildIntoElement<_>` where possible
  and removed extra template/demo `.into_element(cx)` call sites around palette/filter helper paths.
  See
  `docs/workstreams/action-first-authoring-fearless-refactor-v1/TODO.md` (`AFA-adopt-045`).
- Post-v1 repo-shape note: the remaining authoring work is ergonomics + teaching-surface convergence,
  not a Bevy-style single-package root `examples/` rewrite and not an expansion of `ecosystem/fret`
  into the repo?s canonical example host.
- Active post-v1 order: productize the current default path first (onboarding ladder +
  default/comparison/advanced taxonomy), then continue invalidation/local-state guidance and only
  the highest-leverage builder-first seams, while keeping keyed-list / payload-row handler
  ergonomics in maintenance mode unless new evidence appears; macros stay last.
- Active shortlist note: `POST_V1_SURFACE_SHORTLIST.md` now narrows the next truly worthwhile
  surfaces to default-path productization first, invalidation/local-state ergonomics second,
  builder-first last-mile seams third, and keyed-list / payload-row handler ergonomics only after
  those passes.
- Productization update (as of 2026-03-09): `DEFAULT_PATH_PRODUCTIZATION.md` now defines the ladder
  and label contract explicitly, and `README.md`, `docs/first-hour.md`,
  `docs/crate-usage-guide.md`, `docs/ui-ergonomics-and-interop.md`,
  `docs/examples/README.md`, `docs/examples/todo-app-golden-path.md`,
  `apps/fret-cookbook/README.md`, `apps/fret-cookbook/EXAMPLES.md`,
  `apps/fret-ui-gallery/README.md`, the `data_table` gallery page framing, and generated scaffold
  READMEs now align on the same default/comparison/advanced framing.
- Invalidation review update (as of 2026-03-09): `INVALIDATION_LOCAL_STATE_REVIEW.md` now uses
  `apps/fret-cookbook/examples/simple_todo_v2_target.rs`, `apps/fret-cookbook/examples/query_basics.rs`,
  `apps/fret-cookbook/examples/commands_keymap_basics.rs`, and `apps/fret-cookbook/examples/form_basics.rs`
  as a focused medium-surface set and records a split result: keyed-list pressure has shifted from
  invalidation/local-state helpers to root handler placement for payload row actions, query/client
  invalidation still belongs to the explicit render-time escape hatch path, and command/keymap plus
  cross-field form root handling remain intentional ownership boundaries rather than the best sugar
  targets.
- Keyed-list helper prototype update (as of 2026-03-09): `ecosystem/fret/src/view.rs` now adds the
  deliberately narrow `ViewCx::on_payload_action_notify_local_update_if::<A, T>(...)` helper, and
  `apps/fret-cookbook/examples/simple_todo_v2_target.rs`, `apps/fret-examples/src/todo_demo.rs`,
  plus the generated simple-todo scaffold now use it for payload-row local collection mutations.
- Keyed-list follow-up decision (as of 2026-03-09): with the helper adopted on cookbook/app/scaffold
  todo-like surfaces and `INVALIDATION_LOCAL_STATE_REVIEW.md` ruling out command/query/form
  ownership boundaries as valid evidence for broader sugar, `AFA-postv1-003` is now treated as
  closed for this pass. Reopen only if another medium surface shows the same row-local
  handler-placement pressure.
- Business-table note: `DataTable` is now explicitly treated as a separate post-v1 audit/problem
  space. It should not keep the primitive `Table` builder-first cleanup milestone artificially
  open.
- Adjacent examples-side polish is also underway: `apps/fret-examples` now uses the same builder-first
  guidance for decorate-only `test_id` / semantics patches in `todo_demo` and the utility-window /
  hit-test demos where the host sink already accepts builders; raw pointer-region and container roots
  still intentionally land as `AnyElement` boundaries.
- Cookbook polish has narrowed further as well: `async_inbox_basics`, `imui_action_basics`, and
  `utility_window_materials_windows` now keep their progress/root diagnostics hooks on the builder
  path, and `apps/fret-cookbook/src/scaffold.rs` now keeps the shared page-shell root `test_id` on
  that same path. That leaves `date_picker_basics` as the intentionally documented host-boundary
  exception.
- Local-state cleanup has narrowed as well: `overlay_basics` and `imui_action_basics` now use
  `use_local*`, so the default/runtime teaching surfaces no longer demonstrate `use_state::<T>()`
  as the local-state story.

Evidence anchors (verified in-tree as of 2026-03-08):

- `ecosystem/fret/src/view.rs` (`TrackedStateExt::{layout, paint, hit_test}` now covers `LocalState<T>`, `Model<T>`, and `QueryHandle<T>` behind `state-query`; `WatchedState::value*`, `LocalState::{watch, read_in, revision_in}`, documented `LocalState::{update_in, set_in, update_action, set_action}` write semantics, `ViewCx::on_action_notify_*`, and `ViewCx::on_action_notify_local_*` helpers)
- `ecosystem/fret/src/view.rs` test `local_state_update_action_requests_redraw_and_notify` (locks the post-v1 rule that tracked local writes inside action dispatch request redraw and notify, while `notify()` remains an escape hatch rather than a teaching-surface default)
- `ecosystem/fret-ui-kit/src/declarative/model_watch.rs` (`ModelWatchExt` still backs legacy `cx.watch_model(...)`, while `QueryHandleWatchExt` now gives `ElementContext` query surfaces the query-specific handle-side `handle.layout_query(cx).value_*` read shape that mirrors the View runtime)
- `docs/examples/todo-app-golden-path.md`, `docs/integrating-tokio-and-reqwest.md`, `docs/workstreams/imui-state-integration-v1.md` (narrative docs now mirror the same query handle-side read story across `ViewCx` and `ElementContext`, including the `QueryHandleWatchExt` path for `handle.paint_query(cx).value_*` / `handle.layout_query(cx).value_*`)
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/TEACHING_SURFACE_LOCAL_STATE_INVENTORY.md`, `docs/workstreams/action-first-authoring-fearless-refactor-v1/MODEL_CENTERED_WIDGET_CONTRACT_AUDIT.md`, `apps/fret-examples/src/async_playground_demo.rs`, `apps/fret-examples/src/embedded_viewport_demo.rs`, `apps/fret-ui-gallery/src/ui/snippets/card/demo.rs`, `apps/fret-ui-gallery/src/ui/snippets/button_group/popover.rs` (post-v1 audit now distinguishes cookbook/template closure from advanced/runtime-bound surfaces, true model-centered widget contracts, and snippet-level controlled/uncontrolled authoring choices)
- `apps/fret-cookbook/examples/hello.rs` (simple baseline remains the smallest action-first view reference case)
- `apps/fret-cookbook/examples/hello_counter.rs` (first medium cookbook example on the `use_local` + `state.layout(cx).value_*` / `state.paint(cx).value_*` + local-state-specific notify helpers path)
- `apps/fret-cookbook/examples/query_basics.rs` (second medium cookbook example on the same local-state path, while still demonstrating render-time query invalidation and explicit redraw as an escape hatch; query result reads now stay on the `QueryHandle<T>` side via `handle.layout(cx).value_*`)
- `apps/fret-cookbook/examples/commands_keymap_basics.rs` (third medium cookbook example on the same path; validates command availability + keymap gating against `use_local*` / `state.layout(cx).value_*` / `state.paint(cx).value_*` without view-held `Model<bool>` fields, now uses `Switch::from_checked(...).action(...)` for its view-local allow-command toggle plus a disabled snapshot indicator for panel state, and keeps coordinated gate reads on `LocalState::read_in(...)` inside the generic transaction/availability closures)
- `apps/fret-cookbook/examples/toggle_basics.rs` (new focused cookbook example proving `Toggle::from_pressed(...).action(...)` on view-local state without a `Model<bool>` bridge)
- `apps/fret-cookbook/examples/text_input_basics.rs` (validates `use_local*` + `state.layout(cx).value_*` / `state.paint(cx).value_*` + direct `Input::new(&LocalState<String>)` interop while keeping command availability on the generic models path)
- `apps/fret-cookbook/examples/overlay_basics.rs` (now keeps dialog open state and the underlay counter on `use_local*`, using `LocalState::clone_model()` only at the model-centered dialog/widget boundary)
- `apps/fret-cookbook/examples/imui_action_basics.rs` (now keeps the shared counter on `use_local*` while still proving declarative + IMUI + GenUI action dispatch convergence)
- `apps/fret-cookbook/examples/date_picker_basics.rs` (extends the same bridge pattern to `DatePicker::new_controllable(...)`, proving the authoring side can still prefer `use_local*` even when the widget API remains model-centered; the selected row stays builder-first, while the current picker/card sink still lands at the widget host boundary when a concrete `AnyElement` is required)
- `apps/fret-cookbook/examples/form_basics.rs` (extends the same local-state path to multi-field validation/reset flows while intentionally keeping cross-field coordination on `on_action_notify_models`)
- `apps/fret-cookbook/examples/simple_todo.rs` (default cookbook keyed-list lesson now uses `LocalState<Vec<_>>`, payload row toggle, and stable keyed row identity; the older explicit-model split is no longer carried by the boring path)
- `apps/fretboard/src/scaffold/templates.rs` (`simple_todo_template_main_rs` now uses the generated starter default keyed-list path: `LocalState<Vec<_>>`, payload row actions, `Checkbox::from_checked(...)`, direct text-value bridge `Input::new(&draft_state)`, and query-tip handle-side reads for `QueryHandle<T>`)
- `apps/fret-cookbook/examples/drop_shadow_basics.rs` (adds a pure toggle-only renderer demo to the same post-v1 path by using `use_local*` / `state.layout(cx).value_*` / `state.paint(cx).value_*` / `local.clone_model()` for the existing `Switch::new(Model<bool>)` boundary)
- `apps/fret-cookbook/examples/markdown_and_code_basics.rs` (extends the same path to a mixed editor/render-options surface: `Textarea` now accepts `&LocalState<String>` directly, while `ToggleGroup::single` and `Switch` still consume models and the view itself keeps source/wrap/cap-height in local state)
- `apps/fret-cookbook/examples/assets_reload_epoch_basics.rs` (extends the same path to a host/runtime escape-hatch surface: the bump counter now lives in local state, while the actual asset reload epoch bump plus redraw/RAF scheduling intentionally stay render-time)
- `apps/fret-cookbook/examples/drag_basics.rs` / `apps/fret-cookbook/examples/undo_basics.rs` / `apps/fret-cookbook/examples/gizmo_basics.rs` (numeric semantics/test-id decoration now stay on the builder path for badges/text instead of forcing decorate-only `AnyElement` landings)
- `apps/fret-cookbook/examples/virtual_list_basics.rs` (extends the same path to the first virtualization hybrid: the items collection and scroll handle stay explicit, while measure mode / toggles / jump input now use local state and the reorder/scroll commands remain on `on_action_notify_models`)
- `apps/fret-cookbook/examples/theme_switching_basics.rs` (extends the same path to a theme-selection surface: the selected scheme now lives in local state while the actual theme application plus redraw/RAF synchronization intentionally stay render-time)
- `apps/fret-cookbook/examples/icons_and_assets_basics.rs` (extends the same path to an asset demo surface: the reload bump counter now lives in local state while the actual asset reload epoch bump plus redraw/RAF synchronization intentionally stay render-time)
- `apps/fret-cookbook/examples/customv1_basics.rs` (clears the last default-surface renderer hybrid by moving `enabled` / `strength` to `use_local*`, using `on_action_notify_toggle_local_bool` for the simple enable/disable flag, and leaving effect registration, capability checks, and effect-layer plumbing explicit)
- `apps/fretboard/src/scaffold/templates.rs` (`hello_template_main_rs` and `simple_todo_template_main_rs` now avoid template-only palette/filter `.into_element(cx)` cliffs, and `simple_todo_template_main_rs` stays on the `LocalState<Vec<_>>` default path so generated starter code no longer inherits the cookbook comparison split)
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/TEACHING_SURFACE_LOCAL_STATE_INVENTORY.md` (records Queue A + Queue B as cleared and treats the remaining teaching-surface `Model<T>` holders as intentionally advanced/interop-bound)
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/V2_GOLDEN_PATH.md` (documents the intentionally small default v2 surface and the five recommended composition entry points)
- `ecosystem/fret-ui-kit/src/activate.rs` (`on_activate_*` helpers for low-noise `OnActivate` authoring)
- `ecosystem/fret-ui-kit/src/ui.rs` (`UiElementSinkExt` + `UiChildIntoElement` for builder-first `*_build` sink composition and heterogeneous child bridging)
- `ecosystem/fret-ui-shadcn/src/card.rs` (`Card::build(...)` / `CardHeader::build(...)` / `CardContent::build(...)` allow late child landing, host-bound `.ui()` patching, and direct `children!` / `push_ui()` participation across the query-demo card tree)
- `ecosystem/fret-ui-shadcn/src/alert.rs` (`Alert::build(...)` keeps alert title/description content on the builder path, while `AlertAction::build(...)` does the same for the top-right action slot without reopening a broader helper family)
- `ecosystem/fret-ui-shadcn/src/layout.rs` (`container_vstack_build(...)` / `container_hstack_build(...)` / `container_hstack_centered_build(...)` keep older shadcn layout helpers on the same late-landing child pipeline)
- `ecosystem/fret-ui-shadcn/src/table.rs` (`Table::build(...)` / `TableHeader::build(...)` / `TableBody::build(...)` / `TableFooter::build(...)` / `TableRow::build(...)` extend the same late-landing child pipeline into the table composite stack)
- `ecosystem/fret-ui-shadcn/src/table.rs` (`TableCell::build(child)` is the first single-child late-landing sample layered onto the same authoring surface)
- `ecosystem/fret-ui-shadcn/src/dialog.rs` (`DialogTrigger::build(...)` and `Dialog::compose().content_with(...)` keep trigger/content authoring on the late-landing pipeline while supporting `DialogClose::from_scope()` in deferred content)
- `ecosystem/fret-ui-shadcn/src/sheet.rs` (`SheetTrigger::build(...)` and `Sheet::compose().content_with(...)` keep trigger/content authoring on the late-landing pipeline while supporting `SheetClose::from_scope()` in deferred content)
- `apps/fret-cookbook/examples/form_basics.rs`, `apps/fret-cookbook/examples/assets_reload_epoch_basics.rs`, `apps/fret-ui-gallery/src/ui/snippets/alert/basic.rs`, `apps/fret-ui-gallery/src/ui/snippets/alert/action.rs` (the first medium alert-family surfaces now stay on the builder-first path instead of pre-collecting alert children)
- `apps/fret-ui-gallery/src/ui/snippets/typography/table.rs` (gallery snippet now demonstrates the `TableCell::build(ui::text(...))` shape instead of forcing early child landing)
- `ecosystem/fret-genui-shadcn/src/resolver/data.rs` (GenUI data-table rendering now uses the table builder-first path instead of pre-collecting header/body row vectors)
- `apps/fretboard/src/scaffold/templates.rs` (scaffold templates prefer View + typed actions and now late-land their todo-card trees via `Card::build(...)`)
- `docs/first-hour.md` / `docs/examples/README.md` / `docs/examples/todo-app-golden-path.md` / `docs/fearless-refactoring.md` / `docs/crate-usage-guide.md` / `docs/ui-ergonomics-and-interop.md` (first-contact, golden-path, and ergonomics docs now teach the same three entrypoints and defer raw `on_action_notify` to cookbook/reference host-side cases)
- `apps/fret-ui-gallery/src/ui/pages/command.rs` (gallery teaching page now calls out the same default path and keeps advanced host-side cases out of the gallery narrative)
- `apps/fret-cookbook/examples/async_inbox_basics.rs` (`Cancel` uses the default path; `Start` remains advanced for host-side dispatcher/inbox scheduling; the page card now uses the late-landing card builder path)
- `apps/fret-cookbook/examples/commands_keymap_basics.rs` (command/keymap teaching surface now uses the late-landing card builder path for both the outer panel and nested card)
- `apps/fret-cookbook/examples/canvas_pan_zoom_basics.rs` (prefers `on_action_notify*` helpers)
- `apps/fret-cookbook/examples/date_picker_basics.rs` (now uses `use_local*` / `local.clone_model()` for the controlled open/selected state while keeping the existing date-picker widget boundary)
- `apps/fret-cookbook/examples/form_basics.rs` (prefers `on_action_notify_models`, now uses `use_local*` plus the direct text bridge for field inputs, and keeps the late-landing card builder path)
- `apps/fret-cookbook/examples/toast_basics.rs` (intentional advanced reference case for imperative Sonner host integration)
- `apps/fret-cookbook/examples/router_basics.rs` (`ClearIntents` uses the default path; back/forward remain advanced for router availability sync)
- `apps/fret-cookbook/examples/undo_basics.rs` (`Inc`/`Dec`/`Reset` use the default path; `Undo`/`Redo` keep the host-side RAF effect)
- `apps/fret-cookbook/examples/simple_todo.rs` (now matches the default cookbook keyed-list path: `LocalState<Vec<_>>`, payload row toggle, and keyed row identity without explicit row models)
- `apps/fret-cookbook/examples/simple_todo_v2_target.rs` (comparison target now keeps the keyed list in `LocalState<Vec<TodoRow>>`, uses `Checkbox::from_checked(...).action_payload(...)` for row toggles, and makes the remaining visible gap more precise: row-level event ergonomics still prefer root handler registration)
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/POST_V1_AUTHORING_V2_PROPOSAL.md`, `docs/workstreams/action-first-authoring-fearless-refactor-v1/V2_GOLDEN_PATH.md`, `docs/examples/todo-app-golden-path.md` (current baseline vs v2 north-star is now documented explicitly; text widgets and keyed-list default viability are no longer the main blockers, and the next gap is framed as productization/default-path clarity plus narrower event/composition ergonomics)
- `apps/fret-cookbook/examples/virtual_list_basics.rs` (prefers `on_action_notify_models` for scroll actions)
- `apps/fret-cookbook/examples/query_basics.rs` (prefers action helpers)
- `apps/fret-cookbook/examples/markdown_and_code_basics.rs` (prefers action helpers)
- `apps/fret-examples/src/custom_effect_v1_demo.rs` (reset action now uses the default `on_action_notify_models` transaction path)
- `apps/fret-examples/src/liquid_glass_demo.rs` (reset/preset/toggle-inspector actions now use the default `on_action_notify_models` transaction path)
- `apps/fret-examples/src/async_playground_demo.rs` (`ToggleTheme` now uses the default `on_action_notify_models` path and keeps the theme side effect as render-time state synchronization; its query result view now reads from `handle.layout_query(cx).value_*`)
- `apps/fret-examples/src/hello_counter_demo.rs` (first `use_local` prototype; removes explicit model-handle fields, now keeps its generic step read on `LocalState::read_in(...)`, uses the direct text bridge for its step input, and still keeps the default `on_action_notify_models` action surface)
- `apps/fret-examples/src/query_demo.rs` (second `use_local` prototype; also the first builder-first composition experiment using `ui::*_build` sinks plus `UiElementSinkExt`, now paired with late-landing `Card::build(...)` / `CardHeader::build(...)` / `CardContent::build(...)`, while query state reads stay on `QueryHandle<T>` via `query_handle.layout(cx).value_*`)
- `apps/fret-examples/src/query_async_tokio_demo.rs` (third `use_local` prototype; now also mirrors the builder-first composition experiment for the async query path with the same late-landing card tree while keeping transient invalidation and Tokio-backed async spawning, and query state reads stay on `QueryHandle<T>` via `query_handle.layout(cx).value_*`)
- `apps/fret-ui-gallery/src/ui/snippets/collapsible/demo.rs` (gallery snippet now uses the uncontrolled `Collapsible::default_open(false)` path directly; this case no longer counts as a model-centered contract blocker)
- `apps/fret-examples/src/embedded_viewport_demo.rs` (advanced demo now moves its view-local `size_preset` to `use_local_with(...)` + `on_action_notify_local_set(...)`, while embedded viewport interop state and render-time effects stay on the explicit runtime path)
- `apps/fret-examples/src/custom_effect_v2_web_demo.rs` (reset button uses `on_activate_request_redraw`)
- `apps/fret-examples/src/imui_floating_windows_demo.rs` (pressable overlap target uses `on_activate_notify`)
- `tools/gate_no_models_mut_in_action_handlers.py` (teaching-surface regression gate)
- `tools/gate_only_allowed_on_action_notify_in_teaching_surfaces.py` (locks the approved advanced `on_action_notify` teaching-surface exceptions and keeps `fret-examples` plus ui-gallery pages/snippets on the zero-exception path)
- `tools/gate_no_single_model_action_helpers_in_default_teaching_surfaces.py` (keeps `fret-examples` and ui-gallery teaching pages/snippets on the default helper surface without single-model aliases; scaffold templates keep equivalent unit-test assertions)
- `tools/gate_no_mvu_in_tree.py` / `tools/gate_no_mvu_in_cookbook.py` (prevent MVU surfaces from reappearing in code after the M9 hard delete)

Hardening follow-up (open):

- Key-context aware `when` evaluation (`keyctx.*`) is aligned across keymap matching, menus/palette gating, shortcut display, and diagnostics (see TODO `AFA-actions-019`).
- Embedded viewport interop has a view-runtime demo proving `record_engine_frame` composition (see TODO `AFA-adopt-044`).
- Authoring ergonomics: semantics/test IDs/key contexts can be attached before `into_element(cx)`, and `fret-ui-kit::ui::*` constructors are cx-less; cookbook + templates demonstrate the patterns (see TODO “Reduce authoring noise”).
- Teaching-surface convergence: cookbook/examples are gated to avoid legacy `stack::*` layout helpers and teach one layout authoring surface (`fret-ui-kit::ui::*`); ui-gallery migration is in progress (see TODO “Reduce authoring noise” and gates `tools/gate_no_stack_in_cookbook.py`, `tools/gate_no_stack_in_examples.py`).
- Helper-surface convergence: README/docs/templates plus `docs/crate-usage-guide.md` and `docs/ui-ergonomics-and-interop.md` now frame `on_action_notify_models`, `on_action_notify_transient`, and local `on_activate*` as the default mental model; advanced aliases remain available but stay off the default teaching surfaces via `tools/gate_no_single_model_action_helpers_in_default_teaching_surfaces.py` plus scaffold template unit tests, while the remaining advanced raw `on_action_notify` teaching cases are cookbook-only host-side categories locked by `tools/gate_only_allowed_on_action_notify_in_teaching_surfaces.py`.

Post-v1 direction (recommended):

- Close v1 as successful on architecture and default-path convergence; do not keep the migration phase open solely to chase the final authoring-density target.
- Track the remaining ergonomics pressure separately:
  - direct local-state ergonomics beyond `Model<T>`, with the next step being to turn the new todo-like `LocalState<Vec<_>>` comparison path into a boring default rather than a special evidence example; render-side `value_*`, store-side `value_in*`, and handled-aware `update_in_if` now cover the common read/write boilerplate, so the remaining pressure is concentrated on higher-level tracked-write ownership/invalidation defaults rather than syntax noise,
  - the new tracked-write inventory note confirms that the remaining noisy cases mostly live at explicit-model/shared coordination boundaries, so the next milestone step should evaluate those boundaries directly before adding more default action helpers,
  - business-table `DataTable` authoring is now classified separately from primitive `Table` builder cleanup; any future work here should be a curated recipe/productization decision, not another generic builder-helper expansion,
  - the explicit-model collection inventory now records that both `apps/fret-examples/src/todo_demo.rs` and the scaffold simple-todo template have moved onto the v2 local-state keyed-list path, so remaining explicit collection surfaces are comparison-only or intentionally advanced,
  - checkbox/switch/toggle action-only `control_id` parity is now closed, so any future discrete-widget work should point to a narrower regression than label forwarding,
  - skill-level parity guidance remains the shared rubric for any future discrete-widget audits before adding helpers,
  - explicit-vs-implicit invalidation ergonomics (`notify()` stays available, but should not be the default burden after tracked state writes),
  - builder-first composition that reduces `ui::children!` / nested `into_element(cx)`,
  - keyed-list / payload-row handler ergonomics only if a new round of evidence justifies promoting it,
  - narrow UI macros only if builder-first authoring still leaves repeated structural boilerplate; they are optional polish, not a v2 prerequisite.
  - after default-path convergence, shift the next milestone from helper design to productization: onboarding clarity, comparison/advanced-surface positioning, visual defaults, and a future deprecation plan.
  - The current deprecation/hard-delete blockers are now named explicitly in `docs/workstreams/action-first-authoring-fearless-refactor-v1/HARD_DELETE_GAP_ANALYSIS.md`: app-entry closure surfaces, compat runner entry points, `use_state` as a user-visible alias, and `CommandId`-first widget contracts.
  - The app-entry blocker now has a concrete recommended direction in `docs/workstreams/action-first-authoring-fearless-refactor-v1/APP_ENTRY_POLICY_DECISION_DRAFT.md`: `view::<V>()` as the only default path, `.ui(...)` as a temporary advanced bridge on a staged path to deprecation/removal.
  - `docs/workstreams/action-first-authoring-fearless-refactor-v1/APP_ENTRY_CALLER_INVENTORY.md` now records that the in-tree `App::ui*` callers have been migrated; the remaining work is deprecation/removal sequencing rather than demo conversion.
  - Progress update (as of 2026-03-09): `ecosystem/fret/src/app_entry.rs` now deprecates the closure-root app-entry methods, and `ecosystem/fret/src/lib.rs` plus `ecosystem/fret/README.md` are locked by an in-crate policy test so `view::<V>()` remains the only default path.
  - Deprecation-window update (as of 2026-03-09): `APP_ENTRY_POLICY_DECISION_DRAFT.md` now fixes the minimum downstream window for `App::ui*` at 2026-03-09 → earliest removal 2026-06-09, and still requires at least one published release carrying the deprecation warnings before hard delete is allowed.
  - Execution update (as of 2026-03-09): `docs/workstreams/action-first-authoring-fearless-refactor-v1/HARD_DELETE_EXECUTION_CHECKLIST.md` now turns those blockers into an ordered cleanup sequence, with `App::ui*` identified as the closest hard-delete/quarantine candidate and the compat runner / `use_state` / command-first widget decisions left as explicit next policy checkpoints.
  - Status-matrix update (as of 2026-03-09): `docs/workstreams/action-first-authoring-fearless-refactor-v1/HARD_DELETE_STATUS_MATRIX.md` now sharpens that sequence further: `App::ui*` is waiting on deprecation timing rather than migration work, compat runner and `use_state` are currently retained advanced/non-default seams, and the remaining command-first widget family is the main implementation-scoped cleanup track.
  - Compat-runner update (as of 2026-03-09): `docs/workstreams/action-first-authoring-fearless-refactor-v1/COMPAT_DRIVER_CALLER_INVENTORY.md` now shows that `run_native_with_compat_driver(...)` still serves three real in-tree caller families (plot/chart demos, low-level renderer/asset demos, advanced shell demos), so the next step is a keep-vs-quarantine policy call rather than an immediate hard delete.
  - Compat-runner decision draft (as of 2026-03-09): `docs/workstreams/action-first-authoring-fearless-refactor-v1/COMPAT_DRIVER_POLICY_DECISION_DRAFT.md` now recommends keeping `run_native_with_compat_driver(...)` as an advanced low-level interop seam for now, updating docs to mark it as non-default, and deferring any hard delete until a clearer quarantine boundary or replacement path exists.
  - Compat-runner wording update (as of 2026-03-09): the workstream checklist/gap-analysis side is now aligned with README/rustdoc, so Stage 3 is blocked only on future keep-vs-quarantine product decisions rather than wording drift.
  - `use_state` inventory update (as of 2026-03-09): `docs/workstreams/action-first-authoring-fearless-refactor-v1/USE_STATE_CALLER_INVENTORY.md` now shows that the starter/reference leaks are closed (`hello`, the `hello` scaffold template, the gallery action-first snippet, `overlay_basics`, and `imui_action_basics` now use `use_local*`), leaving `use_state` present only as explicit runtime/API substrate plus migration/contract documentation.
  - `use_state` decision draft (as of 2026-03-09): `docs/workstreams/action-first-authoring-fearless-refactor-v1/USE_STATE_POLICY_DECISION_DRAFT.md` now recommends keeping `use_state` for now as an explicit raw-model hook, treating `use_local*` as the only default local-state teaching path, and deferring any deprecation until the repo decides whether that explicit seam is permanent or should later move behind a narrower gate/deprecation path.
  - `use_state` gate update (as of 2026-03-09): `tools/gate_no_use_state_in_default_teaching_surfaces.py` now guards the approved first-contact/reference source files, `apps/fretboard/src/scaffold/templates.rs` keeps template output covered by unit assertions, and the canonical cross-platform runner `tools/pre_release.py` now runs the gate with the other teaching-surface policy checks.
  - Command-first widget audit update (as of 2026-03-09): `docs/workstreams/action-first-authoring-fearless-refactor-v1/COMMAND_FIRST_WIDGET_CONTRACT_AUDIT.md` now scopes the remaining blocker into already-aligned dual-surface widgets (`Button`, `CommandItem`), menu-family alias candidates (`DropdownMenu*`, `ContextMenu*`, `Menubar*`), and low-risk app-facing alias candidates (`NavigationMenu*`, `BreadcrumbItem`, Material `Snackbar`).
  - Command-first alias update (as of 2026-03-09): `BreadcrumbItem::action(...)`, `NavigationMenuLink::action(...)`, and `NavigationMenuItem::action(...)` now exist in `ecosystem/fret-ui-shadcn`, and the navigation-menu gallery snippets now prefer the action-first spelling while command-centric internals remain unchanged.
  - Material snackbar alias update (as of 2026-03-09): `ecosystem/fret-ui-material3/src/snackbar.rs` now exposes `Snackbar::action_id(...)` / `action_command(...)`, and `apps/fret-ui-gallery/src/ui/snippets/material3/snackbar.rs` now uses `action_id(...)` as the default public spelling while toast dispatch internals remain unchanged.
  - Material snackbar gate update (as of 2026-03-09): `tools/gate_material3_snackbar_default_surface.py` now locks that gallery snippet to `action_id(...)`, and the canonical cross-platform runner `tools/pre_release.py` runs the gate so the default snippet does not drift back to compat spellings.
  - Menu-family alias update (as of 2026-03-09): `ContextMenu*` and `Menubar*` item/checkbox/radio builders now also expose `action(...)` aliases, and the broader gallery menu surface now prefers that spelling across the main context-menu / menubar snippets (including `demo`, `checkboxes`/`checkbox`, `radio`, `submenu`, `rtl`, and icon/group variants), so the remaining command-first blocker is narrower and mostly about future docs/default-surface adoption rather than missing builder APIs.
  - Menu-helper follow-up (as of 2026-03-09): `text_edit_context_menu.rs`, workspace tab-strip context menus, and the focused menubar/context-menu keyboard-dismiss tests now also prefer `action(...)`, reducing the remaining command-shaped residue to narrower advanced/internal surfaces plus future gating work.
  - Dropdown-menu follow-up (as of 2026-03-09): `DropdownMenu*` now also exposes action-first aliases in `ecosystem/fret-ui-shadcn/src/dropdown_menu.rs`, the primary dropdown-menu snippets (`basic`, `demo`) plus overlay preview menu surfaces now prefer `action(...)`, and the remaining command-shaped dropdown residue is narrower and mostly internal/advanced.
  - Menu gate update (as of 2026-03-09): `tools/gate_menu_action_default_surfaces.py` now protects the primary ui-gallery dropdown-menu / context-menu / menubar snippets plus overlay preview menu surfaces from drifting back to `.on_select(...)`, and `tools/pre_release.py` runs that gate with the rest of the teaching-surface policy suite.
  - Curated internal menu follow-up (as of 2026-03-09): `ecosystem/fret-workspace/src/tab_strip/overflow.rs` now uses `DropdownMenuItem::action(...)` / `trailing_action(...)`, `ecosystem/fret-genui-shadcn/src/resolver/overlay.rs` now lowers stable unit action ids through `DropdownMenuItem::action(...)`, and `tools/gate_menu_action_curated_internal_surfaces.py` keeps that explicit post-v1 residue slice from drifting back to `.on_select(...)`.
  - Intentional command-surface inventory update (as of 2026-03-09): `COMMAND_FIRST_INTENTIONAL_SURFACES.md` now marks command palette/catalog, `DataTable` business-table wiring, compat/conformance tests, and callback-style non-menu widgets as intentional retained surfaces rather than the next generic migration target, so this track is now in maintenance mode unless a new default-facing leak appears.
  - Current-vs-target v2 note (as of 2026-03-09): `V2_BEST_PRACTICE_GAP.md` now makes the next-stage framing explicit: v1 migration is effectively complete, action/menu residue is no longer the main work item, and the highest-value remaining gap is productization plus tracked-write/invalidation ergonomics.
  - `notify()` policy draft (as of 2026-03-09): `NOTIFY_POLICY_DECISION_DRAFT.md` now fixes the near-term direction for `AFA-postv1-004`: keep `notify()` as a low-level escape hatch, keep tracked writes as the boring default rerender path, and do not reopen generic invalidation helper design unless a new medium-surface contradiction appears.
  - `notify()` default-path gate update (as of 2026-03-09): `tools/gate_no_notify_in_default_teaching_surfaces.py` now keeps the default cookbook ladder plus scaffold templates off explicit `cx.notify(...)` / `host.notify(...)`, and `tools/pre_release.py` runs that policy alongside the other teaching-surface checks.
  - Richer `todo` template audit update (as of 2026-03-09): `apps/fretboard/src/scaffold/templates.rs` (`todo_template_main_rs`) remains intentionally explicit because the template is still teaching selector deps across nested row models plus query invalidation/filter coordination on one tracked graph; this is now documented in `docs/examples/todo-app-golden-path.md` and the generated template README so it does not look like an accidental lagging migration.
  - Productization ingress update (as of 2026-03-09): `DEFAULT_PATH_PRODUCTIZATION.md`, `README.md`, `docs/first-hour.md`, `docs/crate-usage-guide.md`, `docs/ui-ergonomics-and-interop.md`, `docs/examples/README.md`, `apps/fret-cookbook/README.md`, `apps/fret-cookbook/EXAMPLES.md`, `apps/fret-ui-gallery/README.md`, and `ecosystem/fret/README.md` now repeat the same default/comparison/advanced framing plus the same `hello` -> `simple-todo` -> `todo` ladder, reducing the remaining post-v1 work from taxonomy drift to keeping those ingress docs stable.
  - Medium-surface builder follow-up (as of 2026-03-09): `Alert::build(...)` plus `AlertAction::build(...)` now close one deliberately narrow alert-family seam in `ecosystem/fret-ui-shadcn/src/alert.rs`, and the first real surfaces (`form_basics`, `assets_reload_epoch_basics`, and the main ui-gallery alert snippets) now use that late-landing path instead of pre-collecting alert children.
  - Medium-surface builder follow-up (as of 2026-03-09): `ScrollArea::build(...)` now closes the next narrow runtime-owned seam in `ecosystem/fret-ui-shadcn/src/scroll_area.rs`, and the first real surfaces (`markdown_and_code_basics`, `async_playground_demo`, and the main ui-gallery scroll-area demo) now keep viewport children on the late-landing path without widening the helper surface.
  - Medium-surface builder follow-up (as of 2026-03-09): `FieldSet::build(...)`, `FieldGroup::build(...)`, and `Field::build(...)` now close the next dense form-layout seam in `ecosystem/fret-ui-shadcn/src/field.rs`, and the first real surfaces (`ui-gallery` field `input`, field `fieldset`, and form `demo`) now keep field-family children on the late-landing path instead of pre-collecting them.

---

## M0 — Decision gates locked (ADRs accepted)

Exit criteria:

- ADR 0307 and ADR 0308 exist as the canonical contract references (Status: Accepted).
- Keymap strategy (ActionId vs CommandId) is explicit and stable for v1.
- `docs/adr/README.md` jump table points to the new ADRs.

---

## M1 — Action system v1 landed (additive)

Exit criteria:

- Action IDs are stable and debuggable.
- UI can bind to actions without string parsing glue.
- Keymap can trigger actions and diagnostics can explain availability/dispatch outcomes.
- At least one palette/menu trigger path uses the same action dispatch pipeline (no divergence).

Notes:

- v1 may keep `ActionId` == `CommandId` to avoid schema churn; the key is the authoring surface and routing semantics.

---

## M2 — View runtime v1 landed (minimal, ecosystem-level)

Exit criteria:

- A minimal `View` + `ViewCx` exists with:
  - action handler table registration,
  - `notify()` dirty marking,
  - `use_state`, `use_selector`, `use_query` integration surfaces.
- At least one demo renders via the view runtime without MVU.
- At least one gate explains view rebuild reasons (notify vs observed deps vs inspection mode).

---

## M3 — Multi-frontend convergence (imui + GenUI alignment)

Exit criteria:

- imui can dispatch `ActionId` directly (no string commands required).
- GenUI action bindings align with action conventions and can trigger the same action handler surfaces.
- Diagnostics selectors still work across all frontends (stable `test_id` surfaces).

---

## M4 — Adoption (cookbook + gallery)

Exit criteria:

- At least 2 cookbook examples migrated and used as the “before/after” teaching baseline.
- At least 1 ui-gallery page/snippet migrated.
- Docs show a single golden path for new users.
- `fretboard` templates do not teach a conflicting default paradigm (updated or explicitly deferred).

---

## M5 — Editor-grade proof (docking/workspace integration)

Exit criteria:

- At least one editor-grade surface (workspace shell / docking) uses the new action-first routing where appropriate.
- Regression gates exist (tests + diag script) for action routing + availability under overlays/focus changes.

---

## M6 — Cleanup (delete legacy, keep it boring)

Exit criteria:

- Redundant/legacy APIs are removed or clearly quarantined as “legacy”.
- Templates default to action-first + view runtime patterns.
- No in-tree demo requires stringly command routing glue.
- `cargo nextest run` gates remain green.
- “Risk matrix” items (R1–R6) have explicit mitigations/gates or are explicitly deferred.

---

## Post-v1 milestones (proposed)

These milestones are intentionally outside the v1 closure, but define the safe path to reduce
long-term surface area without breaking downstream users.

### M7 — Payload actions (v2) decision + prototype

Exit criteria:

- A concrete contract exists for parameterized/payloaded actions, including determinism and
  diagnostics expectations.
  - ADR: `docs/adr/0312-payload-actions-v2.md`
- At least one in-tree demo migrates from MVU payload routing to payload actions (or an explicit
  alternative is adopted).
  - Demo: `apps/fret-cookbook/examples/payload_actions_basics.rs`
  - Gate: `tools/diag-scripts/cookbook/payload-actions-basics/cookbook-payload-actions-basics-remove.json`

### M8 — MVU deprecation window (warn + migrate)

Exit criteria:

- MVU’s long-term stance is decided (supported vs legacy-only) and reflected in docs/templates.
- If legacy-only: a deprecation window exists (warnings + migrations), and in-tree demos remain
  buildable while migrating.
- If removal is adopted: MVU is hard-deleted in-tree under M9.

### M9 - Hard delete legacy MVU (in-tree) - Met

Exit criteria:

- `LEGACY_MVU_INVENTORY.md` has no remaining in-tree MVU usage.
- `ecosystem/fret` no longer exposes MVU surfaces:
  - delete `mvu` + `mvu_router` + `legacy` modules,
  - remove MVU re-exports from `prelude::*`.
- `apps/fret-examples` and `apps/fret-demo` no longer have legacy MVU demo routing.
- Docs/templates do not mention MVU as an available authoring path.
- A small gate prevents MVU APIs from being reintroduced (grep-based check is sufficient).

Closure note (as of 2026-03-06):

- Exit criteria are met; MVU survives only as historical/external migration context in docs.
