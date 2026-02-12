---
name: fret-component-authoring
description: Component authoring in Fret (`fret-ui` + `fret-ui-kit`). Use when building/refactoring declarative elements, managing element identity/state (`scope`, `keyed`, `with_state`), reading/observing `Model<T>` with correct invalidation, or adopting the `UiBuilder` (`ui()`) patch surface.
---

# Fret component authoring

Fret is **declarative-first**. The primary component authoring API is `fret_ui::ElementContext`.
Upper layers should stay in `fret-ui-kit` / `fret-ui-shadcn` and keep `crates/fret-ui` mechanism-only.

## When to use

- Building or refactoring a declarative element/component.
- Debugging “state sticks to the wrong row” / identity issues.
- Debugging invalidation bugs (“model updated but UI didn’t re-layout/repaint”).
- Adopting the `UiBuilder` (`ui()`) patch surface for ecosystem components.

## Inputs to collect (ask the user)

Ask these before writing code (most bugs are “wrong identity” or “wrong invalidation”):

- Component family: list row/virtualized item, overlay surface, text input, etc?
- Identity: what should the stable key be (data id, not index), and where is it introduced?
- State: what must persist across frames (element-local state vs `Model<T>` vs app-global)?
- Invalidation: should updates trigger `Layout` or only `Paint`?
- Layering: is this mechanism (`crates/fret-ui`) or policy/recipes (`ecosystem/`)?

Defaults if unclear:

- Add stable identity first (`keyed/scope`), then add state, then wire model reads with explicit invalidation.

## Smallest starting point (one command)

- `cargo run -p fretboard -- dev native --bin components_gallery`

## Workflow

1. Ensure stable identity (`keyed` / `scope` / `named`) before adding state.
2. Store cross-frame local state with `with_state*` (not global statics).
3. Read models with explicit invalidation (`Layout` vs `Paint`) so updates are observed.
4. Keep interaction policy in ecosystem layers (action hooks) rather than `crates/fret-ui`.
5. Add at least one regression artifact (unit test or `fretboard diag` script) for tricky behavior.

## Definition of done (what to leave behind)

- Minimum deliverables (3-pack): Repro (smallest component), Gate (test/script), Evidence (anchors). See `fret-skills-playbook`.
- Stable identity is explicit (`keyed`/`scope`) and derived from the model (not incidental indices).
- Cross-frame state lives in `with_state*` or `Model<T>` (no global statics).
- All model reads register the correct invalidation (`Layout` vs `Paint`) and the UI updates deterministically.
- Any non-trivial interaction/state machine has a regression artifact:
  - a focused unit/integration test, and/or
  - a `tools/diag-scripts/*.json` scenario with stable `test_id` targets.
- Layering is preserved: policy stays in `ecosystem/`, mechanisms in `crates/`.

## Overview

**Key concepts:**

- **Stable identity:** `cx.scope(...)` / `cx.keyed(key, ...)` / `cx.named("name", ...)`
- **Element-local state:** `cx.with_state(...)` / `cx.with_state_for(...)`
- **Model observation:** a model read must also register invalidation (`Layout` vs `Paint`)
- **Action hooks:** interaction policy is owned by components (ADR 0074)
- **Unified authoring surface:** `UiBuilder` via `value.ui().px_3().w_full().into_element(cx)` (ADR 0145)

## Quick start

### 1) Identity + local state

Use `keyed` for lists/virtualization and `with_state` for cross-frame local state:

```rust
use fret_ui_kit::prelude::*;

#[derive(Default)]
struct RowState {
    open: Option<Model<bool>>,
}

fn open_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<bool> {
    cx.with_state(RowState::default, |st| {
        st.open
            .get_or_insert_with(|| cx.app.models_mut().insert(false))
            .clone()
    })
}

pub fn row<H: UiHost>(cx: &mut ElementContext<'_, H>, id: u64) -> AnyElement {
    cx.keyed(id, |cx| {
        // Create the model once per row (stored in element-local state).
        let open = open_model(cx);
        let is_open = cx.get_model_copied(&open, Invalidation::Paint);

        ui::h_flex(cx, move |cx| {
            vec![
                ui::text(cx, format!("row {id}")).into_element(cx),
                ui::text(cx, format!("open={is_open}")).into_element(cx),
            ]
        })
        .gap_metric(MetricRef::space(Space::N2))
        .into_element(cx)
    })
}
```

### 2) Read + observe models (avoid “silent no-op” invalidation)

If you read a `Model<T>` during render without observation, UI may not re-layout/re-paint on updates.

Prefer the combined read+observe helpers (see `docs/action-hooks.md`):

- `cx.get_model_cloned(&model, Invalidation::Layout)`
- `cx.get_model_copied(&model, Invalidation::Paint)`
- `cx.read_model_ref(&model, Invalidation::Layout, |v| ...)`

## Common patterns

### Prefer the `ui()` patch surface for ecosystem components

If a component should be styleable/layoutable like shadcn recipes, implement the ecosystem traits:

- `UiPatchTarget` (accepts `{ chrome, layout }` patches)
- `UiIntoElement` (renders to `AnyElement`)
- `UiSupportsChrome` / `UiSupportsLayout` (enables fluent methods)

Reference: `docs/component-authoring-contracts.md` (“Unified authoring builder surface”, ADR 0145).

### Iterator borrow pitfall: use `*_build` constructors

When authoring children from an iterator that captures `&mut cx`, use:

- `ui::h_flex_build(cx, |cx, out| { ... })`
- `ui::v_flex_build(cx, |cx, out| { ... })`
- `ui::container_build(cx, |cx, out| { ... })`

This avoids borrow-checker conflicts while keeping rendering keyed/stable.

## Common pitfalls

- Reading models during render without registering invalidation.
- Using retained widgets as a public component authoring model (see ADR 0066).
- Putting interaction policy (dismiss rules, focus restore, typeahead matching) into `crates/fret-ui`.

## Evidence anchors

- Architecture overview: `docs/architecture.md`
- Component authoring contracts: `docs/component-authoring-contracts.md`
- Action hooks (policy split): `docs/action-hooks.md`
- Runtime contract surface map: `docs/runtime-contract-matrix.md`
- Key APIs:
  - `crates/fret-ui/src/elements/cx.rs` (`ElementContext`)
  - `ecosystem/fret-ui-kit/src/ui_builder.rs` (`UiBuilder`, ADR 0145)
  - `ecosystem/fret-ui-kit/src/declarative/action_hooks.rs` (`ActionHooksExt`)

## Related skills

- `fret-action-hooks` (press/dismiss/roving/typeahead/timers policy wiring)
- `fret-layout-and-style` (layout/style tokens and `UiBuilder` usage)
- `fret-diag-workflow` (turn tricky state bugs into scripted repro gates)
