# Public Authoring State Lanes and Identity Fearless Refactor v1 — TODO

This file is the execution checklist for `DESIGN.md`.

Companion docs:

- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `MIGRATION_MATRIX.md`
- `APP_FACING_RENDER_GAP_AUDIT_2026-04-03.md`
- `API_WORKBENCH_FRAMEWORK_PRIORITY_AUDIT_2026-04-15.md`
- `docs/adr/0319-public-authoring-state-lanes-and-identity-contract-v1.md`

## M0 — Open the lane correctly

- [x] Add the workstream directory with:
  - [x] `DESIGN.md`
  - [x] `TODO.md`
  - [x] `MILESTONES.md`
  - [x] `MIGRATION_MATRIX.md`
- [x] Add ADR 0319 and connect the lane from the docs indices.
- [x] Record the initial migration classes:
  - [x] must migrate to the blessed path,
  - [x] may remain advanced/reference with explicit reason,
  - [x] blocked on lower-level widget/component contract work.
- [x] State explicitly that this lane does not reopen the model-backed `LocalState<T>` decision.

## M1 — Freeze the target public surface

- [ ] Freeze the default lane wording:
  - [ ] `LocalState<T>` is the only blessed first-contact local-state story,
  - [ ] keyed identity is the only taught dynamic-list/subtree rule.
  - [ ] freeze the render-authoring wording for `AppUi` / extracted helper surfaces so the repo
    can distinguish ordinary app-facing render sugar from the raw component/internal
    `ElementContext` lane.
  - [ ] freeze the Todo-surfaced render-gap classification from
    `APP_FACING_RENDER_GAP_AUDIT_2026-04-03.md`:
    - [ ] keep-raw escape hatches,
    - [ ] explicit non-default environment/responsive lane,
    - [ ] missing app-facing render sugar.
- [ ] Freeze the advanced raw-model lane wording:
  - [x] choose the explicit model-oriented replacement name (`AppUiRawModelExt::raw_model::<T>()`),
  - [x] decide whether pre-release migration uses hard delete or a short-lived compatibility alias (`hard delete` for the old name),
  - [x] remove “generic hook” framing from the first migrated public docs.
- [ ] Freeze the bridge/internal lane wording:
  - [x] classify `LocalState::{model, clone_model, *_in(...)}`
  - [x] classify `ElementContext::{slot_state, local_model, model_for, ...}`
  - [x] keep `AppUi` default-lane access to component/internal state helpers behind explicit
    `cx.elements()`
  - [x] keep explicit ownership language for helper-heavy/component/internal surfaces.
- [ ] Freeze the diagnostics posture:
  - [x] evaluation-boundary diagnostics stay internal,
  - [x] `render_pass_id` is not a public concept,
  - [ ] decide whether the internal field name should stay as-is or be renamed to a less GPU-loaded term later.

## M2 — Converge the runtime substrate

- [x] Audit the overlap between:
  - [x] `ecosystem/fret/src/view.rs::raw_model_with(...)`
  - [x] `ecosystem/fret/src/view.rs::local_with(...)`
  - [x] `crates/fret-ui/src/elements/cx.rs::{slot_state, keyed_slot_state, local_model, model_for}`
- [x] Decide the narrowest convergence strategy:
  - [x] direct reuse of kernel slot/model primitives where possible,
  - [x] explicit wrapper-only behavior where extra facade logic remains necessary.
- [x] Remove duplicated repeated-call bookkeeping if kernel-owned identity/evaluation state can own it.
- [x] Keep diagnostics keyed/evaluation-correct after convergence.
- [x] Seal the default `AppUi` lane so component/internal state helpers require explicit
  `cx.elements()`.
- [x] Land the first explicit render-authoring capability slice for extracted helpers:
  - [x] add `fret_ui::ElementContextAccess<'a, H>` as the narrow late-landing contract,
  - [x] move `fret-ui-kit` late-landing helpers onto that contract plus
    `IntoUiElementInExt::into_element_in(...)`,
  - [x] implement the contract for `AppUi`,
  - [x] reexport the capability on `fret::app` / `fret::app::prelude::*`,
  - [x] migrate the cookbook scaffold proof surface to the explicit capability lane,
  - [x] move retained table callback seams (`HeaderAccessoryAt` / `CellAt`) onto the same
    explicit capability lane so app-facing retained authoring helpers do not leak raw
    `ElementContext`.
- [ ] Add targeted tests for:
  - [x] repeated-call diagnostics,
  - [x] keyed child-scope correctness,
  - [x] raw-model compatibility behavior during the migration window.
  - [x] audit the `AppUi` `Deref` removal blast radius against examples/cookbook so the next
    structural step is evidence-based rather than guessed.
  - [x] decide and land the first correct type-level target for extracted helper functions before
    deleting implicit `AppUi -> ElementContext` coercion (`ElementContextAccess<'a, H>` plus
    `IntoUiElementInExt` as the current explicit capability lane).
  - [x] add source-policy tests for the explicit render-authoring capability lane (`fret::app`
    reexports, `AppUi` impl, cookbook scaffold proof surface).
  - [x] re-evaluate the highest-priority framework follow-on against the real
    `api_workbench_lite` consumer probe after the mutation teaching lane closed.
    Result: keep this lane active and prioritize `AppUi` / extracted-helper render-lane
    separation before reopening storage-model or mutation-owner debates. See
    `API_WORKBENCH_FRAMEWORK_PRIORITY_AUDIT_2026-04-15.md`.
  - [x] land the first real consumer-probe helper migration on the new render-authoring
    capability lane:
    `api_workbench_lite_demo` extracted helpers now accept
    `fret::app::RenderContextAccess<'a, App>`, `LocalState`/query/mutation read helpers work
    through that capability, and raw `ElementContext` access is spelled explicitly with
    `cx.elements()` only at late-landing builder boundaries.
  - [x] close the first provider late-builder escape-hatch gap on that lane:
    `DirectionProvider`, `TooltipProvider`, and `SidebarProvider` now expose explicit
    `*_in(...)` capability overloads, `SidebarInset` / `SidebarGroupContent` participate in the
    `IntoUiElement` lane, and `api_workbench_lite_demo` now uses `with_in(...)` /
    `into_element_in(...)` instead of spelling `cx.elements()`.
  - [x] tighten the cookbook shared page-shell proof so ordinary helper theme access stays on the
    app-facing render lane:
    `apps/fret-cookbook/src/scaffold.rs` now accepts
    `fret::app::RenderContextAccess<'a, App>` and uses `cx.theme_snapshot()` instead of reaching
    through `cx.elements().theme().snapshot()` just to read background tokens.
  - [x] extend the same capability story into editor recipe composites where app-facing helper
    extraction still needed a raw editor surface:
    `InspectorPanel`, `PropertyGroup`, and `PropertyGrid` now expose `into_element_in(...)`, and
    `editor_notes_demo` no longer needs `cx.elements()` to mount its shell-owned inspector rail.
  - [x] prove the same lane on a shell-owned product proof instead of a smaller editor-only demo:
    `workspace_shell_demo` now extracts `workspace_shell_editor_rail(...)` on
    `ElementContextAccess<'a, App>`, mounts `InspectorPanel` / `PropertyGroup` / `PropertyGrid`
    through `into_element_in(...)`, and keeps `workspace_shell_command_button(...)` as the
    explicit raw primitive exception instead of confusing the two lanes.
  - [x] stop teaching root render helpers to reach through `cx.app` just to read theme snapshots
    on the editor-notes proof family:
    `editor_notes_demo` and `editor_notes_device_shell_demo` now use the app-facing
    `cx.theme_snapshot()` helper, and their proof gates forbid
    `Theme::global(&*cx.app).snapshot()` from drifting back into those product surfaces.
  - [x] apply the same app-lane theme-read cleanup to default first-contact query/counter proofs
    without widening `fret::app::prelude::*`:
    `hello_counter_demo`, `query_demo`, and `query_async_tokio_demo` now import
    `fret::app::RenderContextAccess as _`, use `cx.theme_snapshot()` at the root render surface,
    and source-policy tests forbid `Theme::global(&*cx.app).snapshot()` on that batch.
  - [x] keep the promoted cookbook first-contact examples on the same render lane too:
    `hello_counter`, `simple_todo`, `simple_todo_v2_target`, and `data_table_basics` now import
    `fret::app::RenderContextAccess as _`, use `cx.theme_snapshot()` in their `AppUi` roots, and
    cookbook source-policy tests forbid `Theme::global(&*cx.app).snapshot()` on that promoted
    teaching set.
  - [ ] remove `AppUi` `Deref` only after ordinary render-authoring sugar has an explicit
    app-facing lane rather than falling back to `cx.elements()` everywhere.
  - [ ] audit the remaining Todo-surfaced render-authoring pressure before any future `Deref`
    removal:
    - [x] prove that the first ordinary app-composition slice (`Progress` / `ScrollArea`) can stay
      on the existing `.ui()` patch-builder lane instead of spelling `LayoutRefinement` directly,
    - [x] remove the shared footer-pill chrome/layout fragments so ordinary app composition no
      longer needs helper-returned `LayoutRefinement` / `ChromeRefinement` for that slice,
    - [x] land the first helper-local hover-region sugar replacement (`ui::hover_region(...)`)
      instead of spelling `HoverRegionProps` plus `cx.elements()` in app-facing helpers,
    - [x] land the first helper-local rich-text sugar replacement (`ui::rich_text(...)`) instead
      of spelling `StyledTextProps` / `styled_text_props(...)` in app-facing helpers,
    - [ ] explicit environment/responsive helpers that should stay off the default lane rather than
      being mistaken for raw debt.
  - [x] land the first justified app-facing render-sugar replacements without widening
    `fret::app::prelude::*` or collapsing the documented `raw` lane.

## M3 — Migrate first-contact docs, templates, and proof surfaces

- [x] Update docs and READMEs:
  - [x] `docs/README.md`
  - [x] `docs/first-hour.md`
  - [x] `docs/examples/README.md`
  - [x] `docs/examples/todo-app-golden-path.md`
  - [x] `docs/authoring-golden-path-v2.md`
  - [x] `docs/crate-usage-guide.md`
  - [x] `docs/fearless-refactoring.md`
  - [x] `ecosystem/fret/README.md`
- [x] Update default generated templates and their tests:
  - [x] `apps/fretboard/src/scaffold/templates.rs`
- [x] Re-verify the default runnable proof surfaces:
  - [x] `apps/fret-examples/src/todo_demo.rs`
  - [x] `apps/fret-examples/src/simple_todo_demo.rs`
  - [x] `apps/fret-cookbook/examples/simple_todo.rs`
- [x] Refresh source-policy gates so first-contact surfaces cannot drift back to raw-model or
  mixed-lane examples.
  - [x] raw-model naming/source-policy docs gate (`ecosystem/fret/tests/raw_state_advanced_surface_docs.rs`)
  - [x] scaffold template README gate (`apps/fretboard/src/scaffold/templates.rs`)

## M4 — Migrate or classify all user-facing examples

- [x] Process the migration matrix end-to-end rather than stopping at first-contact surfaces.
- [x] Migrate straightforward user-facing examples to the blessed path where the component contract
  already supports it, including the current queue:
  - [x] `apps/fret-examples/src/date_picker_demo.rs`
  - [x] `apps/fret-examples/src/ime_smoke_demo.rs`
  - [x] `apps/fret-examples/src/sonner_demo.rs`
  - [x] `apps/fret-examples/src/launcher_utility_window_demo.rs`
  - [x] `apps/fret-examples/src/launcher_utility_window_materials_demo.rs`
  - [x] `apps/fret-examples/src/emoji_conformance_demo.rs`
  - [x] `apps/fret-examples/src/async_playground_demo.rs`
  - [x] `apps/fret-examples/src/form_demo.rs`
  - [x] `apps/fret-examples/src/table_demo.rs`
  - [x] `apps/fret-examples/src/datatable_demo.rs`
  - [x] `apps/fret-cookbook/examples/data_table_basics.rs`
- [x] Split blocked surfaces into explicit follow-on dependency notes instead of leaving them
  ambiguous, including the current queue:
  - [x] `apps/fret-examples/src/markdown_demo.rs`
  - [x] `apps/fret-examples/src/drop_shadow_demo.rs`
  - [x] `apps/fret-cookbook/examples/drop_shadow_basics.rs`
  - [x] `apps/fret-cookbook/examples/overlay_basics.rs`
  - [x] `apps/fret-cookbook/examples/virtual_list_basics.rs`
  - [x] `apps/fret-examples/src/form_demo.rs` is no longer blocked after the dedicated form
    bridge contract landed (`FormRegistry` + `FormField`); the remaining `DatePicker::new_controllable(...)`
    seam stays an explicit controlled/uncontrolled helper, not the default app-lane path.
  - [x] the table-specific blocker is no longer open: `fret_ui_kit::declarative::table`,
    `fret_ui_shadcn::DataTable*`, and related helpers now accept a narrow
    `IntoTableStateModel` bridge instead of requiring `Model<TableState>` on the default lane.
  - [x] the overlay-close-specific blocker is no longer open: `DialogClose`, `SheetClose`, and
    `DrawerClose` now accept the same narrow `IntoBoolModel` bridge as their root/button peers,
    which lets cookbook overlay examples stay on `LocalState<bool>` without reopening a generic
    `IntoModel<T>` seam.
- [x] Explicitly classify advanced/reference surfaces that may legitimately retain raw-model
  bridges, including the current queue:
  - [x] `apps/fret-examples/src/custom_effect_v1_demo.rs`
  - [x] `apps/fret-examples/src/custom_effect_v2_demo.rs`
  - [x] `apps/fret-examples/src/custom_effect_v3_demo.rs`
  - [x] `apps/fret-examples/src/postprocess_theme_demo.rs`
  - [x] `apps/fret-examples/src/liquid_glass_demo.rs`
  - [x] `apps/fret-examples/src/genui_demo.rs`
  - [x] `apps/fret-examples/src/imui_floating_windows_demo.rs`
- [x] Make the advanced/reference classification visible in docs, example indices, and example
  comments where appropriate.

## M5 — Delete residual ambiguity and close cleanly

- [ ] Remove or hard-deprecate transitional generic raw-model names once the replacement surface is
  landed.
- [ ] Refresh docs/tests/source-gates around the chosen target interface.
- [ ] Ensure no first-contact doc/template/example still mentions the old generic raw-model API.
- [ ] Ensure every remaining raw-model example is explicitly advanced/reference and justified by
  ownership or widget-contract reality.
- [ ] Ensure the default render-authoring path no longer depends on implicit `AppUi` `Deref`
  inheritance once the narrowed `UiCx` / app-facing render surface is landed.
- [ ] Replace raw `UiCx = ElementContext<App>` as the default helper-story export once the
  narrowed extracted-helper render lane is ready.
- [ ] Record a closeout audit with:
  - [ ] final target interface,
  - [ ] migration results,
  - [ ] retained advanced seams,
  - [ ] render-authoring lane closure results,
  - [ ] and any intentionally deferred follow-on lanes.

## Standing rules

- [ ] No patch here may reopen the model-backed `LocalState<T>` architecture decision.
- [ ] No patch here may widen `fret::app::prelude::*` with raw-model seams.
- [ ] No patch here may turn `render_pass_id` into a public authoring concept.
- [ ] No user-facing example may remain in a mixed/uncategorized state once this lane closes.
