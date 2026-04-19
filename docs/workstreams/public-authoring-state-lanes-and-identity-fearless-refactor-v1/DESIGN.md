# Public Authoring State Lanes and Identity Fearless Refactor v1 — Design

Status: Draft
Last updated: 2026-04-03

Related:

- `docs/adr/0028-declarative-elements-and-element-state.md`
- `docs/adr/0031-app-owned-models-and-leasing-updates.md`
- `docs/adr/0223-authoring-paradigm-app-owned-models-and-state-helpers-v1.md`
- `docs/adr/0308-view-authoring-runtime-and-hooks-v1.md`
- `docs/adr/0319-public-authoring-state-lanes-and-identity-contract-v1.md`
- `docs/authoring-golden-path-v2.md`
- `docs/examples/todo-app-golden-path.md`
- `docs/workstreams/local-state-architecture-fearless-refactor-v1/DESIGN.md`
- `docs/workstreams/local-state-facade-boundary-hardening-v1/DESIGN.md`
- `docs/workstreams/default-app-productization-fearless-refactor-v1/DESIGN.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/APP_FACING_RENDER_GAP_AUDIT_2026-04-03.md`
- `ecosystem/fret/src/view.rs`
- `crates/fret-ui/src/elements/cx.rs`

---

## 0) Why this lane exists

Fret already closed several state/authoring questions:

- app-owned models remain the base ownership model,
- the ecosystem view runtime remains the golden-path authoring surface,
- `LocalState<T>` remains model-backed,
- and `todo`/`simple-todo` now teach a `LocalState`-first first-contact story.

Those decisions were correct, but recent productization and diagnostics work exposed a narrower
remaining problem:

> the kernel is already identity-first, while the public state surface and some user-facing examples
> still teach, name, or implement state through a partially overlapping hook-style story.

Symptoms now visible in-tree:

- `LocalState<T>` is the default story, but the historical `AppUiRawStateExt::use_state*` naming
  survived long enough to blur the public lane.
- `ecosystem/fret/src/view.rs` still carries a facade wrapper around `raw_model_with(...)` even
  though model allocation itself should converge onto `ElementContext` slot/model primitives in the
  kernel.
- user-facing examples still mix:
  - `cx.state().local*`,
  - `LocalState::from_model(app.models_mut().insert(...))`,
  - and `clone_model()` bridge calls.
- the recent repeated-call warning bug showed that one part of the public facade was still thinking
  in terms of “same frame” rather than “same render evaluation / keyed identity”.

GPUI can tolerate a narrower public story because its authoring surface is more tightly coupled to
one stack. Fret cannot. Fret must support:

- default declarative apps,
- editor-grade/shared-model surfaces,
- reusable recipe/component layers,
- advanced manual assembly,
- and immediate-mode frontends on the same IR/runtime.

That means the repo now needs one explicit public state/identity contract rather than another round
of piecemeal wording fixes.

---

## 1) Problem statement

### 1.1 The public taxonomy is still blurrier than the implementation

The implementation already distinguishes several layers:

- kernel identity/state slots in `fret-ui`,
- ecosystem view/runtime sugar in `fret`,
- app-facing `LocalState<T>` helpers,
- and explicit `Model<T>` / `ModelStore` bridges.

The public surface still blurs them together often enough that new users can infer the wrong mental
model.

### 1.2 The kernel is identity-first, but the facade still leaks hook-shaped lore

`ElementContext` already exposes identity-centered primitives:

- `keyed(...)`,
- `slot_state(...)`,
- `keyed_slot_state(...)`,
- `local_model(...)`,
- `local_model_keyed(...)`,
- `model_for(...)`.

But the current `AppUi` raw-state helper path still layers another state-slot story on top of that
instead of clearly converging on it.

### 1.3 Diagnostics debt is a design-smell signal, not only a warning bug

The `render_pass_id` fix was the correct minimal repair for repeated-call diagnostics, but it also
proved something higher-level:

- the correct boundary is one render evaluation / keyed identity context,
- not a whole frame,
- and not an undocumented pseudo-hook contract that users are expected to infer.

That is a public design problem even if the field itself remains internal.

### 1.4 The migration debt is spread across real user-facing surfaces

This is not a purely internal cleanup. Current user-facing surfaces still carry old or mixed state
stories, including examples, demos, cookbook pages, and advanced reference docs.

Before open source, every such surface must end in one of three states:

1. migrated to the unique blessed path,
2. intentionally marked as advanced/reference with an explicit reason,
3. or removed/archived.

### 1.5 Lane sealing is still partly documentation-level, not type-level

The recent `render_pass_id` / repeated-call cleanup closed one real kernel/facade overlap, but the
next remaining gap is higher-level:

- `AppUi` now blocks helper-local slot/model APIs unless code opts into `cx.elements()`,
- but `AppUi` still reaches the broader `ElementContext` surface through `Deref`,
- and extracted helper functions still use `AppComponentCx`, which is currently a plain
  `ElementContext<App>` alias.

The current closure target for new default-path helper signatures is therefore:

- prefer `fret::app::AppRenderContext<'a>` for ordinary extracted helper functions,
- prefer `fret::app::AppRenderCx<'a>` when closure-local or inline helper families need a
  concrete context carrier,
- keep `RenderContextAccess<'a, App>` as the underlying generic capability,
- keep grouped app helper extensions (`UiCxActionsExt`, `UiCxDataExt`) valid on that lane,
- and treat `UiCx` only as the compatibility old-name alias while migration is still in flight.

That closes the previously open concrete-type ergonomics question without pretending the repo can
now delete every `AppComponentCx` usage immediately:

- `AppRenderContext<'a>` carries the named-helper teaching surface,
- `AppRenderCx<'a>` carries the concrete closure-local helper story on the same app-facing lane,
- and `AppComponentCx` remains only as the migration-era old name.

A compile audit on 2026-04-02 showed that blindly removing `AppUi`'s `Deref` is not yet the right
end state:

- `cargo check -p fret-examples --all-targets` surfaced 100 `AppComponentCx`/`into_element(...)` style
  mismatched-type failures, 31 direct `app` field reads, plus ordinary render-authoring helper
  lookups such as `theme_snapshot`, `container`, `text_props`, `flex`, and
  `environment_viewport_bounds`.
- `cargo check -p fret-cookbook --all-targets` surfaced the same pattern at smaller scale:
  17 `AppComponentCx`/`into_element(...)` mismatches, 11 direct `app` field reads, and ordinary helper calls
  such as `text`, `text_props`, `action_is_enabled`, and `pointer_region`.

This means the next structural task is not a blind `Deref` deletion.
The next structural task is separating:

- app-facing render-authoring sugar,
- from the raw component/internal `ElementContext` lane,
- for both `AppUi` and extracted helper surfaces.

---

## 2) Goals

### G1 — Freeze one explicit blessed path for ordinary app authors

The first-contact story must stay:

- `View`,
- `AppUi`,
- `cx.state()`,
- `cx.actions()`,
- `cx.data()`,
- `cx.effects()`,
- `LocalState<T>`,
- keyed identity for dynamic collections.

No first-contact doc/template/example should require raw `Model<T>` choreography just to explain
view-owned local state.

For driver/init/hybrid surfaces that intentionally sit outside the normal `View::render(...)`
entrypoint but still own view-local slots, the blessed posture is still `LocalState<T>`:

- prefer `cx.state().local*` when the surface already renders through `AppUi`,
- otherwise prefer `LocalState::new_in(models, value)` when init code owns `&mut ModelStore`,
- and reserve `LocalState::from_model(...)` for truly explicit raw-model wrapping.

### G2 — Make the explicit raw-model lane honest

Advanced code still needs raw model handles. That is not the problem.

The problem is generic naming and fuzzy framing.

This lane should leave the repo with a raw-model surface that:

- uses explicit `model` terminology,
- reads as advanced/manual on first contact,
- and no longer looks like a co-equal default hook API.

### G3 — Share one identity contract across declarative, recipes, and IMUI

All authoring frontends must teach the same identity rules:

- static single callsites may rely on callsite convenience,
- dynamic repeated subtrees must establish stable keyed identity,
- diagnostics should point users back to keyed identity, not pseudo-hook folklore.

### G4 — Converge the view runtime substrate onto kernel primitives

The ecosystem view runtime should be authoring sugar over kernel mechanism, not a semantically
parallel slot allocator.

This lane should narrow the gap between:

- `ecosystem/fret/src/view.rs`,
- and `crates/fret-ui/src/elements/cx.rs`.

### G5 — Migrate or classify every user-facing old surface

The workstream must include a repo migration plan, not just a target design.

That includes:

- docs,
- default templates,
- first-contact demos,
- cookbook examples,
- and broader `apps/fret-examples` user-visible proof surfaces.

---

## 3) Non-goals

This lane is not for:

- changing the model-backed storage contract behind `LocalState<T>`,
- switching to a plain-Rust/self-owned default local-state runtime,
- replacing `App` / `ModelStore` ownership with signals or hidden reactive graphs,
- making `render_pass_id` a public concept,
- deleting legitimate advanced `Model<T>` seams when shared ownership is still the point,
- or treating IMUI as a separate state architecture.

---

## 4) Inherited decisions (do not silently reopen)

### 4.1 App-owned models remain the ownership baseline

This lane inherits ADR 0031 and ADR 0223:

- shared mutable state remains app-owned,
- typed `Model<T>` handles remain the explicit shared-state contract,
- driver-boundary apply and explicit invalidation remain the framework stance.

### 4.2 The view runtime remains ecosystem-level

This lane inherits ADR 0308:

- the view/runtime facade stays in `ecosystem/fret`,
- kernel/public policy separation remains intentional,
- and the default authoring loop still composes existing kernel mechanisms.

### 4.3 `LocalState<T>` remains model-backed

This lane inherits the `O1` closeout from
`local-state-architecture-fearless-refactor-v1`:

- the storage model is not being reopened here,
- the question here is public contract clarity and implementation convergence.

### 4.4 IMUI still compiles to the same IR and identity rules

This lane inherits the current immediate-mode posture:

- IMUI remains an authoring frontend,
- it still targets the same declarative/runtime substrate,
- and it must not fork identity semantics or invent a separate local-state doctrine.

---

## 5) Core decisions for this lane

### D1 — Public state ownership is explained as three lanes

The repo should explicitly teach three lanes:

1. **Default app lane**
   - `LocalState<T>`
   - `cx.state()`
   - `cx.actions()`
   - `cx.data()`
   - `cx.effects()`
2. **Explicit model lane**
   - raw `Model<T>` handles
   - `ModelStore`
   - explicit bridge APIs such as `model()` / `clone_model()` / `*_in(...)`
   - advanced docs/imports only
3. **Component/internal identity lane**
   - `ElementContext::{keyed, scope, slot_state, local_model, model_for, ...}`
   - reusable recipe/helper/component internals
   - diagnostics and identity substrate

The public problem is not that these lanes exist.
The public problem is that they are not yet named and migrated consistently.

### D1.1 — `AppUi` and extracted helper functions should converge on one app-facing render lane

The default app lane is not only about state helpers.
It also includes ordinary render-authoring operations such as:

- app/theme/window reads,
- viewport/environment queries,
- builder composition,
- and helper-function extraction for child subtrees.

Therefore, removing `AppUi`'s `Deref` is only correct once the repo has an explicit app-facing
render-authoring surface for those ordinary operations.

The target is not:

- `AppUi` is narrow,
- while extracted helper functions still take raw `ElementContext<App>` through `AppComponentCx`.

The target is:

- one explicit app-facing render-authoring lane,
- one explicit component/internal identity lane,
- and deliberate escape hatches between them.

`todo_demo` now provides a useful release-facing proof set for this split:

- some low-level usage still belongs on explicit raw lanes (`shadcn::raw::button::ButtonStyle`,
  low-level icon helpers, state-style graphs),
- some low-level usage belongs on explicit but non-default environment/responsive lanes
  (`viewport_width_at_least(...)`, pointer-capability queries, hysteresis nouns),
- and the remaining pressure is now narrower than the first audit suggested:
  `Progress` and `ScrollArea` already fit the existing `.ui()` patch-builder lane, and the first
  helper-local follow-on now also lands on ecosystem recipe sugar rather than a new facade proxy:
  `todo_row(...)` uses `ui::hover_region(...)` and `ui::rich_text(...)`. The still-open pressure is
  now mostly deliberate raw style escape hatches and explicit environment/responsive helpers,
  rather than ordinary helper-local composition.

That means this lane must classify Todo-surfaced render pressure precisely instead of treating every
low-level noun as the same kind of problem.

### D2 — Generic `use_state` naming is no longer the target public raw-model contract

The advanced raw-model seam should use explicit model-oriented naming.

This workstream therefore assumes:

- the historical `use_state*` naming is transitional,
- the current target advanced seam is `AppUiRawModelExt::raw_model::<T>()`,
- the repo may rename/delete it directly pre-release,
- and documentation should stop treating “raw state hook” language as the stable north star.

The exact target spelling should be frozen in milestone `M1` before broad code migration begins.

### D3 — Identity is the contract; callsite convenience is only a convenience

Across all authoring frontends:

- stable keyed identity is the real rule,
- callsite-derived state is just a convenience for static positions,
- repeated collections/subtrees must establish keyed identity explicitly.

This is the main lesson to preserve from GPUI, but translated to Fret’s broader multi-frontend
surface.

### D4 — Diagnostics must align to evaluation scope, not frame scope

Repeated-call diagnostics should reason about one render evaluation / keyed subtree pass.

That means:

- the recent `render_pass_id` repair is correct as an internal fix,
- but the long-term contract is the evaluation boundary itself,
- not the field name or a public authoring token.

### D5 — The ecosystem facade should converge downward, not fork upward

Where possible, `AppUi` local/raw-state helpers should reduce to the existing kernel identity/model
primitives rather than duplicating a semantically parallel slot model.

If a facade helper still needs extra bookkeeping, that difference must be explicit and justified.

The same rule now applies to the render-authoring surface split:

- ordinary app-facing render sugar should become an explicit surface in `fret`,
- raw `ElementContext` should remain the component/internal substrate,
- and the repo should stop relying on `Deref` + type aliases as the main lane-sealing mechanism.

### D6 — Every user-facing old surface must be migrated or classified

This lane does not allow “quiet leftovers”.

For every user-visible surface with older state patterns, the repo must decide one of:

- migrate to the default blessed path,
- keep explicit model usage but mark the surface advanced/reference and explain why,
- or delete/archive the surface.

---

## 6) Execution batches

### B0 — Freeze the contract and the migration matrix

Deliverables:

- ADR 0319,
- this workstream,
- one migration matrix covering docs, templates, demos, cookbook examples, and advanced/reference
  surfaces.

### B1 — Freeze the target raw-model naming and compatibility posture

Before broad code edits:

- choose the explicit model-oriented replacement name,
- decide whether pre-release migration uses hard delete or a short-lived compatibility alias,
- and freeze the wording for default vs advanced lanes.

### B2 — Converge the runtime substrate

Audit and narrow the overlap between:

- `AppUi::raw_model_with(...)` / `local_with(...)`,
- and `ElementContext` slot/model primitives.

Goal:

- one identity model,
- one diagnostics story,
- fewer parallel slot systems.

This batch now also owns the next audit/freeze step for the render-authoring lane:

- quantify the `AppUi` `Deref` blast radius,
- classify which inherited operations are legitimate app-facing render sugar,
- classify Todo-surfaced low-level pressure into:
  - keep-raw escape hatch,
  - explicit non-default render lane,
  - missing app-facing sugar,
- and decide the correct target for extracted helper functions (`AppComponentCx` wrapper / trait surface /
  equivalent) before deleting implicit coercions.

### B3 — Migrate the default ladder and first-contact docs

This batch covers:

- docs,
- READMEs,
- templates,
- `todo_demo`,
- `simple_todo_demo`,
- cookbook first-contact examples,
- and the gates that protect them.

### B4 — Migrate or classify the broader user-facing example suite

This batch covers the remaining user-visible demos and cookbook examples:

- migrate simple/ordinary cases to the blessed path,
- split blocked cases by missing widget/bridge contracts,
- classify advanced/reference examples explicitly when they legitimately keep raw models.

### B5 — Delete residual ambiguity

Close the lane only when:

- compatibility names are gone or clearly temporary,
- diagnostics are internalized,
- source-policy tests protect the blessed path,
- and no user-facing surface is left in an unnamed mixed state.

---

## 7) Definition of done

This lane is done when all of the following are true:

1. first-contact docs/examples/templates teach only one default local-state story,
2. the advanced raw-model lane uses explicit model terminology,
3. kernel identity rules are explained consistently across declarative, recipe, and IMUI surfaces,
4. `AppUi` local-state substrate no longer duplicates kernel identity semantics without a justified
   reason,
5. the default app-facing render-authoring lane no longer depends on `AppUi` `Deref` or a raw
   `UiCx = ElementContext<App>` alias without an explicit justification,
6. `render_pass_id` remains internal-only,
7. and the migration matrix has no user-facing uncategorized leftovers.
