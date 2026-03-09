# Action-First Authoring + View Runtime (Fearless Refactor v1) — Migration Guide

Last updated: 2026-03-08

This guide is intentionally practical: it describes how to migrate in-tree demos and ecosystem code
in small, reviewable slices.

Inventory:

- Remaining teaching-surface `Model<T>` holders are classified in
  `docs/workstreams/action-first-authoring-fearless-refactor-v1/TEACHING_SURFACE_LOCAL_STATE_INVENTORY.md`.

Note:

- In-tree MVU has already been removed. The `MVU -> View runtime` section is retained only as a
  mapping guide for migrating older external codebases and as historical context for the completed
  M9 hard-delete path.

---

## 1) Migration sequence (recommended)

1) Migrate **event identities** first (commands → actions).
2) Migrate the authoring loop (legacy MVU → view runtime) only after action IDs are stable (if applicable).
3) Add/upgrade gates (tests + diag scripts) while migrating, not after.

Rationale:

- action-first is the cross-frontend convergence seam (declarative + imui + GenUI),
- view runtime is the “authoring density” win, but it builds on stable action semantics.

---

## 2) Commands → Actions (authoring-level refactor)

Target outcome:

- UI triggers and keybindings reference an `ActionId` (string-visible, stable).

Migration steps:

1) Introduce action IDs for the existing command IDs (prefer keeping the same string).
2) Update UI widgets to bind `.action(...)` rather than `.on_click(cmd_id)` where appropriate.
3) Update handler registration:
   - v1: prefer `on_action` hooks backed by the action handler table (authoring-level).
   - if migrating older code: keep existing command hooks temporarily, but treat them as compat-only.

### 2.1 Typed unit action IDs (recommended v1 authoring style)

Define typed unit actions with explicit stable ID strings:

```rust,ignore
mod act {
    fret::actions!([
        EditorSave = "app.editor.save.v1",
        WorkspaceTabClose = "workspace.tabs.close.v1",
    ]);
}
```

Bind a shadcn button to the action:

```rust,ignore
shadcn::Button::new("Save").action(act::EditorSave);
```

### 2.2 Register metadata + default keybindings (v1: via the command registry)

In v1, `ActionId == CommandId` and action metadata is published via the existing command registry
surface (see ADR 0307). This keeps keymap, command palette, menus, and diagnostics aligned.

Practical checklist for a demo / app:

1) Register command/action metadata first (title, category, scope, default keybindings).
2) Then install the default keybindings into the keymap (the app may have already installed
   defaults for previously-known commands during bootstrap).

Example:

```rust,ignore
fn install_commands(app: &mut App) {
    let cmd: CommandId = act::EditorSave.into();

    let meta = CommandMeta::new("Save")
        .with_category("Editor")
        .with_scope(CommandScope::Widget)
        .with_default_keybindings([DefaultKeybinding::single(
            PlatformFilter::All,
            KeyChord::new(KeyCode::KeyS, Modifiers { ctrl: true, ..Default::default() }),
        )]);

    app.commands_mut().register(cmd, meta);
    fret_app::install_command_default_keybindings_into_keymap(app);
}
```

---

## 3) Legacy MVU → View runtime (minimal refactor)

Target outcome:

- the “one file demo” does not need:
  - `MessageRouter`,
  - `enum Msg`,
  - the `update(msg)` boilerplate.

Recommended v1 entry point:

```rust,ignore
fn main() -> anyhow::Result<()> {
    FretApp::new("my-demo")
        .window("my-demo", (560.0, 360.0))
        .install_app(fret_cookbook::install_cookbook_defaults)
        .run_view::<MyView>()
        .map_err(anyhow::Error::from)
}
```

Template reference:

- `cargo run -p fretboard -- new hello` uses this pattern (View runtime + typed unit actions):
  `apps/fretboard/src/scaffold/templates.rs` (`hello_template_main_rs`)
- `cargo run -p fretboard -- new todo` extends the same pattern with selector/query hooks:
  `apps/fretboard/src/scaffold/templates.rs` (`todo_template_main_rs`)
- `cargo run -p fretboard -- new simple-todo` provides the smallest View+actions baseline:
  `apps/fretboard/src/scaffold/templates.rs` (`simple_todo_template_main_rs`)

UI gallery reference:

- `apps/fret-ui-gallery/src/ui/snippets/command/action_first_view.rs` (Action-first + view runtime snippet, used by the `Command` page).

Migration steps:

1) Choose state placement explicitly.
   - Shared state should stay in explicit models.
   - For view-local state, prefer `cx.use_local*` + `state.layout(cx).value_*` / `state.paint(cx).value_*` on the default post-v1 path.
   - For keyed dynamic collections that still allocate nested row models, keep the collection itself in an
     explicit `Model<T>` for now and move adjacent draft/filter/counter state to `use_local*`.
   - Use the teaching-surface inventory document to decide whether an example should migrate fully,
     migrate partially, or remain an explicit-model reference case.
   - Use `Input::new(&local_text)` / `Textarea::new(&local_text)` for text widgets on the post-v1
     local-state path.
   - Use `local.clone_model()` when an existing non-text widget constructor still expects `Model<T>`
     (for example `Switch::new(...)` or `DatePicker::new_controllable(...)`).
   - When a generic `ModelStore` closure still needs to read local state or compare revisions, prefer
     `local.read_in(models, ...)` / `local.revision_in(models)` over leaking `local.model()` back into
     the default path.
   - For query resources, prefer reading the returned handle from the handle side as well:
     `query_handle.layout(cx).value_or_else(QueryState::<T>::default)` instead of
     `cx.watch_model(query_handle.model())...` at the teaching surface.
   - `cx.use_state::<T>()` remains available when you intentionally want the raw `Model<T>` handle,
     but it is no longer the first teaching-surface recommendation.
2) Replace:
   - `msg.cmd(Msg::X)` with `act::X` action references.
3) Replace `update(...)` with `cx.on_action...` handlers.
   - Tip: for most state-mutating handlers, start with `cx.on_action_notify_models::<A>(|models| ...)`.
   - If the action is a straightforward write to one local handle, `cx.on_action_notify_local_set`,
     `cx.on_action_notify_local_update`, or `cx.on_action_notify_toggle_local_bool` are acceptable.
   - Stay on `cx.on_action_notify_models::<A>(...)` when command/keymap gating, form-style
     validation/reset flows, or broader coordination across multiple state slots is involved.
   - Use `cx.on_action_notify::<A>(...)` only for advanced host-only cases where the built-in
     model/transient shorthands do not fit.
4) Replace manual “force refresh” hacks with:
   - first-class tracked writes (`cx.on_action_notify_models::<A>(...)`,
     `cx.on_action_notify_local_*`, or `LocalState::update_action(...)`) when the change itself is
     the rerender reason,
   - `cx.notify()` only for imperative/cache-boundary invalidation that is not represented by a
     tracked write, and/or
   - selector/query hooks that carry proper dependency observation.

Helper layering for migration code:

### Default entrypoints (recommended mental model)

If you want the closest v1 equivalent to the GPUI/Zed authoring feel, start with only these three
entrypoints and treat the rest as convenience aliases:

1. `cx.on_action_notify_models::<A>(|models| ...)`
   - Default for most typed UI actions that mutate app/view models.
   - Think: ?normal button / shortcut / menu action?.
2. `cx.on_action_notify_transient::<A>(...)`
   - Default when the action must trigger an `App`-only effect in `render()`.
   - Think: query invalidation, driver interaction, effect scheduling.
3. `on_activate(...)` / `on_activate_notify(...)`
   - Default only for local pressable/widget glue that is not worth promoting to a typed action.
   - Think: small internal widget activation, close buttons, immediate-mode/imui local affordances.

Everything else (`on_action_notify_model_update`, `on_action_notify_model_set`,
`on_action_notify_toggle_bool`, `on_activate_request_redraw`, ...) should be treated as optional
shorthand, not as the first thing new users need to memorize.

### North-star vs landed v1

The original north-star discussion remains valid, but it is important to separate the landed v1
surface from the post-v1 density goals:

- Landed in v1: `View` + typed actions, `use_selector` / `use_query`, cx-less `ui::*` constructors,
  semantics/test IDs before `into_element(cx)`, and a narrowed default helper surface.
- Not yet the default story: plain-Rust local state, builder-only composition that removes most
  `ui::children!`, and widget-local `listener` / `dispatch` / `shortcut` sugar.
- Recommendation: migrate to the landed v1 surface first, then evaluate post-v1 ergonomics changes
  with side-by-side demo evidence rather than mixing them into the migration baseline.

### Late-landing child composition (v1 best practice)

Avoid forcing early `into_element(cx)` when the only reason is “I need a `Vec<AnyElement>` now”.
Prefer collecting children at `into_element(cx)` time:

- layout wrappers: prefer `ui::children![cx; ...]` inside `ui::{h_flex,v_flex}(|cx| ...)` closures.
- shadcn composites: prefer `*_::build(...)` variants when available:
  - `Card::build(...)`, `CardHeader::build(...)`, `CardContent::build(...)`
  - `Table::build(...)`, `TableRow::build(...)`, `TableCell::build(child)`
  - overlay triggers like `DialogTrigger::build(...)`, `SheetTrigger::build(...)`.

Example (card, late-landing):

```rust,ignore
let card = shadcn::Card::build(|cx, out| {
    out.push_ui(
        cx,
        shadcn::CardHeader::build(|cx, out| {
            out.push_ui(cx, shadcn::CardTitle::new("Title"));
            out.push_ui(cx, shadcn::CardDescription::new("Description"));
        }),
    );
    out.push_ui(
        cx,
        shadcn::CardContent::build(|_cx, out| {
            out.push(body);
        }),
    );
})
.ui()
.w_full()
.into_element(cx);
```

### Overlay composition note (Dialog/Sheet)

When composing shadcn overlays via `.compose()`, prefer `.content_with(|cx| ...)` when the content
needs to resolve scope-only affordances such as `DialogClose::from_scope()` / `SheetClose::from_scope()`.

This keeps authoring on the late-landing pipeline while allowing the close affordance to resolve
its `open` model from the active overlay scope.

Example:

```rust,ignore
use fret_ui_shadcn as shadcn;

shadcn::Dialog::new(open.clone())
    .compose()
    .trigger(shadcn::DialogTrigger::build(shadcn::Button::new("Open")))
    .content_with(|cx| {
        let close = shadcn::DialogClose::from_scope().into_element(cx);
        shadcn::DialogContent::new(vec![close]).into_element(cx)
    })
    .into_element(cx);
```

### Helper visibility policy (docs/templates)

- Default onboarding material should teach only the three entrypoints above.
- Keep raw `on_action` / `on_action_notify`, the single-model aliases, payload hooks, and
  redraw-oriented `on_activate_request_redraw*` helpers in advanced/reference material unless the
  example truly needs them.
- A helper should graduate into first-contact docs/templates only after it solves repeated noise
  across multiple real demos/templates, not a single local call site.

- Optional advanced shorthand for obviously single-model handlers (keep these out of first-contact teaching unless they are materially clearer):

```rust,ignore
let count = cx.use_state::<u32>();
cx.on_action_notify_model_update::<act::Click, u32>(count.clone(), |v| {
    *v = v.saturating_add(1);
});

let open = cx.use_state::<bool>();
cx.on_action_notify_toggle_bool::<act::TogglePanel>(open.clone());
```

- For common multi-model flows, prefer `on_action_notify_models::<A>(|models| ...)`:

```rust,ignore
cx.on_action_notify_models::<act::Add>({
    let todos = self.todos.clone();
    let draft = self.draft.clone();
    move |models| {
        let text = models.read(&draft, |v| v.trim().to_string()).ok().unwrap_or_default();
        if text.is_empty() {
            return false;
        }
        let _ = models.update(&todos, |todos| todos.push(text));
        let _ = models.update(&draft, String::clear);
        true
    }
});
```

Choosing the helper:

- Start with `on_action_notify_models` unless you have a strong reason not to.
- Use `on_action_notify_model_update` / `on_action_notify_model_set` / `on_action_notify_toggle_bool`
  only when the single-model shape is obviously clearer than the generic `models` transaction.
- Use `on_action_notify_transient` when the real work must happen with `&mut App` in `render()`.
- Use `on_action_notify` (or raw `on_action`) for advanced host-only cases (focus, timers, clipboard,
  custom effects) where the built-in shorthands do not fit.
  - Current intentional cookbook cases fall into four host-side categories:
    - `toast_basics`: imperative host integration (`Sonner` toast dispatch needs `UiActionHost` + window).
    - `router_basics` back/forward: router command availability sync on the host path.
    - `async_inbox_basics::Start`: background dispatcher/inbox scheduling plus wake integration.
    - `undo_basics::Undo` / `Redo`: history traversal plus an explicit RAF effect.
- Use `on_activate` / `on_activate_notify` for local pressable/widget glue, not as the default
  replacement for typed action handlers.

Side effects that need `App` access (v1 note):

- Some operations (e.g. `fret-query` invalidation via `with_query_client`) require `&mut App`.
- View action handlers (`cx.on_action*`) run on a restricted host (`UiActionHost`) by design, so they
  should avoid direct `App`-only calls.

Recommended v1 patterns:

- For event-like App effects, prefer transient events (one-shot flags) to schedule work for the next render pass:
  - In the action handler: record a transient event (see `ViewCx::on_action_notify_transient`).
  - In `render()`: consume the transient flag (see `ViewCx::take_transient_on_action_root`) and
    apply the `App`-scoped effect.
- If the App-only effect is a pure projection of model state, keep the action as a normal model
  transaction and synchronize the effect idempotently in `render()` from that state.
- If you need payload/data (not just a boolean flag), use a small `pending effect` model value
  instead.

Example:

- `ecosystem/fret/src/view.rs` (`ViewCx::on_action_notify_transient`, `ViewCx::take_transient_on_action_root`).
- `apps/fret-examples/src/query_demo.rs` (uses transient events + `with_query_client`).
- `apps/fret-examples/src/query_async_tokio_demo.rs` (same, but with `use_query_async`).
- `apps/fret-cookbook/examples/commands_keymap_basics.rs` (shows `use_local*` + `state.layout(cx)` / `state.paint(cx)` + `local.clone_model()` for command availability and switch widgets).
- `apps/fret-cookbook/examples/drop_shadow_basics.rs` (shows the same bridge on a pure toggle-only renderer demo, keeping `Switch::new(Model<bool>)` unchanged while removing view-held `Model<bool>` fields).
- `apps/fret-cookbook/examples/markdown_and_code_basics.rs` (extends the bridge to a mixed editor/render-options page: `Textarea` now accepts `&LocalState<String>` directly, while `ToggleGroup::single` and `Switch` still keep their existing model-centered widget contracts and the view itself now prefers `use_local*` / `state.layout(cx)` / `state.paint(cx)`).
- `apps/fret-cookbook/examples/assets_reload_epoch_basics.rs` (shows the same local-state path for a pure trigger counter while intentionally keeping the asset reload bump, redraw request, and RAF scheduling as render-time host/runtime effects).
- `apps/fret-cookbook/examples/virtual_list_basics.rs` (shows the first virtualization hybrid on the post-v1 path: the items collection and scroll handle stay explicit, while mode/toggle/jump controls move to `use_local*` / `state.layout(cx)` / `state.paint(cx)` and scroll/reorder coordination continues on `on_action_notify_models`; the jump `Input` now accepts `&LocalState<String>` directly).
- `apps/fret-cookbook/examples/theme_switching_basics.rs` (shows the same local-state path for a theme-selection surface: the selected scheme now lives in `use_local*`, while the actual theme application plus redraw/RAF sync intentionally stay render-time host effects).
- `apps/fret-cookbook/examples/icons_and_assets_basics.rs` (shows the same hybrid rule for asset demos: the reload bump counter now lives in local state, while the actual asset reload epoch bump plus redraw/RAF synchronization intentionally stay render-time host/runtime effects).
- `apps/fret-cookbook/examples/customv1_basics.rs` (shows the matching renderer/effect hybrid: `enabled` and `strength` now live in `use_local*`, while effect registration, capability checks, and effect-layer plumbing intentionally stay render-time/runtime-owned).
- `apps/fret-cookbook/examples/text_input_basics.rs` (shows the narrow text bridge: `Input::new(&LocalState<String>)` on the default path, while submit/clear gating stays on `on_action_notify_models`).
- `apps/fret-cookbook/examples/date_picker_basics.rs` (shows the same bridge for `DatePicker::new_controllable(...)` while keeping the component boundary unchanged).
- `apps/fret-cookbook/examples/form_basics.rs` (shows multi-field local-state reads plus generic `on_action_notify_models` coordination for validation/reset).
- `apps/fret-cookbook/examples/simple_todo.rs` (kept intentionally as the keyed-list explicit-model comparison/reference surface: local draft / `next_id`, explicit collection model, and keyed row identity).
- `apps/fret-examples/src/todo_demo.rs` (shows the default app-grade keyed-list path: `LocalState<Vec<_>>`, payload row actions, and snapshot checkbox rendering).
- `apps/fretboard/src/scaffold/templates.rs` (`simple_todo_template_main_rs` now matches that default keyed-list path for generated starter apps instead of mirroring the cookbook comparison split).
- `apps/fret-examples/src/async_playground_demo.rs` (theme mirrors `Model<bool>`; `render()` applies the theme when the value changes).

### Current authoring review notes

Based on the current template/demo pass (`hello_template_main_rs`, `hello_counter_demo`, `query_demo`):

- Prefer reading tracked state once near the top of `render()` into plain locals, then render from those locals below.
  - Good: collect `draft_value`, `filter_value`, `count`, `step_text`, or `panel_open` up front.
  - Prefer `state.layout(cx).value_*` / `state.paint(cx).value_*` for the default read path, and keep raw `watch(...)` only when you need custom invalidation, `observe()`, or `revision()` access.
- Prefer one `AnyElement` landing per composed subtree boundary.
  - Good: build `header`, `content`, `footer`, then land each section once with `.into_element(cx)`.
  - Use `ui::children![cx; ...]` to keep heterogeneous child lists readable before the final landing.
- For App-only effects, keep the action handler data-only and schedule the effect explicitly.
  - Good: `cx.on_action_notify_transient::<A>(...)` in the handler, then `take_transient_on_action_root(...)` + `with_query_client(...)` in `render()`.
  - Use a small pending-effect model when you need payload/data rather than a boolean transient.
- Do not add more tiny helper APIs until these patterns prove insufficient across another round of real demos/templates.

---

## 3.1) Per-item dispatch: payload actions v2

If you previously relied on MVU routers for per-item/payloaded routing, prefer payload actions v2
(ADR 0312) for pointer/programmatic dispatch:

- See: `docs/adr/0312-payload-actions-v2.md`
- Example: `apps/fret-cookbook/examples/payload_actions_basics.rs`

## 3.2) MVU removal status (M9 landed)

MVU authoring is no longer available in-tree.

Policy:

- Do not reintroduce MVU code paths into the repo.
- Use this guide as the mapping reference only when migrating an external MVU-based codebase.
- Prefer payload actions v2 plus the view runtime for any remaining per-item or payloaded dispatch needs.

Completed in-tree state:

- `ecosystem/fret` no longer exposes MVU modules or legacy re-exports.
- `apps/fret-examples`, `apps/fret-demo`, and scaffold templates no longer carry MVU demo routing.
- Guardrails prevent MVU from drifting back into code surfaces.
- New in-tree work should target `View` + typed unit/payload actions only; keep this guide as an
  external migration mapping reference.

Evidence anchors:

- `ecosystem/fret/src/view.rs` (current view-runtime authoring hooks)
- `ecosystem/fret/src/actions.rs` (unit + payload action macros/traits)
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/LEGACY_MVU_INVENTORY.md`
- `tools/gate_no_mvu_in_tree.py`
- `tools/gate_no_mvu_in_cookbook.py`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/TODO.md` (M9 closure checklist)

---

## 4) imui alignment (imui widgets dispatch actions)

Target outcome:

- imui widgets can trigger the same action IDs as declarative UI.

Migration steps:

1) Add a `UiWriter` helper to emit an action trigger (no string glue).
2) Ensure imui outputs stable `test_id`/semantics for diag scripts.
3) Keep policy in ecosystem components, not in `fret-imui`.

---

## 4.1) Stamping semantics/test IDs without early `into_element(cx)`

Target outcome:

- authoring code can attach `test_id` / a11y semantics decorations on *any* `UiIntoElement` value,
  without forcing an early `into_element(cx)` call.

Recommended pattern (ecosystem authoring surface, ADR 0160):

```rust,ignore
use fret_core::SemanticsRole;
use fret_ui_kit::prelude::*;

let button = shadcn::Button::new("Save")
    .action(act::EditorSave)
    .role(SemanticsRole::Button)
    .test_id("editor.save")
    .key_context("editor");

// Only convert to `AnyElement` at the end:
let el = button.into_element(cx);
```

Notes:

- `role(...)` is available on `UiBuilder<T>` and on `AnyElement` (after `into_element(cx)`).
  - For arbitrary `UiIntoElement` values, prefer `a11y_role(...)` / `a11y(...)`.
- `a11y_*` decorations are applied via layout-transparent `SemanticsDecoration` on `AnyElement`
  (no extra layout node required).
- `key_context(...)` participates in `when` expressions via `keyctx.*` (ADR 0022).

## 4.2) cx-less `fret-ui-kit::ui::*` constructors (authoring noise reduction)

Target outcome:

- stop threading an outer `cx` argument into `fret-ui-kit::ui::*` constructors when it is only used
  for type anchoring, not for logic.

Migration examples:

```rust,ignore
// Before (older signature; removed):
// ui::v_flex(cx, |cx| ui::children![cx; shadcn::Label::new("Title")])

// After:
ui::v_flex(|cx| ui::children![cx; shadcn::Label::new("Title")])
```

Notes:

- The closure still receives `cx`; this is where keyed elements, observation, and conversion to
  `AnyElement` happen.
- In rare cases where Rust cannot infer the host type from context (typically when a builder is
  stored in a `let` binding without an immediate `into_element(cx)` boundary), add an explicit
  host-type anchor. Preferred (less generic noise): annotate the closure argument type:
  - `ui::v_flex(|cx: &mut ElementContext<'_, App>| { ... })`
  Alternative (turbofish):
  - `ui::v_flex::<App, _, _>(|cx| { ... })`

## 4.3) Authoring “golden style” (recommended)

This is a style guide, not a contract, but it is the repo’s default teaching baseline.

- Prefer `ui::v_flex(|cx| ...)` / `ui::h_flex(|cx| ...)` (no outer `cx` argument).
- If you need a horizontal row that does not force `width: fill`, prefer `ui::h_row(|cx| ...)`.
- If you need a vertical stack that does not force `width: fill`, prefer `ui::v_stack(|cx| ...)`.
- Prefer `ui::children![cx; ...]` for heterogeneous child lists to avoid decorate-only early
  `into_element(cx)` calls.
- For old `stack::*` call sites, the mapping is typically:
  - `stack::v_flex(...)` → `ui::v_flex(...)` (forces `width: fill`)
  - `stack::v_stack(...)` → `ui::v_stack(...)` (does **not** force `width: fill`)
  - `stack::h_flex(...)` → `ui::h_flex(...)` (forces `width: fill`)
  - `stack::h_row(...)` → `ui::h_row(...)` (does **not** force `width: fill`)
  - `stack::container_vstack(...)` → `ui::container(...)` + `ui::v_stack(...)` (explicit composition)
    - Internal policy/helper code that still wants a sink-based composition path can use `container_vstack_build(...)` / `container_hstack_build(...)` in `ecosystem/fret-ui-shadcn::layout` to stay on the same late-landing child pipeline without rebuilding a temporary `Vec<AnyElement>`.
    - Example (closest behavior match):

```rust,ignore
// Before:
// stack::container_vstack(cx, props, stack_props, children)

// After (explicit composition):
ui::container(|cx| {
    vec![ui::v_stack(|_cx| children).layout(stack_layout)]
})
.layout(container_layout)
.into_element(cx);
```
- When rendering dynamic lists, prefer `*_build(|cx, out| out.push_ui(cx, ui::keyed(id, |cx| ...)))`
  so keyed identity stays stable without forcing an eager `AnyElement` landing inside the sink. Keep
  raw `cx.keyed(id, |cx| ...)` for direct element construction paths that are not already on the
  builder-first surface.
- For router outlets, prefer `RouterOutlet::into_element_by_leaf_ui(...)` / `into_element_ui(...)` when route cards or not-found panels are still builder-first values; keep `into_element_by_leaf(...)` / `into_element(...)` for compatibility paths that already materialize `AnyElement`s.
- For low-level raw container seams that still need explicit `ContainerProps`, prefer `ui::container_props(props, |cx| [...])` / `ui::container_props_build(props, |cx, out| ...)` so the child subtree can remain builder-first until the final host `cx.container(...)` boundary. Keep raw `cx.container(props, |_cx| [child])` for compatibility paths that already hold concrete `AnyElement`s or that intentionally bypass the authoring helpers.
- For table-like composite trees, prefer `Table::build(...)` / `TableHeader::build(...)` / `TableBody::build(...)` / `TableFooter::build(...)` / `TableRow::build(...)` when the children naturally come from loops or generated data; when the final cell child is itself a builder, prefer `TableCell::build(child)` over early `into_element(cx)` and keep `TableCell::new(child)` only for already-landed `AnyElement` values.
- For overlay trigger/anchor wrappers used in sink-based or direct late-landing paths, prefer `DialogTrigger::build(child)` / `SheetTrigger::build(child)` / `AlertDialogTrigger::build(child)` / `DrawerTrigger::build(child)` / `PopoverTrigger::build(child)` / `PopoverAnchor::build(child)` / `HoverCardTrigger::build(child)` / `HoverCardAnchor::build(child)` / `TooltipTrigger::build(child)` / `TooltipAnchor::build(child)` when the wrapped child is still a builder or `UiIntoElement`; keep `*_Trigger::new(child)` / `*_Anchor::new(child)` for already-landed `AnyElement` values. `Dialog::compose().trigger(...)` / `Sheet::compose().trigger(...)` / `AlertDialog::compose().trigger(...)` / `Drawer::compose().trigger(...)` now also accept those trigger build values directly, removing the old eager landing cliff from the composition surface. For anchor wrappers that need `element_id()` before final landing, use `*_AnchorBuild::into_anchor(cx)` or stay on the eager `*_Anchor::new(child)` path.
- For dialog / sheet content trees that still need nested builders or `push_ui(...)`, prefer `DialogContent::build(...)` / `DialogHeader::build(...)` / `DialogFooter::build(...)` and `SheetContent::build(...)` / `SheetHeader::build(...)` / `SheetFooter::build(...)` so the overlay body stays builder-first until the final root `into_element(cx)` boundary. Keep raw `DialogContent::new(...)` / `SheetContent::new(...)` for already-landed `AnyElement` children or compatibility paths that still build the overlay body eagerly.
- For alert-dialog / drawer content trees that still need nested builders or `push_ui(...)`, prefer `AlertDialogContent::build(...)` / `AlertDialogHeader::build(...)` / `AlertDialogFooter::build(...)` and `DrawerContent::build(...)` / `DrawerHeader::build(...)` / `DrawerFooter::build(...)` so those overlay sections stay builder-first until the existing alert-dialog / drawer root `into_element(cx)` boundary. Keep raw `AlertDialogContent::new(...)` / `DrawerContent::new(...)` for already-landed `AnyElement` children or compatibility paths that still build the overlay body eagerly.
- For alert-dialog / drawer root authoring, prefer `AlertDialog::build(cx, trigger, content)` / `Drawer::build(cx, trigger, content)` when the trigger and content are still builder-first values. Those helpers keep the old runtime boundary but remove the eager trigger/content closure shell from straightforward gallery/cookbook-style call sites.
- For popover / hover-card / tooltip root authoring, prefer the host-bound late-landing constructors `Popover::build(cx, trigger, content)` / `HoverCard::build(cx, trigger, content)` / `HoverCard::build_controllable(cx, open, default_open, trigger, content)` / `Tooltip::build(cx, trigger, content)` when the trigger or content is still a builder-first value. `PopoverContent::test_id(...)` and `Tooltip::new(trigger, content)` now both cross that root boundary without forcing an early `into_element(cx)` just to attach semantics or diagnostics hooks.
- For dropdown-menu root / parts authoring, prefer `DropdownMenu::build(cx, trigger, entries)` for direct trigger wiring and `DropdownMenu::build_parts(cx, DropdownMenuTrigger::build(child), DropdownMenuContent::new()..., entries)` for the shadcn part surface. Keep `DropdownMenuTrigger::new(child)` / `into_element_parts(...)` only for already-landed `AnyElement` triggers or compatibility call sites that still need the old trigger closure shape.
- Attach `test_id` / `a11y_*` / `key_context` on builders before `into_element(cx)` whenever the
  surrounding sink accepts `UiIntoElement`; if the sink/root still requires a concrete `AnyElement`
  (for example `Vec<AnyElement>::push(...)` or a widget-host boundary), land exactly there instead of
  introducing extra adapters.
- Keep the teaching surfaces consistent: the repo gates forbid `stack::*` authoring helpers in
  cookbook/examples (and the UI gallery shell):
  - `tools/gate_no_stack_in_cookbook.py`
  - `tools/gate_no_stack_in_examples.py`
  - `tools/gate_no_stack_in_ui_gallery_shell.py` (preview pages migrate in batches)
- Legacy stack helpers are hard-deleted from `fret-ui-kit` and gated to prevent regressions.
  - Gate: `tools/gate_no_public_stack_in_ui_kit.py`
- If host type inference fails, first try annotating the closure argument type
  (`|cx: &mut ElementContext<'_, App>| ...`) before reaching for turbofish.

## 5) GenUI alignment (spec bindings reuse action IDs)

Target outcome:

- GenUI specs and Rust UI can share action IDs and metadata where appropriate.

Migration steps:

1) Standardize action ID naming conventions (namespace + `.v1` suffix).
2) Expose action metadata to the GenUI inspector surfaces (optional v1).
3) Keep GenUI guardrails: do not allow specs to dispatch arbitrary actions without catalog approval.

---

## 6) Embedded viewport interop (advanced)

This applies to demos/apps that embed an `EmbeddedViewportSurface` and need a custom per-frame
engine recording hook.

Key constraint:

- `UiAppDriver` supports a single `record_engine_frame(...)` hook.
  - View runtime installs a hook today (v1) to enable the view cache on the `UiTree`.
    - See: `ecosystem/fret/src/app_entry.rs` (`App::view::<V>()`)
    - See: `ecosystem/fret/src/view.rs` (`view_record_engine_frame`)
  - Embedded viewport interop installs a hook to record the engine/offscreen pass.
    - See: `ecosystem/fret/src/interop/embedded_viewport.rs` (`EmbeddedViewportUiAppDriverExt`)

Recommended migration pattern:

1) Keep `viewport_input(handle_viewport_input)` installed (embedded viewport input forwarding).
2) Install a *composed* `record_engine_frame(...)` that performs both responsibilities:
   - ensure view-cache enablement (view runtime v1 behavior), and
   - record the embedded viewport engine pass.
3) Add a scripted diagnostics gate that proves the composition works end-to-end (pointer input +
   engine recording + view-cache tracing).
