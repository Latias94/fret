# App-Facing Render Gap Audit — 2026-04-03

Status: Initial classification audit with first proof correction

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

- the first ordinary app-composition proof has already closed:
  `todo_demo` now expresses `Progress` width/fill/rounding and `ScrollArea` width/max-height through
  the existing `.ui()` patch-builder lane instead of spelling `LayoutRefinement` directly,
- the first helper-local hover/text proof has also closed:
  `todo_row(...)` now uses `ui::hover_region(...)` and `ui::rich_text(...)` instead of spelling
  `HoverRegionProps`, `StyledTextProps`, or `cx.elements()` directly,
- `todo_demo` no longer keeps shared footer-pill chrome/layout fragments on direct refinement
  helper returns; the remaining direct refinement spelling is now narrower app-local style escape
  hatch usage (`subtle_destructive_button_style(...)` and the dashed empty-state border style).

Why this matters:

- this was not a missing widget contract for `Progress` or `ScrollArea`; the `.ui()` lane already
  existed and the proof surface had simply not migrated to it,
- this was also not a reason to widen `fret::app::prelude::*`; the correct first follow-on was to
  add narrower `fret-ui-kit::ui` render sugar for hover regions and attributed text,
- these findings are not all justification for new public recipes,
- but the remaining helper-local pressure still shows that the repo lacks one fully explicit
  app-facing render-authoring lane covering “ordinary app helper extraction” without falling
  through to raw `ElementContext` too early.

Evidence from the first proof correction:

- `apps/fret-examples/src/todo_demo.rs`
- `apps/fret-examples/src/lib.rs` (`todo_demo_prefers_default_app_surface`)
- `ecosystem/fret-ui-kit/src/ui.rs`
- `ecosystem/fret-ui-kit/src/ui_builder.rs`
- `ecosystem/fret-ui-shadcn/src/progress.rs`
- `ecosystem/fret-ui-shadcn/src/scroll_area.rs`
- `cargo nextest run -p fret-ui-kit rich_text_builder_renders_styled_text_element hover_region_builder_renders_hover_region_element`
- `cargo nextest run -p fret-ui-shadcn --lib progress_supports_ui_patch_builder_lane scroll_area_supports_ui_patch_builder_lane`
- `cargo nextest run -p fret-examples --lib todo_demo_prefers_default_app_surface`
- `cargo check -p fret-demo --bin todo_demo`

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

- explicit environment/responsive helpers should stay off the default lane rather than being
  mistaken for raw debt
- app-local raw style escape hatches should stay explicit unless a real reusable recipe emerges
- narrowing the ordinary `AppUi` / extracted-helper surface before any future `Deref` removal

## Outcome

No new top-level workstream is required.

The correct follow-on is to extend the existing
`public-authoring-state-lanes-and-identity-fearless-refactor-v1` lane with:

1. an explicit classification for these Todo-surfaced render gaps,
2. a narrow app-facing render-sugar follow-up under the current `AppUi` / `UiCx` split work,
3. and a standing rule that this cleanup must not widen the default prelude or collapse the
   deliberate `raw` lane.
