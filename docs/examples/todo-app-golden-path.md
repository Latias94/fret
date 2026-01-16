# Todo App (Golden Path) — User View

This document shows what we want a first-time Fret user to write when building a small “Todo” app:

- a text input,
- a list of items with checkboxes,
- a couple of icons (add/remove),
- no direct knowledge of `winit`, `wgpu`, effect flushing, or runner internals.

It is intentionally “golden path”: advanced apps may assemble crates manually.

Related ADRs:

- Golden-path driver/pipelines: `docs/adr/0112-golden-path-ui-app-driver-and-pipelines.md`
- Ecosystem bootstrap and tooling: `docs/adr/0108-ecosystem-bootstrap-ui-assets-and-dev-tools.md`
- Dev hotpatch safety: `docs/adr/0107-dev-hotpatch-subsecond-and-hot-reload-safety.md`
- Resource handle boundary: `docs/adr/0004-resource-handles.md`

## Recommended dependencies (native)

- `fret-kit` (desktop-first batteries-included entry point)
  - wraps `fret-bootstrap` (golden-path wiring) + `fret-ui-shadcn` (default component surface)
  - enables a practical desktop-first default stack via `fret/native-wgpu`

## Quick start (template)

If you are working inside this repository, you can generate a runnable todo template:

```bash
fretboard new todo --name my-todo
cargo run --manifest-path local/my-todo/Cargo.toml
```

To enable UI render asset caches (images/SVG), add `--ui-assets`:

```bash
fretboard new todo --name my-todo --ui-assets
```

Notes:

- `fret-kit` defaults to a practical desktop setup (diagnostics + icons + optional caches).
- Advanced apps can depend on `fret` + `fret-bootstrap` directly for finer-grained control.

## Minimal `Cargo.toml`

This repo is not published to crates.io yet, so the examples below use workspace `path` dependencies.

```toml
[package]
name = "todo"
version = "0.1.0"
edition = "2024"

[dependencies]
anyhow = "1"
fret-kit = { path = "../../ecosystem/fret-kit" }
```

## Minimal startup

```rust,ignore
fn main() -> anyhow::Result<()> {
    fret_kit::app_with_hooks("todo", init_window, view, |d| d.on_command(on_command))?
        .with_main_window("todo", (560.0, 520.0))
        .run()?;

    Ok(())
}
```

Notes:

- `FnDriver` is the recommended authoring surface for Subsecond-style hotpatch (ADR 0107).
- `fret-kit::app_with_hooks` applies conservative defaults while keeping the underlying driver hotpatch-friendly.

## App state (models)

```rust,ignore
use std::sync::Arc;
use fret_runtime::Model;

#[derive(Clone)]
struct TodoItem {
    id: u64,
    done: bool,
    text: Arc<str>,
}

struct TodoWindowState {
    ui: fret_ui::UiTree<fret_app::App>,
    root: Option<fret_core::NodeId>,
    todos: Model<Vec<TodoItem>>,
    draft: Model<String>,
}
```

## Commands (UI → app logic)

We recommend keeping UI → app communication as command IDs:

```rust,ignore
use fret_runtime::CommandId;

const CMD_ADD: &str = "todo.add";
const CMD_CLEAR_DONE: &str = "todo.clear_done";
const CMD_TOGGLE_PREFIX: &str = "todo.toggle.";
const CMD_REMOVE_PREFIX: &str = "todo.remove.";

fn toggle_cmd(id: u64) -> CommandId {
    CommandId::new(format!("{CMD_TOGGLE_PREFIX}{id}"))
}

fn remove_cmd(id: u64) -> CommandId {
    CommandId::new(format!("{CMD_REMOVE_PREFIX}{id}"))
}
```

This avoids storing long-lived closures at arbitrary nodes and keeps hot reload resets predictable (ADR 0107).

## View (build retained UI tree)

This is high-level pseudocode showing the intent; exact component APIs may vary.

```rust,ignore
use fret_kit::prelude::*;

fn view(cx: &mut ElementContext<'_, fret_app::App>, st: &mut TodoWindowState) -> Vec<AnyElement> {
    let add_icon = IconId::new("lucide.plus");
    let trash_icon = IconId::new("lucide.trash-2");

    // 1) Input row: text field + add button
    // 2) List: checkbox + label + remove button per item
    // 3) Footer: “clear done”
    vec![todo_root(cx, st, add_icon, trash_icon)]
}
```

Note: `fret-kit::prelude` includes the shadcn authoring vocabulary (layout/styling + common types) so app code can stay
on a single dependency for the default story.

## Event pipeline (platform → UI)

In a typical window driver:

- deliver `Event` to the UI tree first (focus, text input, overlays, etc),
- then apply any app-specific event handling.

## Command handler (app logic)

Commands are the boundary where you mutate models and emit effects:

```rust,ignore
use fret_runtime::CommandId;

fn on_command(
    app: &mut fret_app::App,
    services: &mut dyn fret_core::UiServices,
    window: fret_core::AppWindowId,
    ui: &mut fret_ui::UiTree<fret_app::App>,
    state: &mut TodoWindowState,
    cmd: CommandId,
) {
    // App commands: update models.
    match cmd.as_str() {
        CMD_ADD => { /* read draft, push todo, clear draft */ }
        CMD_CLEAR_DONE => { /* retain only !done */ }
        other => {
            if let Some(id) = other.strip_prefix(CMD_TOGGLE_PREFIX) { /* toggle */ }
            if let Some(id) = other.strip_prefix(CMD_REMOVE_PREFIX) { /* remove */ }
        }
    }
}
```

## Async / background work (two patterns)

For apps that need background work (I/O, indexing, etc), we recommend:

1) **Inbox + timer/RAF** (portable):
   - background thread sends messages to a channel,
   - main thread polls the inbox via timers and applies updates to models.

2) **External runtime (Tokio) + message channel** (heavy editor):
   - run a separate runtime thread,
   - send pure data messages to UI thread,
   - UI thread applies updates to models and requests redraw.

See ADR 0112 for rationale and constraints.

## Hotpatch (Subsecond) integration

When using hotpatch (ADR 0107):

- prefer `FnDriver` entry points (function pointers),
- treat action hook registries and overlay registries as disposable,
- use the runner’s hot reload hooks to reset retained UI state on patch applied.

## Asset caches (images / SVGs)

If you want UI render asset conveniences (not an editor/project asset pipeline):

- `fret-kit` enables UI asset caches by default; disable via features if you want a smaller build.
- Optionally call `.with_ui_assets_budgets(...)` (on the returned builder) to override budgets.
- If you want to call cache APIs directly (stats, keyed helpers), add an explicit dependency on
  `fret-ui-assets` and enable its `app-integration` feature.

See the runnable demo: `apps/fret-demo/src/bin/assets_demo.rs`.

## Icon packs (Lucide / Radix / custom)

Recommended for apps:

- `fret-kit` enables Lucide by default. To change packs, configure `fret-kit` features:
  - enable `fret-kit/icons-lucide` (default), or
  - enable `fret-kit/icons-radix`.

If you need a custom pack, call `.register_icon_pack(...)` with your own `fn(&mut IconRegistry)` implementation.
