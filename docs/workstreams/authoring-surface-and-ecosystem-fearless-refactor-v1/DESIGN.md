# Authoring Surface + Ecosystem (Fearless Refactor v1)

Status: proposed pre-release reset

This workstream is about making Fret's public authoring story dramatically simpler before the
first public release.

It is intentionally a **fearless refactor** plan:

- Fret is not published yet.
- We do not need compatibility shims for external users.
- If an old public-looking helper makes the product surface worse, we should delete it rather than
  preserve it.

This document is **not** an ADR. If this workstream changes hard contracts in `crates/fret-ui`,
`fret-runtime`, input/focus, overlays, text, or renderer semantics, the relevant ADRs must be
updated separately.

## Problem Statement

### From an app author's perspective

Fret already has the right power, but the surface is still too wide:

- too many things are reachable from the default prelude,
- low-level mechanism types leak into first-contact imports,
- multiple action/state patterns still look "equally official",
- users can reach advanced seams before they understand the boring path.

The result is that small apps still feel more ceremonial than they should.

### From an ecosystem author's perspective

We have strong first-party ecosystems (`fret-ui-shadcn`, `fret-docking`, `fret-query`,
`fret-selector`, `fret-router`), but the smallest stable contract for third-party crates is not
sharp enough yet.

Today, an ecosystem author can reasonably ask:

1. Which surface is the stable one for reusable components?
2. Which names are safe to teach in docs?
3. Which extension seams should query/router/state libraries plug into?
4. How do I avoid depending on app- or runner-owned concepts accidentally?

### From a maintainer's perspective

The architecture is mostly correct, but the product surface is not encoded tightly enough:

- documentation has to do too much of the curation work,
- "default" vs "advanced" is a prose convention more than a type-system boundary,
- first-party crates can still drift into teaching slightly different mental models,
- dead aliases and redundant helpers make cleanup harder.

## Guiding Constraints

- Keep `crates/fret-ui` mechanism-only.
- Keep policy and recipe behavior in ecosystem crates.
- Make the default app surface small enough to memorize.
- Make the reusable component surface explicit and portable.
- Make advanced seams explicit instead of accidentally discoverable.
- First-party ecosystem crates must use the same extension seams we expect third parties to use.
- Prefer deletion over deprecation while the project is pre-release.

## Non-goals

- Rewriting the renderer architecture.
- Replacing the runtime model with a second authoring runtime.
- Moving interaction policy into `crates/fret-ui`.
- Introducing a macro DSL as the default authoring model.
- Preserving old naming if a better product surface exists.

## Personas

### 1) App author

Wants one obvious way to build:

- a native desktop app,
- a general-purpose productivity tool,
- a dashboard/settings/workspace app,
- a medium-size app without learning runner internals on day one.

### 2) Component / design-system author

Wants:

- a portable reusable component contract,
- stable styling/layout hooks,
- clean semantics/focus/overlay composition,
- no forced dependency on app-owned globals or launch glue.

### 3) First-party ecosystem maintainer

Examples:

- `fret-ui-shadcn`
- `fret-docking`
- `fret-query`
- `fret-selector`
- `fret-router`

Needs a shared authoring vocabulary so first-party crates do not invent parallel mini-frameworks.

### 4) Third-party ecosystem author

Wants to ship:

- another component kit,
- another design system,
- a workflow library,
- a router/query/state helper,
- a domain UI package.

Needs a small, documented, stable extension surface.

## Product Surface Reset (Target Shape)

We should intentionally define **three** public authoring layers.

| Surface | Audience | Import posture | What it exposes |
| --- | --- | --- | --- |
| App surface | ordinary app authors | `use fret::app::prelude::*;` | app builder, app-facing view runtime, default UI composition, default action/state patterns |
| Component surface | reusable component authors | `use fret::component::prelude::*;` | component composition contracts, styling/layout patch surface, semantics/layout helpers |
| Advanced surface | runner/interop/power users | explicit `fret::advanced::*` imports | driver hooks, viewport/runner/manual assembly seams, low-level runtime types |

Important rule:

- `fret::app::prelude::*` must become the canonical app import.
- `fret::prelude::*` may exist only as a temporary in-repo migration bridge and must stop acting as a transitive "everything prelude".

## Naming Reset

Because we do not need compatibility, this workstream should make the names unambiguous.

### Target names

- `FretApp`: the canonical app builder entry point.
- `KernelApp`: the app runtime type currently known as `fret_app::App`.
- `AppUi`: the app-facing render/action/state context currently exposed as `ViewCx`.
- `Ui`: canonical app-facing return alias for rendered UI (`Elements` underneath).

### Names to stop teaching

- `fret::App` as the default user-facing entry name.
- bare `App` in the default app prelude when it refers to the kernel runtime.
- `ViewElements` as a first-contact alias.
- broad mechanism names in app-level docs when a narrower app-facing alias exists.

## App Authoring Surface (Target)

The app surface should optimize for the smallest stable mental model:

1. `FretApp`
2. `View`
3. `AppUi`
4. `LocalState`
5. `ui` / `shadcn`

### Default imports

The app prelude should expose only:

- `FretApp`
- `View`
- `AppUi`
- `LocalState`
- `Ui`
- `ui`
- `shadcn` (when enabled)
- `ThemeSnapshot`
- typed action macros and identities needed by ordinary apps

It should **not** expose:

- `ElementContext`
- `UiTree`
- `UiServices`
- raw `ModelStore`
- low-level runner/viewport traits
- broad component-author or maintainer-only helpers

### Default `AppUi` shape

Instead of dozens of flat helpers, the default app-facing API should be grouped by intent:

- `ui.state()`
- `ui.actions()`
- `ui.data()`
- `ui.effects()`

Suggested default operations:

- `ui.state().local::<T>()`
- `ui.state().local_init(|| ...)`
- `ui.state().watch(&state)`
- `ui.actions().locals::<A>(...)`
- `ui.actions().payload::<A>().local(&state, ...)`
- `ui.actions().transient::<A>(...)`
- `ui.data().selector(...)`
- `ui.data().query(...)`

Advanced operations may still exist, but must not be the main surface.

### Default action posture

The default mental model should become:

- view-owned state: `LocalState<T>`
- coordinated writes: `actions().locals::<A>(...)`
- app-bound side effects: `actions().transient::<A>(...)`
- shared cross-view graphs: explicit advanced/escape hatch

The old split between multiple near-duplicate handler helpers should be reduced aggressively.

## Component Surface (Target)

Reusable components should have an explicit contract separate from the app surface.

### Ownership

- app authoring stays in `ecosystem/fret`
- reusable component infrastructure stays in `ecosystem/fret-ui-kit`
- recipe/taxonomy layers stay in crates such as `fret-ui-shadcn` and future design-system crates

### Goal

Third-party component authors should only need:

- the component prelude,
- stable layout/style patch hooks,
- semantics and composition helpers,
- documented boundaries for overlay/focus/input behavior.

### Rules

- reusable components should not depend on the app prelude,
- reusable components should not need runner types,
- component kits should build on the same primitives used by first-party kits,
- component docs should not teach app-level glue unless explicitly marked as integration guidance.

## Ecosystem Strategy

This workstream must support both first-party and third-party ecosystems.

### `fret-ui-shadcn`

Target posture:

- remains a recipe/taxonomy crate,
- depends on the component surface, not the app surface,
- never becomes the place where app-runtime policy leaks in by accident,
- keeps `app-integration` explicitly optional and clearly separated from component composition.

### `fret-docking`

Target posture:

- core docking model/ops stay below the policy layer,
- docking UI behavior stays in ecosystem,
- multi-window and viewport hooks are explicit advanced seams,
- docking should not redefine the app authoring model.

### `fret-selector`, `fret-query`, `fret-router`

Target posture:

- remain separate crates,
- integrate through small documented extension seams,
- use the same app-facing grouped context model as ordinary apps,
- avoid requiring direct low-level runtime imports in normal usage.

Concrete rule:

- first-party state/router libraries should extend `AppUi` (or its grouped subcontexts) through
  explicit extension traits, not through ad-hoc hidden knowledge.

### Third-party ecosystem crates

A third-party crate should be able to choose one of these product positions:

1. **App addon**
   - depends on the app surface and optionally higher-level first-party ecosystems.
2. **Reusable component kit**
   - depends on the component surface only.
3. **Advanced integration crate**
   - depends on advanced/manual assembly surfaces explicitly.

We should document this split so external ecosystems know where they belong.

## Deletion Strategy

Because compatibility is not required, we should remove redundant public-looking surfaces instead
of carrying them forward.

### Delete or demote from the default app surface

- broad transitive re-exports from `fret_ui_kit::prelude::*`
- mechanism-layer types in `fret::app::prelude::*`
- duplicate top-level aliases that increase naming ambiguity
- flat action helper variants that are no longer part of the blessed path
- old examples/docs that teach superseded authoring patterns

### Keep, but move to explicit advanced imports

- driver hooks
- viewport/interop seams
- manual assembly surfaces
- low-level runtime/service types

## Documentation and Template Reset

This workstream is not complete until the docs and generated templates all teach the same story.

Required outcomes:

- `README.md`
- `docs/README.md`
- `docs/first-hour.md`
- templates in `apps/fretboard`
- cookbook examples

must all teach the same default surface and avoid low-level leakage.

## Example: Current vs Target Authoring Feel

### Current shape (representative)

```rust
fn render(&mut self, cx: &mut ViewCx<'_, '_, App>) -> Elements {
    let draft = cx.use_local::<String>();
    let todos = cx.use_local_with(seed_todos);

    cx.on_action_notify_locals::<act::Add>({
        let draft = draft.clone();
        let todos = todos.clone();
        move |tx| {
            let text = tx.value_or_else(&draft, String::new).trim().to_string();
            if text.is_empty() {
                return false;
            }
            let _ = tx.update(&todos, |rows| rows.push(TodoRow::new(text)));
            tx.set(&draft, String::new())
        }
    });

    let input = shadcn::Input::new(&draft).submit_command(act::Add.into());

    ui::v_flex(|cx| ui::children![cx; input])
        .into_element(cx)
        .into()
}
```

### Target shape

```rust
fn render(&mut self, ui: &mut AppUi<'_>) -> Ui {
    let draft = ui.state().local::<String>();
    let todos = ui.state().local_init(seed_todos);

    ui.actions().locals::<act::Add>(|tx| {
        let text = tx.get(&draft).trim().to_string();
        if text.is_empty() {
            return false;
        }

        tx.update(&todos, |rows| rows.push(TodoRow::new(text)));
        tx.set(&draft, String::new());
        true
    });

    vstack![
        input(&draft).submit(act::Add),
        todo_list(&todos),
    ]
}
```

The goal is not "more magic". The goal is:

- fewer names to remember,
- less boilerplate at the call site,
- clearer separation between default and advanced paths.

## Gates

This workstream should leave behind enforcement, not only prose.

Examples:

- gate that `fret::app::prelude` does not expose low-level mechanism types,
- gate that templates use only the blessed app surface,
- gate that README/docs/first-hour agree on the default action model,
- gate that ecosystem crates use the documented extension seams instead of private shortcuts.

## Success Criteria

- A new app author can stay productive on one small import surface.
- A third-party component author can identify the correct reusable contract in under 10 minutes.
- First-party ecosystem crates no longer teach parallel authoring dialects.
- The default prelude becomes materially smaller and more intentional.
- Dead aliases and redundant helpers are removed, not merely hidden in prose.
