# ADR 0319: Public Authoring State Lanes and Identity Contract (v1)

Status: Accepted

## Context

Fret already locked the main architectural pieces relevant to UI state:

- GPUI-style declarative element rebuilding with stable identity and externalized cross-frame state
  (ADR 0028),
- app-owned shared state via `Model<T>` handles (ADR 0031),
- ecosystem-level authoring/runtime ergonomics rather than kernel-owned policy (ADR 0223),
- and the `View` + `AppUi` runtime as the default app-facing authoring loop (ADR 0308).

This is the correct architectural direction for Fret’s goals:

- general-purpose application UIs,
- editor-grade/shared-model surfaces,
- reusable recipe/component layers,
- and immediate-mode frontends that still target the same IR/runtime.

However, pre-release productization and diagnostics work exposed a narrower but hard-to-change
contract gap:

1. `LocalState<T>` is already the default first-contact story, but the public raw-model seam still
   survives under transitional naming/history that reads more default than it should.
2. `ecosystem/fret/src/view.rs` still carries a facade-local raw/local-state substrate even though
   `ElementContext` already has kernel identity/state primitives such as:
   - `keyed(...)`,
   - `slot_state(...)`,
   - `keyed_slot_state(...)`,
   - `local_model(...)`,
   - `local_model_keyed(...)`,
   - `model_for(...)`.
3. User-facing examples still mix:
   - the `LocalState`-first blessed path,
   - explicit `LocalState::from_model(...)`,
   - and `clone_model()` bridge usage.
4. The recent repeated-call warning bug proved that the right diagnostics boundary is one render
   evaluation / keyed subtree pass, not “same frame”.

GPUI’s narrower public story is not enough as-is for Fret, because Fret must support more than one
authoring frontend while keeping one identity contract.

## Decision

### D1 — Fret exposes three public state/identity lanes, not one blended story

The repo should explicitly teach and maintain three lanes:

1. **Default app lane**
   - `View`
   - `AppUi`
   - `cx.state()`
   - `cx.actions()`
   - `cx.data()`
   - `cx.effects()`
   - `LocalState<T>`
   - component/internal identity/state helpers stay behind explicit `cx.elements()`
2. **Explicit model lane**
   - raw `Model<T>` handles
   - `ModelStore`
   - advanced/manual bridge APIs such as `model()`, `clone_model()`, and `*_in(...)`
3. **Component/internal identity lane**
   - `ElementContext` identity and slot/model primitives
   - helper/component/recipe internals
   - diagnostics substrate

No first-contact doc/template/example may blend these lanes without explicitly naming the escape
hatch and why it exists.

### D1.1 — `AppUi` keeps the component/internal lane explicit

`AppUi` may still expose broad render-authoring sugar, but the component/internal state helpers are
not part of the default lane.

Therefore:

- `slot_state(...)`,
- `local_model(...)`,
- `local_model_keyed(...)`,
- `state_for(...)`,
- `model_for(...)`,
- and lower-level identity helpers such as `scope(...)` / `named(...)`

must require an explicit `cx.elements()` escape hatch from app-facing code.

This ADR does not require deleting broad render-authoring sugar in one step.

That separation is now part of the shipped contract: `AppUi` no longer exposes a `Deref` bridge
to `ElementContext`, and ordinary builder/helper APIs are expected to close over explicit
app-facing capability lanes (`AppRenderContext<'a>`, `AppRenderCx<'a>`, `into_element_in(...)`)
rather than forcing default-path code through `cx.elements()`.

Low-level environment/responsive reads are a separate explicit secondary lane on
`fret::env::{...}`. Keep query-configuration nouns such as `ContainerQueryHysteresis`,
`ViewportQueryHysteresis`, and `ViewportOrientation` on that same explicit lane rather than
mistaking them for raw component/internal `ElementContext` debt or widening the default prelude.

Small explicit runtime/command helpers may still live on the ordinary `AppUi` lane when they do
not reopen the broader component/internal substrate. In particular, frame-driven progression such
as `request_animation_frame()` and explicitly imported command-gating reads/dispatch
(`fret::actions::ElementCommandGatingExt`) belong on the app-facing lane rather than depending on
implicit `AppUi -> ElementContext` inheritance.
Committed geometry-query helpers may also live on that app-facing lane when they only observe or
create named layout-query regions rather than exposing raw identity/state primitives. In
particular, `layout_query_bounds(...)`, `layout_query_region(...)`, and
`layout_query_region_with_id(...)` are part of the ordinary app authoring surface, not the
component/internal state lane. Likewise, direct late-builder roots that only need to land
authored trigger/content children should expose explicit capability-first `*_in(...)` entry points
instead of forcing app-facing code back through raw `ElementContext` or implicit `AppUi ->
ElementContext` inheritance.
Conversely, low-level raw text/leaf authoring that still depends on `ElementContext` primitives
such as `text_props(...)` or direct raw `into_element(...)` landing remains on the explicit
`cx.elements()` lane until the repo commits a narrower app-facing alternative.
Manual `render_root_with_app_ui(...)` proofs that still own a raw layout/build phase should keep
tracked reads on `AppUi` first and then enter `cx.elements()` explicitly for that phase rather
than widening the default façade. Mixed app-facing/interop roots may instead keep ordinary
late-landing on the existing capability-first lane (`into_element_in(...)`) and reserve
`cx.elements()` only for the genuinely raw interop seam.

### D1.2 — Extracted helper surfaces should converge on the same narrowed render-authoring lane

Extracted child-builder helpers on the default app surface should converge on the same narrowed
render-authoring lane as `AppUi`.

During the current migration window:

- the default teaching surface for new helper signatures is
  `fret::app::AppRenderContext<'a>`,
- the default concrete carrier for closure-local or inline helper families is
  `fret::app::AppRenderCx<'a>`,
- app-hosted component/snippet helpers that deliberately target the default `fret::app::App`
  host should prefer `AppComponentCx<'a>`,
- `RenderContextAccess<'a, App>` remains the underlying generic capability,
- grouped app-facing helper namespaces such as `AppRenderActionsExt` / `AppRenderDataExt` may continue to
  power that lane,
- and `UiCx` should be treated only as the compatibility old-name alias when an older helper still
  has not migrated to `AppRenderContext<'a>`, `AppRenderCx<'a>`, or `AppComponentCx<'a>`.

Named helper families that only need grouped state/query access plus ordinary late-landing should
migrate from `UiCx` or `AppComponentCx` to `fret::app::AppRenderContext<'a>` rather than widening
`AppUi` again or silently reclassifying the whole surface as raw-owner authoring.

The long-term target is not “`AppUi` is narrow but `AppComponentCx` remains a raw `ElementContext` alias”.
The long-term target is one explicit app-facing render-authoring lane plus one explicit
component/internal identity lane.
In the shipped default prelude, `AppComponentCx` is no longer reexported; reaching for it requires explicit
import / advanced-surface intent.

### D2 — `LocalState<T>` remains the only blessed first-contact local-state story

The default app lane remains:

- `LocalState<T>` for view-owned local mutable state,
- keyed identity for dynamic repeated subtrees,
- and explicit `Model<T>` graphs only when ownership is genuinely shared or runtime-owned.

Raw model handles are not co-equal with `LocalState<T>` in first-contact materials.
First-contact docs/templates/examples should not teach `LocalState::from_model(...)`,
`clone_model()`, or raw `Model<T>` choreography on this lane.

When a manual driver/init/hybrid surface needs a `LocalState<T>` handle before the first `AppUi`
render, prefer `LocalState::new_in(models, value)` rather than allocating a raw `Model<T>` first
and then wrapping it with `LocalState::from_model(...)`.

### D3 — Generic hook-style raw-model naming is not the long-term public contract

The public advanced raw-model lane must use explicit `model` terminology rather than generic
hook-style `state` naming.

Therefore:

- the historical `AppUiRawStateExt::use_state*` naming is transitional,
- the target advanced seam is `AppUiRawModelExt::raw_model::<T>()`,
- it is a migration target rather than the target interface,
- and the repo may rename/delete it directly pre-release once the replacement surface is ready.

### D4 — Identity is the real contract across declarative views, recipes, and IMUI

Across all authoring frontends:

- callsite convenience is valid only for static single positions,
- dynamic repeated subtrees must establish stable keyed identity,
- and diagnostics should guide users toward keyed identity rather than undocumented hook-order
  folklore.

On the default app lane, dynamic lists/subtrees should teach `ui::for_each_keyed(...)`,
`ui::for_each_keyed_with_cx(...)`, or `ui.id(key, ...)` as the identity-bearing default.
Unkeyed iteration remains an explicit exception only for static, order-stable collections.

### D5 — The kernel owns the identity/model substrate

The mechanism vocabulary lives in kernel/runtime primitives such as:

- `ElementContext::{keyed, scope, slot_state, keyed_slot_state, local_model, local_model_keyed, model_for}`.

Ecosystem authoring/runtime layers should converge onto those primitives rather than maintain a
semantically parallel slot/state substrate unless a remaining difference is explicit and justified.

### D6 — Evaluation tokens remain internal diagnostics machinery

Fields such as `render_pass_id` are internal diagnostics substrate only.

The current `render_pass_id` name is acceptable while it remains private repeated-call
bookkeeping. It may still be replaced by a different internal mechanism later, but it must not
become a public authoring concept or a user-visible escape hatch.

## Consequences

### Benefits

- Fret keeps the correct GPUI-inspired skeleton:
  - app-owned models,
  - stable identity,
  - explicit invalidation.
- The public state surface becomes more honest for first-contact users.
- Declarative views, recipes, and IMUI keep one identity contract instead of drifting into
  frontend-specific rules.
- Diagnostics and substrate convergence become reviewable, rather than hidden inside facade-local
  behavior.

### Costs / Risks

- The repo must migrate a non-trivial set of user-facing examples and docs.
- Some widget/component contracts may need smaller follow-on cleanups before specific examples can
  migrate cleanly.
- Pre-release renaming/deletion of generic raw-model APIs will cause intentionally breaking in-tree
  changes.

## Migration posture

This ADR is executed through:

- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/DESIGN.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/TODO.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/MILESTONES.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/MIGRATION_MATRIX.md`

Every user-facing surface must end in one of these states:

1. migrated to the blessed path,
2. explicitly marked advanced/reference with rationale,
3. or deleted/archived.

## Alternatives considered

- **Keep the current mixed public surface**
  - Rejected because it leaves first-contact users learning two overlapping state doctrines.
- **Move the default state story to a plain-Rust/self-owned runtime**
  - Rejected because this ADR does not reopen the already-closed model-backed `LocalState<T>`
    decision.
- **Copy GPUI’s narrower public `use_state` story directly**
  - Rejected because Fret must support more authoring fronts and therefore needs a clearer
    lane-based public contract.
