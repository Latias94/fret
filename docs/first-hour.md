# First Hour with Fret (Native) — A Boring, Repeatable Path

This guide is the “30–60 minute” onboarding path for a first-time Fret user.

Goals:

- Build and run a small native UI app quickly.
- Learn the minimum mental model needed to keep making progress.
- Avoid early concept overload (selectors/queries/advanced interop).

This guide is the **default** path, not the comparison or maintainer path.

Non-goals:

- Web/wasm setup (follow [docs/ui-ergonomics-and-interop.md](./ui-ergonomics-and-interop.md) and [apps/fret-demo-web](../apps/fret-demo-web)).
- Engine/editor-grade features (docking, multi-window, viewport interop) — those come later.

## 0) Prereqs

- Rust toolchain pinned by this repo (`rust-toolchain.toml`).
- A working desktop GPU stack (wgpu/Winit).
- For OS setup and faster local builds, see: [docs/setup.md](./setup.md).

## 1) Generate a runnable app (recommended: `simple-todo`)

In this repository:

```bash
cargo run -p fretboard -- new simple-todo --name my-simple-todo
cargo run --manifest-path local/my-simple-todo/Cargo.toml
```

If you prefer an in-tree example (no local scaffold), run the cookbook version:

```bash
cargo run -p fretboard -- dev native --example simple_todo
```

Why `simple-todo`?

- It is **LocalState + view runtime + typed actions + keyed lists** only.
- It intentionally does **not** pull in `fret-selector` or `fret-query`.
- It is the boring default path for a first real app, while `todo` is the richer follow-up and
  `simple_todo_v2_target` remains comparison-only.

## 2) Where to edit

Open:

- `local/my-simple-todo/src/main.rs`

The template is intentionally small:

- `TodoView` keeps view-owned draft text and keyed list state in `LocalState<T>` / `LocalState<Vec<_>>`.
- `act::*` are typed actions: unit actions for top-level intents and payload actions for per-row list interactions.
- `TodoView` wires the view runtime (`init`, `render`) and starts with `cx.actions().locals(...)`, keyed-row payload binding via `.action_payload(...)`, `payload_local_update_if::<A>(...)` as the default row-write path, `payload_locals::<A>(...)` only when one payload action coordinates multiple locals, `cx.actions().transient(...)` for App-only effects, and widget-local `.action(...)` / `.action_payload(...)` / `.listen(...)` when a control only exposes activation glue. Add `use fret::app::AppActivateExt as _;` explicitly for that bridge; the explicit `.dispatch::<A>()` / `.dispatch_payload::<A>(...)` aliases remain available when you want the type-directed wording. Drop down to `cx.actions().models(...)` when coordinating shared `Model<T>` graphs.
- Treat raw `on_action_notify` as cookbook/reference material for host-side integrations, not as the first-hour default.

Memorize the default app surface before you start editing:

- import from `use fret::app::prelude::*;`
- startup path: `FretApp::new("my-simple-todo").window("my-simple-todo", (...)).view::<TodoView>()?.run()`
- render signature: `fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui`
- grouped namespaces first: `cx.state()`, `cx.actions()`, `cx.data()`, `cx.effects()`
- if you intentionally need the raw model-backed hook, make that an advanced choice via
  `use fret::advanced::AppUiRawStateExt;`
- if you later graduate to the richer `todo` rung and need explicit selector/query nouns, add
  `fret::selector::{DepsBuilder, DepsSignature}` or `fret::query::{QueryKey, QueryPolicy, ...}`
  intentionally instead of expecting them from the default prelude

### Path taxonomy

- **Default**: `hello`, `simple-todo`, `todo`
- **Comparison**: `simple_todo_v2_target`
- **Advanced**: gallery, viewport/interop, docking, renderer-heavy demos

## 3) The three things you should learn first

### A) Keyed identity for dynamic lists

When rendering a list that can insert/remove/reorder, use keys:

- Prefer `ui::for_each_keyed(cx, items, |item| id, |item| row)` for list rows.

Rule of thumb:

- If the list can change shape over time, assume it needs keys.

Why this matters:

- Keys make element identity stable across frames, so per-row state stays attached to the same item.
- Without keys, inserting/removing/reordering rows can cause state to “jump” between rows (focus, hover, local state, etc.).

Minimal pattern:

```rust
let rows = ui::v_flex(|cx| {
    ui::for_each_keyed(cx, items.iter(), |item| item.id, |item| render_row(item))
});
```

If a row really needs the inner keyed child scope, use:

```rust
let rows = ui::v_flex(|cx| {
    ui::for_each_keyed_with_cx(cx, items.iter(), |item| item.id, |cx, item| {
        render_row_with_cx(cx, item)
    })
});
```

### B) The authoring dialect: one small default surface

In the onboarding path, stay on one small surface:

- `LocalState` for view-owned state
- typed actions for intent
- `cx.actions().locals(...)` for coordinated LocalState writes
- `.action_payload(...)` plus `payload_local_update_if::<A>(...)` for view-owned keyed-row interactions; use `payload_locals::<A>(...)` only when one payload action must coordinate multiple locals
- `cx.actions().transient(...)` only for App-bound effects
- widget-local `.action(...)` / `.action_payload(...)` / `.listen(...)` only when a control truly needs activation glue, with an explicit `use fret::app::AppActivateExt as _;`

This all lives on the default app import surface:

- `use fret::app::prelude::*;`
- `shadcn` and `ui` come from that prelude on the default app path

For UI composition, you will mostly author via `ui::*` constructors from that app prelude.

Key points:

- `ui::*` constructors return typed builder/child values that usually stay typed until a sink
  lands them.
- Apply layout/chrome refinement via fluent methods (`px_2()`, `gap(Space::N2)`, `rounded_md()`, ...).
- Prefer `ui::children![cx; ...]` for heterogeneous child groups.
- If a render root or wrapper closure only needs to late-land one typed child, prefer
  `ui::single(cx, child)` over `ui::children![cx; child].into()`.
- Treat explicit `.into_element(cx)` / `AnyElement` seams as advanced helper or interop boundaries,
  not as the first thing to memorize on the default path.
- If a local helper actually reads state, emits text/layout nodes, or otherwise needs runtime
  access, give it `cx: &mut UiCx<'_>`.
- If a local helper is only a pure page shell around already-typed children, prefer
  `fn page(...) -> impl UiChild` and let `render(...)` late-land it through
  `ui::single(cx, page(...))`.
- If you have a patchable component type (implements `UiPatchTarget`), you can opt into the same fluent
  authoring style with `.ui()`.
- Most `ui::*` layout constructors accept children through `IntoUiElement<H>`, so you can pass `UiBuilder` values
  directly (use `ui::children![cx; ...]` for heterogeneous lists).

Minimal pattern:

```rust
let header = ui::single(
    cx,
    ui::h_flex(|_cx| [ui::text("Hello")])
        .gap(Space::N2)
        .px_3(),
);
```

### C) Iterator helpers for child collection

For dynamic keyed lists, the default path is still the dedicated keyed helper:

```rust
let rows = ui::v_flex(|cx| {
    ui::for_each_keyed(cx, items.iter(), |item| item.id, |item| render_row(item))
});
```

If you already have an iterator that yields `AnyElement`, collect it with `.elements()`:

```rust
use fret_ui::element::AnyElementIterExt;

let rows = prebuilt_rows.into_iter().elements();
```

If you truly need manual sink-style collection, keep `*_build(...)` as an explicit advanced escape
hatch rather than the default keyed-list story:

```rust
use fret::children::UiElementSinkExt as _;

let list = ui::v_flex_build(|cx, out| {
    for it in items.iter() {
        out.push_ui(cx, expensive_manual_child(it));
    }
});
```

### D) Heterogeneous children without adapter noise

When building a list of children, prefer:

- `ui::children![cx; a, b, c]`

This is the intended “composition macro” surface: it keeps call sites readable while conversions
remain explicit at the ecosystem boundary.

### E) Invalidation rules of thumb

Fret uses explicit invalidation (this is a contract, not an optimization detail).

When observing tracked state in views:

| If the value affects… | Choose | Notes |
| --- | --- | --- |
| visuals only | `Paint` | default; cheapest |
| layout (size/flow/scroll extents) | `Layout` | safe when in doubt |
| hit regions only | `HitTest` | pointer-only changes without layout changes |

Keep the default path handle-first:

- for view-owned state, prefer `local.paint(cx)` / `local.layout(cx)` / `local.hit_test(cx)`
- when you intentionally drop to explicit shared handles on helper-heavy surfaces, keep the same
  handle-side shape (`model.paint_in(cx)` / `model.layout_in(cx)` / `model.hit_test_in(cx)`)

Examples:

```rust
let clicks = clicks_state.paint(cx).value_or_default();
let label = label_state.layout(cx).value_or_default();
```

If you are unsure, start with `Layout` and tighten later.

## 4) Next steps (progressive disclosure ladder)

1) **Hello UI** (minimal): `cargo run -p fretboard -- new hello --name hello-world`
2) **Simple baseline**: `simple-todo` (this guide)
3) **Best-practice baseline**: `todo` (selectors + queries)
   - See: `docs/examples/todo-app-golden-path.md`
4) **Comparison only**: `simple_todo_v2_target`
   - Use this only to compare authoring density or local-state/list tradeoffs against the default path.
   - Do not treat it as the main onboarding surface.
5) **Interop (Tier A embedding)**: viewport surfaces + input forwarding
   - See: `docs/ui-ergonomics-and-interop.md`
   - See: `ecosystem/fret/src/interop/embedded_viewport.rs`
6) **Examples index**: templates + cookbook + gallery + labs
   - See: `docs/examples/README.md`
   - Workstream notes: `docs/workstreams/example-suite-fearless-refactor-v1/design.md`

### Template matrix (what each scaffold teaches)

| Template | Generate | Teaches | Avoids |
| --- | --- | --- | --- |
| `hello` | `cargo run -p fretboard -- new hello` | view runtime + typed actions (smallest runnable UI surface) | selectors/queries |
| `simple-todo` | `cargo run -p fretboard -- new simple-todo` | view runtime + typed actions + keyed lists | selectors/queries |
| `todo` | `cargo run -p fretboard -- new todo` | best-practice baseline: selectors + queries + LocalState transactions | being minimal |
