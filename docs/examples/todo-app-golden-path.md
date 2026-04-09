# Todo App (Golden Path) — User View

This document shows what we want a first-time Fret user to write when building a small “Todo” app:

- a text input,
- a list of items with checkboxes,
- a couple of icons (add/remove),
- no direct knowledge of `winit`, `wgpu`, effect flushing, or runner internals.

It is intentionally “golden path”: advanced apps may assemble crates manually.
This document teaches one default path only: LocalState-first app code on the `fret::app`
surface. Explicit raw `Model<T>` handles stay on the advanced lane.

Taxonomy:

- **Default** follow-up: this document is the richer third rung (`todo`) after `hello` and
  `simple-todo`.
- **Comparison**: `simple_todo_v2_target` remains a side-by-side evaluation surface, not the
  onboarding default.
- **Advanced**: gallery/interop/renderer/docking surfaces are outside this document's scope.

## Onboarding ladder (progressive disclosure)

Prefer an explicit ladder instead of starting with the full baseline on minute 1:

1. `hello` — the smallest runnable “Hello UI”
2. `simple-todo` - **View runtime + typed actions + keyed lists** (no selectors/queries; current practical shape is `LocalState<Vec<_>>` + payload row actions for view-owned lists)
3. `todo` — the richer third-rung product baseline once you need derived or async state, with
   selector/query slices kept visible but easy to delete

Templates (public/product spelling):

```bash
fretboard new hello --name hello-world
fretboard new simple-todo --name my-simple-todo
fretboard new todo --name my-todo
```

In this repository, the public-surface equivalent is `cargo run -p fretboard -- new ...`.
`cargo run -p fretboard-dev -- new ...` remains the repo-local maintainer variant and writes under
`local/` by default.

Maintainer comparison target (not the onboarding default):

- `cargo run -p fretboard-dev -- dev native --example simple_todo_v2_target`
- It remains useful as the smallest side-by-side comparison surface, but the same keyed-list
  direction now also ships in `apps/fret-examples/src/simple_todo_demo.rs`,
  `apps/fret-examples/src/todo_demo.rs`, and the `fretboard` simple-todo scaffold. Its value is
  comparison, not proving the default path is still missing.

Related ADRs:

- Golden-path driver/pipelines: `docs/adr/0110-golden-path-ui-app-driver-and-pipelines.md`
- Ecosystem bootstrap and tooling: `docs/adr/0106-ecosystem-bootstrap-ui-assets-and-dev-tools.md`
- Resource handle boundary: `docs/adr/0004-resource-handles.md`

## Recommended dependencies (native)

- `fret` (desktop-first batteries-included entry point)
  - wraps `fret-bootstrap` (golden-path wiring) + `fret-ui-shadcn` (default component surface)
  - enables a practical desktop-first default stack via `fret-framework/native-wgpu`
- Optional ecosystem helpers (recommended defaults):
  - `fret-selector` (memoized derived state)
  - `fret-query` (async resource state + caching, TanStack Query-like)

## Quick start (third-rung template)

If this is your first Fret app, stop at `simple-todo` first. Generate `todo` only when you
intentionally want selector/query on day 1.

Treat the generated `todo` scaffold as a product baseline first, not as the new minimum starter:
keep the extra selector/query seams only when they help the app you are actually building.

If you are working inside this repository and want the public product path, you can generate a
runnable todo template with:

```bash
cargo run -p fretboard -- new todo --name my-todo
cargo run --manifest-path my-todo/Cargo.toml
```

To enable UI render asset caches (images/SVG), add `--ui-assets`:

```bash
cargo run -p fretboard -- new todo --name my-todo --ui-assets
```

Repo-local maintainer mode remains available:

```bash
cargo run -p fretboard-dev -- new todo --name my-todo
cargo run --manifest-path local/my-todo/Cargo.toml
```

That scaffold now also creates an `assets/` directory and mounts it through
a checked-in `src/generated_assets.rs` stub, so app-owned bundle assets already have a default
logical lane plus a portable packaged path.

Notes:

- `fret` defaults to a practical desktop setup (`desktop` + `app`).
- Advanced apps can depend on `fret-framework` + `fret-bootstrap` directly for finer-grained control.

## Invalidation rules of thumb (keep it simple)

When observing tracked state in views:

- Visual-only changes → `Paint`
- Affects sizing/flow/scroll extents → `Layout`
- Affects hit regions only → `HitTest`

If you are unsure, start with `Layout` and tighten later.

## Identity rules of thumb (keyed lists)

Dynamic lists should use stable keys:

- Prefer `ui::for_each_keyed(cx, items, |item| id, |item| row)` for list rows.
- If a list can insert/remove/reorder, assume it needs keys.

## Minimal `Cargo.toml`

This repo is not published to crates.io yet, so the examples below use workspace `path` dependencies.

```toml
[package]
name = "todo"
version = "0.1.0"
edition = "2024"

[dependencies]
anyhow = "1"
fret = { path = "../../ecosystem/fret", features = ["state"] }

# Optional: depend on these directly only if you need their APIs outside of `AppUi`.
fret-selector = { path = "../../ecosystem/fret-selector", features = ["ui"], optional = true }
fret-query = { path = "../../ecosystem/fret-query", features = ["ui"], optional = true }
```

## Minimal startup

```rust,ignore
use fret::app::prelude::*;

fn main() -> anyhow::Result<()> {
    FretApp::new("todo")
        .window("todo", (560.0, 520.0))
        .view::<TodoView>()?
        .run()
        .map_err(anyhow::Error::from)
}
```

## Extending the entry (recommended seams)

The builder chain is ecosystem-level and intentionally provides a few stable seams for extending
apps without dropping down to `fret-bootstrap`:

```rust,ignore
use fret::app::prelude::*;

fn install_todo_app(app: &mut App) {
    // Register app-owned globals, commands, services, etc.
    // Example:
    // app.set_global(MyService::default());
}

 fn main() -> anyhow::Result<()> {
    FretApp::new("todo")
        .window("todo", (560.0, 520.0))
        .setup(install_todo_app)
        // Disable filesystem config loading for embedding/minimal builds:
        .config_files(false)
        // If you use images/SVG in UI, tune budgets:
        .ui_assets_budgets(64 * 1024 * 1024, 4096, 16 * 1024 * 1024, 4096)
        .view::<TodoView>()?
        .run()
        .map_err(anyhow::Error::from)
}
```

Notes:

- The action-first + view runtime path is the recommended golden path for new apps (ADRs 0307/0308).
- Keep one or two trivial local slots inline; once a view owns several related `LocalState<T>` slots, prefer a small `*Locals` bundle with `new(cx)` and optional `bind_actions(&self, cx)`. Inside that bundle, start with `cx.actions().locals_with((...)).on::<A>(|tx, (...)| ...)` for multi-slot transactions, use `cx.actions().transient(...)` for app-only effects, and keep widget-local `.action(...)` / `.action_payload(...)` / `.listen(...)` when a control only exposes activation glue. Add `use fret::app::AppActivateExt as _;` explicitly for that bridge. Drop down to `cx.actions().models(...)` when coordinating shared `Model<T>` graphs.
- In-tree MVU is removed; if you are migrating an older external MVU codebase, use the workstream migration guide as a mapping reference rather than treating MVU as a current option.
- Use typed unit actions for globally addressable intents and typed payload actions for per-item UI intents.

## App state (LocalState-first)

```rust,ignore
use std::sync::Arc;

mod act {
    fret::actions!([
        Add = "todo.todo.add.v1",
        ClearDone = "todo.todo.clear_done.v1",
        RefreshTip = "todo.todo.refresh_tip.v1",
        FilterAll = "todo.todo.filter_all.v1",
        FilterActive = "todo.todo.filter_active.v1",
        FilterCompleted = "todo.todo.filter_completed.v1"
    ]);
}

#[derive(Clone)]
struct TodoRow {
    id: u64,
    done: bool,
    text: Arc<str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TodoFilter {
    All,
    Active,
    Completed,
}

struct TodoView;
```


## Three-layer state split (recommended)

This section describes the richer third rung (`todo`) and the `fretboard new todo`
scaffold template.

The `simple-todo` template intentionally stops earlier (no selector/query).

Current status note (as of 2026-03-17): the `todo` scaffold is **LocalState-first** (view-owned
slots) and uses typed payload actions + keyed lists for per-row interaction, while still
showcasing selector and query hooks. On the current third rung, selector dependencies now stay on
the LocalState-first teaching path via `cx.data().selector_layout(...)`, so the default authoring
surface no longer teaches `clone_model()` or raw `DepsBuilder` choreography as the first selector
story.

Productization note (2026-04-02): the generated `todo` starter should now read like a deletable
product baseline first. Keep the selector/query slices visible, but treat them as secondary seams:
remove the query-backed focus note first if you do not need async state yet, then remove filters
and the derived projection if your first version only needs one plain task list.

The official baseline uses a 3-layer state split:

1. Local mutable state (`LocalState<T>`):
   - canonical source for user edits and UI interaction state (`draft`, `todos`, `filter`, `tip_nonce`) in this baseline.
2. Derived state (`fret-selector`):
   - memoized projections/counters/filtered views derived from tracked locals.
3. Async resource state (`fret-query`):
   - loading/error/success/cache lifecycle for remote or background resources.

Boundary rule:

- keep domain mutations in typed action handlers,
- keep selector/query as read-side helpers,
- pass plain values/snapshots into components whenever practical.
- prefer `LocalState<Vec<_>>` + payload actions for view-owned keyed lists; keep explicit `Model<T>` graphs for shared ownership or cross-view coordination.
- for non-payload multi-slot `LocalState<T>` coordination, prefer a small `*Locals` bundle with `new(cx)` and optional `bind_actions(&self, cx)`, then use `cx.actions().locals_with((...)).on::<A>(|tx, (...)| ...)` inside that bundle.
- for keyed-row payload writes, start with `cx.actions().local(&rows_state).payload_update_if::<A>(...)`.

## Actions (UI -> app logic)

Use typed unit actions with stable IDs as the default boundary between UI intents and app mutations:

- UI binds actions (button clicks, submit, menu items, keymap shortcuts) via `ActionId`.
- View runtime installs typed action handlers at a chosen root, keeping dispatch explainable and consistent.

High-level sketch:

```rust,ignore
use fret::app::prelude::*;

mod act {
    fret::actions!([Add = "todo.todo.add.v1"]);
}

impl View for TodoView {
    fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {
        let draft = cx.state().local::<String>();
        cx.actions().local(&draft).set::<act::Add>(String::new());

        ui::single(
            cx,
            shadcn::Button::new("Add")
                .action(act::Add),
        )
    }
}
```

## View (render a retained UI tree)

The view runtime renders the same declarative IR (`Ui`, backed by `Elements`) but provides a cohesive authoring loop:

- grouped app helpers (`state()`, `actions()`, `data()`, `effects()`),
- LocalState/query/selector helpers behind those grouped entrypoints,
- `notify → dirty → reuse` semantics via view cache roots.

Default helper rule on this path:

- keep `fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui` as the root signature,
- give a helper `&mut UiCx<'_>` only when the helper body actually needs runtime/context access,
- if a helper is only wrapping already-typed children into page chrome, prefer
  `fn page(...) -> impl UiChild` and late-land it from `render(...)` with
  `ui::single(cx, page(...))`.

If a product intentionally needs the raw model-backed hook, keep that on the explicit
`fret::advanced` lane rather than reintroducing it into the default todo authoring path.
Make that choice explicit:

```rust,ignore
use fret::advanced::AppUiRawModelExt;

let raw_model = cx.raw_model::<T>();
```

For the full runnable baseline, see the `fretboard new todo` scaffold template.

## Derived state (selectors)

For editor-style UIs, views often need derived values (counts, filters, projections). We recommend
memoizing these computations with selectors instead of:

- recomputing every frame, or
- introducing user-managed “tick models” to force refresh.

High-level sketch (matching the current third-rung scaffold):

```rust,ignore
let derived = cx
    .data()
    .selector_layout((&todos_state, &filter_state), |(todos, filter)| {
        compute(&todos, filter)
    });
```

## Async resource state (queries)

For async data (network, disk, indexing), we recommend `cx.data().query(...)` so the UI can observe
loading/error/success/cache state consistently. Internally this still rides on tracked query state,
but app code should stay handle-first.

High-level sketch:

```rust,ignore
use fret::query::{QueryKey, QueryPolicy};

let handle = cx.data().query(key, policy, move |token| fetch(token));
let state = handle.read_layout(cx);

let status_label = state.status.as_str();
let is_refreshing = state.is_refreshing();
```

To invalidate/refetch from app logic:

```rust,ignore
// If refetch is just a pure state projection, keep it on the LocalState-first lane
// (for example, bump a local nonce like `locals.tip_nonce`).
cx.actions()
    .local(&locals.tip_nonce)
    .update::<act::RefreshTip>(|value| {
        *value = value.saturating_add(1);
    });

// then include the nonce in the query key:
let tip_nonce_value = locals.tip_nonce.paint_value(cx);
let handle = cx.data().query(tip_key(tip_nonce_value), policy, move |token| fetch(token));
```

## Event pipeline (platform → UI)

In a typical window driver:

- deliver `Event` to the UI tree first (focus, text input, overlays, etc),
- then apply any app-specific event handling.

## Action handlers (logic)

In the view runtime shape, typed action handlers are the boundary where you mutate tracked state and
request UI updates. Keep one or two trivial locals inline, but once a view owns several related
slots, prefer bundling them into a small `*Locals` helper and install grouped handlers from there.
Within that bundle, use `cx.actions().locals_with((...)).on::<A>(|tx, (...)| ...)` for
LocalState-first flows, keep `cx.actions().local(&rows_state).payload_update_if::<A>(...)` as the
default keyed-row payload write path, drop to `cx.actions().models(...)` when you intentionally
coordinate explicit shared model graphs, use `cx.actions().transient(...)` when the real work must
happen with `&mut App` in `render()`, and keep raw `on_action_notify` plus lower-level
payload/model seams for cookbook/reference host-side cases only:

```rust,ignore
struct TodoLocals {
    draft: LocalState<String>,
    next_id: LocalState<u64>,
    todos: LocalState<Vec<TodoRow>>,
}

impl TodoLocals {
    fn bind_actions(&self, cx: &mut AppUi<'_, '_>) {
        cx.actions()
            .locals_with((&self.draft, &self.next_id, &self.todos))
            .on::<act::Add>(|tx, (draft, next_id, todos)| {
                let text = tx.value(&draft).trim().to_string();
                if text.is_empty() {
                    return false;
                }

                let id = tx.value(&next_id);
                let _ = tx.update(&next_id, |value| *value = value.saturating_add(1));

                if !tx.update(&todos, |rows| {
                    rows.insert(0, TodoRow {
                        id,
                        done: false,
                        text: Arc::from(text),
                    });
                }) {
                    return false;
                }

                tx.set(&draft, String::new())
            });

        cx.actions()
            .local(&self.todos)
            .payload_update_if::<act::Toggle>(|rows, id| {
                if let Some(row) = rows.iter_mut().find(|row| row.id == id) {
                    row.done = !row.done;
                    true
                } else {
                    false
                }
            });
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

See ADR 0110 for rationale and constraints.

## Asset caches (images / SVGs)

If you want UI render asset conveniences (not an editor/project asset pipeline):

- Enable `fret/ui-assets` (or scaffold with `fretboard new todo --ui-assets`) so the golden-path
  driver wires caches + budgets.
- Prefer generated `src/generated_assets.rs` modules plus `generated_assets::mount(builder)` for
  the default portable packaged lane; if startup needs one explicit development-vs-packaged
  contract, use `fret::assets::{AssetStartupPlan, AssetStartupMode}` with
  `FretApp::asset_startup(...)`.
- Keep file-backed development startup on `FretApp::asset_startup(...)` plus
  `AssetStartupPlan::development_dir(...)` only when you intentionally want a native/package-dev
  source on the builder path.
- Optionally call `.ui_assets_budgets(...)` on `FretApp` to override budgets.
- If you want to call cache APIs directly (stats, keyed helpers), add an explicit dependency on
  `fret-ui-assets` and enable its `app-integration` feature.

See the in-tree cookbook examples:

- `apps/fret-cookbook/examples/icons_and_assets_basics.rs`
- `apps/fret-cookbook/examples/assets_reload_epoch_basics.rs`

## Icon packs (Lucide / Radix / custom)

Recommended for apps:

- `fret` enables a default icon pack via `fret/icons` (Lucide).
- To use another pack, add it as an explicit dependency and install it via the entry seams:
  - `.setup(fret_icons_radix::app::install)`.

If you need a custom pack, expose the same app-facing seam from your own crate and call
`.setup(my_icons::app::install)`.

To bootstrap that custom crate from a local SVG directory, run
`fretboard icons import svg-dir --source ./icons --crate-name my-icons --vendor-namespace app`.

The same command is shown below as a copy/paste block:

```bash
fretboard icons import svg-dir --source ./icons --crate-name my-icons --vendor-namespace app
```

If your source already exists as a local Iconify collection snapshot, use
`fretboard icons import iconify-collection --source ./iconify/lucide.json --crate-name lucide-icons --vendor-namespace lucide`
instead of the SVG-directory command above.

If the pack should also publish semantic `ui.*` aliases, pass
`--semantic-aliases ./semantic-aliases.json` with a committed JSON file:

```json
{
  "schema_version": 1,
  "semantic_aliases": [
    { "semantic_id": "ui.search", "target_icon": "actions-search" }
  ]
}
```

`target_icon` should match the generated icon name.

If imported icons should keep authored colors by default, pass
`--presentation-defaults ./presentation-defaults.json` with a committed JSON file:

```json
{
  "schema_version": 1,
  "default_render_mode": "mask",
  "icon_overrides": [
    { "icon_name": "brand-logo", "render_mode": "original-colors" }
  ]
}
```

`icon_name` should also match the generated icon name. Unlisted icons use `default_render_mode`;
when it is omitted, generated packs default to `mask`.

That generated crate already emits `PACK_METADATA`, `PACK` / `VENDOR_PACK`, and explicit
`my_icons::app::install(...)` wiring.

If you are on `fret_bootstrap` directly, prefer an explicit pack contract such as
`.register_icon_pack_contract(my_icons::PACK)`.

If you intentionally need raw registry control, drop to
`fret_bootstrap::BootstrapBuilder::register_icon_pack(...)`.
