# Public Authoring State Lanes and Identity Fearless Refactor v1 — Milestones

Last updated: 2026-04-15

Related:

- Design: `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/DESIGN.md`
- TODO: `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/TODO.md`
- Migration matrix: `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/MIGRATION_MATRIX.md`
- App-facing render gap audit: `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/APP_FACING_RENDER_GAP_AUDIT_2026-04-03.md`
- API workbench framework priority audit: `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/API_WORKBENCH_FRAMEWORK_PRIORITY_AUDIT_2026-04-15.md`
- AppUi root accessor audit: `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/APP_UI_ROOT_ACCESSOR_AUDIT_2026-04-15.md`
- AppUi Deref pressure classification audit: `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/APP_UI_DEREF_PRESSURE_CLASSIFICATION_AUDIT_2026-04-15.md`
- UI assets capability adapter audit: `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/UI_ASSETS_CAPABILITY_ADAPTER_AUDIT_2026-04-15.md`
- Query grouped maintenance surface audit: `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/QUERY_GROUPED_MAINTENANCE_SURFACE_AUDIT_2026-04-15.md`
- Cookbook theme context owner audit: `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/COOKBOOK_THEME_CONTEXT_OWNER_AUDIT_2026-04-15.md`
- Model store render read owner audit: `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/MODEL_STORE_RENDER_READ_OWNER_AUDIT_2026-04-15.md`
- ADR 0319: `docs/adr/0319-public-authoring-state-lanes-and-identity-contract-v1.md`

---

## Current status snapshot (as of 2026-04-15)

- **M0**: Met
  - the lane now exists,
  - ADR 0319 is written,
  - and the migration matrix explicitly includes user-facing examples.
- **M1**: In progress
  - target raw-model naming is frozen,
  - bridge/internal lane wording is now source-gated in `ecosystem/fret/src/view.rs` and
    `crates/fret-ui/src/declarative/tests/identity.rs`,
  - but the remaining default-lane wording cleanup, the `AppUi` / `UiCx` render-authoring lane
    wording, the Todo-surfaced render-gap classification, and the internal `render_pass_id`
    naming decision are still open.
- **M2**: In progress
  - kernel/facade substrate convergence is partially landed and the default `AppUi` lane now
    requires explicit `elements()` escape-hatch access for component/internal state helpers.
  - repeated-call diagnostics bookkeeping is now kernel-owned through
    `ElementContext::note_repeated_call_in_render_evaluation_at(...)`, so `raw_model_with(...)`
    no longer carries a facade-local render-pass tracker.
  - the first explicit render-authoring capability slice is now landed:
    `fret_ui::ElementContextAccess<'a, H>` exists as the narrow late-landing contract,
    `fret-ui-kit` late-landing helpers and `IntoUiElementInExt::into_element_in(...)` accept it,
    `AppUi` implements it, and `fret::app` / `fret::app::prelude::*` reexport the capability
    needed by extracted helper surfaces.
  - retained table callback seams are now on the same capability lane where they touch app-facing
    authoring:
    `fret_ui_kit::declarative::table::{HeaderAccessoryAt, CellAt}` no longer require raw
    `ElementContext<'_, H>`, `fret_ui_shadcn::DataTable` keeps the raw surface only as an
    internal adapter boundary, and `components_gallery` now demonstrates the explicit helper path.
  - keyed child-scope correctness is now covered directly in
    `crates/fret-ui/src/declarative/tests/identity.rs` for helper-local `slot_state(...)` and
    `local_model(...)` under keyed child scopes.
  - a direct compile audit now shows that deleting `AppUi`'s `Deref` is not yet the right closeout:
    `cargo check -p fret-examples --all-targets` surfaced 100 `UiCx`/`into_element(...)`
    mismatched-type failures, 31 direct `app` field reads, and ordinary render-authoring helpers
    such as `theme_snapshot`, `container`, `text_props`, `flex`, and
    `environment_viewport_bounds`; `cargo check -p fret-cookbook --all-targets` showed the same
    pattern at smaller scale.
  - the next remaining structural gap is therefore explicit render-authoring lane separation for
    `AppUi` and extracted helper surfaces, not a blind `Deref` deletion.
  - the 2026-04-15 `api_workbench_lite` priority audit now freezes that judgment as the current
    framework priority:
    keep this lane active, treat `AppUi` / extracted-helper render-lane separation as the next
    major refactor, and do not reopen mutation ownership or `LocalState<T>` storage from that
    evidence alone.
  - `APP_FACING_RENDER_GAP_AUDIT_2026-04-03.md` now classifies the current Todo-derived pressure
    into:
    - keep-raw escape hatches,
    - explicit environment/responsive lanes that should stay non-default,
    - and missing app-facing render sugar for ordinary app helper extraction.
  - the first proof correction from that audit is now landed:
    `todo_demo` no longer spells `LayoutRefinement` directly for `Progress` or `ScrollArea`,
    dedicated smoke tests in `fret-ui-shadcn` prove that both widgets already support the `.ui()`
    patch-builder lane, and a second proof correction now lands the first app-facing render-sugar
    replacements in `fret-ui-kit::ui`: `todo_row(...)` uses `ui::hover_region(...)` and
    `ui::rich_text(...)` instead of spelling `HoverRegionProps`, `StyledTextProps`, or
    `cx.elements()` directly. A follow-on cleanup also removes the shared footer-pill
    `ChromeRefinement` / `LayoutRefinement` helpers from `todo_demo`, so the remaining pressure is
    now mostly deliberate raw style escape hatches plus explicit environment/responsive helpers.
  - the cookbook scaffold proof surface and dedicated source-policy tests now lock this minimal
    capability lane so future cleanup can continue without regressing to implicit `Deref`.
  - the first real consumer probe is now also on that lane:
    `api_workbench_lite_demo` extracted helpers accept
    `fret::app::RenderContextAccess<'a, App>`, `LocalState::{layout_value, paint_value,
    layout_read_ref, paint_read_ref}`, `QueryHandleReadLayoutExt`, `MutationHandleReadLayoutExt`,
    and `UiCxDataExt`/`UiCxActionsExt` now work through the same capability, and the remaining raw
    `ElementContext` requirements are explicit `cx.elements()` escape hatches at
    `into_element(...)` / `with(...)` late-landing boundaries instead of implicit `AppUi` `Deref`
    inheritance.
  - the cookbook first-contact scaffold now also stays on that same app-facing render lane for
    theme reads:
    `apps/fret-cookbook/src/scaffold.rs` uses
    `fret::app::RenderContextAccess<'a, App>` plus `cx.theme_snapshot()` for page-shell
    background tokens instead of reopening the raw element surface through
    `cx.elements().theme().snapshot()`.
  - the next closure step on that same probe is now landed:
    `DirectionProvider`, `TooltipProvider`, and `SidebarProvider` expose explicit capability
    overloads (`into_element_in(...)`, `with_in(...)`, `with_elements_in(...)`) for helper-heavy
    render surfaces, `SidebarInset` / `SidebarGroupContent` now participate in the same
    `IntoUiElement` lane as the rest of the sidebar facade, and the
    `api_workbench_lite_demo` proof no longer spells `cx.elements()` at all on the extracted
    helper path.
  - the same lane now extends into editor-grade ecosystem composites:
    `fret-ui-editor::{InspectorPanel, PropertyGroup, PropertyGrid}` expose
    `into_element_in(...)` capability overloads, a new `fret-ui-editor` source-policy gate locks
    those signatures, and `editor_notes_demo` now mounts its inspector rail through the explicit
    helper lane instead of dropping to `cx.elements()`.
  - the same capability-first helper story now also holds on the broader workspace-shell product
    proof:
    `workspace_shell_demo` extracts `workspace_shell_editor_rail(...)` on
    `ElementContextAccess<'a, App>`, mounts the right-rail `InspectorPanel` / `PropertyGroup` /
    `PropertyGrid` tree through `into_element_in(...)`, and keeps
    `workspace_shell_command_button(...)` as the intentional raw primitive/helper exception
    instead of mixing that low-level case into the editor-composite lane.
  - the editor-notes product proofs now also keep root theme reads on the app-facing render lane:
    `editor_notes_demo` and `editor_notes_device_shell_demo` use `cx.theme_snapshot()` instead of
    `Theme::global(&*cx.app).snapshot()`, and their proof gates now lock that root-render
    authoring choice so those examples stop teaching raw host/theme access for ordinary app
    surfaces.
  - the same root-render theme-read correction now also covers the default counter/query teaching
    batch without widening `fret::app::prelude::*`:
    `hello_counter_demo`, `query_demo`, and `query_async_tokio_demo` import
    `fret::app::RenderContextAccess as _`, use `cx.theme_snapshot()` in their `AppUi` render
    roots, and source-policy gates now forbid `Theme::global(&*cx.app).snapshot()` on those
    first-contact/default-facing examples.
  - the promoted cookbook first-contact set now teaches the same root-render lane:
    `hello_counter`, `simple_todo`, `simple_todo_v2_target`, and `data_table_basics` import
    `fret::app::RenderContextAccess as _`, use `cx.theme_snapshot()` in their `AppUi` render
    roots, and cookbook source-policy gates now forbid `Theme::global(&*cx.app).snapshot()` on
    that promoted teaching batch.
  - the same root-render theme-read correction now also covers the async/runtime and manual-root
    proof surfaces without widening `fret::app::prelude::*`:
    `async_playground_demo` and `ime_smoke_demo` import
    `fret::app::RenderContextAccess as _`, use `cx.theme_snapshot()` at their root render
    surfaces, and `apps/fret-examples/src/lib.rs` source-policy gates now forbid
    `Theme::global(&*cx.app).snapshot()` on both proofs.
  - selected advanced/runtime demos now also stop teaching direct app/theme global reads when the
    code path is only ordinary chrome or inspector theme token access:
    `embedded_viewport_demo`, `custom_effect_v1_demo`, `custom_effect_v2_demo`,
    `markdown_demo`, and `genui_demo` use `cx.theme_snapshot()` on those ordinary surfaces, while
    renderer/theme-bridge proofs such as `postprocess_theme_demo` and `liquid_glass_demo` stay on
    their explicit advanced lane instead of being folded into the default cleanup batch.
  - low-level interop/direct-leaf examples now also read theme snapshots from their owned element
    context instead of reopening host-global theme access:
    `external_texture_imports_demo`, `external_texture_imports_web_demo`,
    `external_video_imports_avf_demo`, and `external_video_imports_mf_demo` use
    `cx.theme().snapshot()` on their `ElementContext` roots, and the low-level interop
    source-policy gate now forbids `Theme::global(&*cx.app).snapshot()` on that batch.
  - the web Custom Effect V2 product-validation family now follows the same context-owned theme
    rule for ordinary chrome and inspector reads:
    `custom_effect_v2_web_demo`, `custom_effect_v2_identity_web_demo`,
    `custom_effect_v2_lut_web_demo`, and `custom_effect_v2_glass_chrome_web_demo` use
    `cx.theme().snapshot()` on their `ElementContext` helpers/root chrome, and the grouped-helper
    source-policy gates now forbid `Theme::global(&*cx.app).snapshot()` on that batch.
  - the remaining selected `ElementContext` product/stress proofs now also stay on context-owned
    theme reads:
    `canvas_datagrid_stress_demo` and `imui_interaction_showcase_demo` use
    `cx.theme().snapshot()`, and source-policy gates now keep those surfaces out of the
    host-global theme lane too.
  - the only remaining host-global theme reads in `apps/fret-examples/src` are now explicit
    renderer/theme-bridge proofs, not cleanup leftovers:
    `postprocess_theme_demo` and `liquid_glass_demo` keep
    `Theme::global(&*cx.app).snapshot()`, and source-policy gates now lock that fact as the
    intentional tail of this theme-read closure work.
  - `components_gallery` now also distinguishes the two lanes it really owns instead of mixing
    them implicitly:
    ordinary theme reads use `cx.theme_snapshot()` / `cx.theme()`, while the gallery keeps
    explicit raw state access only where it is intentionally probing lower-level retained table
    seams (`cx.elements().slot_state(...)`, `local_model_keyed(...)`).
  - ordinary root/helper seams now also stop teaching `AppUi` call sites to drop immediately to
    `cx.elements()` when the escape hatch belongs inside the helper boundary instead:
    `embedded_viewport_demo` and `hello_world_compare_demo` take
    `fret::app::ElementContextAccess<'a, KernelApp>` for their extracted page/root helpers,
    `assets_demo` takes `fret::app::RenderContextAccess<'a, KernelApp>` at `render_view(...)`
    entry and keeps `ThemeSnapshot` on that helper lane, and `image_heavy_memory_demo` takes
    `fret::app::ElementContextAccess<'a, KernelApp>` at `render_view(...)` entry; source-policy
    gates now forbid `embedded_viewport_page(cx.elements(), ...)`,
    `hello_world_compare_root(cx.elements(), ...)`, and `render_view(cx.elements())` from
    drifting back into this batch.
  - the same boundary closure now also covers the editor-grade IMUI proof wrapper without
    flattening its advanced helper story:
    `imui_editor_proof_demo` takes
    `fret::app::ElementContextAccess<'a, KernelApp>` at the outer `render_view(...)` boundary,
    leaves its internal `UiCx` helper family on the advanced lane, and source-policy gates now
    forbid `render_view(cx.elements())` from drifting back into that root.
  - the 2026-04-15 advanced-entry audit has now landed as the next framework slice rather than
    staying a planning note:
    `fret_imui::{imui_in, imui_raw_in, imui_build_in}`,
    `fret_chart::declarative::chart_canvas_panel_in`,
    `fret_node::ui::declarative::node_graph_surface_in`, and
    `NodeGraphSurfaceBinding::observe_in(...)` now provide the capability-first caller-facing
    adapters for the remaining advanced public entry surfaces.
  - first-party proof callsites that only needed those entry adapters are now migrated too:
    immediate-mode teaching surfaces (`imui_hello_demo`, `imui_floating_windows_demo`,
    `imui_response_signals_demo`, `imui_shadcn_adapter_demo`, `imui_node_graph_demo`) now lock
    `fret_imui::imui_in(cx, |ui| {`, while `chart_declarative_demo` and `node_graph_demo` keep
    their advanced direct-leaf ownership but no longer spell raw `cx.elements()` at the `AppUi`
    root just to enter the surface.
  - the post-adapter follow-on is now also landed and explicitly classified:
    selected `AppUi` roots (`embedded_viewport_demo`, `async_playground_demo`,
    `markdown_demo`, `postprocess_theme_demo`, `genui_demo`, and
    `hello_world_compare_demo`) now use explicit `AppUi::{app, app_mut, window_id}` accessors
    instead of `cx.app` / `cx.window` through the `Deref` bridge when they only need host/window
    access, and `apps/fret-examples/src/lib.rs` locks that batch with a source-policy gate.
  - the new 2026-04-15 root-accessor audit also records the current boundary clearly:
    this cleanup removes real compatibility syntax debt, but it still does **not** make direct
    `AppUi` `Deref` deletion correct yet because the remaining pressure is now concentrated in
    intentionally advanced/raw surfaces rather than one uniform default-lane mistake.
  - a second 2026-04-15 follow-on audit now makes that remaining pressure more precise:
    the selected root batch is effectively closed, `markdown_demo` no longer teaches the
    trait-UFCS variant at the root, and the remaining direct `cx.app` / `cx.window` grep surface
    is now classified across advanced renderer/effect owners, docking/multi-window owners, and
    helper-local raw seams rather than one more broad `AppUi` root cleanup queue.
  - the next helper-local follow-on now also has a first landed ecosystem-level adapter result:
    `fret-ui-assets::ui` owns `use_rgba8_image_state_in(...)`, `image_stats_in(...)`, and
    `svg_stats_in(...)`, first-party proof surfaces (`assets_demo`, `markdown_demo`, and cookbook
    `assets_reload_epoch_basics`) now teach that lane, and the direct `fret-ui-assets`
    dependency in `fret-examples` now enables the `ui` feature explicitly so the intended
    capability surface is available without widening the `fret` facade.
  - the grouped app-facing query maintenance lane is now closed over the remaining first-party
    proof seams too:
    `fret::view::{AppUiData, UiCxData}` now own `query_snapshot()`,
    `query_snapshot_entry(...)`, and `cancel_query(...)`, `async_playground_demo` uses grouped
    invalidation/cancel/snapshot helpers instead of raw `with_query_client(...)`, and
    `markdown_demo` now uses grouped namespace invalidation too. The owner split is now recorded
    explicitly in `QUERY_GROUPED_MAINTENANCE_SURFACE_AUDIT_2026-04-15.md`:
    grouped UI maintenance stays on `fret`, while raw `with_query_client(...)` remains the pure
    app/driver or lower-level authoring seam.
  - the cookbook theme-read owner split is now explicit too:
    cookbook `AppUi` / `UiCx` ordinary token reads use `cx.theme_snapshot()`, while direct-leaf
    `ElementContext` interop roots use `cx.theme().snapshot()`. The repo no longer teaches
    `Theme::global(&*cx.app).snapshot()` on cookbook examples, cookbook source-policy tests lock
    both owner classes, and `COOKBOOK_THEME_CONTEXT_OWNER_AUDIT_2026-04-15.md` records the split.
  - the remaining render-time raw model-store read tail is now split by owner rather than grep
    bucket:
    cookbook `virtual_list_basics` uses tracked builder `.revision()` reads on the default
    `AppUi` lane, direct-leaf/manual render roots (`custom_effect_v2_*_web_demo`,
    `external_*_imports*_demo`) use `cx.data().selector_model_layout(...)` for render-time
    visibility reads, and pure driver/app loops such as `record_engine_frame(...)` keep raw
    `app.models().read(...)`. `MODEL_STORE_RENDER_READ_OWNER_AUDIT_2026-04-15.md` records why
    this slice did not add a new framework API.
  - the same owner split now also covers the selected stress render roots that previously read
    tracked state before entering the render tree:
    `virtual_list_stress_demo` and `canvas_datagrid_stress_demo` now use
    `cx.data().selector_model_layout(...)` for their render-time state bags, while their driver
    loops remain unchanged. The render-time raw `app.models()` tail is therefore further reduced
    to deliberate driver/retained owners instead of mixed stress-demo drift.
  - the advanced/reference GenUI message lane now makes its remaining store ownership explicit at
    the app surface too:
    `GenUiState` owns helper methods for queue clearing, queued invocation reads, and
    `LocalState` reads consumed by `handle_msg(...)`, so the demo no longer mixes repeated raw
    `app.models()` calls through message handling just to reach its own state. This narrows the
    remaining grep tail further without misclassifying the GenUI integration surface as default
    render-lane authoring.
- **M3**: Met
  - first-contact docs, scaffold tests, and Todo proof surfaces now all teach the same
    LocalState-first default lane and the same explicit `AppUiRawModelExt::raw_model::<T>()`
    advanced seam.
- **M4**: Met
  - the user-facing migration matrix now has explicit dispositions for all tracked surfaces:
    the straightforward example queue is migrated, the previous blocker queue is cleared, and the
    remaining high-ceiling surfaces are explicitly classified as advanced/reference in source docs,
    `docs/examples/README.md`, and `apps/fret-examples/src/lib.rs` source-policy gates.
  - a first migration batch is now landed:
    `date_picker_demo`, `ime_smoke_demo`, `sonner_demo`, `launcher_utility_window_demo`,
    `launcher_utility_window_materials_demo`, `emoji_conformance_demo`,
    `async_playground_demo`, `form_demo`, `table_demo`, `datatable_demo`, cookbook
    `data_table_basics`, `drop_shadow_basics`, `overlay_basics`, `virtual_list_basics`, and
    example `markdown_demo` / `drop_shadow_demo`.
  - the form-specific bridge cleanup is now explicit:
    `FormRegistry` accepts a narrow `IntoFormValueModel<T>` bridge and `FormField::new(...)`
    accepts `IntoFormStateModel`, which lets default app-lane examples stay on `LocalState`
    without introducing a crate-wide `IntoModel<T>` story.
  - the table-specific bridge cleanup is now explicit:
    `fret_ui_kit::declarative::table` owns `IntoTableStateModel`, and the default-facing
    `DataTable`, `DataTableToolbar`, `DataTablePagination`, `DataTableViewOptions`, and related
    builder helpers now accept that narrow bridge so app/cookbook examples can keep
    `TableState` on `LocalState<TableState>`.
  - the overlay-close-specific bridge cleanup is now explicit:
    `DialogClose`, `SheetClose`, and `DrawerClose` now accept `IntoBoolModel`, which brings the
    close affordance path back in line with `Dialog::new(...)`, `Button::toggle_model(...)`, and
    other narrow bool bridges on the default-facing authoring lane.
  - the previous M4 blocked queue is cleared: the markdown/drop-shadow examples now bind control
    toggles directly from `&LocalState<bool>`, and `virtual_list_basics` keeps imperative scroll
    helpers on `LocalState` by using explicit `LocalState::{value_in_or,value_in_or_default}`
    store reads instead of reopening `clone_model()`. For date pickers specifically, the default
    app-lane guidance is now to prefer `DatePicker::new(&open, &month, &selected)` when the app
    owns all three state slots; `new_controllable(...)` remains the explicit
    controlled/uncontrolled seam.
  - the advanced/reference roster is now explicit instead of implicit:
    `custom_effect_v1_demo`, `custom_effect_v2_demo`, `custom_effect_v3_demo`,
    `postprocess_theme_demo`, `liquid_glass_demo`, `genui_demo`, and
    `imui_floating_windows_demo` each carry top-level classification docs and are locked by the
    `advanced_reference_demos_are_explicitly_classified` gate.
- **M5**: Planned
  - no hard-delete or final closeout has happened yet.

Execution rule:

- treat this as a public-surface and migration lane,
- not as a storage-model redesign lane,
- and do not call the migration done until user-visible examples are either migrated or explicitly
  classified.

---

## Milestone 0 — Open the lane and freeze the problem

Exit target:

- one workstream exists for the public state/identity cleanup,
- the ADR locks the contract direction,
- and the migration plan explicitly includes old code plus user-visible examples.

Current result:

- this directory now exists,
- ADR 0319 is added,
- and `MIGRATION_MATRIX.md` now records the completed M4 dispositions for the tracked surfaces.

## Milestone 1 — Freeze the target public interface

Exit target:

- the repo can say, in one stable sentence each, what belongs to:
  - the default app lane,
  - the advanced raw-model lane,
  - and the component/internal identity lane.

Decision target:

- stop treating historical `use_state` naming as the long-term public raw-model contract,
- choose explicit model-oriented naming,
- and freeze the pre-release compatibility posture before broad migration starts.

## Milestone 2 — Converge the substrate

Exit target:

- `AppUi` local/raw-state helpers clearly reduce to kernel identity/model primitives, or
- any remaining extra facade bookkeeping is small, explicit, and justified.

What this milestone proves:

- the public contract is not hiding a second parallel slot model,
- diagnostics line up with keyed render evaluation rather than whole-frame heuristics,
- app-facing code must opt into the component/internal lane explicitly through `elements()`,
- and the repo has evidence for the remaining `AppUi` / `UiCx` render-authoring split instead of
  guessing at it.
- the repo can now also distinguish three different follow-on categories for Todo-surfaced
  low-level pressure:
  - keep-raw escape hatch,
  - explicit but non-default render lane,
  - and missing app-facing sugar.

## Milestone 3 — Re-land the first-contact story

Exit target:

- first-contact docs, templates, and Todo proof surfaces all teach the same public contract.

Required evidence:

- docs/README and onboarding docs updated,
- template tests updated,
- `todo_demo` / `simple_todo_demo` still pass their source-policy and runtime validation,
- no first-contact surface uses old generic raw-model naming.

## Milestone 4 — Finish the user-facing migration matrix

Exit target:

- every user-visible example and cookbook surface has one explicit disposition:
  - migrated,
  - advanced/reference with rationale,
  - or blocked on a separately named lower-level contract.

Required evidence:

- migration matrix completed,
- example index / docs updated to reflect classification,
- no ambiguous mixed-lane examples remain.

Current result:

- the migration matrix is classified end-to-end for the tracked user-facing queue,
- `docs/examples/README.md` names the advanced/reference roster explicitly,
- and `apps/fret-examples/src/lib.rs` now locks the advanced/reference comments with a dedicated
  source-policy gate.

## Milestone 5 — Close the public cleanup cleanly

Exit target:

- the old generic raw-model public story is gone or clearly transitional,
- diagnostics internals are not exposed as public concepts,
- and the repo has one credible open-source-ready state/identity story.

Definition of done:

- default app lane is stable and singular,
- advanced raw-model lane is explicit and honestly named,
- kernel identity rules are shared across declarative, recipe, and IMUI fronts,
- the app-facing render-authoring lane no longer depends on implicit `AppUi` `Deref` / raw `UiCx`
  alias inheritance without an explicit justification,
- the workbench/tool-app consumer probe no longer needs raw `cx.app` / raw helper-context leakage
  to stay on the default app authoring story,
- and the migration backlog has no uncategorized user-facing leftovers.
