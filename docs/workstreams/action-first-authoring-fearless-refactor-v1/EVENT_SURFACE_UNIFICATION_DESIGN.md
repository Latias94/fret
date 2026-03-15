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
2. **Dispatch that action from widget-local activation glue**
   - if a widget already has `.action(...)` / `.action_payload(...)`, prefer those direct action
     slots first,
   - use `.on_activate(cx.actions().dispatch::<act::Save>())` only for activation-only surfaces
     that do not already expose a stable action slot,
   - use `.on_activate(cx.actions().dispatch_payload::<act::ToggleTodo>(todo.id))` only when the
     widget-local surface is activation-only but the action still needs payload dispatch.
3. **Handle actions at the view/root layer**
   - `cx.actions().locals::<A>(...)`
   - `cx.actions().models::<A>(...)`
   - `cx.actions().payload::<A>().locals(...)`
   - `cx.actions().transient::<A>(...)`
4. **Use an explicit listener escape hatch for local imperative glue**
   - `.on_activate(cx.actions().listener(|host, acx| { ... }))`

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
    .on_activate(cx.actions().dispatch::<act::Save>());

widget_that_only_exposes_on_activate()
    .on_activate(cx.actions().dispatch_payload::<act::RemoveTodo>(todo.id));

shadcn::Button::new("Close")
    .on_activate(cx.actions().listener(|host, acx| {
        host.request_redraw(acx.window);
        host.notify(acx);
    }));
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
- widget-local activation via `cx.actions().dispatch/listener` only for activation-only surfaces,
- root/view handling via `cx.actions().locals/models/payload/transient`

Demote:

- raw `on_activate*` helper names to advanced/reference material.

### Phase 2.5 — Add only thin app-facing sugar where the activation-only seam still feels too hard

If first-party examples still show repeated activation-only boilerplate after the docs/template
rewrite, add a thin app-facing extension trait rather than another helper family.

Recommended shape:

```rust,ignore
shadcn::DrawerTrigger::build(...)
    .dispatch::<act::OpenPalette>(cx);

custom_canvas_hotspot(...)
    .listen(cx, |host, acx| {
        host.request_redraw(acx.window);
        host.notify(acx);
    });
```

Recommended trait boundary:

- app-facing extension trait in `ecosystem/fret`,
- implemented only for widgets/types that already expose `on_activate(...)`,
- powered internally by `cx.actions().dispatch(...)` / `dispatch_payload(...)` / `listener(...)`,
- kept off `crates/fret-ui` and off component-policy crates.

Non-goals for this thin sugar lane:

- do not replace `.action(...)` as the default for widgets that already have action slots,
- do not add a new family like `click`, `submit`, `select`, `listener_notify`, `listener_redraw`,
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
