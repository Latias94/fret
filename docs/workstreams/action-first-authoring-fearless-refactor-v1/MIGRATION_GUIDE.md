# Action-First Authoring + View Runtime (Fearless Refactor v1) — Migration Guide

Last updated: 2026-03-03

This guide is intentionally practical: it describes how to migrate in-tree demos and ecosystem code
in small, reviewable slices.

---

## 1) Migration sequence (recommended)

1) Migrate **event identities** first (commands → actions).
2) Migrate authoring loop (MVU → view runtime) only after action IDs are stable.
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
   - v1: keep using the existing command dispatch pipeline (`on_command`) while we land the
     view/runtime-level handler table work.
   - later (post-M1/M2): converge toward `on_action` hooks backed by an action handler table.

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

## 3) MVU → View runtime (minimal refactor)

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

- `fretboard new hello` uses this pattern (View runtime + typed unit actions):
  `apps/fretboard/src/scaffold/templates.rs` (`hello_template_main_rs`)
- `fretboard new todo` extends the same pattern with selector/query hooks:
  `apps/fretboard/src/scaffold/templates.rs` (`todo_template_main_rs`)
- `fretboard new simple-todo` provides the smallest View+actions baseline:
  `apps/fretboard/src/scaffold/templates.rs` (`simple_todo_template_main_rs`)

UI gallery reference:

- `apps/fret-ui-gallery/src/ui/snippets/command/action_first_view.rs` (Action-first + view runtime snippet, used by the `Command` page).

Migration steps:

1) Move state into:
   - app-owned models (recommended for shared state), or
   - view-local state slots for simple demos.
2) Replace:
   - `msg.cmd(Msg::X)` with `act::X` action references.
3) Replace `update(...)` with `cx.on_action(...)` handlers.
4) Replace manual “force refresh” hacks with:
   - `cx.notify()` and/or
   - selector/query hooks that carry proper dependency observation.

---

## 4) imui alignment (imui widgets dispatch actions)

Target outcome:

- imui widgets can trigger the same action IDs as declarative UI.

Migration steps:

1) Add a `UiWriter` helper to emit an action trigger (no string glue).
2) Ensure imui outputs stable `test_id`/semantics for diag scripts.
3) Keep policy in ecosystem components, not in `fret-imui`.

---

## 5) GenUI alignment (spec bindings reuse action IDs)

Target outcome:

- GenUI specs and Rust UI can share action IDs and metadata where appropriate.

Migration steps:

1) Standardize action ID naming conventions (namespace + `.v1` suffix).
2) Expose action metadata to the GenUI inspector surfaces (optional v1).
3) Keep GenUI guardrails: do not allow specs to dispatch arbitrary actions without catalog approval.
