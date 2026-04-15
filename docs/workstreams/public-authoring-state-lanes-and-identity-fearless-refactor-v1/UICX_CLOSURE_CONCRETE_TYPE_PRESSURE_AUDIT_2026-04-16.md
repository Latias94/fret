# UiCx Closure Concrete-Type Pressure Audit — 2026-04-16

Status: post-facade pressure audit for the active public authoring lane

Related:

- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/APP_RENDER_CONTEXT_FACADE_AUDIT_2026-04-16.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/EXTRACTED_HELPER_RENDER_GUIDANCE_AUDIT_2026-04-16.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/DESIGN.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/TODO.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/MILESTONES.md`
- `ecosystem/fret/src/view.rs`
- `ecosystem/fret/src/lib.rs`
- `apps/fret-examples/src/api_workbench_lite_demo.rs`
- `apps/fret-cookbook/src/scaffold.rs`
- `apps/fret-ui-gallery/src/ui/snippets/ai/model_selector_demo.rs`
- `apps/fret-ui-gallery/src/ui/snippets/item/demo.rs`
- `apps/fret-examples/src/markdown_demo.rs`
- `apps/fret-examples/src/hello_world_compare_demo.rs`

## Question

Now that `AppRenderContext<'a>` exists, is the remaining `UiCx` pressure mostly migration noise,
or does it still reveal a real concrete-type ergonomics need on the default authoring lane?

## Verdict

`AppRenderContext<'a>` is the correct named façade for extracted helper **functions**, but it does
not close the remaining `UiCx` pressure by itself.

The still-open pressure is specifically:

- closure-local helper extraction,
- snippet/gallery authoring that wants a concrete parameter type,
- and small inline helper families where generic trait bounds are materially less ergonomic than a
  concrete context carrier.

Therefore the next structural question is not "rename more docs from `RenderContextAccess` to
`AppRenderContext`".

The next structural question is:

- whether the default app lane ultimately needs a concrete extracted-helper carrier,
- or whether `UiCx` remains the compatibility-only concrete type for closure-local helpers while
  named helper functions keep migrating to `AppRenderContext<'a>`.

## Findings

### 1) The façade landing only touches a tiny proof set compared with the remaining concrete-type stock

Current landed `AppRenderContext<'a>` proof surfaces:

- `apps/fret-cookbook/src/scaffold.rs`
- `apps/fret-examples/src/api_workbench_lite_demo.rs`
- first-contact docs and source-policy tests

Quick repo counts on 2026-04-16:

- `AppRenderContext<'a>` occurrences across the landed proof/docs set used in this slice: 18
- `UiCx` closure/function helper matches in Rust sources:
  - `apps/fret-ui-gallery/src`: 1220 matches across 913 Rust files
  - `apps/fret-examples/src`: 107 matches across 10 Rust files
  - `apps/fret-cookbook/examples`: 5 matches across 4 Rust files

Conclusion:

- the façade is real,
- but the remaining concrete-type authoring stock is still much larger.

### 2) Closure-local helpers are the main unresolved pressure, not named helper functions

Representative patterns:

- `let item = |cx: &mut UiCx<'_>, ...| { ... }`
- `let icon = |cx: &mut UiCx<'_>, ...| { ... }`
- `fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`

Representative files:

- `apps/fret-ui-gallery/src/ui/snippets/item/demo.rs`
- `apps/fret-ui-gallery/src/ui/snippets/ai/model_selector_demo.rs`
- `apps/fret-examples/src/markdown_demo.rs`
- `apps/fret-examples/src/hello_world_compare_demo.rs`

These are not all advanced/raw proofs.
Many are ordinary snippet or chrome helpers that happen to prefer concrete Rust parameter syntax.

Conclusion:

- the remaining `UiCx` pressure is not just stale docs,
- it is partly a language/ergonomics pressure around concrete helper carriers.

### 3) This pressure does not invalidate `AppRenderContext<'a>`

`AppRenderContext<'a>` is still the correct app-facing name for:

- extracted helper functions,
- scaffold/shared helper APIs,
- and docs that teach the default lane.

What it does **not** prove is that the repo can now delete or fully demote every concrete `UiCx`
authoring surface.

Conclusion:

- keep the façade,
- but do not misread that façade landing as evidence that `UiCx` is now purely accidental debt.

## Implication for the next fearless refactor

Before attempting any broad `UiCx` demotion or `AppUi` `Deref` removal follow-on, the repo should
answer one explicit question:

- does Fret want one concrete app-facing extracted-helper context for closure-heavy authoring,
- or is it acceptable that the concrete helper story stays on compatibility `UiCx` while the
  named-function/default-doc story moves to `AppRenderContext<'a>`?

That question should be answered with a narrow proof slice, not another blind grep cleanup.
