# Action-First Authoring + View Runtime (Fearless Refactor v1) — Event Surface Unification

Status: in progress, post-v1 productization lane
Last updated: 2026-03-15

Related:

- `docs/workstreams/action-first-authoring-fearless-refactor-v1/V2_GOLDEN_PATH.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/POST_V1_AUTHORING_V2_PROPOSAL.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/TODO.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/COMMAND_FIRST_RETAINED_SEAMS_DECISION_DRAFT.md`

---

## Problem statement

The repo already has a coherent action pipeline, but the public event authoring story is still too
wide.

Today an app author can encounter several overlapping spellings for "user interaction causes work":

- widget action identity: `.action(...)`, `.action_payload(...)`
- widget-local callback glue: `.on_activate(...)`
- historical command-shaped naming: `.on_click(...)`, `.on_select(...)`
- root/view action registration: `cx.actions().locals/models/transient/payload/availability(...)`
- lower-level callback helpers: `on_activate(...)`, `on_activate_notify(...)`,
  `on_activate_request_redraw(...)`

The runtime/mechanism layer is not the main problem.
The real product-surface problem is that **the same author intent still maps to too many public
shapes**.

---

## Non-goals

This lane does **not** aim to:

- redesign the command routing/runtime dispatch mechanism,
- replace `ActionId == CommandId` in v1,
- rewrite `pressable_*` internals in `crates/fret-ui`,
- remove advanced/internal callback seams needed by component policy code,
- force command palette/catalog surfaces to stop being command-centric.

This is a public-surface/productization refactor, not a mechanism rewrite.

---

## Target mental model

The default app-facing event story should collapse to four concepts:

1. **Bind stable action identity on widgets**
   - `.action(act::Save)`
   - `.action_payload(todo.id)`
2. **Dispatch that action from widget-local activation sugar**
   - if a widget already has `.action(...)` / `.action_payload(...)`, prefer those direct action
     slots first,
   - use `widget.dispatch::<act::Save>(cx)` only for activation-only surfaces that do not already
     expose a stable action slot,
   - use `widget.dispatch_payload::<act::ToggleTodo>(cx, todo.id)` only when the widget-local
     surface is activation-only but the action still needs payload dispatch,
   - treat raw `.on_activate(...)` wired with `cx.actions().dispatch* / listener(...)` as the
     lower-level building block behind that sugar, not as the default teaching lane.
3. **Handle actions at the view/root layer**
   - `cx.actions().locals::<A>(...)`
   - `cx.actions().models::<A>(...)`
   - `cx.actions().payload::<A>().locals(...)`
   - `cx.actions().transient::<A>(...)`
4. **Use an explicit listener escape hatch for local imperative glue**
   - `widget.listen(cx, |host, acx| { ... })`

Everything else should read as advanced or retained seam.

---

## API direction

### Default-facing surface

Preferred widget-side authoring:

```rust,ignore
shadcn::Button::new("Save")
    .action(act::Save);

shadcn::Checkbox::from_checked(todo.done)
    .action(act::ToggleTodo)
    .action_payload(todo.id);

widget_that_only_exposes_on_activate()
    .dispatch::<act::Save>(cx);

widget_that_only_exposes_on_activate()
    .dispatch_payload::<act::RemoveTodo>(cx, todo.id);

shadcn::Button::new("Close")
    .listen(cx, |host, acx| {
        host.request_redraw(acx.window);
        host.notify(acx);
    });
```

Preferred root/view handling:

```rust,ignore
cx.actions().locals::<act::Add>(|tx| { ... });
cx.actions().models::<act::Save>(|models| { ... });
cx.actions().payload::<act::ToggleTodo>().locals(|tx, id| { ... });
cx.actions().transient::<act::Refresh>(TRANSIENT_REFRESH_KEY);
```

### Retained advanced seams

The following remain valid, but should not be the first-contact story:

- raw `AppUi::on_action(...)` / `on_action_notify(...)`
- `fret_ui_kit::on_activate*` helpers
- raw `Arc<dyn Fn(&mut dyn UiActionHost, ...)>`
- `pressable_on_activate(...)` and other `ElementContext`-level hooks
- command-catalog/menu internals that stay command-centric by design

---

## Sequencing

### Phase 1 — Add grouped widget-side glue under `cx.actions()`

Land:

- `cx.actions().dispatch::<A>()`
- `cx.actions().dispatch_payload::<A>(payload)`
- `cx.actions().listener(...)`

Goal:

- stop forcing app authors to reopen the raw `OnActivate` closure shape for common widget glue.

### Phase 2 — Re-teach the docs/templates

Move default docs toward:

- widget binding via `.action(...)` / `.action_payload(...)` whenever the widget already exposes a
  stable action slot,
- widget-local activation via `.dispatch::<A>(cx)` / `.dispatch_payload::<A>(cx, ...)` /
  `.listen(cx, ...)` only for activation-only surfaces,
- root/view handling via `cx.actions().locals/models/payload/transient`

Demote:

- raw `on_activate*` helper names and direct `.on_activate(cx.actions()....)` glue to
  advanced/reference material.

### Phase 2.5 — Land thin app-facing sugar for activation-only seams

Status (as of 2026-03-15): landed in `ecosystem/fret` as `fret::app::AppActivateSurface` plus the
blanket `AppActivateExt` methods imported by `fret::app::prelude::*`, with an explicit app-lane
import path available as `use fret::app::AppActivateExt as _;`.

Default shape:

```rust,ignore
activation_only_widget()
    .dispatch::<act::OpenPalette>(cx);

activation_only_widget()
    .listen(cx, |host, acx| {
        host.request_redraw(acx.window);
        host.notify(acx);
    });
```

Trait boundary:

- app-facing extension trait in `ecosystem/fret`,
- implemented only for widgets/types that already expose `on_activate(...)`,
- current first-party coverage intentionally stays narrow and real: `shadcn::Badge`,
  `shadcn::Button`, `shadcn::SidebarMenuButton`, `shadcn::extras::{BannerAction, BannerClose, Ticker}`,
  optional `fret_ui_ai::{ArtifactAction, ArtifactClose, CheckpointTrigger, ConfirmationAction, ConversationDownload, MessageAction, PromptInputButton, WebPreviewNavigationButton, WorkflowControlsButton}`,
  plus Material 3 `Card`, `DialogAction`, and `TopAppBarAction`,
- powered internally by `cx.actions().dispatch(...)` / `dispatch_payload(...)` / `listener(...)`,
- kept off `crates/fret-ui` and off component-policy crates.
- custom widgets join this lane by implementing `fret::app::AppActivateSurface` and forwarding
  their `on_activate(...)` slot.
- intentionally excluded: surfaces whose callback contract already carries extra domain data or a
  specialized `ActionCx`-only seam, such as `fret_ui_ai::Attachment` (`Arc<str>` id payload),
  `QueueItemAction`, `Test`, `FileTreeAction`, `Suggestion`, `MessageBranch`, and terminal/file-tree
  helper actions. Those remain component-owned typed callbacks rather than joining a second generic
  app-facing trait family.

Current first-party teaching evidence (as of 2026-03-15):

- selected UI Gallery activation-only snippets now import `fret::app::AppActivateExt as _;`,
- those snippets prefer `.listen(cx, |host, acx| { ... })` over reopening raw `.on_activate(...)`,
- `apps/fret-ui-gallery/tests/ui_authoring_surface_default_app.rs` locks that default teaching
  surface with `selected_activation_snippets_prefer_app_activate_listen`, including the primary
  `sonner/demo` snippet, the data-table pagination demos, `scroll_area/nested_scroll_routing`, and
  the AI `artifact_code_display` / `artifact_demo` / `chat_demo` / `checkpoint_demo` /
  `confirmation_demo` / `conversation_demo` / `message_usage` / `prompt_input_docs_demo` /
  `prompt_input_referenced_sources_demo` / `reasoning_demo` / `transcript_torture` /
  `web_preview_demo` / `workflow_controls_demo` / `workflow_node_graph_demo` / `message_demo` /
  `task_demo` / `persona_demo` snippets when `fret`'s optional `ui-ai` lane is enabled.

Non-goals for this thin sugar lane:

- do not replace `.action(...)` as the default for widgets that already have action slots,
- do not add a new family like `click`, `submit`, `select`, `listener_notify`, `listener_redraw`,
- do not introduce a parallel `AppActionCxSurface`-style trait family for custom callback
  signatures; keep payload/context-carrying widget contracts typed and component-local,
- do not flatten the grouped `cx.actions()` namespace back into another flat helper taxonomy.

### Phase 3 — Shrink command-shaped widget naming

Continue alias-first cleanup where default-facing widgets still teach command-shaped names.

This is now a narrow residue lane, not a repo-wide grep target.

### Phase 4 — Evaluate whether local pressable/component author seams also need grouped sugar

Only revisit if real component-author evidence says:

- `ElementContext`-level `pressable_on_activate(...)` authoring is still too noisy,
- and the retained low-level seam is leaking into first-party default-facing teaching surfaces.

---

## Guardrails

Do not add a large family of tiny event helpers all at once.

The promotion rule for new event helpers is:

1. at least two real demos/templates need the same shape,
2. the helper removes a real repeated seam rather than renaming one-off code,
3. the helper fits under the existing grouped `cx.actions()` story,
4. the helper does not reintroduce command-vs-action ambiguity.

That means this lane should prefer:

- `dispatch`
- `dispatch_payload`
- `listener`

over a broad new family like:

- `dispatch_if_enabled`
- `listener_notify`
- `listener_redraw`
- `listener_redraw_notify`
- `click`
- `select`
- `submit`

unless later evidence clearly shows they are needed.

---

## Current decision

Recommended current stance:

- keep the runtime dispatch mechanism unchanged,
- keep command/catalog and low-level pressable seams as retained advanced surfaces,
- move the default widget-side event story into `cx.actions()`,
- then update docs/templates before considering any broader event-helper expansion.
