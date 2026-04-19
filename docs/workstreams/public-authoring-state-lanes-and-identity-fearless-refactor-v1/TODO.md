# Public Authoring State Lanes and Identity Fearless Refactor v1 — TODO

This file is the execution checklist for `DESIGN.md`.

Companion docs:

- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `MIGRATION_MATRIX.md`
- `APP_FACING_RENDER_GAP_AUDIT_2026-04-03.md`
- `DEFAULT_LANE_LOCALSTATE_KEYED_IDENTITY_FREEZE_AUDIT_2026-04-16.md`
- `TODO_ENV_RESPONSIVE_LANE_FREEZE_AUDIT_2026-04-16.md`
- `API_WORKBENCH_FRAMEWORK_PRIORITY_AUDIT_2026-04-15.md`
- `ADVANCED_ENTRY_CAPABILITY_AUDIT_2026-04-15.md`
- `APP_UI_ROOT_ACCESSOR_AUDIT_2026-04-15.md`
- `APP_UI_DEREF_PRESSURE_CLASSIFICATION_AUDIT_2026-04-15.md`
- `APP_UI_DEREF_COMPILE_AUDIT_2026-04-17.md`
- `EXTRACTED_HELPER_RENDER_GUIDANCE_AUDIT_2026-04-16.md`
- `APP_RENDER_CONTEXT_FACADE_AUDIT_2026-04-16.md`
- `UICX_CLOSURE_CONCRETE_TYPE_PRESSURE_AUDIT_2026-04-16.md`
- `APP_RENDER_CX_CONCRETE_CARRIER_AUDIT_2026-04-16.md`
- `RENDER_PASS_ID_INTERNAL_NAMING_AUDIT_2026-04-16.md`
- `UICX_DEFAULT_PRELUDE_DEMOTION_AUDIT_2026-04-17.md`
- `APP_UI_RUNTIME_GATING_AND_FRAME_OWNER_AUDIT_2026-04-17.md`
- `APP_UI_RAW_OWNER_CALLSITE_EXPLICITNESS_AUDIT_2026-04-17.md`
- `APP_UI_LAYOUT_QUERY_OWNER_AUDIT_2026-04-17.md`
- `APP_UI_OVERLAY_ROOT_CAPABILITY_SURFACE_AUDIT_2026-04-17.md`
- `APP_UI_RAW_TEXT_AUTHORING_OWNER_AUDIT_2026-04-17.md`
- `APP_UI_DEFAULT_TEXT_BUILDER_SURFACE_AUDIT_2026-04-17.md`
- `APP_UI_MANUAL_FORM_RAW_OWNER_AUDIT_2026-04-17.md`
- `APP_UI_ADVANCED_HELPER_RAW_OWNER_AUDIT_2026-04-17.md`
- `APP_UI_MANUAL_DATE_PICKER_RAW_OWNER_AUDIT_2026-04-17.md`
- `APP_UI_EMBEDDED_VIEWPORT_INTEROP_CAPABILITY_AUDIT_2026-04-17.md`
- `APP_UI_FINAL_NO_DEREF_TAIL_OWNER_AUDIT_2026-04-17.md`
- `APP_UI_DEREF_REMOVAL_PROOF_AUDIT_2026-04-18.md`
- `APP_COMPONENT_CX_UI_GALLERY_MIGRATION_AUDIT_2026-04-18.md`
- `UICX_ADVANCED_PRELUDE_AND_FIRST_PARTY_TAIL_AUDIT_2026-04-18.md`
- `APP_RENDER_GROUPED_HELPER_EXT_NAMING_AUDIT_2026-04-19.md`
- `APP_UI_TODO_ROOT_CAPABILITY_LANDING_AUDIT_2026-04-17.md`
- `APP_UI_ASYNC_PLAYGROUND_HELPER_CAPABILITY_AUDIT_2026-04-17.md`
- `APP_UI_MARKDOWN_ROOT_CAPABILITY_LANDING_AUDIT_2026-04-17.md`
- `UI_ASSETS_CAPABILITY_ADAPTER_AUDIT_2026-04-15.md`
- `QUERY_GROUPED_MAINTENANCE_SURFACE_AUDIT_2026-04-15.md`
- `COOKBOOK_THEME_CONTEXT_OWNER_AUDIT_2026-04-15.md`
- `MODEL_STORE_RENDER_READ_OWNER_AUDIT_2026-04-15.md`
- `IMUI_IMMEDIATE_LOCALSTATE_BRIDGE_OWNER_AUDIT_2026-04-15.md`
- `APP_DRIVER_RAW_MODEL_OWNER_AUDIT_2026-04-15.md`
- `COMPONENTS_GALLERY_OWNER_SPLIT_AUDIT_2026-04-16.md`
- `IMUI_EDITOR_PROOF_APP_OWNER_AUDIT_2026-04-16.md`
- `IMUI_EDITOR_PROOF_GROUPED_PAINT_READ_AUDIT_2026-04-16.md`
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

- [x] Freeze the default lane wording:
  - [x] `LocalState<T>` is the only blessed first-contact local-state story,
  - [x] keyed identity is the only taught dynamic-list/subtree rule.
    Result: the default docs (`docs/authoring-golden-path-v2.md`, `docs/first-hour.md`,
    `docs/examples/README.md`, `docs/examples/todo-app-golden-path.md`), ADR 0319, and the
    `fretboard` onboarding templates now all freeze the same LocalState-first + keyed-dynamic-list
    posture, with `default_state_identity_docs.rs` as the source-policy gate. See
    `DEFAULT_LANE_LOCALSTATE_KEYED_IDENTITY_FREEZE_AUDIT_2026-04-16.md`.
  - [x] freeze the render-authoring wording for `AppUi` / extracted helper surfaces so the repo
    can distinguish ordinary app-facing render sugar from the raw component/internal
    `ElementContext` lane.
    - frozen posture:
      - `AppUi` remains the root default lane,
      - extracted helper signatures on the default lane prefer
        `fret::app::AppRenderContext<'a>`,
      - closure-local / inline helper families on that lane may use
        `fret::app::AppRenderCx<'a>` as the concrete carrier,
      - `RenderContextAccess<'a, App>` remains the underlying generic capability,
      - `UiCx` is a compatibility old-name alias rather than the taught default.
  - [x] classify the remaining closure-local helper pressure after the `AppRenderContext<'a>`
    façade landing:
    - [x] decide whether closure-heavy default authoring should eventually gain a concrete
      app-facing helper carrier,
    - [x] `AppRenderCx<'a>` is the concrete app-facing helper carrier for closure-local /
      inline helpers on the default lane.
    - [x] `UiCx` remains compatibility-only as the old-name alias while
      named helper functions keep migrating to `AppRenderContext<'a>`.
  - [x] freeze the Todo-surfaced render-gap classification from
    `APP_FACING_RENDER_GAP_AUDIT_2026-04-03.md`:
    - [x] keep-raw escape hatches,
    - [x] explicit non-default environment/responsive lane,
    - [x] missing app-facing render sugar.
- [x] Freeze the advanced raw-model lane wording:
  - [x] choose the explicit model-oriented replacement name (`AppUiRawModelExt::raw_model::<T>()`),
  - [x] decide whether pre-release migration uses hard delete or a short-lived compatibility alias (`hard delete` for the old name),
  - [x] remove “generic hook” framing from the first migrated public docs.
- [x] Freeze the bridge/internal lane wording:
  - [x] classify `LocalState::{model, clone_model, *_in(...)}`
  - [x] classify `ElementContext::{slot_state, local_model, model_for, ...}`
  - [x] keep `AppUi` default-lane access to component/internal state helpers behind explicit
    `cx.elements()`
  - [x] keep explicit ownership language for helper-heavy/component/internal surfaces.
- [x] Freeze the diagnostics posture:
  - [x] evaluation-boundary diagnostics stay internal,
  - [x] `render_pass_id` is not a public concept,
  - [x] keep the internal field name as-is while it remains private repeated-call bookkeeping.
    Result: the diagnostics posture is now frozen as “evaluation-boundary diagnostics stay
    internal, and `render_pass_id` stays a private bookkeeping name rather than a public
    authoring term.” See `RENDER_PASS_ID_INTERNAL_NAMING_AUDIT_2026-04-16.md`.

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
  - [x] keep the async/runtime and manual-root proof surfaces on that same root render lane too:
    `async_playground_demo` and `ime_smoke_demo` now import
    `fret::app::RenderContextAccess as _`, use `cx.theme_snapshot()` at their root render
    surfaces, and `apps/fret-examples/src/lib.rs` source-policy tests forbid
    `Theme::global(&*cx.app).snapshot()` from drifting back into either proof without widening
    `fret::app::prelude::*`.
  - [x] keep selected advanced/runtime ordinary chrome reads on context theme helpers when the
    surface is not actually proving renderer/theme-bridge ownership:
    `embedded_viewport_demo`, `custom_effect_v1_demo`, `custom_effect_v2_demo`,
    `markdown_demo`, and `genui_demo` now use `cx.theme_snapshot()` for their ordinary
    inspector/chrome/image-placeholder theme token reads, and
    `apps/fret-examples/src/lib.rs` source-policy tests lock that choice without pretending
    `postprocess_theme_demo` or `liquid_glass_demo` should leave their explicit theme-bridge
    lane.
  - [x] keep low-level direct-leaf interop roots on context-owned theme reads instead of host
    global escape hatches:
    `external_texture_imports_demo`, `external_texture_imports_web_demo`,
    `external_video_imports_avf_demo`, and `external_video_imports_mf_demo` now read
    `cx.theme().snapshot()` from `ElementContext`, and the direct-leaf root source-policy gate
    forbids `Theme::global(&*cx.app).snapshot()` on that batch.
  - [x] keep the web Custom Effect V2 family on the same context-owned theme lane for ordinary
    chrome/inspector reads:
    `custom_effect_v2_web_demo`, `custom_effect_v2_identity_web_demo`,
    `custom_effect_v2_lut_web_demo`, and `custom_effect_v2_glass_chrome_web_demo` now use
    `cx.theme().snapshot()` from `ElementContext` for lens/inspector/root chrome token reads, and
    the existing grouped-helper source-policy gates forbid `Theme::global(&*cx.app).snapshot()`
    on that batch.
  - [x] keep the remaining selected `ElementContext` product/stress proofs on context-owned theme
    reads too:
    `canvas_datagrid_stress_demo` and `imui_interaction_showcase_demo` now use
    `cx.theme().snapshot()`, and `apps/fret-examples/src/lib.rs` source-policy tests lock that
    choice so the lane can distinguish real renderer/theme-bridge proofs from ordinary
    `ElementContext` chrome reads.
  - [x] lock the remaining explicit renderer/theme-bridge lane too:
    `postprocess_theme_demo` and `liquid_glass_demo` still keep
    `Theme::global(&*cx.app).snapshot()`, and `apps/fret-examples/src/lib.rs` now has a source
    policy gate that treats those two files as the intentional remaining host-global theme proofs
    instead of accidental cleanup leftovers.
  - [x] keep `components_gallery` on the same split lane it actually needs:
    ordinary theme reads now use `cx.theme_snapshot()` / `cx.theme()` for
    `theme_name`, font-style token reads, and hover-card chrome, while the file still keeps its
    explicit raw state lane (`cx.elements().slot_state(...)`, `local_model_keyed(...)`) where the
    gallery is intentionally exercising lower-level retained table paths.
  - [x] stop ordinary root/helper seams from forcing `AppUi` call sites back onto raw
    `ElementContext` when the helper boundary itself can own that escape hatch:
    `embedded_viewport_demo` and `hello_world_compare_demo` now accept
    `fret::app::ElementContextAccess<'a, KernelApp>` for their page/root helpers,
    `assets_demo` now accepts `fret::app::RenderContextAccess<'a, KernelApp>` at
    `render_view(...)` entry and keeps `ThemeSnapshot` on the helper lane, and
    `image_heavy_memory_demo` now accepts `fret::app::ElementContextAccess<'a, KernelApp>` at its
    `render_view(...)` entry; source-policy gates now forbid
    `embedded_viewport_page(cx.elements(), ...)`,
    `hello_world_compare_root(cx.elements(), ...)`, and `render_view(cx.elements())` on this
    batch.
  - [x] keep the same root/helper boundary rule on the editor-grade IMUI proof without widening
    or flattening its advanced helper lane:
    `imui_editor_proof_demo` now accepts
    `fret::app::ElementContextAccess<'a, KernelApp>` at the outer `render_view(...)` boundary,
    keeps the internal `AppComponentCx` helper family unchanged, and source-policy gates now forbid the root
    from spelling `render_view(cx.elements())`.
  - [x] classify and then migrate the remaining `AppUi`-root advanced entry seams in
    `apps/fret-examples/src` so they stop using raw `cx.elements()` only as an entry adapter:
    immediate-mode teaching surfaces (`imui_hello_demo`, `imui_floating_windows_demo`,
    `imui_response_signals_demo`, `imui_shadcn_adapter_demo`, `imui_node_graph_demo`) now
    explicitly require `fret_imui::imui_in(cx, |ui| {`, while the advanced direct-leaf
    chart/node demos use `chart_canvas_panel_in(cx, props)` and
    `node_graph_surface_in(cx, props)` plus `NodeGraphSurfaceBinding::observe_in(cx)`.
  - [x] audit the remaining intentional lanes before reopening any `AppUi` `Deref` removal work:
    `ADVANCED_ENTRY_CAPABILITY_AUDIT_2026-04-15.md` concludes that the next correct framework
    slice is capability-first adapters for advanced public entry surfaces (`fret_imui`,
    `fret_chart`, `fret_node`) rather than another blind `Deref` removal attempt.
  - [x] add capability-first advanced-entry adapters for the remaining ecosystem public surfaces
    that still forced `AppUi` roots to spell `cx.elements()` only to enter the advanced lane:
    `fret_imui::{imui_in, imui_raw_in, imui_build_in}`,
    `fret_chart::declarative::chart_canvas_panel_in`,
    `fret_node::ui::declarative::node_graph_surface_in`,
    and `NodeGraphSurfaceBinding::observe_in(...)` are now landed and first-party proof callsites
    are migrated onto that lane.
  - [x] shrink the remaining selected `AppUi` root bridge-syntax dependence after that adapter
    batch before reopening direct `Deref` removal:
    `embedded_viewport_demo`, `async_playground_demo`, `markdown_demo`,
    `postprocess_theme_demo`, `genui_demo`, and `hello_world_compare_demo` now use explicit
    `AppUi::{app, app_mut, window_id}` accessors instead of `cx.app` / `cx.window` bridge syntax
    at the root render surface, and `apps/fret-examples/src/lib.rs` now locks that batch with
    `selected_app_ui_roots_prefer_explicit_render_context_accessors_over_deref`.
  - [x] classify the remaining post-cleanup `Deref` pressure before arguing for any direct
    `AppUi` `Deref` deletion:
    `APP_UI_DEREF_PRESSURE_CLASSIFICATION_AUDIT_2026-04-15.md` now records that the selected root
    batch is effectively closed, `markdown_demo` no longer teaches the trait-UFCS variant at the
    root, and the remaining grep surface is split across advanced/reference owner surfaces,
    docking/multi-window proofs, and helper-local raw seams rather than one unfinished default
    lane.
  - [x] close the first repeated helper-local asset state/stats seam on the same capability-first
    authoring lane:
    `fret-ui-assets::ui` now owns `use_rgba8_image_state_in(...)`, `image_stats_in(...)`, and
    `svg_stats_in(...)`; `assets_demo`, `markdown_demo`, and cookbook
    `assets_reload_epoch_basics` now use those adapters instead of spelling `cx.app + cx.window`
    or `UiAssets::image_stats(&mut *cx.app)` directly just to enter the asset helper surface.
  - [x] close the remaining grouped app-facing query maintenance seam that still forced real
    `AppUi` / `AppComponentCx` examples back onto raw client shell code:
    `fret::view::{AppUiData, AppRenderData}` now own `query_snapshot()`,
    `query_snapshot_entry(...)`, and `cancel_query(...)`; `async_playground_demo` uses grouped
    invalidation/cancel/snapshot helpers end-to-end, `markdown_demo` uses grouped namespace
    invalidation, and `QUERY_GROUPED_MAINTENANCE_SURFACE_AUDIT_2026-04-15.md` records why this
    owner stays on the `fret` app-facing lane rather than moving into `fret-query`.
  - [x] close the remaining cookbook host-global theme snapshot tail by owner class instead of one
    flat grep bucket:
    ordinary `AppUi` / `AppComponentCx` cookbook examples now use `cx.theme_snapshot()`, direct-leaf
    `ElementContext` interop roots now use `cx.theme().snapshot()`, cookbook source-policy tests
    lock the split, and `COOKBOOK_THEME_CONTEXT_OWNER_AUDIT_2026-04-15.md` records the owner
    classification so future cleanup does not mix app-facing and interop lanes.
  - [x] close the remaining render-time raw `ModelStore` reads only where an existing render-lane
    helper already owns the story:
    cookbook `virtual_list_basics` now uses tracked builder `.revision()` reads on `AppUi`,
    direct-leaf/manual render roots (`custom_effect_v2_*_web_demo`,
    `external_*_imports*_demo`) now use `cx.data().selector_model_layout(...)` for their
    render-time `show` toggles, source-policy tests lock both owners, and
    `MODEL_STORE_RENDER_READ_OWNER_AUDIT_2026-04-15.md` records why driver-side
    `record_engine_frame(...)` reads stay on raw `app.models()`.
  - [x] extend the same grouped-selector render-lane cleanup to the selected stress roots that
    were still reading tracked models before entering the render tree:
    `virtual_list_stress_demo` and `canvas_datagrid_stress_demo` now derive their render-time
    state through `cx.data().selector_model_layout(...)`, source-policy tests lock that choice,
    and the remaining `app.models()` grep tail is narrower driver/retained owner surface instead
    of stress-render drift.
  - [x] make the advanced/reference GenUI message lane own its remaining local/model store reads
    explicitly instead of mixing raw `app.models()` calls through `handle_msg(...)`:
    `GenUiState` now owns helper methods such as `auto_apply_enabled(...)`,
    `auto_fix_enabled(...)`, `editor_text_value(...)`, `stream_text_value(...)`,
    `stream_patch_only_enabled(...)`, `queued_invocations(...)`, and `clear_action_queue(...)`;
    `handle_msg(...)` routes through those helpers, and source-policy tests lock the message-lane
    owner without pretending this advanced surface should migrate onto render-time grouped helpers.
  - [x] close the IMUI immediate-mode closure-local raw `ModelStore` read tail without flattening
    app/driver ownership:
    `imui_hello_demo` now reads its checkbox status with `paint_value_in(ui.cx_mut())`,
    `imui_interaction_showcase_demo` now reads closure-local bookmark/tool/autosave/exposure/
    review/tab/context state with `layout_value_in(ui.cx_mut())`, `push_showcase_event(...)`
    keeps raw `app.models()` as the app-owned helper seam, and
    `IMUI_IMMEDIATE_LOCALSTATE_BRIDGE_OWNER_AUDIT_2026-04-15.md` records the split.
  - [x] freeze the remaining pure app/driver-loop raw `ModelStore` reads so future cleanup does
    not misclassify them as render-lane debt:
    embedded viewport recorders, external import `record_engine_frame(...)` loops, workspace/utility
    command handlers, and `plot_stress_demo` driver helpers now have dedicated source-policy gates,
    cookbook `embedded_viewport_basics` keeps the same explicit raw owner, and
    `APP_DRIVER_RAW_MODEL_OWNER_AUDIT_2026-04-15.md` records that `components_gallery` is a
    separate retained/component follow-on rather than part of this owner class.
  - [x] split the remaining `components_gallery` mixed owner surface instead of treating it as one
    more raw grep tail:
    the retained table-torture subtree now uses `table_state.layout(cx).revision()` for its
    render-time revision read, app/theme sync and overlay aggregation now route through explicit
    demo-local owner helpers (`selected_theme_preset(app)`, `overlays_open(app)`), driver/event
    tree-key reads stay raw, and
    `COMPONENTS_GALLERY_OWNER_SPLIT_AUDIT_2026-04-16.md` records the three-way owner split.
  - [x] keep the last `imui_editor_proof_demo` raw model tail on explicit app-owned helper seams
    instead of scattering store reads through the advanced proof:
    outliner reorder math now routes through `proof_outliner_items_snapshot(...)` and
    `proof_outliner_order_line_for_model(...)`, dock/bootstrap target lookup routes through
    `embedded_target_for_window(...)`, and
    `IMUI_EDITOR_PROOF_APP_OWNER_AUDIT_2026-04-16.md` records why this remains demo-local owner
    code rather than a new framework surface.
  - [x] move the remaining `imui_editor_proof_demo` paint-only shared-model readouts onto the
    grouped selector lane instead of leaving them on raw `get_model_*` helpers:
    text assist / text field / string readouts, authoring parity shared state, gradient-stop
    snapshots, and dock-panel embedded target reads now use `cx.data().selector_model_paint(...)`,
    while `IMUI_EDITOR_PROOF_GROUPED_PAINT_READ_AUDIT_2026-04-16.md` records why this does not
    reopen the app-owned reorder/bootstrap exceptions.
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
    - [x] explicit environment/responsive helpers that should stay off the default lane rather than
      being mistaken for raw debt.
      Result: `fret::env::{...}` now also carries the needed query-configuration nouns
      (`ContainerQueryHysteresis`, `ViewportQueryHysteresis`, `ViewportOrientation`), and
      `todo_demo` no longer needs a direct `fret_ui_kit::declarative` import for that ordinary
      app-facing responsive slice. See
      `TODO_ENV_RESPONSIVE_LANE_FREEZE_AUDIT_2026-04-16.md`.
  - [x] land the first justified app-facing render-sugar replacements without widening
    `fret::app::prelude::*` or collapsing the documented `raw` lane.
  - [x] stop teaching `UiCx` as a default import once `AppRenderCx<'a>` is the blessed concrete
    closure-local carrier:
    `fret::app::prelude::*` now exports `AppRenderCx` but no longer reexports `UiCx`, while the
    compatibility alias remains available only through explicit import / advanced surfaces. See
    `UICX_DEFAULT_PRELUDE_DEMOTION_AUDIT_2026-04-17.md`.
  - [x] land the first explicit runtime/gating slice that no longer depends on the temporary
    `AppUi -> ElementContext` `Deref` bridge:
    `AppUi` now owns an inherent `request_animation_frame()` helper, the explicitly imported
    `fret::actions::ElementCommandGatingExt` trait now implements directly on `AppUi`, and the
    owner classification for the remaining raw method-family tail is recorded in
    `APP_UI_RUNTIME_GATING_AND_FRAME_OWNER_AUDIT_2026-04-17.md`.
  - [x] move the already-classified raw-owner and grouped-selector tail callsites without widening
    the framework surface:
    `components_gallery` now enters its retained branch through `let cx = cx.elements();`,
    cookbook `drag_basics` / `gizmo_basics` now spell `cx.elements().pointer_region(...)`,
    `editor_notes_device_shell_demo` now uses `cx.data().selector_model_paint(...)`, and the
    resulting no-`Deref` compile-audit reduction is recorded in
    `APP_UI_RAW_OWNER_CALLSITE_EXPLICITNESS_AUDIT_2026-04-17.md`.
  - [x] land the app-facing layout-query owner slice instead of forcing geometry-query authoring
    through raw `cx.elements()`:
    `AppUi` now owns `layout_query_bounds(...)`, `layout_query_region(...)`, and
    `layout_query_region_with_id(...)`; the nested region-builder closure carries grouped
    action-handler state instead of depending on implicit `Deref`; `markdown_demo` now keeps its
    remaining mutable host bridge on `cx.app_mut().models_mut()`; and the no-`Deref` evidence is
    recorded in `APP_UI_LAYOUT_QUERY_OWNER_AUDIT_2026-04-17.md`.
  - [x] land the direct overlay-root capability slice instead of forcing real app roots back
    through raw `ElementContext` entry points:
    `Dialog`, `AlertDialog`, `Sheet`, and `Drawer` now expose explicit `*_in(...)` late-builder
    entry points, the matching `UiBuilder` overlay adapters expose the same capability lane,
    `WorkspaceFrame` now exposes `into_element_in(...)`, `api_workbench_lite_demo` /
    `editor_notes_device_shell_demo` / `emoji_conformance_demo` use explicit `app()` /
    `into_element_in(...)` accessors, and the targeted evidence is recorded in
    `APP_UI_OVERLAY_ROOT_CAPABILITY_SURFACE_AUDIT_2026-04-17.md`.
  - [x] classify the current raw text/builder conformance tail at the callsite instead of
    widening `AppUi`:
    `emoji_conformance_demo` now keeps theme/state/global reads on `AppUi`, then explicitly
    enters `let cx = cx.elements();` for `text_props(...)` and direct
    `into_element(...)` late-landing (`Card*`, `Separator`, `ScrollArea`); the targeted evidence
    is recorded in `APP_UI_RAW_TEXT_AUTHORING_OWNER_AUDIT_2026-04-17.md`.
  - [x] keep the default counter teaching surfaces on existing app-lane text builders and
    capability-first late landing instead of reopening raw `TextProps` or `cx.elements()`:
    `apps/fret-examples/src/hello_counter_demo.rs` and
    `apps/fret-cookbook/examples/hello_counter.rs` now use `ui::text(...)` /
    `ui::text_block(...)`; the demo uses `IntoUiElementInExt::into_element_in(cx)` for ordinary
    late-landing roots; and the targeted evidence is recorded in
    `APP_UI_DEFAULT_TEXT_BUILDER_SURFACE_AUDIT_2026-04-17.md`.
  - [x] classify the manual `form_demo` raw layout/build tail at the callsite instead of widening
    `AppUi`:
    `form_demo` now keeps its `form_state` / `status` reads on `AppUi`, then explicitly enters
    `let cx = cx.elements();` for `cx.text(...)`, direct form-control late-landing, and
    `cx.container(...)` / `cx.flex(...)`; the targeted evidence is recorded in
    `APP_UI_MANUAL_FORM_RAW_OWNER_AUDIT_2026-04-17.md`.
  - [x] classify the next advanced helper/render-shell clusters at the callsite instead of
    widening `AppUi`:
    `postprocess_theme_demo` and `imui_interaction_showcase_demo` now keep tracked reads on
    `AppUi`, then explicitly enter `let cx = cx.elements();` before their raw helper/render-shell
    phase; the targeted evidence is recorded in
    `APP_UI_ADVANCED_HELPER_RAW_OWNER_AUDIT_2026-04-17.md`.
  - [x] classify the next manual `render_root_with_app_ui(...)` tail at the callsite instead of
    widening `AppUi`:
    `date_picker_demo` now keeps theme/local-state/layout reads on `AppUi`, then explicitly
    enters `let cx = cx.elements();` before its manual header/toggle/picker/calendar/container
    build phase; the targeted evidence is recorded in
    `APP_UI_MANUAL_DATE_PICKER_RAW_OWNER_AUDIT_2026-04-17.md`.
  - [x] classify the next mixed app-facing/interop root as capability-first late landing plus one
    explicit raw seam instead of widening `AppUi` or collapsing the whole render path onto raw
    `ElementContext`:
    `embedded_viewport_demo` now keeps tracked reads and ordinary builders on `AppUi`, uses
    `IntoUiElementInExt::into_element_in(cx)` plus `ui::text(...).into_element_in(cx)` for normal
    late-landing, and keeps `cx.elements()` only at `EmbeddedViewportSurface::panel(...)`; the
    targeted evidence is recorded in
    `APP_UI_EMBEDDED_VIEWPORT_INTEROP_CAPABILITY_AUDIT_2026-04-17.md`.
  - [x] keep the default Todo app root on the existing capability-first late-landing lane instead
    of letting ordinary builders fall back to implicit `Deref`:
    `todo_demo` now uses `into_element_in(cx)` for root-level status/progress/scroll/footer
    landing on `AppUi`, while helper-local `ElementContext` closures keep their existing raw
    builder usage; the targeted evidence is recorded in
    `APP_UI_TODO_ROOT_CAPABILITY_LANDING_AUDIT_2026-04-17.md`.
  - [x] classify the next named-helper tail as app-facing capability debt instead of reopening a
    raw-owner lane:
    `async_playground_demo` now moves its named helpers from `UiCx<'_>` onto
    `fret::app::AppRenderContext<'a>`, uses `IntoUiElementInExt::into_element_in(cx)` for
    ordinary late-landing, keeps `cx.elements().pressable(...)` explicit at the one real raw
    leaf, and records the targeted evidence in
    `APP_UI_ASYNC_PLAYGROUND_HELPER_CAPABILITY_AUDIT_2026-04-17.md`.
  - [x] keep the markdown app root and nested layout-query shells on the same capability-first
    late-landing lane instead of falling back to implicit `Deref`:
    `markdown_demo` now uses `IntoUiElementInExt::into_element_in(cx)` for `toggles`, the nested
    `layout_query_region_with_id(...)` shell/container, `content`, and the outer page shell,
    while the explicit image-hook raw helper remains on `UiCx<'_>`; the targeted evidence is
    recorded in `APP_UI_MARKDOWN_ROOT_CAPABILITY_LANDING_AUDIT_2026-04-17.md`.
  - [x] keep the editor-notes workbench root on that same capability-first late-landing lane
    instead of letting ordinary shell rails fall back to implicit `Deref`:
    `editor_notes_demo` now keeps its reusable panels generic on
    `ElementContextAccess<'a, App>`, imports `IntoUiElementInExt as _`, uses
    `into_element_in(cx)` for both rails, `WorkspaceFrame`, and the outer page shell, and records
    the targeted evidence in `APP_UI_EDITOR_NOTES_ROOT_CAPABILITY_LANDING_AUDIT_2026-04-17.md`.
  - [x] extend the app-facing runtime/frame lane to cover continuous frame leases instead of
    forcing real proof surfaces back through raw scheduling helpers:
    `AppUi` now owns `set_continuous_frames(enabled)`, `hello_world_compare_demo` calls that
    helper directly, keeps its closure-local swatches on `AppRenderCx<'_>`, uses `ui::text(...)`
    for the title, uses `into_element_in(cx)` for `layout_probe`, `swatch_row`, and the root
    shell, and records the targeted evidence in
    `APP_UI_CONTINUOUS_FRAMES_RUNTIME_OWNER_AUDIT_2026-04-17.md`.
  - [x] keep the first-contact query proofs on that same capability-first root landing lane
    instead of letting ordinary detail builders fall back to implicit `Deref`:
    `query_demo` and `query_async_tokio_demo` now import `IntoUiElementInExt as _`, use
    `into_element_in(cx)` for `status_row`, `buttons`, and `detail_body`, keep grouped
    state/effects/data ownership on `AppUi`, and record the targeted evidence in
    `APP_UI_QUERY_ROOT_CAPABILITY_LANDING_AUDIT_2026-04-17.md`.
  - [x] keep the advanced raw builder triplet on explicit escape hatches instead of widening
    `AppUi` with `container(...)` / `flex(...)` / `text(...)`:
    `drop_shadow_demo`, `ime_smoke_demo`, and `sonner_demo` now perform app-lane
    state/theme/grouped reads first, then enter `let cx = cx.elements();` before the advanced raw
    builder phase, and record the targeted evidence in
    `APP_UI_ADVANCED_RAW_BUILDER_OWNER_AUDIT_2026-04-17.md`.
  - [x] close the remaining `fret-examples` no-`Deref` tail at the owner boundary instead of
    widening `AppUi` again:
    `custom_effect_v1_demo`, `custom_effect_v2_demo`, `custom_effect_v3_demo`, `genui_demo`, and
    `liquid_glass_demo` now enter their advanced `view(...)` helpers through `cx.elements()`,
    `components_gallery` keeps tracked reads on `AppUi` then enters `let cx = cx.elements();`
    before the normal-branch raw builder/theme-name phase, the source-policy gates in
    `apps/fret-examples/src/lib.rs` lock that split, and the temporary no-`Deref` spot-check now
    leaves `fret-examples` clean. See
    `APP_UI_FINAL_NO_DEREF_TAIL_OWNER_AUDIT_2026-04-17.md`.

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
- [x] Ensure the default render-authoring path no longer depends on implicit `AppUi` `Deref`
  inheritance once the narrowed app-facing render surface is landed.
  Result: `ecosystem/fret/src/view.rs` no longer implements `std::ops::Deref` /
  `std::ops::DerefMut` for `AppUi`, and the follow-on package/test proof is recorded in
  `APP_UI_DEREF_REMOVAL_PROOF_AUDIT_2026-04-18.md`.
- [x] Demote raw `UiCx = ElementContext<App>` from the default helper-story export surface once
  `AppRenderCx<'a>` is the blessed concrete carrier.
  Result: `UiCx` now remains compatibility-only behind explicit import / advanced-surface intent,
  while `fret::app::prelude::*` keeps `AppRenderCx<'a>` as the default concrete helper carrier on
  the app-facing lane.
- [x] Land an explicit app-hosted component/snippet helper alias for first-party exemplar
  surfaces.
  Result: `ecosystem/fret/src/lib.rs` now exports `AppComponentCx<'a>`, `UiCx<'a>` remains the
  compatibility old-name alias to that surface, and the shipped gallery/docs exemplar no longer
  needs to teach `UiCx` as its first-party app-hosted snippet lane.
- [x] Migrate the first-party UI Gallery snippet/page/internal-preview surface off the explicit
  `UiCx` compatibility alias onto `AppComponentCx<'a>`.
  Result: `apps/fret-ui-gallery/src/ui/**` plus the matching source-policy tests now use
  `AppComponentCx<'a>`, and the migration proof is recorded in
  `APP_COMPONENT_CX_UI_GALLERY_MIGRATION_AUDIT_2026-04-18.md`.
- [x] Demote `UiCx` from `fret::advanced::prelude::*` once the first-party app-hosted example tail
  is migrated to `AppComponentCx<'a>`.
  Result: `fret::advanced::prelude::*` now exports `AppComponentCx<'a>` instead of `UiCx<'a>`, the
  remaining first-party example/cookbook runtime tail now uses `AppComponentCx<'a>`, and the proof
  is recorded in `UICX_ADVANCED_PRELUDE_AND_FIRST_PARTY_TAIL_AUDIT_2026-04-18.md`.
- [x] Retire `UiCx` from the grouped app-render helper extension names before auditing the root
  alias itself.
  Result: `AppRenderActionsExt` / `AppRenderDataExt` are the canonical grouped helper extension
  names, the first-party examples and UI Gallery snippets now import those names, and
  `UiCxActionsExt` / `UiCxDataExt` remain only as explicit deprecated compatibility aliases.
  Proof: `APP_RENDER_GROUPED_HELPER_EXT_NAMING_AUDIT_2026-04-19.md`.
- [x] Audit the remaining explicit-import `UiCx` tail and either classify it as
  advanced/compatibility-only or migrate it to `AppRenderContext<'a>` / `AppRenderCx<'a>` /
  `AppComponentCx<'a>`.
  Result: `UiCx<'a>` now remains only as an explicitly deprecated compatibility alias at the root
  surface, public docs/tooling describe it that way, and the default teaching-snippet gate now
  enforces `AppComponentCx<'a>` instead of `UiCx<'a>`.
  Proof: `UICX_COMPAT_ALIAS_DEPRECATION_AUDIT_2026-04-19.md`.
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
