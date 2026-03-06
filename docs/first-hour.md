# First Hour with Fret (Native) — A Boring, Repeatable Path

This guide is the “30–60 minute” onboarding path for a first-time Fret user.

Goals:

- Build and run a small native UI app quickly.
- Learn the minimum mental model needed to keep making progress.
- Avoid early concept overload (selectors/queries/advanced interop).

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

- It is **Model + view runtime + typed actions + keyed lists** only.
- It intentionally does **not** pull in `fret-selector` or `fret-query`.

## 2) Where to edit

Open:

- `local/my-simple-todo/src/main.rs`

The template is intentionally small:

- `TodoState` holds app-owned `Model<T>` state.
- `act::*` are typed unit actions (stable IDs).
- `TodoView` wires the view runtime (`init`, `render`) and starts with `cx.on_action_notify_models`, `cx.on_action_notify_transient`, plus local `on_activate*` only when widget glue truly needs it.
- Treat raw `on_action_notify` as cookbook/reference material for host-side integrations, not as the first-hour default.

## 3) The three things you should learn first

### A) Keyed identity for dynamic lists

When rendering a list that can insert/remove/reorder, use keys:

- Prefer `cx.keyed(id, |cx| ...)` for list rows.

Rule of thumb:

- If the list can change shape over time, assume it needs keys.

Why this matters:

- Keys make element identity stable across frames, so per-row state stays attached to the same item.
- Without keys, inserting/removing/reordering rows can cause state to “jump” between rows (focus, hover, local state, etc.).

Minimal pattern:

```rust
for item in items {
    cx.keyed(item.id, |cx| render_row(cx, item));
}
```

### B) The authoring dialect: `ui::*` constructors + `UiBuilder`

In the onboarding path, you will mostly author UI via `ui::*` constructors from `fret_ui_shadcn::prelude::*`.

Key points:

- `ui::*` constructors return `UiBuilder<T>` (a patchable builder surface).
- Apply layout/chrome refinement via fluent methods (`px_2()`, `gap(Space::N2)`, `rounded_md()`, ...).
- Convert into `AnyElement` at the boundary via `.into_element(cx)`.
- If you have a patchable component type (implements `UiPatchTarget`), you can opt into the same fluent
  authoring style with `.ui()`.
- Most `ui::*` layout constructors accept children as `UiIntoElement`, so you can pass `UiBuilder` values
  directly (use `ui::children![cx; ...]` for heterogeneous lists).

Minimal pattern:

```rust
let header = ui::h_flex(|_cx| [ui::text("Hello")])
    .gap(Space::N2)
    .px_3()
    .into_element(cx);
```

### C) Iterator helpers for child collection

If you already have an iterator that yields `AnyElement`, collect it with `.elements()`:

```rust
use fret_ui::element::AnyElementIterExt;

let rows = items
    .iter()
    .map(|it| cx.keyed(it.id, |cx| render_row(cx, it)))
    .elements();
```

If you run into iterator borrow pitfalls (because a closure needs `&mut cx`), prefer the `*_build`
constructors that collect into a sink:

```rust
let list = ui::v_flex_build(cx, |cx, out| {
    for it in items.iter() {
        out.push(cx.keyed(it.id, |cx| render_row(cx, it)));
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

When observing models (via `cx.watch_model(...)`):

| If the value affects… | Choose | Notes |
| --- | --- | --- |
| visuals only | `Paint` | default; cheapest |
| layout (size/flow/scroll extents) | `Layout` | safe when in doubt |
| hit regions only | `HitTest` | pointer-only changes without layout changes |

Examples:

```rust
let clicks = cx.watch_model(&models.clicks).paint().copied_or_default();
let label = cx.watch_model(&models.label).layout().cloned_or_default();
```

If you are unsure, start with `Layout` and tighten later.

## 4) Next steps (progressive disclosure ladder)

1) **Hello UI** (minimal): `cargo run -p fretboard -- new hello --name hello-world`
2) **Simple baseline**: `simple-todo` (this guide)
3) **Best-practice baseline**: `todo` (selectors + queries)
   - See: `docs/examples/todo-app-golden-path.md`
4) **Interop (Tier A embedding)**: viewport surfaces + input forwarding
   - See: `docs/ui-ergonomics-and-interop.md`
   - See: `ecosystem/fret/src/interop/embedded_viewport.rs`
5) **Examples index**: templates + cookbook + gallery + labs
   - See: `docs/examples/README.md`
   - Workstream notes: `docs/workstreams/example-suite-fearless-refactor-v1/design.md`

### Template matrix (what each scaffold teaches)

| Template | Generate | Teaches | Avoids |
| --- | --- | --- | --- |
| `hello` | `cargo run -p fretboard -- new hello` | view runtime + typed actions (smallest runnable UI surface) | selectors/queries |
| `simple-todo` | `cargo run -p fretboard -- new simple-todo` | view runtime + typed actions + keyed lists | selectors/queries |
| `todo` | `cargo run -p fretboard -- new todo` | “best practice baseline” with selectors/queries | being minimal |
