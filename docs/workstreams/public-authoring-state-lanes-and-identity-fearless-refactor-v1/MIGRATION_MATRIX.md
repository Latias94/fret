# Public Authoring State Lanes and Identity Fearless Refactor v1 — Migration Matrix

Last updated: 2026-04-02

This matrix exists to prevent the lane from stopping at abstract design.

Every user-visible surface should end in one of these states:

- **Migrate**: move to the unique blessed `LocalState`-first path.
- **Advanced**: retain explicit model/bridge usage, but label the surface advanced/reference and
  explain why.
- **Blocked**: migration depends on another lower-level widget/component/runtime contract and must
  not be hand-waved as “done”.

## A) First-contact and canonical docs — Must migrate

| Surface | Current drift | Target state | Batch | Notes |
| --- | --- | --- | --- | --- |
| `docs/README.md` | docs index now points to the explicit `LocalState`-first blessed path and the current `AppUiRawModelExt::raw_model::<T>()` advanced seam | Migrate | M3 | Done 2026-04-02. |
| `docs/first-hour.md` | onboarding guide now keeps the default path narrow while naming `AppUiRawModelExt::raw_model::<T>()` as an explicit advanced choice | Migrate | M3 | Done 2026-04-02. |
| `docs/examples/README.md` | examples index now keeps the ladder/classification wording aligned with one default path and one explicit advanced raw-model escape hatch | Migrate | M3 | Done 2026-04-02. |
| `docs/examples/todo-app-golden-path.md` | todo golden path now explicitly teaches one LocalState-first path and keeps raw `Model<T>` handles on the advanced lane | Migrate | M3 | Done 2026-04-02. |
| `docs/authoring-golden-path-v2.md` | main public wording anchor now states that the LocalState-first lane is the only blessed first-contact story | Migrate | M3 | Done 2026-04-02. |
| `docs/crate-usage-guide.md` | crate guidance now names the single advanced raw-model seam explicitly instead of leaving mixed-lane wording implied | Migrate | M3 | Done 2026-04-02. |
| `docs/fearless-refactoring.md` | advanced guidance already points to `AppUiRawModelExt::raw_model::<T>()` and remains aligned after the naming cleanup audit | Migrate | M3 | Done 2026-04-02. |
| `ecosystem/fret/README.md` | package README now states the same LocalState-first default path and explicit advanced raw-model seam as the canonical docs | Migrate | M3 | Done 2026-04-02. |
| `apps/fretboard/src/scaffold/templates.rs` | template readmes/tests now lock the one blessed starter story and forbid raw-model seam drift back into `hello` / `simple-todo` / `todo` | Migrate | M3 | Done 2026-04-02. |
| `apps/fret-examples/src/todo_demo.rs` | app-grade proof surface reverified against the final naming/gate story (`todo_demo_prefers_default_app_surface`) | Migrate | M3 | Done 2026-04-02. |
| `apps/fret-examples/src/simple_todo_demo.rs` | second-rung proof surface reverified against the final naming/gate story (`simple_todo_demo_prefers_default_app_surface`) | Migrate | M3 | Done 2026-04-02. |
| `apps/fret-cookbook/examples/simple_todo.rs` | cookbook onboarding proof surface reverified on the same blessed path (`onboarding_examples_use_the_new_app_surface`) | Migrate | M3 | Done 2026-04-02. |

## B) Straightforward user-facing examples — Migrate

| Surface | Current drift | Target state | Batch | Notes |
| --- | --- | --- | --- | --- |
| `apps/fret-examples/src/date_picker_demo.rs` | migrated to a render-time `DatePickerDemoLocals::new(cx)` bundle backed by `cx.state().local_init(...)` | Migrate | M4 | Done 2026-04-02. |
| `apps/fret-examples/src/ime_smoke_demo.rs` | migrated to a render-time `ImeSmokeLocals::new(cx)` bundle; text widgets now bind `&LocalState<String>` directly | Migrate | M4 | Done 2026-04-02. |
| `apps/fret-examples/src/sonner_demo.rs` | migrated to a render-time `SonnerDemoLocals::new(cx)` bundle | Migrate | M4 | Done 2026-04-02. |
| `apps/fret-examples/src/launcher_utility_window_demo.rs` | migrated init-time locals to `LocalState::new_in(app.models_mut(), ...)` | Migrate | M4 | Done 2026-04-02. |
| `apps/fret-examples/src/launcher_utility_window_materials_demo.rs` | migrated init-time status local to `LocalState::new_in(app.models_mut(), ...)` | Migrate | M4 | Done 2026-04-02. |
| `apps/fret-examples/src/emoji_conformance_demo.rs` | migrated to a render-time `EmojiConformanceLocals::new(cx)` bundle backed by `cx.state().local_init(...)` | Migrate | M4 | Done 2026-04-02. |
| `apps/fret-examples/src/async_playground_demo.rs` | migrated init-time config locals to `LocalState::new_in(...)`; the surface now stays on `LocalState` + grouped query helpers without raw-model widget bridges | Migrate | M4 | Done 2026-04-02. |
| `apps/fret-examples/src/form_demo.rs` | migrated init-time locals to `LocalState::new_in(...)`; `FormRegistry` and `FormField` now accept narrow local-state bridges, and the demo uses `DatePicker::new(&open, &month, &selected)` on the blessed lane | Migrate | M4 | Done 2026-04-02. |
| `apps/fret-examples/src/table_demo.rs` | migrated init-time and render-time table state to `LocalState<TableState>` after the table family landed a narrow `IntoTableStateModel` bridge | Migrate | M4 | Done 2026-04-02. |
| `apps/fret-examples/src/datatable_demo.rs` | migrated to `LocalState<TableState>` and now exercises `DataTable`, `DataTableToolbar`, and `DataTablePagination` on the bridged default lane | Migrate | M4 | Done 2026-04-02. |
| `apps/fret-cookbook/examples/data_table_basics.rs` | cookbook data-table example now keeps `TableState` on `LocalState<TableState>` while staying on the default `shadcn::DataTable*` authoring surface | Migrate | M4 | Done 2026-04-02. |
| `apps/fret-examples/src/components_gallery.rs` | retained table demo path no longer writes body-cell callbacks directly against raw `ElementContext<'_, App>`; the gallery now lands retained cells through an explicit `ElementContextAccess` helper | Migrate | M4 | Done 2026-04-02. |
| `apps/fret-examples/src/markdown_demo.rs` | markdown toolbar switches now bind `&LocalState<bool>` directly instead of reopening `clone_model()` | Migrate | M4 | Done 2026-04-02. |
| `apps/fret-examples/src/drop_shadow_demo.rs` | drop-shadow toggles now bind `&LocalState<bool>` directly on the advanced demo surface | Migrate | M4 | Done 2026-04-02. |
| `apps/fret-cookbook/examples/drop_shadow_basics.rs` | cookbook drop-shadow example now keeps its control toggles on the same `LocalState<bool>` bridge path | Migrate | M4 | Done 2026-04-02. |
| `apps/fret-cookbook/examples/overlay_basics.rs` | overlay example now stays on `LocalState<bool>` end-to-end after `DialogClose` / `SheetClose` / `DrawerClose` adopted the narrow `IntoBoolModel` bridge | Migrate | M4 | Done 2026-04-02. |
| `apps/fret-cookbook/examples/virtual_list_basics.rs` | virtual-list control state now stays on `LocalState` even inside imperative `models::<...>` handlers by using `LocalState::{value_in_or,value_in_or_default}` instead of `clone_model()` | Migrate | M4 | Done 2026-04-02. |

## C) User-facing surfaces blocked on lower-level contract cleanup — Blocked

| Surface | Current drift | Target state | Batch | Notes |
| --- | --- | --- | --- | --- |
| None in the 2026-04-02 M4 queue | The previous markdown/drop-shadow/overlay/virtual-list queue is now migrated | Blocked | M4 | Keep this section for newly discovered lower-level contract blockers only. |

## D) Advanced/reference examples — Keep explicit, but classify

| Surface | Current drift | Target state | Batch | Notes |
| --- | --- | --- | --- | --- |
| `apps/fret-examples/src/custom_effect_v1_demo.rs` | now explicitly labeled advanced/reference; explicit effect/runtime ownership remains intentional | Advanced | M4 | Done 2026-04-02. Source gate and example docs classify it as a renderer/effect reference surface. |
| `apps/fret-examples/src/custom_effect_v2_demo.rs` | now explicitly labeled advanced/reference; explicit effect/runtime ownership remains intentional | Advanced | M4 | Done 2026-04-02. Source gate and example docs classify it as a renderer/effect reference surface. |
| `apps/fret-examples/src/custom_effect_v3_demo.rs` | now explicitly labeled advanced/reference; explicit effect/runtime ownership remains intentional | Advanced | M4 | Done 2026-04-02. Source gate and example docs classify it as a renderer/effect + diagnostics reference surface. |
| `apps/fret-examples/src/postprocess_theme_demo.rs` | now explicitly labeled advanced/reference; explicit renderer/theme bridge ownership remains intentional | Advanced | M4 | Done 2026-04-02. Source gate and example docs classify it as a renderer/theme reference surface. |
| `apps/fret-examples/src/liquid_glass_demo.rs` | now explicitly labeled advanced/reference; explicit renderer capability and effect/control ownership remain intentional | Advanced | M4 | Done 2026-04-02. Source gate and example docs classify it as a renderer capability reference surface. |
| `apps/fret-examples/src/genui_demo.rs` | now explicitly labeled advanced/reference; generator/editor integration still intentionally keeps explicit model ownership | Advanced | M4 | Done 2026-04-02. Source gate and example docs classify it as a GenUI integration reference surface. |
| `apps/fret-examples/src/imui_floating_windows_demo.rs` | now explicitly labeled advanced/reference; immediate-mode overlap/floating proof remains intentional | Advanced | M4 | Done 2026-04-02. Source gate and example docs classify it as an IMUI floating-window reference surface. |

## E) Internal/public-surface convergence work — Must migrate

| Surface | Current drift | Target state | Batch | Notes |
| --- | --- | --- | --- | --- |
| `ecosystem/fret/src/view.rs` | `raw_model_with(...)` still carries a facade-local diagnostics shell even after model allocation converges onto kernel primitives | Migrate | M2 | Keep converging until only the justified wrapper behavior remains. |
| `ecosystem/fret/src/view.rs` | `AppUi` still relies on `Deref` to inherit ordinary render-authoring sugar and ambient app/window access from `ElementContext`, but it now also implements the narrower `ElementContextAccess<'a, H>` capability for late-landing helper surfaces | Migrate | M2/M5 | 2026-04-02 compile audit: removing `Deref` surfaced 100 mismatched `UiCx`/`into_element(...)` style failures in `fret-examples`, 31 direct `app` field reads, and helper lookups such as `theme_snapshot`, `container`, `text_props`, `flex`, and `environment_viewport_bounds`. The first capability slice is now landed and source-gated; full `Deref` removal remains follow-on work. |
| `ecosystem/fret-ui-kit/src/declarative/table.rs` | retained table callback seams previously forced app-facing retained cell/header helpers to accept raw `ElementContext<'_, H>` | Migrate | M2/M4 | Done 2026-04-02 for retained table surfaces: `HeaderAccessoryAt` and `CellAt` now accept `ElementContextAccess<'a, H>`, and `components_gallery` proves the app-facing usage. |
| `crates/fret-ui/src/elements/cx.rs` | internal `render_pass_id` exists only as diagnostics substrate | Migrate | M2 | Keep internal-only; possibly rename internally later if clearer. |
| `ecosystem/fret/src/lib.rs` | `UiCx<'a>` is still a raw `ElementContext<App>` alias rather than a narrowed extracted-helper render surface, but `fret::app::ElementContextAccess` and `IntoUiElementInExt` are now the blessed minimal explicit helper lane | Migrate | M2/M5 | Long-term target is one explicit app-facing render-authoring lane for both `AppUi` and extracted helper functions, not a split where `AppUi` is narrow but `UiCx` remains raw. The cookbook scaffold proof surface already uses the explicit capability lane. |
| `ecosystem/fret/src/lib.rs` | advanced-lane wording must track the final raw-model rename/compat story | Migrate | M1/M3 | Keep export policy aligned with docs and tests. |
| `apps/fret-examples/src/lib.rs` | source-policy tests now cover both the blessed default lane and the explicit advanced/reference roster | Migrate | M3/M5 | Done 2026-04-02. `advanced_reference_demos_are_explicitly_classified` locks the roster and rationale markers. |
| `ecosystem/fret/tests/raw_state_advanced_surface_docs.rs` | tests currently lock the old raw-model naming | Migrate | M3/M5 | Update when the final naming is frozen. |

## Matrix rule

This matrix is complete only when:

- every row has reached its target state,
- or a row has been removed because the owning surface was deleted/archived,
- and no new user-facing mixed-lane surface has been added without entering this file.
