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

- `fret` (facade; portable core re-exports)
- `fret-ui-shadcn` (components) or `fret-ui-kit` (lower-level building blocks)
- `fret-bootstrap` (recommended; startup wiring, enable the `ui-app-driver` feature)
- `fret-ui-assets` (optional): UI render asset caches (images / SVGs)
- `fret-icons-lucide` (icon pack data)

Notes:

- `fret-bootstrap` features:
  - `ui-app-driver`: enables `UiAppDriver`.
  - `ui-assets`: drives `fret-ui-assets` caches from the event pipeline (recommended if you load images/SVGs).
  - `preload-icon-svgs`: enables `preload_icon_svgs_on_gpu_ready`.

## Minimal `Cargo.toml`

This repo is not published to crates.io yet, so the examples below use workspace `path` dependencies.

```toml
[package]
name = "todo"
version = "0.1.0"
edition = "2024"

[dependencies]
anyhow = "1"
fret-app = { path = "../../crates/fret-app" }
fret-bootstrap = { path = "../../ecosystem/fret-bootstrap", features = ["ui-app-driver", "preload-icon-svgs"] }
fret-ui-shadcn = { path = "../../ecosystem/fret-ui-shadcn" }
fret-icons-lucide = { path = "../../ecosystem/fret-icons-lucide" }
```

If you want images/SVG caches out-of-the-box, enable `fret-bootstrap/ui-assets`:

```toml
fret-bootstrap = { path = "../../ecosystem/fret-bootstrap", features = ["ui-app-driver", "preload-icon-svgs", "ui-assets"] }
fret-ui-assets = { path = "../../ecosystem/fret-ui-assets" }
```

## Minimal startup

```rust,ignore
use fret_app::App;
use fret_ui_shadcn::shadcn_themes::{ShadcnBaseColor, ShadcnColorScheme};

fn main() -> anyhow::Result<()> {
    fret_bootstrap::ui_app_with_hooks("todo", init_window, view, |d| d.on_command(on_command))
        .with_default_settings_json()?
        .init_app(|app| {
            // Optional: apply a built-in shadcn theme preset for a “new-york-v4” look.
            fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
                app,
                ShadcnBaseColor::Slate,
                ShadcnColorScheme::Light,
            );
        })
        .register_icon_pack(fret_icons_lucide::register_icons)
        .preload_icon_svgs_on_gpu_ready()
        .run()?;

    Ok(())
}
```

Notes:

- `FnDriver` is the recommended authoring surface for Subsecond-style hotpatch (ADR 0107). `ui_app_with_hooks` wraps the
  boilerplate while keeping the underlying driver hotpatch-friendly.
- Icons are data-only (`IconRegistry`); rendering remains in the renderer layer.
- `UiAppDriver` closes the window by default on `Event::WindowCloseRequested` (clicking the window X).
  Disable this via `UiAppDriver::close_on_window_close_requested(false)` if you need an unsaved-changes prompt.
- If you enable the `fret-bootstrap` feature `ui-assets`, `UiAppDriver` will also drive `fret-ui-assets` caches from
  the event pipeline so `ImageAssetCache` works without additional boilerplate.

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
use fret_icons::IconId;
use fret_ui::AnyElement;
use fret_ui::ElementContext;
use fret_ui_shadcn as shadcn;

fn view(cx: &mut ElementContext<'_, fret_app::App>, st: &mut TodoWindowState) -> Vec<AnyElement> {
    let add_icon = IconId::new("lucide.plus");
    let trash_icon = IconId::new("lucide.trash-2");

    // 1) Input row: text field + add button
    // 2) List: checkbox + label + remove button per item
    // 3) Footer: “clear done”
    vec![todo_root(cx, st, add_icon, trash_icon)]
}
```

Note: `fret-ui-shadcn` re-exports common declarative authoring helpers (stack/style/icon) from `fret-ui-kit` so app code
can stay on `fret-bootstrap` + `fret-ui-shadcn` for the default story.

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

If you want UI render asset conveniences (not an editor/project asset pipeline), enable the bootstrap `ui-assets` feature.
It wires `ImageAssetCache` / `SvgAssetCache` as globals and (via `UiAppDriver`) drives the image cache from the event
pipeline.

See the runnable demo: `apps/fret-demo/src/bin/assets_demo.rs`.
