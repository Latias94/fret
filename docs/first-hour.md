# First Hour with Fret (Native) — A Boring, Repeatable Path

This guide is the “30–60 minute” onboarding path for a first-time Fret user.

Goals:

- Build and run a small native UI app quickly.
- Learn the minimum mental model needed to keep making progress.
- Avoid early concept overload (selectors/queries/advanced interop).

Non-goals:

- Web/wasm setup (follow `docs/ui-ergonomics-and-interop.md` and `apps/fret-demo-web`).
- Engine/editor-grade features (docking, multi-window, viewport interop) — those come later.

## 0) Prereqs

- Rust toolchain pinned by this repo (`rust-toolchain.toml`).
- A working desktop GPU stack (wgpu/Winit).

## 1) Generate a runnable app (recommended: `simple-todo`)

In this repository:

```bash
cargo run -p fretboard -- new simple-todo --name my-simple-todo
cargo run --manifest-path local/my-simple-todo/Cargo.toml
```

Why `simple-todo`?

- It is **Model + MVU messages + keyed lists** only.
- It intentionally does **not** pull in `fret-selector` or `fret-query`.

## 2) Where to edit

Open:

- `local/my-simple-todo/src/main.rs`

The template is intentionally small:

- `TodoState` holds app-owned `Model<T>` state.
- `Msg` is the typed intent enum (UI → app logic).
- `TodoProgram` wires MVU (`init`, `update`, `view`).

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

### B) Heterogeneous children without adapter noise

When building a list of children, prefer:

- `ui::children![cx; a, b, c]`

This is the intended “composition macro” surface: it keeps call sites readable while conversions
remain explicit at the ecosystem boundary.

### C) Invalidation rules of thumb

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
2) **Simple MVU baseline**: `simple-todo` (this guide)
3) **Best-practice baseline**: `todo` (selectors + queries)
   - See: `docs/examples/todo-app-golden-path.md`
4) **Interop (Tier A embedding)**: viewport surfaces + input forwarding
   - See: `docs/ui-ergonomics-and-interop.md`
   - See: `ecosystem/fret/src/interop/embedded_viewport.rs`
