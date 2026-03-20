# Fearless Refactoring (App Authors)

Fret is a documentation/ADR-driven project. The goal is to lock in *hard-to-change* editor-grade UI
contracts early (multi-window, docking, viewports, GPU-layered rendering), so we can iterate on
ergonomics and component surfaces without forcing late rewrites.

This document is a user-facing guide for writing Fret apps that remain easy to upgrade as the
project evolves.

## TL;DR

- Prefer **ecosystem entry points** (`fret`, `fret-ui-shadcn`, `fret-ui-kit`) for app code.
- Treat `crates/*` runtime internals as **mechanism-only** and subject to reshaping.
- When a breaking refactor is necessary, expect it to be tracked by a roadmap doc + supported by
  templates/demos so upgrades are “compile-guided”.

Related workstream examples:

- Foreground style context refactor: `docs/workstreams/foreground-style-context-fearless-refactor-v1/DESIGN.md`

## Stability boundaries (what is safe to depend on)

### Stable entry points (what we try to keep user-facing)

These crate names are intentionally treated as public “front doors”:

- `fret`: batteries-included app entry points (desktop-first).
- `fret-ui-shadcn`: default component taxonomy + recipes (fast iteration surface).
- `fret-ui-kit`: component authoring glue (tokens, headless helpers, overlay policy helpers).
- `fret-framework`: advanced/manual assembly and integrations.
- `fretboard`: tooling (templates + demo runner + diagnostics workflows).

### Mechanism vs policy (why the split matters)

- `crates/fret-ui` is **mechanisms/contracts only**: element tree, layout, hit-testing, event
  routing, focus, semantics, overlay roots/layers, placement solver, scroll/virtualization, text
  input engines.
- Policy-heavy behavior (dismiss rules, focus restore, hover intent, default sizing/padding, roving
  selection writes, typeahead matching) lives in `ecosystem/`.

If you find yourself reaching into runtime structs for policy, treat it as a smell: there is
probably an ecosystem-level hook/helper that should be used instead (or added).

## How to write app UI that refactors well

### 1) Keep your `view` shape boring

Golden path:

- `impl View for MyView { fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui { ... } }`
- Return `Ui` (the app-facing alias over `Elements`). For a single root element, return `root.into()`.

Why:

- It keeps the default app-author contract concrete and nameable (`AppUi` / `Ui`).
- The explicit advanced/manual path still maps cleanly to a concrete
  `fn(&mut ElementContext<'_, App>, &mut State) -> ViewElements` seam when you need
  hotpatch-friendly function-pointer hooks.
- It leaves room for internal storage optimizations while keeping call sites stable.

#### Why not `-> impl Trait` for `render()`?

Returning `impl Into<Ui>` (or `impl View`) can look nicer in small examples, but it pushes app
code toward generic-heavy patterns that are harder to version over time:

- It makes the default `View::render(...)` contract less nameable in app code and docs.
- It still does not help the explicit advanced/manual path, which needs a concrete `fn` seam for
  hotpatching and predictable dispatch.
- It tends to increase type churn and compile times (monomorphization + larger error surfaces).
- It makes it harder to standardize tooling/diagnostics around a single authoring return type.

Instead, we keep `Ui` concrete and provide small ergonomics (e.g. `root.into()` and array /
iterator conversions) so call sites stay compact without paying the generic tax.

### 2) Prefer typed actions (golden path)

If you want MVU-style clarity while keeping a unified command/key/palette pipeline, prefer typed
actions:

- Define typed unit actions with stable IDs via `fret::actions!([..])`.
- Bind UI triggers via `.action(act::Something)` (or `cx.dispatch(...)` for programmatic dispatch).
- Handle actions via `cx.actions().locals_with((...)).on::<A>(...)`, single-local `cx.actions().local(&local).set::<A>(...)` / `.update::<A>(...)` / `.toggle_bool::<A>()`, keyed-row `.action_payload(...)` plus `cx.actions().local(&rows_state).payload_update_if::<A>(...)` as the default view-owned row-write path, `cx.actions().models::<A>(...)` / `cx.actions().payload_models::<A>(...)` (shared graphs, with or without payload actions), `cx.actions().transient::<A>(...)`, plus widget-local `.action(...)` / `.action_payload(...)` / `.listen(...)` for activation-only surfaces after explicitly importing `use fret::app::AppActivateExt as _;`; keep raw `AppUi::on_action_notify*`, raw `AppUi::on_payload_action_notify*`, and low-level `on_activate*` helpers for cookbook/reference host-side cases.
- If advanced code intentionally wants the raw model-backed hook, import `use fret::advanced::AppUiRawStateExt;` and call `cx.use_state::<T>()` explicitly instead of treating it as part of the default `AppUi` surface.

Authoring and historical note:

- New code should use `View` + typed unit actions (`fret::actions!`) and typed payload actions
  (`fret::payload_actions!`) instead of historical MVU-era helpers.
- In-tree `fret::mvu::*` has been removed; keep MVU discussion only as an external migration/history reference.
- If you are migrating an older external MVU codebase, use `docs/workstreams/action-first-authoring-fearless-refactor-v1/MIGRATION_GUIDE.md` as the mapping guide rather than as an alternative authoring recommendation.

### 3) Use identity helpers early

When rendering collections:

- Use `cx.keyed(key, |cx| ...)` for list rows, menus, tabs, virtual list items.
- Use `cx.scope(|cx| ...)` at component boundaries where stable element-local state matters.

This makes later refactors (virtualization, reordering, splitting components) safer.

### 4) Keep layout/styling in the `ui()` surface

Prefer ecosystem layout helpers and patches:

- `ui::h_flex(...)`, `ui::v_flex(...)`, `ui::container(...)`
- `.ui().px_3().w_full().rounded(...).into_element(cx)`

This limits churn when we refine token conventions or introduce new style defaults.

### 4.5) Treat inherited styling as context, not as a layout fragment

If a helper exists only to install inherited text/icon foreground for descendants, do not assume it
is layout-transparent unless the contract says so explicitly.

Why:

- a style-looking helper that inserts a real wrapper node can silently change subtree ownership,
- which in turn can affect fill/shrink behavior, wrapped-text width resolution, and overlay content
  composition.

For the current cleanup direction and migration plan, see:

- `docs/workstreams/foreground-style-context-fearless-refactor-v1/DESIGN.md`

### 5) Use action hooks for interaction policy

If you’re wiring press/hover/dismiss/roving/typeahead/timers:

- Use `ElementContext` action hook helpers (or `fret-ui-kit` conveniences), not runtime-owned
  shortcut fields.

References:

- `docs/action-hooks.md` (why policy lives in components)

## Upgrade playbook (when things change)

### Compile-guided upgrades

Most refactors should be navigable by:

1. `cargo check` (fast compile errors first)
2. Fix API renames/signature shifts (usually ecosystem-level)
3. Run the relevant demo/tooling command to confirm runtime behavior

Use `rg` to find patterns:

- `.into_element(cx)` call sites
- `Theme::global(...)` usage
- `watch_model(...).layout()` chains
- typed action handlers (`cx.actions().locals_with((...)).on::<A>(...)`, single-local `cx.actions().local(&local).set::<A>(...)` / `.update::<A>(...)` / `.toggle_bool::<A>()`, keyed-row `.action_payload(...)` plus `cx.actions().local(&rows_state).payload_update_if::<A>(...)` as the default row-write path, `cx.actions().models::<A>(...)`, `cx.actions().payload_models::<A>(...)`, `cx.actions().transient::<A>(...)`, widget-local `.action(...)` / `.action_payload(...)` / `.listen(...)` after an explicit `use fret::app::AppActivateExt as _;`, and rare cookbook/reference seams such as `AppUi::on_action_notify*` and `AppUi::on_payload_action_notify*`) plus action IDs

### Prefer templates and golden demos as migration references

If a change impacts authoring ergonomics, we try to update at least one of:

- `apps/fret-examples/src/todo_demo.rs` (golden path)
- `fretboard new ...` templates (`apps/fretboard/src/scaffold/templates.rs`)

## What counts as a “small change” vs a “fearless refactor”

### Small changes (expected to be common)

These are typically additive or mechanical updates:

- Ergonomic sugar in `fret-ui-kit` / `fret-ui-shadcn` (new builder methods, better defaults).
- New conversion helpers (e.g. allowing a single `AnyElement` to convert into `Elements`).
- New component methods for common semantics/test hooks.
- Refactoring templates/demos to demonstrate best practices.

### Fearless refactors (breaking, but planned and documented)

Some areas are intrinsically hard-to-change and may require a coordinated refactor when the
contracts mature. These should be treated as “project events” with explicit roadmap docs.

Current examples (see the linked documents for status and migration notes):

- Overlay + input arbitration v2: `docs/overlay-and-input-arbitration-v2-refactor-roadmap.md`
- Layout engine refactor: `docs/layout-engine-refactor-roadmap.md`
- Renderer refactor: `docs/renderer-refactor-roadmap.md`
- Declarative-only migration tracker: `docs/declarative-only-migration.md`

When one of these lands, expect:

- a migration inventory/checklist (what changes, what stays stable),
- updated golden demos and templates,
- explicit decision gates in ADRs where needed.

## Common refactor recipes

### Recipe: returning a single root element

Prefer returning one root element as `Elements`:

- `root.into()`

This avoids `vec![root].into()` boilerplate and makes it obvious your view has a single root.

### Recipe: model reads during render (reduce boilerplate, keep invalidation explicit)

When reading a `Model<T>` during rendering, prefer the “watch + value_*” helpers so invalidation is
registered and the default read endings stay concise:

- `cx.watch_model(&model).layout().value_or_default()`
- `cx.watch_model(&model).paint().value_or_default()`
- `cx.watch_model(&model).layout().value_or(fallback)`

### Recipe: iterator-heavy children (borrow checker)

If you need to build children from iterators that capture `&mut cx`, prefer `*_build` variants:

- `ui::v_flex_build(cx, |cx, out| { ... })`
- `ui::h_flex_build(cx, |cx, out| { ... })`

This keeps the authoring surface predictable while avoiding common borrow pitfalls.

### Recipe: stamping `test_id` / a11y without layout wrappers

Prefer layout-transparent semantics decoration on `AnyElement` over introducing extra wrapper
nodes:

- `element.test_id("my-test-id")`
- `element.role(SemanticsRole::Button).test_id("my-button")`

### Recipe: embedding other UI systems

If you need to embed `egui`/`iced`/custom engines:

- Prefer viewport-surface embedding (foreign runtime renders into a texture; Fret hosts it and
  forwards input).

Reference:

- `docs/ui-ergonomics-and-interop.md`

## “When should I do a big refactor?”

Use this heuristic:

- If your change is **policy/ergonomics** (layout sugar, default sizing, recipes, interaction
  outcomes), keep it in `ecosystem/`.
- If your change needs **new runtime contracts** (focus, semantics, overlay substrate, text
  editing, input capture), treat it as an ADR-worthy change and expect a fearless refactor path.

If you are unsure, start by updating a golden demo + writing down the desired behavior as a small
doc note. If the contract is truly hard-to-change, promote it to an ADR before it spreads.
