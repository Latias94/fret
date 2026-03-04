# Action-First Authoring + View Runtime (Fearless Refactor v1) — Migration Guide

Last updated: 2026-03-04

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
   - v1: prefer `on_action` hooks backed by the action handler table (authoring-level).
   - compat: keep `on_command` for legacy MVU demos while migrating incrementally.

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

- `cargo run -p fretboard -- new hello` uses this pattern (View runtime + typed unit actions):
  `apps/fretboard/src/scaffold/templates.rs` (`hello_template_main_rs`)
- `cargo run -p fretboard -- new todo` extends the same pattern with selector/query hooks:
  `apps/fretboard/src/scaffold/templates.rs` (`todo_template_main_rs`)
- `cargo run -p fretboard -- new simple-todo` provides the smallest View+actions baseline:
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
   - Tip: for most state-mutating handlers, prefer `cx.on_action_notify::<A>(...)` to request a
     redraw + notify automatically when `handled=true`.
4) Replace manual “force refresh” hacks with:
   - `cx.notify()` and/or
   - selector/query hooks that carry proper dependency observation.

Side effects that need `App` access (v1 note):

- Some operations (e.g. `fret-query` invalidation via `with_query_client`) require `&mut App`.
- View action handlers (`cx.on_action*`) run on a restricted host (`UiActionHost`) by design, so they
  should avoid direct `App`-only calls.

Recommended v1 pattern (schedule in handler, execute in `render()`):

- Preferred: use transient events (one-shot flags) to schedule work for the next render pass:
  - In the action handler: record a transient event (see `ViewCx::on_action_notify_transient`).
  - In `render()`: consume the transient flag (see `ViewCx::take_transient_on_action_root`) and
    apply the `App`-scoped effect.
- If you need payload/data (not just a boolean flag), use a small “pending effect” model value
  instead.

Example:

- `ecosystem/fret/src/view.rs` (`ViewCx::on_action_notify_transient`, `ViewCx::take_transient_on_action_root`).
- `apps/fret-examples/src/query_demo.rs` (uses transient events + `with_query_client`).
- `apps/fret-examples/src/query_async_tokio_demo.rs` (same, but with `use_query_async`).

---

## 3.1) When to use MVU vs View (v1 guidance)

Recommended default (new code):

- Use **View runtime + typed actions** for new templates, cookbook examples, and app code.

Keep MVU (compat / legacy) when:

- You need **per-item/payloaded** routing semantics that v1 typed actions do not support yet
  (v1 is intentionally *unit actions only*; see ADR 0307 “v1 decision snapshot”),
  and the payload actions v2 prototype (ADR 0312) is not sufficient for your use case.
- You are maintaining an existing MVU-based demo and migration would not add new evidence/gates.
- You are exploring authoring patterns quickly and want a minimal “single-file loop” while prototyping.

Prefer payload actions v2 (post-v1) when:

- You want action-first per-item dispatch without routers, and you can accept:
  - payload is pointer/programmatic-only (no keymap schema changes),
  - payload is transient/best-effort (pending store + TTL),
  - missing payload should be handled safely (recommended: treat as not handled).
- See: `docs/adr/0312-payload-actions-v2.md`
- Example: `apps/fret-cookbook/examples/payload_actions_basics.rs`

Policy note:

- MVU is legacy-only (compat), not a supported alternative golden path.
- See: `docs/workstreams/action-first-authoring-fearless-refactor-v1/MVU_POLICY.md`

If you choose MVU in 2026:

- Label it explicitly as legacy/compat in docs and avoid copy-pasting it into new templates.
- Prefer action-first IDs (`ActionId == CommandId` in v1) even inside MVU code where feasible, so
  keymap/palette/menus/diagnostics stay aligned.

### 3.2) Enabling legacy MVU surfaces (opt-in)

MVU is feature-gated and compile-time deprecated.

To use it (legacy demos only), opt in explicitly:

- Enable the `fret` feature `legacy-mvu` in your `Cargo.toml`.
- Import MVU through `fret::legacy::prelude::*` (do not rely on `fret::prelude::*`).
- Expect `deprecated` warnings; in in-tree legacy demo crates we typically add `#![allow(deprecated)]`.

Inventory:

- Track remaining in-tree MVU usage here:
  `docs/workstreams/action-first-authoring-fearless-refactor-v1/LEGACY_MVU_INVENTORY.md`

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
    .a11y_role(SemanticsRole::Button)
    .test_id("editor.save")
    .key_context("editor");

// Only convert to `AnyElement` at the end:
let el = button.into_element(cx);
```

Notes:

- `a11y_*` decorations are applied via layout-transparent `SemanticsDecoration` on `AnyElement`
  (no extra layout node required).
- `key_context(...)` participates in `when` expressions via `keyctx.*` (ADR 0022).

## 5) GenUI alignment (spec bindings reuse action IDs)

Target outcome:

- GenUI specs and Rust UI can share action IDs and metadata where appropriate.

Migration steps:

1) Standardize action ID naming conventions (namespace + `.v1` suffix).
2) Expose action metadata to the GenUI inspector surfaces (optional v1).
3) Keep GenUI guardrails: do not allow specs to dispatch arbitrary actions without catalog approval.
