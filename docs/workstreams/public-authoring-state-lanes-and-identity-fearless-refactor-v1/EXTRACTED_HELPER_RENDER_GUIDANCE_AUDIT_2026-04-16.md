# Extracted Helper Render Guidance Audit — 2026-04-16

Status: wording + source-policy audit for the active public authoring lane

Related:

- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/DESIGN.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/TODO.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/MILESTONES.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/API_WORKBENCH_FRAMEWORK_PRIORITY_AUDIT_2026-04-15.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/APP_UI_DEREF_PRESSURE_CLASSIFICATION_AUDIT_2026-04-15.md`
- `docs/adr/0319-public-authoring-state-lanes-and-identity-contract-v1.md`
- `docs/first-hour.md`
- `docs/authoring-golden-path-v2.md`
- `docs/examples/todo-app-golden-path.md`
- `docs/crate-usage-guide.md`
- `ecosystem/fret/src/lib.rs`
- `ecosystem/fret/tests/render_authoring_capability_surface.rs`
- `ecosystem/fret/tests/raw_state_advanced_surface_docs.rs`

## Assumptions First

### A1) The current capability lane is already strong enough to freeze default guidance

Confidence: Confident

Evidence:

- `ecosystem/fret/src/view.rs`
- `ecosystem/fret/tests/render_authoring_capability_surface.rs`
- `apps/fret-cookbook/src/scaffold.rs`

If wrong:

- this audit would be freezing wording before the repo had any explicit extracted-helper contract
  to point users toward.

### A2) Introducing a second concrete helper wrapper now would be premature

Confidence: Likely

Evidence:

- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/API_WORKBENCH_FRAMEWORK_PRIORITY_AUDIT_2026-04-15.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/APP_UI_DEREF_PRESSURE_CLASSIFICATION_AUDIT_2026-04-15.md`
- `ecosystem/fret/src/lib.rs`

If wrong:

- the repo would still need another public helper-type pivot after this wording freeze, which
  would mean this slice chose the wrong abstraction boundary.

### A3) The remaining `UiCx` usage is mostly migration stock, not proof that `UiCx` should stay the
default teaching surface

Confidence: Confident

Evidence:

- `apps/fret-examples/src/lib.rs`
- `apps/fret-cookbook/src/lib.rs`
- `apps/fret-ui-gallery/src/ui/snippets/**`

If wrong:

- the repo would be hiding a still-unmet default helper capability behind wording rather than
  exposing the missing surface honestly.

## Question

Before removing `AppUi`'s implicit `Deref`, what should the repo teach as the default extracted
helper signature on the app-facing render lane?

## Verdict

Freeze the wording now:

- `AppUi` remains the default root render lane.
- New default-path extracted helpers should prefer
  `fret::app::RenderContextAccess<'a, App>`.
- `UiCx` is reclassified as a compatibility raw alias for `ElementContext<App>`, not as the
  default helper story.
- `UiCxActionsExt` and `UiCxDataExt` keep their current names because they still carry grouped
  app-facing behavior on that capability lane and on compatibility helper code.

This is the right slice because it closes the teaching ambiguity without pretending the repo is
ready for a blind `UiCx` delete or a fresh helper-wrapper type.

## Findings

### 1) The repo already has one real extracted-helper capability lane

`RenderContextAccess<'a, App>` plus `ElementContextAccess<'a, App>` already cover:

- explicit app/window/theme/environment reads,
- grouped data/action helper extension traits,
- and late landing via `IntoUiElementInExt::into_element_in(...)`.

Conclusion:

- default guidance can now point at an explicit lane instead of a raw alias.

### 2) `UiCx` still exists for compatibility, but that is no longer the same thing as default
guidance

`ecosystem/fret/src/lib.rs` still exports `UiCx = ElementContext<App>`.

That export remains necessary because first-party examples and gallery snippets still contain a
large migration stock of helper signatures that intentionally or historically use the raw shape.

Conclusion:

- keep the alias for now,
- but stop teaching it as the first thing a new app author should reach for.

### 3) The right next constraint is wording + source policy, not a second large API move

The current evidence does **not** justify:

- deleting `UiCx`,
- deleting `AppUi` `Deref`,
- or inventing a new helper wrapper type without stronger callsite evidence.

The current evidence **does** justify:

- freezing default docs around `RenderContextAccess<'a, App>`,
- reclassifying `UiCx` as compatibility raw alias wording,
- and leaving the remaining migration stock explicit and reviewable.

## Landed Slice

This audit lands the wording freeze in:

- `docs/first-hour.md`
- `docs/authoring-golden-path-v2.md`
- `docs/examples/todo-app-golden-path.md`
- `docs/crate-usage-guide.md`
- `docs/adr/0319-public-authoring-state-lanes-and-identity-contract-v1.md`
- `ecosystem/fret/src/lib.rs`

and locks it with source-policy tests in:

- `ecosystem/fret/tests/render_authoring_capability_surface.rs`
- `ecosystem/fret/tests/raw_state_advanced_surface_docs.rs`
- `ecosystem/fret/tests/crate_usage_grouped_query_surface.rs`

## Repro, Gate, Evidence

Repro target:

- read `docs/first-hour.md` and `docs/examples/todo-app-golden-path.md` as a first-contact app
  author looking for how to type an extracted helper signature

Primary gates:

- `cargo nextest run -p fret --test render_authoring_capability_surface --test raw_state_advanced_surface_docs --test crate_usage_grouped_query_surface`

What these prove:

- the `fret` facade classifies `UiCx` as a compatibility raw alias,
- default docs no longer teach `&mut UiCx<'_>` as the new-helper default,
- and grouped query/action guidance still points extracted helpers at the app-facing capability
  lane instead of reopening raw `ElementContext` or query-client shell code.
