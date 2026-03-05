# Action-First Authoring + View Runtime (Fearless Refactor v1) — Migration Guide

Last updated: 2026-03-05

This guide is intentionally practical: it describes how to migrate in-tree demos and ecosystem code
in small, reviewable slices.

Note:

- This repo no longer ships MVU authoring surfaces in-tree (M9 hard delete). The “MVU → View runtime”
  section is retained only as a mapping guide for migrating older external codebases.

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

## 3.1) Per-item dispatch: payload actions v2

If you previously relied on MVU routers for per-item/payloaded routing, prefer payload actions v2
(ADR 0312) for pointer/programmatic dispatch:

- See: `docs/adr/0312-payload-actions-v2.md`
- Example: `apps/fret-cookbook/examples/payload_actions_basics.rs`

## 3.2) MVU removal note

MVU authoring surfaces were hard-deleted in-tree as part of milestone M9.

- Do not add MVU back into this repo.
- If you are migrating an external MVU-based codebase, treat this guide as the mapping reference and
  prefer converging on View runtime + typed actions.

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

## 4.2) cx-less `fret-ui-kit::ui::*` constructors (authoring noise reduction)

Target outcome:

- stop threading an outer `cx` argument into `fret-ui-kit::ui::*` constructors when it is only used
  for type anchoring, not for logic.

Migration examples:

```rust,ignore
// Before:
ui::v_flex(cx, |cx| ui::children![cx; shadcn::Label::new("Title")])

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
- Prefer `ui::children![cx; ...]` for heterogeneous child lists to avoid decorate-only early
  `into_element(cx)` calls.
- When rendering dynamic lists, prefer `*_build(|cx, out| ...)` + `cx.keyed(id, |cx| ...)` to keep
  identity stable and reduce allocation noise.
- Attach `test_id` / `a11y_*` / `key_context` on builders before `into_element(cx)`; only land to
  `AnyElement` at the end of a subtree boundary.
- Keep the teaching surfaces consistent: the repo gates forbid `stack::*` authoring helpers in
  cookbook/examples (and the UI gallery shell):
  - `tools/gate_no_stack_in_cookbook.ps1`
  - `tools/gate_no_stack_in_examples.ps1`
  - `tools/gate_no_stack_in_ui_gallery_shell.ps1` (shell-only; preview pages migrate in batches)
- Keep legacy stack helpers internal-only: `fret-ui-kit::declarative::stack` is not part of the
  public API and is scheduled for hard deletion after internal migrations.
  - Gate: `tools/gate_no_public_stack_in_ui_kit.ps1`
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
