# App-Facing Render Gap Audit — 2026-04-03

Status: Initial classification audit

Related:

- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/DESIGN.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/TODO.md`
- `docs/workstreams/default-app-productization-fearless-refactor-v1/RECIPE_PROMOTION_AUDIT_2026-04-02.md`
- `apps/fret-examples/src/todo_demo.rs`

## Scope

This audit answers one narrower follow-on question from the Todo/default-path productization work:

- when `todo_demo` still needs low-level render-authoring nouns, which ones are:
  - legitimate `raw` escape hatches,
  - explicit but non-default app lanes,
  - or signals that the app-facing render-authoring surface is still missing a narrower helper?

This is **not** a recipe-promotion audit and **not** a request to widen the default prelude.

## Primary evidence

- `apps/fret-examples/src/todo_demo.rs`
- `docs/crate-usage-guide.md`
- `docs/shadcn-declarative-progress.md`
- `docs/adr/0319-public-authoring-state-lanes-and-identity-contract-v1.md`

## Findings

### 1) Some `raw` usage is correct and should stay `raw`

Current `todo_demo` usage that still fits the documented raw lane:

- `shadcn::raw::button::ButtonStyle`
- low-level `shadcn::raw::icon::*` element helpers
- state-style graph nouns such as `WidgetStateProperty` / `WidgetStates` when the caller is
  intentionally building a module-local style override rather than using a recipe-owned variant

Why these should stay explicit:

- they are module-level styling internals, not first-contact app vocabulary,
- promoting them into the default lane would widen discovery instead of clarifying it,
- and the current default-path docs already define this family as a deliberate escape hatch rather
  than a peer path.

Todo-specific evidence:

- `apps/fret-examples/src/todo_demo.rs` (`subtle_destructive_button_style(...)`)

### 2) Some low-level nouns are not `raw`, but should remain explicit non-default lanes

Current `todo_demo` usage that should stay explicit without becoming default-prelude vocabulary:

- `ViewportQueryHysteresis`
- `viewport_width_at_least(...)`
- `primary_pointer_can_hover(...)`

Why these should stay explicit:

- they belong to environment/responsive policy, not baseline app authoring,
- they are more advanced than the `hello` / `simple-todo` lane, but they are not component-family
  raw internals either,
- and moving them into the default lane would blur the repo's current “small blessed path, explicit
  extra lanes” story.

Preferred target posture:

- keep them on an explicit environment/responsive import lane,
- do not reframe them as `raw`,
- do not widen `fret::app::prelude::*` with them.

### 3) `todo_demo` still exposes real app-facing render-sugar gaps

The remaining gap is not “too much raw” in the abstract.
It is narrower:

- ordinary app code still drops to lower-level render-authoring nouns for mundane work,
- and extracted helpers still fall through to the component/internal lane too early.

Current pressure points:

- ordinary layout/style refinement in app code still imports `LayoutRefinement` for common
  operations such as width fill, max height, and fixed control height,
- extracted app helpers such as `todo_row(...)` still require `ElementContextAccess`,
  `HoverRegionProps`, `StyledTextProps`, and `cx.elements()` to express hover-region and styled-text
  assembly.

Why this matters:

- these are not all justification for new public recipes,
- but they do show that the repo still lacks one fully explicit app-facing render-authoring lane
  covering “ordinary app helper extraction” without falling through to raw `ElementContext` too
  early.

### 4) This audit does not reopen the closed keep-local recipe verdicts

This audit does **not** change the current recipe-promotion answers:

- responsive page shell stays app-owned,
- Todo/card header stays app-owned,
- hover-reveal destructive row stays app-owned for now.

Those productization decisions remain correct.

The follow-on question is different:

- if the repo later narrows the app-facing render lane, those app-owned helpers should become easier
  to write without accidentally reopening raw/component-internal surfaces.

## Classification summary

### Keep `raw`

- `shadcn::raw::button::ButtonStyle`
- low-level `shadcn::raw::icon::*`
- explicit state-style graph nouns when intentionally authoring a module-local style override

### Keep explicit, but off the default lane

- viewport/environment query helpers
- pointer-capability queries
- responsive hysteresis/configuration nouns

### Follow-on app-facing render-sugar audit/cleanup

- ordinary layout/style refinement for app-owned component composition
- extracted helper support for hover-region assembly
- extracted helper support for styled text / rich inline text composition
- narrowing the ordinary `AppUi` / extracted-helper surface before any future `Deref` removal

## Outcome

No new top-level workstream is required.

The correct follow-on is to extend the existing
`public-authoring-state-lanes-and-identity-fearless-refactor-v1` lane with:

1. an explicit classification for these Todo-surfaced render gaps,
2. a narrow app-facing render-sugar follow-up under the current `AppUi` / `UiCx` split work,
3. and a standing rule that this cleanup must not widen the default prelude or collapse the
   deliberate `raw` lane.
