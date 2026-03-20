# Into-Element Surface (Fearless Refactor v1)

Status: maintenance-only historical design record
Last updated: 2026-03-18

Related:

- `docs/workstreams/authoring-surface-and-ecosystem-fearless-refactor-v1/DESIGN.md`
- `docs/workstreams/authoring-surface-and-ecosystem-fearless-refactor-v1/TARGET_INTERFACE_STATE.md`
- `docs/workstreams/ecosystem-integration-traits-v1/DESIGN.md`
- `docs/adr/0066-fret-ui-runtime-contract-surface.md`
- `docs/adr/0148-component-ecosystem-authoring-conventions-v1.md`
- `CLOSEOUT_AUDIT_2026-03-20.md`

Closeout reading rule on 2026-03-16:

- treat this document as the settled design record for a closed conversion-surface cleanup lane
- the broad `IntoUiElement<H>` migration and public conversion-taxonomy collapse are already
  landed; use `MILESTONES.md`, `TODO.md`, and `TARGET_INTERFACE_STATE.md` as the current closure
  evidence
- remaining work here is maintenance only: explicit raw-seam inventory, low-noise wrapper cleanup,
  and source-gate/docs alignment

Closeout note on 2026-03-18:

- a sampled re-audit confirmed that the remaining `MIGRATION_MATRIX.md` rows were stale status
  lag, not active product-surface migration,
- representative source-policy gates now lock the settled posture across reusable helpers, default
  app snippets/pages, and intentional raw preview seams:
  - `ecosystem/fret/tests/reusable_component_helper_surface.rs`
  - `apps/fret-ui-gallery/tests/ui_authoring_surface_default_app.rs`
  - `apps/fret-ui-gallery/tests/ui_authoring_surface_internal_previews.rs`
  - `ecosystem/fret-ui-shadcn/src/surface_policy_tests.rs`
- this document should now be read strictly as the historical explanation for why the lane existed,
  not as a live design owner for further authoring-surface work.

Closeout audit note on 2026-03-20:

- read `CLOSEOUT_AUDIT_2026-03-20.md` as the final lane-level verdict;
- this folder no longer owns an active migration queue, only explicit seam inventory and drift
  control.

This workstream focused on one narrow product-surface problem:

- how authoring values become `AnyElement`,
- which conversion concepts are public,
- and which of those concepts ordinary app authors should never need to learn.

This is intentionally a **fearless refactor** plan:

- Fret is still pre-release.
- We do not need compatibility shims for external users.
- If multiple public-looking conversion traits describe the same mental operation, we should
  collapse them rather than document around them.

This document is not an ADR. It does not change the retained/declarative runtime contract by
itself. If implementation work changes `fret-ui` tree semantics, overlay ownership, or other hard
runtime contracts, the relevant ADRs must be updated separately.

## 1. Problem Statement

The app-facing authoring reset had already improved the main story:

- `FretApp`
- `App`
- `AppUi`
- `Ui`
- `UiCx`
- `UiChild`
- grouped `state/actions/data/effects`

But the **value-to-element conversion surface** was still fragmented when this lane opened.

At the time this lane opened, first-party code and public exports exposed multiple overlapping
concepts:

- `UiIntoElement`
- `UiHostBoundIntoElement<H>`
- `UiBuilderHostBoundIntoElementExt<H>`
- `UiChildIntoElement<H>`
- `AnyElement`
- `Elements`

Those names do not encode one crisp product story. They encode implementation history.

### 1.1 Why this matters for app authors

Ordinary app authors should learn:

- `Ui` for a view return,
- `UiChild` for extracted app-facing helpers,
- `.ui()` for fluent refinement,
- `.into_element(cx)` only as a landing operation, not as a taxonomy lesson.

When the default path still depended on several conversion traits being in scope, the surface felt
more mechanical than it should.

### 1.2 Why this matters for component authors

Reusable component authors do need a conversion contract, but they should not need to understand
why one trait is host-agnostic, another is host-bound, and a third exists only to restore method
call syntax on `UiBuilder<T>`.

The component surface should expose **one** public conversion concept, not a stack of bridge
concepts.

### 1.3 Evidence from the tree

The original problem was easy to see in the public surface:

- `fret::app::prelude::*` used to rely on a hidden `UiBuilderHostBoundIntoElementExt` bridge so
  method syntax kept working on the app path.
- `fret::component::prelude::*` used to expose `UiIntoElement`, `UiHostBoundIntoElement`,
  `UiChildIntoElement`, `UiBuilder`, and `AnyElement` together.

That split is now deleted from production code. The remaining closeout evidence is narrower:

- some historical workstream/docs text still describes the old split as if it were current,
- a few intentionally non-raw first-party helpers can still drift back to `AnyElement` even though
  the unified `IntoUiElement<H>` / `UiChild` story is already available.

First-party usage also showed that `AnyElement` had become the default authoring currency for many
component/snippet surfaces:

- `apps/fret-cookbook/examples/**`: the official cookbook is already much closer to the target
  app-facing story.
- `apps/fret-ui-gallery/src/ui/snippets/**`: helper/snippet surfaces still overwhelmingly returned
  `AnyElement` and landed children explicitly with `.into_element(cx)`.

That is acceptable for advanced/raw surfaces. It is not the product surface we should teach by
default.

### 1.4 Why this is the highest-leverage remaining UI-authoring gap

If we separate three questions:

- "how does it feel to write one piece of UI?",
- "how good is the full app-authoring story?",
- "how well can the framework support external ecosystems?",

the repo is now in different states on purpose:

- Fret already has a clearer app-facing product story than GPUI for many general-purpose apps
  (`FretApp`, `AppUi`, `Ui`, grouped `state/actions/data/effects`).
- Fret already has a cleaner mechanism/policy/ecosystem split than GPUI for open third-party
  ecosystem growth.
- The clearest place where GPUI still feels more unified is the **local authoring loop** for
  helpers, snippets, and reusable components.

That remaining gap is not mainly about `Ui = Elements`.
It is not mainly about the grouped app-facing state/action APIs either.

It is mainly about the public conversion taxonomy still leaking into real authoring code.

That is why this workstream is the highest-leverage remaining surface cleanup for "make writing UI
feel like one obvious language" without giving up Fret's layered architecture.

## 2. Goals

1. Keep `Ui = Elements` as the app-facing render return alias.
2. Keep `UiChild` as the app-facing extracted-helper concept.
3. Collapse the component-facing conversion API to **one** public trait.
4. Preserve method-call ergonomics for `.into_element(cx)` without teaching bridge traits.
5. Demote `AnyElement` to an explicit raw/advanced tool rather than the default first-contact
   return type.
6. Delete compatibility-only conversion traits after first-party migration is complete.

## 3. Non-goals

- Replacing the declarative IR (`AnyElement`, `Elements`) with a new IR.
- Rewriting the `UiBuilder` fluent patch system.
- Introducing a macro DSL as the default authoring model.
- Forcing all low-level code to stop using `AnyElement`.
- Hiding advanced/runtime escape hatches from power users.

## 4. Design Rules

### 4.1 `Ui` stays

`Ui = Elements` is not the problem.

The problem is not that the default app path has a render alias.
The problem is that the lower conversion ladder is too visible and too fragmented.

This workstream therefore keeps:

- `Ui` on the app path,
- `Elements` as the explicit raw type,
- `AnyElement` as the explicit raw child/unit type.

### 4.2 App authors should learn nouns, not conversion taxonomy

The app-facing teaching surface should stay:

- `fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui`
- `fn helper(...) -> impl UiChild`

Clarification on 2026-03-15:

- spell `cx: &mut UiCx<'_>` on an app helper only when the helper body actually needs
  runtime/context access,
- if a page-shell helper only composes existing `UiChild` values, prefer dropping `cx` and
  late-landing it at the render root via `ui::children![cx; helper(...)]`.

App docs should not need to explain:

- why a helper returns `UiChildIntoElement<App>`,
- why `UiBuilderHostBoundIntoElementExt` must be in scope,
- or why one builder is "host-bound" while another is not.

### 4.3 Component authors need one public conversion trait

The component surface should converge on one concept:

```rust
pub trait IntoUiElement<H: UiHost>: Sized {
    fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement;
}
```

The exact spelling can still change during implementation, but the contract should remain:

- one public conversion trait,
- parameterized by host at the trait level,
- usable for both host-agnostic values and host-bound builder values,
- method-call compatible with `.into_element(cx)`.

### 4.4 Host-bound bridging is an implementation detail

The current split between:

- `UiIntoElement`
- `UiHostBoundIntoElement<H>`
- `UiBuilderHostBoundIntoElementExt<H>`

exists because Rust type constraints differ between:

- values that can convert for any host, and
- values that store a host-typed build closure.

That distinction can remain internally if Rust requires it.

It should not remain as a multi-concept public authoring taxonomy.

### 4.5 `UiChildIntoElement<H>` should disappear from the public story

`UiChildIntoElement<H>` exists mainly because heterogeneous child pipelines and host-bound builder
wrappers currently need a wider landing contract than `UiIntoElement`.

After the unified conversion trait exists, child pipelines should consume that unified trait
instead.

The app-facing `UiChild` marker can remain as a narrow app-owned alias/convention.

### 4.6 `AnyElement` remains explicit, not default

`AnyElement` is still the right raw representation for:

- overlay/controller internals,
- layout engines that assemble heterogeneous child vectors,
- diagnostics and harness code,
- advanced manual assembly,
- low-level reusable helpers that truly need to traffic in raw landed elements.

But first-party docs and default examples should prefer:

- `Ui` for view return values,
- `impl UiChild` for app-facing extracted helpers,
- `impl IntoUiElement<H>` for reusable/generic component helpers,
- `AnyElement` only when a raw landed element is truly required.

## 5. Target Shape

### 5.1 App surface

Keep:

- `Ui`
- `UiChild`
- `UiCx`
- anonymous import support so `.into_element(cx)` continues to work

Remove from the taught surface:

- `AnyElement`
- `Elements`
- `UiIntoElement`
- `UiHostBoundIntoElement`
- `UiChildIntoElement`
- `UiBuilderHostBoundIntoElementExt`

### 5.2 Component surface

Keep:

- `UiBuilder`
- `UiPatchTarget`
- layout/style refinement types
- `AnyElement` as an explicit raw tool
- one public conversion trait (`IntoUiElement<H>` or final equivalent)

Delete from the curated component-surface vocabulary:

- separate public host-agnostic vs host-bound conversion traits
- public bridge-only extension traits whose only job is restoring method syntax
- child-specific conversion traits that duplicate the main conversion contract

### 5.3 Advanced/raw surface

Keep explicit:

- `AnyElement`
- `Elements`
- raw `ElementContext<'_, H>`
- internal/runtime-only helpers when they are genuinely needed

This workstream does not try to erase the raw layer. It tries to stop that raw layer from being the
main product vocabulary.

## 6. Recommended API Shape

### 6.1 App-facing examples

```rust
use fret::app::prelude::*;

fn toolbar(cx: &mut UiCx<'_>) -> impl UiChild {
    ui::h_flex(|cx| {
        ui::children![
            cx;
            shadcn::Button::new("Save").ui().mr_2(),
            shadcn::Button::new("Export").ui(),
        ]
    })
}

impl View for MyView {
    fn init(_app: &mut App, _window: WindowId) -> Self {
        Self
    }

    fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {
        ui::v_flex(|cx| ui::children![cx; toolbar(cx)])
            .into()
    }
}
```

### 6.2 Component-facing examples

```rust
use fret::component::prelude::*;

fn inspector_row<H: UiHost>(
    label: &'static str,
    control: impl IntoUiElement<H>,
) -> impl IntoUiElement<H> {
    ui::h_flex(|cx| {
        ui::children![
            cx;
            ui::text(label).ui().w_px(120.0),
            control,
        ]
    })
}
```

### 6.3 Advanced/raw examples

```rust
use fret::advanced::kernel::*;
use fret_ui::element::AnyElement;

fn landed_overlay<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    ui::container(|cx| ui::children![cx; ui::text("Raw overlay")]).into_element(cx)
}
```

## 7. Migration Strategy

### Phase 0: lock the public target

- publish this workstream,
- decide the final name of the unified conversion trait,
- explicitly mark the old conversion traits as delete-targets.

### Phase 1: introduce the unified public trait

- add the new public conversion trait in `fret-ui-kit`,
- provide temporary blanket impls/adapters from the old internal traits,
- re-export only the new trait from curated surfaces.

### Phase 2: migrate builders and child pipelines

- make `ui::children!` and related heterogeneous child helpers consume the unified trait,
- update `UiBuilder<T>` conversion paths so both host-agnostic and host-bound builders land through
  the same public concept,
- keep any extra internal traits private or advanced-only.

### Phase 3: migrate first-party code

Prioritize:

1. `ecosystem/fret`
2. `ecosystem/fret-ui-kit`
3. `ecosystem/fret-ui-shadcn`
4. `apps/fret-cookbook`
5. `apps/fret-examples`
6. `apps/fret-ui-gallery`

The gallery should migrate in two lanes:

- app-facing teaching snippets move toward `UiChild`,
- reusable/generic snippets move toward `impl IntoUiElement<H>`,
- diagnostics/harness/raw surfaces may keep `AnyElement` explicitly where justified.

### Phase 4: delete the old public surface

Delete:

- `UiIntoElement` from the curated public surface,
- `UiHostBoundIntoElement` from the curated public surface,
- `UiBuilderHostBoundIntoElementExt` from the curated public surface,
- `UiChildIntoElement` from the curated public surface,
- stale docs that still teach those names.

## 8. Risks and Constraints

### 8.1 Rust coherence and blanket impl overlap

The new public trait must avoid coherence conflicts between:

- host-agnostic values,
- host-bound builder values,
- `UiBuilder<T>` blanket impls,
- `AnyElement` passthrough impls.

We should prefer one temporary internal adapter layer over several permanent public traits.

### 8.2 Method resolution stability

If method-call ergonomics depends on an anonymous prelude import, that is acceptable.
What is not acceptable is requiring app/component authors to reason about multiple conversion
traits to predict which `.into_element(cx)` call will compile.

### 8.3 Do not over-normalize raw code

Some low-level helpers are clearer as `AnyElement`.
This workstream should not create churn merely to replace every raw helper return type with
`impl Trait`.

The target is a cleaner public product surface, not the eradication of all explicit IR usage.

## 9. Gates

Add or extend gates so the surface stays clean:

- app prelude does not publicly re-export old conversion traits,
- component prelude exports exactly one public conversion trait,
- first-party default docs/examples prefer `Ui` / `UiChild`,
- first-party generic component examples prefer the unified conversion trait,
- UI Gallery source gates distinguish justified raw `AnyElement` surfaces from default teaching
  surfaces,
- stale names (`UiChildIntoElement`, `UiHostBoundIntoElement`, `UiBuilderHostBoundIntoElementExt`)
  fail fast once deletion starts.
