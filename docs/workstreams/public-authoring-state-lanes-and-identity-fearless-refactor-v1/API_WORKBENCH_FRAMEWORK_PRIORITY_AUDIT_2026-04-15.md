# API Workbench Framework Priority Audit — 2026-04-15

Status: priority audit for the active public authoring lane

Related:

- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/DESIGN.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/TODO.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/MILESTONES.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/MIGRATION_MATRIX.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/APP_FACING_RENDER_GAP_AUDIT_2026-04-03.md`
- `docs/adr/0319-public-authoring-state-lanes-and-identity-contract-v1.md`
- `docs/audits/postman-like-api-client-first-contact.md`
- `apps/fret-examples/src/api_workbench_lite_demo.rs`
- `ecosystem/fret/src/view.rs`
- `ecosystem/fret/src/lib.rs`
- `docs/first-hour.md`

## Question

After closing the narrow mutation teaching follow-on, what is the highest-priority remaining
framework refactor for first-contact Fret users building a real tool app such as a Postman-like API
workbench?

## Verdict

Keep `public-authoring-state-lanes-and-identity-fearless-refactor-v1` as the primary active
framework lane.

The next correct fearless refactor is:

- separate the default app-facing render-authoring lane from the raw `ElementContext` lane for both
  `AppUi` and extracted helper functions,
- then remove the remaining implicit `AppUi -> ElementContext` inheritance,
- without reopening the closed mutation-owner split or the closed `LocalState<T>` storage decision.

This is the highest-leverage framework change because it is now the main place where the public
surface still teaches two different mental models at once:

- `cx.state()` / `cx.actions()` / `cx.data()` / `cx.effects()` as the blessed default lane,
- but also raw `ElementContext` inheritance through `AppUi` and `UiCx`.

## Findings

### 1) `AppUi` now says the right thing, but still leaks the raw lane structurally

`ecosystem/fret/src/view.rs` already exposes the intended default surface:

- grouped default namespaces (`state`, `actions`, `data`, `effects`),
- explicit `elements()` as the component/internal escape hatch,
- explicit `app()`, `app_mut()`, `window_id()`, and `environment_viewport_bounds(...)`,
- and lane-sealing barriers for `slot_state(...)`, `local_model(...)`, `model_for(...)`, and
  related raw identity helpers.

However, the same file still ends with the temporary compatibility bridge:

- `impl Deref for AppUi<'_, '_, _> -> ElementContext<'_, _>`
- `impl DerefMut for AppUi<'_, '_, _> -> ElementContext<'_, _>`

That means the public contract is still partly enforced by wording and source policy rather than by
the type shape itself.

Conclusion:

- the repo has already finished the "what the default lane should be" design work,
- but it has not yet finished the type-level closure that makes the default lane hard to misuse.

### 2) `UiCx` still teaches the old raw helper story

`ecosystem/fret/src/lib.rs` still exports:

- `pub type UiCx<'a> = fret_ui::ElementContext<'a, crate::app::App>;`

This keeps extracted helper functions on a raw `ElementContext<App>` alias even though ADR 0319 says
the long-term target is one explicit app-facing render-authoring lane plus one explicit
component/internal identity lane.

The onboarding/docs story still reflects that transitional state:

- `docs/first-hour.md` tells users to give a helper `&mut UiCx<'_>` when it needs runtime access.

Conclusion:

- first-contact docs still route ordinary helper extraction through a raw alias,
- so the repo keeps re-teaching the very boundary that the default `AppUi` surface is trying to
  narrow.

### 3) The real `api_workbench_lite` probe still falls through the leak

The Postman-like workbench probe now uses the correct submit owner and the correct default
state/data surfaces, but the render body still shows the unresolved lane leak:

- `WorkbenchLocals::new(cx)` uses `cx.state().local_init(...)`, which is correct,
- but the same render function also reaches `cx.app.global::<HistoryDbGlobal>()` and
  `Theme::global(&*cx.app)`,
- which only works because `AppUi` still inherits `ElementContext` members through `Deref`.

This matters because the probe is exactly the kind of real, non-toy app that Fret claims to
support:

- toolbar actions,
- async requests,
- response views,
- persisted history,
- settings/environment state,
- and a shell-like multi-pane layout.

Conclusion:

- the biggest remaining framework friction is not "how do mutations work?" anymore,
- it is "which context surface am I actually writing against when the app becomes real?".

### 4) Workspace-scale evidence says this is still a systemic lane problem

A quick repo scan on 2026-04-15 found:

- `UiCx<'_>` occurrences:
  - `apps/fret-examples/src`: 166
  - `apps/fret-ui-gallery/src`: 1454
  - `apps/fret-cookbook/examples`: 13
- `cx.app` occurrences:
  - `apps/fret-examples/src`: 217
  - `apps/fret-ui-gallery/src`: 228
  - `apps/fret-cookbook/examples`: 48

These counts do not mean every occurrence is wrong.
They do prove the current public/default helper lane is still structurally intertwined with raw
context inheritance across the repo's own proof surfaces.

Conclusion:

- this is large enough to justify keeping the current active framework lane focused on render-lane
  closure rather than opening a fresh state-model redesign.

### 5) Other probe findings remain real, but they are not the top framework refactor

The same workbench probe also exposed:

- command-scope confusion for tool-app chrome actions,
- `locals_with((...))` pressure once a real submit flow coordinates many `LocalState<T>` slots,
- and first-contact productization gaps for shell + async + settings composition.

Those findings still matter, but they do not outrank the render-lane split:

- command-scope pressure is currently a docs/example/teaching issue first,
- `locals_with((...))` pressure is a narrower ergonomics follow-on and can be revisited after the
  default render-authoring lane is explicit,
- shell/async/settings composition is still largely a productization/example lane rather than a
  kernel state rewrite.

Conclusion:

- do not mix these issues into another broad "rewrite state again" workstream.

## Required next slices

### A) Land one explicit extracted-helper render context

Add the real type-level target for ordinary helper functions so the default app surface no longer
depends on:

- raw `UiCx = ElementContext<App>`,
- or implicit `AppUi` `Deref`.

The target should keep:

- ordinary render-authoring sugar,
- explicit app/window/theme/environment access,
- and the late-landing capability contract (`ElementContextAccess` / `IntoUiElementInExt`) where
  appropriate,

while keeping raw identity/state helpers on the explicit `cx.elements()` lane only.

### B) Migrate default-path helper surfaces onto that lane

Migrate the ordinary helper-facing proof surfaces first:

- onboarding/default docs,
- cookbook/default examples,
- default-friendly `fret-examples`,
- and app-facing gallery snippets where they still model the default lane.

The purpose is not to remove all `UiCx`-shaped helper signatures in one batch.
The purpose is to stop teaching raw helper context as the default app-facing story.

### C) Remove `AppUi` `Deref` only after the helper lane is real

Do not repeat the earlier blind `Deref` removal attempt.

The correct sequence remains:

1. define the narrowed helper lane,
2. migrate ordinary helper surfaces,
3. keep advanced/component/internal paths explicit through `cx.elements()`,
4. only then remove `AppUi` `Deref`.

### D) Re-evaluate secondary ergonomics after lane closure

After the render-authoring lane is explicit, re-check:

- whether `locals_with((...))` still needs a broader capture surface for real tool apps,
- whether command-scope authoring needs better default affordances,
- and whether any remaining `cx.app` or `UiCx` usage is actually legitimate advanced/component
  code rather than default-lane drift.

## Non-goals reaffirmed

This audit does **not** justify:

- reopening the closed mutation-owner split,
- turning Sonner or query helpers into submit owners,
- reopening the model-backed `LocalState<T>` storage decision,
- or widening `fret::app::prelude::*` with raw `ElementContext` state/model helpers.

## Decision

Treat the current framework priority as:

- keep `public-authoring-state-lanes-and-identity-fearless-refactor-v1` active,
- make `AppUi` / extracted-helper render-lane separation the next major refactor inside it,
- and keep `locals_with((...))`, command-scope guidance, and broader workbench productization as
  follow-on pressure sets unless new evidence proves the render-lane split is no longer the main
  blocker.
