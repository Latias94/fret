# Into-Element Surface — Migration Matrix

Status: execution tracker
Last updated: 2026-03-14

This matrix tracks how the current conversion surface should move toward the target state.

It focuses on:

- public trait vocabulary,
- helper return types,
- first-party teaching surfaces,
- and which old names become delete-ready.

## Status Legend

| Status | Meaning |
| --- | --- |
| Not started | no migration code landed |
| Scaffolding only | a new path exists, but public teaching/call sites still use the old path |
| In progress | first-party migration is underway |
| Migrated | official first-party call sites use the new path |
| Delete-ready | migrated and guarded; old path can be removed |
| Deleted | old public path is gone |

## Global Deletion Rule

An old conversion name is eligible for deletion only when all of the following are true:

1. app docs/templates no longer teach it,
2. component docs/examples no longer teach it,
3. first-party reusable crates no longer depend on it as public API,
4. a gate exists to prevent it from reappearing on curated surfaces.

## Current Name Classification (2026-03-13)

| Name | Intended posture | Current reality | Status |
| --- | --- | --- | --- |
| `Ui` | keep publicly on the app surface | app-facing alias over `Elements` | Kept publicly |
| `UiChild` | keep publicly on the app surface | app-owned marker over `IntoUiElement<App>` | Kept publicly |
| `IntoUiElement<H>` | keep publicly on the component surface | curated conversion name on `fret-ui-kit` / `fret::component::prelude::*` | Kept publicly |
| `AnyElement` | keep publicly as an explicit raw type | still legal and intentional on advanced/raw seams | Moved to advanced/raw only |
| `Elements` | keep publicly as an explicit raw type; teach `Ui` instead on the app surface | still present as the raw container type behind `Ui` | Moved to advanced/raw only |
| `UiIntoElement` | stop teaching publicly; delete the name entirely once migration lands | deleted from code; only historical docs and negative source-policy assertions still mention the legacy name while audits finish | Deleted |
| `UiHostBoundIntoElement<H>` | stop teaching publicly; compatibility bridge only | deleted from code; no curated or root-level export remains | Deleted |
| `UiChildIntoElement<H>` | stop teaching publicly; app-internal/component-internal mechanism only | deleted from code; child pipelines now consume `IntoUiElement<H>` directly | Deleted |
| `UiBuilderHostBoundIntoElementExt<H>` | hidden bridge only, then delete | deleted from code; method syntax now lands through `IntoUiElement<H>` directly | Deleted |
| legacy split public conversion vocabulary | delete from curated product surfaces | absent from curated `fret` component exports and no longer root-exported from `fret-ui-kit`; only module-level scaffolding remains | Deleted entirely from curated surfaces / root cleanup landed |

## Surface Lanes

| Lane | Current surface | Target surface | Migration tactic | Delete trigger | Status | Evidence anchors |
| --- | --- | --- | --- | --- | --- | --- |
| App render return | `Ui = Elements` alias already exists, but raw `Elements` still appears in some checks and historical docs | keep `Ui` as the app-facing render alias | continue treating `Ui` as canonical and delete stale `Elements` teaching where it survives | default app docs/examples only teach `Ui` | Migrated | `ecosystem/fret/src/lib.rs`, `docs/workstreams/authoring-surface-and-ecosystem-fearless-refactor-v1/TARGET_INTERFACE_STATE.md` |
| App helper child return | `UiChild` already exists as an app-owned marker over `IntoUiElement<App>` | keep `UiChild` as the only app-facing child concept | migrate app-facing helper docs/examples to `impl UiChild` and stop teaching the underlying trait | default app docs/examples never spell `UiChildIntoElement<App>` | Migrated | `ecosystem/fret/src/lib.rs`, `apps/fret-cookbook/src/lib.rs`, `apps/fret-examples/src/lib.rs` |
| Component conversion contract | public split across `UiIntoElement`, `UiHostBoundIntoElement<H>`, and `UiChildIntoElement<H>` | one public conversion trait generic over `H: UiHost` | introduce unified trait, temporarily adapt old impls, then delete the old public names | curated component prelude exports only one public conversion trait | Migrated | `ecosystem/fret/src/lib.rs`, `ecosystem/fret-ui-kit/src/lib.rs`, `ecosystem/fret-ui-kit/src/ui_builder.rs`, `docs/component-authoring-contracts.md` |
| Host-bound builder landing | host-bound builders previously needed `UiBuilderHostBoundIntoElementExt<H>` to recover `.into_element(cx)` syntax | method syntax provided through the unified conversion trait | move host-bound builder landing behind the new public trait and keep any extra bridging internal | app/component preludes stop importing the old extension trait | Migrated | `ecosystem/fret/src/lib.rs`, `ecosystem/fret-ui-kit/src/ui_builder.rs` |
| Child pipelines | `ui::children!`, `UiElementSinkExt`, `imui::add_ui(...)`, and first-party single-child builders now consume `IntoUiElement<H>` directly | heterogeneous child collection consumes the unified contract semantics without parallel component-specific impls | delete the old child bridge and migrate downstream single-child helpers to the unified trait | no curated child helper depends on `UiChildIntoElement<H>` publicly | Migrated | `ecosystem/fret-ui-kit/src/lib.rs`, `ecosystem/fret-ui-kit/src/ui.rs`, `ecosystem/fret-ui-kit/src/imui.rs`, `ecosystem/fret-ui-shadcn/src/`, `ecosystem/fret-router-ui/src/lib.rs` |
| Component helper signatures | first-party reusable snippets often return `AnyElement` even when raw landing is not conceptually required | generic helpers prefer `impl IntoUiElement<H>` | migrate first-party reusable helpers opportunistically during snippet/component audits; keep explicit `ui_builder_ext/*::into_element(...)` outputs on `AnyElement` when the API itself is the intended landing seam | first-party reusable docs/snippets reserve `AnyElement` for justified raw seams and explicit landing-seam helpers | In progress | `ecosystem/fret-ui-shadcn/src/ui_ext/support.rs`, `ecosystem/fret-ui-shadcn/src/ui_ext/data.rs`, `ecosystem/fret-ui-shadcn/src/ui_builder_ext/`, `ecosystem/fret-ui-shadcn/src/surface_policy_tests.rs`, `ecosystem/fret/tests/reusable_component_helper_surface.rs`, `apps/fret-ui-gallery/src/ui/snippets/` |
| App helper signatures | most official app surfaces already prefer `Ui` and `UiCx`, but some helpers still land raw children; advanced/manual-assembly examples historically leaked `AnyElement` even for non-raw helpers; page-local Gallery doc helpers also tended to early-land to satisfy `DocSection::new(...)`; default-app Gallery reusable helpers sometimes still spelled raw `AnyElement` instead of a typed landing contract | app-facing helpers prefer `impl UiChild`; advanced/manual-assembly helpers prefer `impl IntoUiElement<KernelApp>` when `UiChild` is not the teaching surface; default-app reusable helpers that need a concrete host spell `impl IntoUiElement<fret_app::App>` rather than private `KernelApp`; wrapper/composer helpers should also accept `IntoUiElement<H>` inputs instead of pre-landed `AnyElement` | continue app-surface cleanup in cookbook/examples/gallery teaching surfaces, keep advanced examples off raw landed helper returns unless the seam is genuinely low-level, keep Gallery page-local helpers on `UiChild` even when the doc scaffold still consumes `AnyElement` at the section boundary, and migrate default-app reusable snippet helpers off `AnyElement` where no raw seam is intended | app teaching surfaces no longer need raw child return types by default, advanced helper surfaces reserve `AnyElement` for justified raw seams, default-app reusable helpers use an explicit typed landing contract instead of raw `AnyElement`, wrapper/composer helpers accept `IntoUiElement<H>` rather than pre-landed `AnyElement`, and Gallery page/helpers do not early-land just because the doc scaffold or snippet registry has an explicit raw boundary | In progress | `apps/fret-cookbook/examples/`, `apps/fret-examples/src/assets_demo.rs`, `apps/fret-examples/src/async_playground_demo.rs`, `apps/fret-examples/src/custom_effect_v1_demo.rs`, `apps/fret-examples/src/custom_effect_v2_demo.rs`, `apps/fret-examples/src/custom_effect_v3_demo.rs`, `apps/fret-examples/src/postprocess_theme_demo.rs`, `apps/fretboard/src/scaffold/templates.rs`, `apps/fret-ui-gallery/src/ui/pages/ai_persona_demo.rs`, `apps/fret-ui-gallery/src/ui/pages/ai_commit_demo.rs`, `apps/fret-ui-gallery/src/ui/pages/ai_context_demo.rs`, `apps/fret-ui-gallery/src/ui/pages/ai_file_tree_demo.rs`, `apps/fret-ui-gallery/src/ui/snippets/ai/context_default.rs`, `apps/fret-ui-gallery/src/ui/snippets/ai/context_demo.rs`, `apps/fret-ui-gallery/src/ui/snippets/ai/file_tree_basic.rs`, `apps/fret-ui-gallery/src/ui/snippets/ai/file_tree_expanded.rs`, `apps/fret-ui-gallery/src/ui/snippets/ai/test_results_demo.rs`, `apps/fret-ui-gallery/src/ui/snippets/avatar/demo.rs`, `apps/fret-ui-gallery/src/ui/snippets/avatar/group.rs`, `apps/fret-ui-gallery/src/ui/snippets/avatar/with_badge.rs`, `apps/fret-ui-gallery/src/ui/snippets/avatar/fallback_only.rs`, `apps/fret-ui-gallery/src/ui/snippets/avatar/sizes.rs`, `apps/fret-ui-gallery/src/ui/snippets/avatar/group_count.rs`, `apps/fret-ui-gallery/src/ui/snippets/avatar/group_count_icon.rs`, `apps/fret-ui-gallery/src/ui/snippets/avatar/badge_icon.rs`, `apps/fret-ui-gallery/src/ui/snippets/avatar/dropdown.rs`, `apps/fret-ui-gallery/src/ui/snippets/badge/demo.rs`, `apps/fret-ui-gallery/src/ui/snippets/badge/spinner.rs`, `apps/fret-ui-gallery/src/ui/snippets/badge/rtl.rs`, `apps/fret-ui-gallery/src/ui/snippets/badge/counts.rs`, `apps/fret-ui-gallery/src/ui/snippets/badge/colors.rs`, `apps/fret-ui-gallery/src/ui/snippets/badge/icon.rs`, `apps/fret-ui-gallery/src/ui/snippets/badge/variants.rs`, `apps/fret-ui-gallery/src/ui/snippets/breadcrumb/dropdown.rs`, `apps/fret-ui-gallery/src/ui/snippets/button/demo.rs`, `apps/fret-ui-gallery/src/ui/snippets/button/size.rs`, `apps/fret-ui-gallery/src/ui/snippets/button/with_icon.rs`, `apps/fret-ui-gallery/src/ui/snippets/button/link_render.rs`, `apps/fret-ui-gallery/src/ui/snippets/button/rtl.rs`, `apps/fret-ui-gallery/src/ui/snippets/button/loading.rs`, `apps/fret-ui-gallery/src/ui/snippets/button/variants.rs`, `apps/fret-ui-gallery/src/ui/snippets/button/button_group.rs`, `apps/fret-ui-gallery/src/ui/snippets/button/rounded.rs`, `apps/fret-ui-gallery/src/ui/snippets/hover_card/sides.rs`, `apps/fret-ui-gallery/src/ui/snippets/hover_card/trigger_delays.rs`, `apps/fret-ui-gallery/src/ui/snippets/context_menu/basic.rs`, `apps/fret-ui-gallery/src/ui/snippets/context_menu/radio.rs`, `apps/fret-ui-gallery/src/ui/snippets/context_menu/checkboxes.rs`, `apps/fret-ui-gallery/src/ui/snippets/context_menu/groups.rs`, `apps/fret-ui-gallery/src/ui/snippets/context_menu/icons.rs`, `apps/fret-ui-gallery/src/ui/snippets/context_menu/shortcuts.rs`, `apps/fret-ui-gallery/src/ui/snippets/context_menu/destructive.rs`, `apps/fret-ui-gallery/src/ui/snippets/context_menu/demo.rs`, `apps/fret-ui-gallery/src/ui/snippets/context_menu/rtl.rs`, `apps/fret-ui-gallery/src/ui/snippets/context_menu/submenu.rs`, `apps/fret-ui-gallery/src/ui/snippets/combobox/long_list.rs`, `apps/fret-ui-gallery/src/ui/snippets/combobox/input_group.rs`, `apps/fret-ui-gallery/src/ui/snippets/combobox/trigger_button.rs`, `apps/fret-ui-gallery/src/ui/snippets/combobox/groups_with_separator.rs`, `apps/fret-ui-gallery/src/ui/snippets/combobox/groups.rs`, `apps/fret-ui-gallery/src/ui/snippets/combobox/disabled.rs`, `apps/fret-ui-gallery/src/ui/snippets/combobox/custom_items.rs`, `apps/fret-ui-gallery/src/ui/snippets/combobox/clear_button.rs`, `apps/fret-ui-gallery/src/ui/snippets/combobox/invalid.rs`, `apps/fret-ui-gallery/src/ui/snippets/pagination/simple.rs`, `apps/fret-ui-gallery/src/ui/snippets/pagination/usage.rs`, `apps/fret-ui-gallery/src/ui/snippets/carousel/basic.rs`, `apps/fret-ui-gallery/src/ui/snippets/carousel/sizes.rs`, `apps/fret-ui-gallery/src/ui/snippets/carousel/plugin_wheel_gestures.rs`, `apps/fret-ui-gallery/src/ui/snippets/carousel/spacing_responsive.rs`, `apps/fret-ui-gallery/src/ui/snippets/carousel/loop_carousel.rs`, `apps/fret-ui-gallery/src/ui/snippets/carousel/options.rs`, `apps/fret-ui-gallery/src/ui/snippets/carousel/loop_downgrade_cannot_loop.rs`, `apps/fret-ui-gallery/src/ui/snippets/carousel/spacing.rs`, `apps/fret-ui-gallery/src/ui/snippets/carousel/usage.rs`, `apps/fret-ui-gallery/src/ui/snippets/carousel/sizes_thirds.rs`, `apps/fret-ui-gallery/src/ui/snippets/carousel/parts.rs`, `apps/fret-ui-gallery/src/ui/snippets/carousel/api.rs`, `apps/fret-ui-gallery/src/ui/snippets/carousel/duration_embla.rs`, `apps/fret-ui-gallery/src/ui/snippets/carousel/rtl.rs`, `apps/fret-ui-gallery/src/ui/snippets/carousel/plugin_autoplay.rs`, `apps/fret-ui-gallery/src/ui/snippets/carousel/plugin_autoplay_delays.rs`, `apps/fret-ui-gallery/src/ui/snippets/carousel/plugin_autoplay_controlled.rs`, `apps/fret-ui-gallery/src/ui/snippets/carousel/plugin_autoplay_stop_on_focus.rs`, `apps/fret-ui-gallery/src/ui/snippets/carousel/plugin_autoplay_stop_on_last_snap.rs`, `apps/fret-ui-gallery/src/ui/snippets/carousel/events.rs`, `apps/fret-ui-gallery/src/ui/snippets/skeleton/avatar.rs`, `apps/fret-ui-gallery/src/ui/snippets/skeleton/rtl.rs`, `apps/fret-ui-gallery/src/ui/snippets/skeleton/form.rs`, `apps/fret-ui-gallery/src/ui/snippets/skeleton/table.rs`, `apps/fret-ui-gallery/src/ui/snippets/popover/basic.rs`, `apps/fret-ui-gallery/src/ui/snippets/popover/demo.rs`, `apps/fret-ui-gallery/src/ui/snippets/popover/with_form.rs`, `apps/fret-ui-gallery/src/ui/snippets/resizable/usage.rs`, `apps/fret-ui-gallery/src/ui/snippets/resizable/vertical.rs`, `apps/fret-ui-gallery/src/ui/snippets/resizable/handle.rs`, `apps/fret-ui-gallery/src/ui/snippets/data_table/basic_demo.rs`, `apps/fret-ui-gallery/src/ui/snippets/data_table/default_demo.rs`, `apps/fret-ui-gallery/src/ui/snippets/data_table/guide_demo.rs`, `apps/fret-ui-gallery/src/ui/snippets/data_table/rtl_demo.rs`, `apps/fret-ui-gallery/src/ui/snippets/table/demo.rs`, `apps/fret-ui-gallery/src/ui/snippets/table/footer.rs`, `apps/fret-ui-gallery/src/ui/snippets/table/rtl.rs`, `apps/fret-ui-gallery/src/ui/snippets/table/actions.rs`, `apps/fret-ui-gallery/src/ui/snippets/dropdown_menu/mod.rs`, `apps/fret-ui-gallery/src/ui/snippets/dropdown_menu/avatar.rs`, `apps/fret-ui-gallery/src/ui/snippets/dropdown_menu/basic.rs`, `apps/fret-ui-gallery/src/ui/snippets/dropdown_menu/checkboxes.rs`, `apps/fret-ui-gallery/src/ui/snippets/dropdown_menu/checkboxes_icons.rs`, `apps/fret-ui-gallery/src/ui/snippets/dropdown_menu/complex.rs`, `apps/fret-ui-gallery/src/ui/snippets/dropdown_menu/demo.rs`, `apps/fret-ui-gallery/src/ui/snippets/dropdown_menu/destructive.rs`, `apps/fret-ui-gallery/src/ui/snippets/dropdown_menu/icons.rs`, `apps/fret-ui-gallery/src/ui/snippets/dropdown_menu/parts.rs`, `apps/fret-ui-gallery/src/ui/snippets/dropdown_menu/radio_group.rs`, `apps/fret-ui-gallery/src/ui/snippets/dropdown_menu/radio_icons.rs`, `apps/fret-ui-gallery/src/ui/snippets/dropdown_menu/rtl.rs`, `apps/fret-ui-gallery/src/ui/snippets/dropdown_menu/shortcuts.rs`, `apps/fret-ui-gallery/src/ui/snippets/dropdown_menu/submenu.rs`, `apps/fret-ui-gallery/src/ui/snippets/dropdown_menu/usage.rs`, `apps/fret-ui-gallery/src/ui/snippets/button_group/api_reference.rs`, `apps/fret-ui-gallery/src/ui/snippets/toggle_group/size.rs`, `apps/fret-ui-gallery/src/ui/snippets/drawer/demo.rs`, `apps/fret-ui-gallery/src/ui/snippets/drawer/responsive_dialog.rs`, `apps/fret-ui-gallery/src/ui/snippets/drawer/sides.rs`, `apps/fret-ui-gallery/src/ui/snippets/drawer/scrollable_content.rs`, `apps/fret-ui-gallery/src/ui/snippets/dialog/demo.rs`, `apps/fret-ui-gallery/src/ui/snippets/dialog/rtl.rs`, `apps/fret-ui-gallery/src/ui/snippets/dialog/scrollable_content.rs`, `apps/fret-ui-gallery/src/ui/snippets/dialog/sticky_footer.rs`, `apps/fret-ui-gallery/src/ui/snippets/separator/menu.rs`, `apps/fret-ui-gallery/src/ui/snippets/separator/list.rs`, `apps/fret-ui-gallery/src/ui/snippets/sheet/demo.rs`, `apps/fret-ui-gallery/src/ui/snippets/sheet/rtl.rs`, `apps/fret-ui-gallery/src/ui/snippets/item/avatar.rs`, `apps/fret-ui-gallery/src/ui/snippets/item/icon.rs`, `apps/fret-ui-gallery/src/ui/snippets/item/link.rs`, `apps/fret-ui-gallery/src/ui/snippets/item/link_render.rs`, `apps/fret-ui-gallery/src/ui/snippets/item/dropdown.rs`, `apps/fret-ui-gallery/src/ui/snippets/item/extras_rtl.rs`, `apps/fret-ui-gallery/src/ui/snippets/item/gallery.rs`, `apps/fret-ui-gallery/src/ui/snippets/toast/deprecated.rs`, `apps/fret-ui-gallery/src/ui/snippets/motion_presets/fluid_tabs_demo.rs`, `apps/fret-ui-gallery/src/ui/snippets/tooltip/rtl.rs`, `apps/fret-ui-gallery/src/ui/snippets/tooltip/sides.rs`, `apps/fret-ui-gallery/src/ui/snippets/tabs/demo.rs`, `apps/fret-ui-gallery/src/ui/snippets/collapsible/basic.rs`, `apps/fret-ui-gallery/src/ui/snippets/collapsible/settings_panel.rs`, `apps/fret-ui-gallery/src/ui/snippets/collapsible/rtl.rs`, `apps/fret-ui-gallery/src/ui/snippets/collapsible/file_tree.rs`, `apps/fret-ui-gallery/tests/ui_authoring_surface_default_app.rs` |
| Raw explicit IR | `AnyElement` and `Elements` are used widely in low-level helpers, tests, overlays, diagnostics, and manual assembly | retain raw types explicitly on advanced/internal surfaces | document raw use as intentional rather than accidental | raw surfaces are clearly documented as advanced/internal rather than default teaching | In progress | `ecosystem/fret/src/lib.rs`, `crates/fret-ui/src/`, `apps/fret-ui-gallery/src/driver/` |

Execution note on 2026-03-13:

- the focused `selected_*` source gate now also covers
  `apps/fret-ui-gallery/src/ui/snippets/ai/{attachments_usage,file_tree_demo,speech_input_demo}.rs`,
  where `render_grid_attachment(...)`, `invisible_marker(...)`, `body_text(...)`, and
  `clear_action(...)` now prefer typed helper signatures, with the speech-input pair now on the
  default-app `UiCx -> impl UiChild` posture, while keeping
  explicit `.into_element(cx)` only at `Attachments::new(...)`, `Vec<AnyElement>` child arrays,
  and semantics-decoration seams.
- the focused `selected_*` source gate now also covers
  `apps/fret-ui-gallery/src/ui/snippets/ai/{attachments_grid,attachments_list,file_tree_large}.rs`,
  where `render_grid_attachment(...)`, `render_list_attachment(...)`, `invisible_marker(...)`,
  and `preview(...)` now prefer `IntoUiElement<H>`-based helper signatures while keeping explicit
  `.into_element(cx)` only at `Attachments::new(...)`, `Vec<AnyElement>` child arrays,
  semantics-decoration seams, and the intentional `render(cx)` landing boundary used by
  `file_tree_large::preview(...)`.
- the internal gallery wrapper gate now also records
  `apps/fret-ui-gallery/src/ui/doc_layout.rs` and
  `apps/fret-ui-gallery/src/ui/previews/pages/editors/code_editor/mvp/gates.rs`,
  where `demo_shell<B>(...)` and `gate_panel<B>(...)` now accept `IntoUiElement<fret_app::App>`
  inputs and reserve `.into_element(cx)` for the shell/panel child-landing seams instead of
  forcing callers to pre-land `AnyElement`.
- the same gallery helper inventory now also records the internal docs prose/layout helpers:
  `apps/fret-ui-gallery/src/ui/doc_layout.rs::{wrap_row,wrap_controls_row,text_table,muted_full_width,muted_inline}` and the local
  `notes_block(... )::muted_flex_1_min_w_0(...)` now use the default-app
  `UiCx -> impl UiChild` posture instead of host-generic `ElementContext<'_, H> -> AnyElement`.
- the same gallery helper inventory now also records the intentionally retained raw doc-layout
  boundaries:
  `apps/fret-ui-gallery/src/ui/doc_layout.rs::render_doc_page(...)` now also stays on a typed
  helper signature, with final landing moved back to each page/preview return site.
  The remaining intentional raw storage on this lane is now the landed `DocSection.preview` field
  plus the tuple-return `gap_card(...)` placeholder seam. `wrap_preview_page(...)`, `icon(...)`,
  `render_section(...)`, `preview_code_tabs(...)`, `code_block_shell(...)`, and
  `section_title(...)` also stay on typed helper signatures, with explicit landing moved either
  into the surrounding scaffold or back to the preview-page call sites;
  `ui_authoring_surface_default_app::gallery_doc_layout_retains_only_intentional_raw_boundaries`,
  `ui_authoring_surface_default_app::render_doc_page_callers_land_the_typed_doc_page_explicitly`, and
  `ui_authoring_surface_internal_previews::wrap_preview_page_callers_land_the_typed_preview_shell_explicitly`
  plus
  `ui_authoring_surface_internal_previews::render_doc_page_callers_land_the_typed_doc_page_explicitly`
  now lock that audited split.
- the same AI teaching lane now also records the page-level section cleanup:
  `apps/fret-ui-gallery/src/ui/pages/ai_*.rs` now use `DocSection::build(cx, ...)` instead of
  `DocSection::new(...)` for first-party demo sections, and
  `ui_authoring_surface_default_app::curated_ai_doc_pages_use_typed_doc_sections` now locks that
  typed docs posture across the full AI page surface.
- the same gallery helper inventory now also records the intentionally retained internal overlay
  preview seams:
  `apps/fret-ui-gallery/src/ui/previews/gallery/overlays/overlay.rs`,
  `overlay/layout.rs`,
  `overlay/widgets.rs`, and
  `overlay/flags.rs` no longer stay fully raw: `layout.rs::{row,row_end,compose_body}` plus
  `flags.rs::last_action_status(...)` now expose typed helper outputs, while
  `preview_overlay(...)`, `OverlayWidgets`, and `status_flags(...)` remain raw because this
  diagnostics surface still stores landed overlay roots and returns a concrete result vector;
  `ui_authoring_surface_internal_previews::gallery_overlay_preview_retains_intentional_raw_boundaries`
  now locks that audited inventory.
- the same default-app helper inventory now also records the low-traffic notification cleanup:
  `apps/fret-ui-gallery/src/ui/snippets/toast/deprecated.rs` now exposes the typed
  `UiCx -> impl UiChild` render surface,
  `apps/fret-ui-gallery/src/ui/pages/toast.rs` now uses `DocSection::build(cx, ...)`, and
  `apps/fret-ui-gallery/src/ui/snippets/sonner/mod.rs::local_toaster(...)` now keeps the local
  toaster helper on `UiChild` with the explicit landing seam living only at
  `apps/fret-ui-gallery/src/ui/pages/sonner.rs`.
- the same internal preview helper inventory now also records the code-editor MVP cleanup:
  `apps/fret-ui-gallery/src/ui/previews/pages/editors/code_editor/mvp/{header,word_boundary,gates}.rs`
  now keep `build_header(...)`, `word_boundary_controls(...)`,
  `word_boundary_debug_view(...)`, `gate_panel(...)`, and the `*_gate(...)` helpers on the typed
  `UiChild` lane, while
  `apps/fret-ui-gallery/src/ui/previews/pages/editors/code_editor/mvp.rs` performs the explicit
  `.into_element(cx)` only when feeding those helpers into the final preview page raw boundary.
- the same internal preview helper inventory now also records a few lower-traffic cleanups:
  `apps/fret-ui-gallery/src/ui/previews/pages/editors/text/outline_stroke.rs::toggle_button(...)`
  and
  `apps/fret-ui-gallery/src/ui/previews/pages/editors/text/mixed_script_fallback.rs::sample_row(...)`
  plus
  `apps/fret-ui-gallery/src/ui/previews/pages/editors/text/feature_toggles.rs::{toggle_button,sample_text}(...)`
  now keep those local preview helpers on `UiChild`, and
  `apps/fret-ui-gallery/src/ui/previews/pages/harness/intro.rs::{card(...),preview_intro(...)}`
  now use the same typed-helper posture while keeping `DocSection::build(cx, ...)` for overview
  registration instead of the eager `DocSection::new(...)` path.
- the shadcn surface gate now also records
  `ecosystem/fret-ui-shadcn/src/{context_menu,dropdown_menu,menubar}.rs`,
  where the internal `menu_icon_slot(...)` wrappers now accept `IntoUiElement<H>` inputs instead
  of forcing pre-landed `AnyElement`, while keeping the wrapper output itself as the explicit
  landed menu-slot seam.
- the shadcn surface gate now also records the current thin public constructor/wrapper trial:
  `ecosystem/fret-ui-shadcn/src/badge.rs::badge<H, T>(...)`,
  `ecosystem/fret-ui-shadcn/src/command.rs::command<H, I, F, T>(...)`,
  `ecosystem/fret-ui-shadcn/src/input_group.rs::input_group<H>(...)`,
  `ecosystem/fret-ui-shadcn/src/input_otp.rs::input_otp<H>(...)`,
  `ecosystem/fret-ui-shadcn/src/kbd.rs::kbd<H, T>(...)`, and
  `ecosystem/fret-ui-shadcn/src/separator.rs::separator<H>()` now expose typed constructor or
  wrapper outputs, while `ecosystem/fret-ui-shadcn/src/kbd.rs::kbd_icon<H>(...)` remains
  intentionally raw because `Kbd::from_children(...)` still owns a concrete `Vec<AnyElement>`
  child seam.
- the same shadcn surface gate now also records the final wrapper-seam decision for
  `ecosystem/fret-ui-shadcn/src/text_edit_context_menu.rs::{text_edit_context_menu,
  text_selection_context_menu, text_edit_context_menu_controllable,
  text_selection_context_menu_controllable}`:
  these helpers keep `-> AnyElement` deliberately because `ContextMenu::build(...)` is the actual
  root landing boundary, while the trigger input stays on `IntoUiElement<H>`.
- the canonical compare/example helper lane is now closed outside intentional retained seams:
  `apps/fret-examples/src/todo_demo.rs::todo_page(...)`,
  `apps/fret-examples/src/simple_todo_demo.rs::todo_row(...)`,
  `apps/fret-examples/src/imui_editor_proof_demo.rs::{render_editor_name_assist_surface,render_authoring_parity_surface,render_authoring_parity_shared_state,render_authoring_parity_declarative_group,render_authoring_parity_imui_group,render_authoring_parity_imui_host}`,
  and `apps/fretboard/src/scaffold/templates.rs::{todo_page(...),simple_todo::todo_page(...)}`
  now stay on `UiChild` / `IntoUiElement<...>`-based signatures, and the current non-`lib.rs`
  scan of `apps/fret-examples/src` plus `apps/fret-cookbook/examples` leaves only
  `apps/fret-cookbook/examples/chart_interactions_basics.rs::chart_canvas(...) -> AnyElement` as
  the remaining intentional helper seam on that lane.
- the internal declarative semantics surface now also records
  `ecosystem/fret-ui-kit/src/declarative/semantics.rs`,
  where `UiElementTestIdExt`, `UiElementA11yExt`, and `UiElementKeyContextExt` wrappers now land
  through `IntoUiElement<H>` directly, with a source-policy gate asserting that the production
  semantics helper surface no longer depends on `UiIntoElement`.
- the built-in text primitive surface now also records
  `ecosystem/fret-ui-kit/src/ui.rs`,
  where `TextBox` and `RawTextBox` now implement `IntoUiElement<H>` directly, with a source-policy
  gate asserting that the production `ui.rs` surface no longer depends on `UiIntoElement`.
- `ecosystem/fret-ui-kit/src/ui_builder.rs` now also records the final bridge cleanup:
  the legacy `UiIntoElement` trait is deleted entirely, and explicit raw seams now rely on the
  direct `IntoUiElement<H> for AnyElement` implementation instead of an internal legacy bridge.
- the focused `selected_*` source gate now also covers
  `apps/fret-ui-gallery/src/ui/snippets/sidebar/{demo,controlled,mobile,rtl}.rs`,
  where `menu_button(...)` moved from raw `AnyElement` returns to `IntoUiElement`-based helper
  signatures while keeping `SidebarMenuItem::new(...)` as the explicit landing seam.
- the focused `selected_*` source gate now also covers
  `apps/fret-ui-gallery/src/ui/snippets/aspect_ratio/{portrait,square,rtl}.rs`,
  where `portrait_image(...)`, `square_image(...)`, `rtl_image(...)`, and `ratio_example(...)`
  moved from raw `AnyElement` returns to `IntoUiElement`-based helper signatures while keeping
  `AspectRatio::with_child(...)` and render boundaries as explicit landing seams.
- the focused `selected_*` source gate now also covers
  `apps/fret-ui-gallery/src/ui/snippets/combobox/{long_list,input_group,trigger_button,groups_with_separator,groups,disabled,custom_items,clear_button,invalid}.rs`,
  where `state_rows(...)` joined `state_row(...)` on `IntoUiElement<fret_app::App>`-based helper
  signatures while keeping explicit `.into_element(cx)` only at sibling child-collection seams.
- the UI Gallery default-app source gate now also records the first top-level snippet-family move
  from eager landing to typed app-facing return values:
  `apps/fret-ui-gallery/src/ui/snippets/accordion/{basic,borders,card,demo,disabled,extras,multiple,rtl,usage}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, while
  `apps/fret-ui-gallery/src/ui/pages/accordion.rs` consumes those previews through
  `DocSection::build(cx, ...)`; the old `pub fn render(...) -> AnyElement` teaching pattern is now
  forbidden for that family by
  `ui_authoring_surface_default_app::{accordion_app_facing_snippets_prefer_ui_cx_on_the_default_app_surface,accordion_page_uses_typed_doc_sections_for_app_facing_snippets}`.
- the same UI Gallery default-app source gate now also records the next top-level snippet-family
  move on the tabs lane:
  `apps/fret-ui-gallery/src/ui/snippets/tabs/{demo,disabled,extras,icons,line,list,rtl,usage,vertical,vertical_line}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, while
  `apps/fret-ui-gallery/src/ui/pages/tabs.rs` consumes those previews through
  `DocSection::build(cx, ...)`; the old `pub fn render(...) -> AnyElement` teaching pattern is now
  forbidden for that family by
  `ui_authoring_surface_default_app::{tabs_app_facing_snippets_prefer_ui_cx_on_the_default_app_surface,tabs_page_uses_typed_doc_sections_for_app_facing_snippets}`.
- the same UI Gallery default-app source gate now also records the next top-level snippet-family
  move on the toggle lane:
  `apps/fret-ui-gallery/src/ui/snippets/toggle/{demo,disabled,label,outline,rtl,size,usage,with_text}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, while
  `apps/fret-ui-gallery/src/ui/pages/toggle.rs` consumes those previews through
  `DocSection::build(cx, ...)`; the old `pub fn render(...) -> AnyElement` teaching pattern is now
  forbidden for that family by
  `ui_authoring_surface_default_app::{toggle_app_facing_snippets_prefer_ui_cx_on_the_default_app_surface,toggle_page_uses_typed_doc_sections_for_app_facing_snippets}`.
- the same UI Gallery default-app source gate now also records the next top-level snippet-family
  move on the radio-group lane:
  `apps/fret-ui-gallery/src/ui/snippets/radio_group/{choice_card,demo,description,disabled,extras,fieldset,invalid,label,plans,rtl,usage}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, while
  `apps/fret-ui-gallery/src/ui/pages/radio_group.rs` consumes those previews through
  `DocSection::build(cx, ...)`; the old `pub fn render(...) -> AnyElement` teaching pattern is now
  forbidden for that family by
  `ui_authoring_surface_default_app::{radio_group_app_facing_snippets_prefer_ui_cx_on_the_default_app_surface,radio_group_page_uses_typed_doc_sections_for_app_facing_snippets}`.
- the same UI Gallery default-app source gate now also records the next top-level snippet-family
  move on the slider lane:
  `apps/fret-ui-gallery/src/ui/snippets/slider/{demo,extras,label,usage}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, keep their local model
  state inside the snippet instead of routing it through `pages/slider.rs`, while
  `apps/fret-ui-gallery/src/ui/pages/slider.rs` consumes those previews through
  `DocSection::build(cx, ...)`; the old `pub fn render(...) -> AnyElement` teaching pattern is now
  forbidden for that family by
  `ui_authoring_surface_default_app::{slider_app_facing_snippets_prefer_ui_cx_on_the_default_app_surface,slider_page_uses_typed_doc_sections_for_app_facing_snippets}`.
- the same UI Gallery default-app source gate now also records the next top-level snippet-family
  move on the native-select lane:
  `apps/fret-ui-gallery/src/ui/snippets/native_select/{demo,disabled,invalid,label,rtl,usage,with_groups}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, keep value/open model
  state inside each snippet instead of routing it through `pages/native_select.rs`, while
  `apps/fret-ui-gallery/src/ui/pages/native_select.rs` consumes those previews through
  `DocSection::build(cx, ...)`; the old `pub fn render(...) -> AnyElement` teaching pattern is now
  forbidden for that family by
  `ui_authoring_surface_default_app::{native_select_app_facing_snippets_prefer_ui_cx_on_the_default_app_surface,native_select_page_uses_typed_doc_sections_for_app_facing_snippets}`.
- the same UI Gallery default-app source gate now also records the next top-level snippet-family
  move on the resizable lane:
  `apps/fret-ui-gallery/src/ui/snippets/resizable/{demo,handle,notes,rtl,usage,vertical}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, keep fractions model
  state inside the snippet instead of routing it through `pages/resizable.rs`,
  `ui/content.rs`, `ui/models.rs`, and runtime-driver bootstrap relay fields, while
  `apps/fret-ui-gallery/src/ui/pages/resizable.rs` consumes those previews through
  `DocSection::build(cx, ...)`; the old `pub fn render(...) -> AnyElement` teaching pattern is now
  forbidden for that family by
  `ui_authoring_surface_default_app::{resizable_app_facing_snippets_prefer_ui_cx_on_the_default_app_surface,resizable_page_uses_typed_doc_sections_for_app_facing_snippets}`.
- the same UI Gallery default-app source gate now also records the next top-level snippet-family
  move on the navigation-menu lane:
  `apps/fret-ui-gallery/src/ui/snippets/navigation_menu/{demo,docs_demo,link_component,rtl,usage}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, keep local menu value
  state inside the snippet instead of routing it through a page-local relay, while
  `apps/fret-ui-gallery/src/ui/pages/navigation_menu.rs` consumes those previews through
  `DocSection::build(cx, ...)`; the old `pub fn render(...) -> AnyElement` teaching pattern is now
  forbidden for that family by
  `ui_authoring_surface_default_app::{navigation_menu_app_facing_snippets_prefer_ui_cx_on_the_default_app_surface,navigation_menu_page_uses_typed_doc_sections_for_app_facing_snippets}`.
- the same UI Gallery default-app source gate now also records the next top-level snippet-family
  move on the scroll-area lane:
  `apps/fret-ui-gallery/src/ui/snippets/scroll_area/{demo,usage,horizontal,nested_scroll_routing,rtl}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, keep the app-facing docs
  examples on `shadcn::scroll_area(cx, |_cx| [...])`, while
  `apps/fret-ui-gallery/src/ui/pages/scroll_area.rs` consumes those previews through
  `DocSection::build(cx, ...)` and intentionally keeps
  `drag_baseline` / `expand_at_bottom` on diagnostics-owned `DocSection::new(...)`; the old
  `pub fn render(...) -> AnyElement` teaching pattern is now forbidden for that family by
  `ui_authoring_surface_default_app::{scroll_area_app_facing_snippets_prefer_ui_cx_on_the_default_app_surface,scroll_area_page_uses_typed_doc_sections_for_app_facing_snippets,scroll_area_diagnostics_snippets_remain_intentional_raw_boundaries}`, while
  `selected_scroll_area_snippet_helpers_prefer_into_ui_element_over_anyelement` keeps the
  non-diagnostics helper family on the typed `shadcn::scroll_area(...)` lane.
- the same UI Gallery default-app source gate now also records the next top-level snippet-family
  move on the progress lane:
  `apps/fret-ui-gallery/src/ui/snippets/progress/{demo,usage,label,controlled,rtl}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, while
  `apps/fret-ui-gallery/src/ui/pages/progress.rs` consumes those previews through
  `DocSection::build(cx, ...)`; the old `pub fn render(...) -> AnyElement` teaching pattern is now
  forbidden for that family by
  `ui_authoring_surface_default_app::{progress_snippets_prefer_ui_cx_on_the_default_app_surface,progress_page_uses_typed_doc_sections_for_app_facing_snippets}`.
- the same UI Gallery default-app source gate now also records the next top-level snippet-family
  move on the chart lane:
  `apps/fret-ui-gallery/src/ui/snippets/chart/{demo,usage,contracts,tooltip,legend,rtl}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, while
  `apps/fret-ui-gallery/src/ui/pages/chart.rs` consumes those previews through
  `DocSection::build(cx, ...)`; the old `pub fn render(...) -> AnyElement` teaching pattern is now
  forbidden for that family by
  `ui_authoring_surface_default_app::{chart_snippets_prefer_ui_cx_on_the_default_app_surface,chart_page_uses_typed_doc_sections_for_app_facing_snippets}`.
- the same UI Gallery default-app source gate now also records the next top-level snippet-family
  move on the combobox lane:
  `apps/fret-ui-gallery/src/ui/snippets/combobox/{conformance_demo,basic,usage,label,auto_highlight,clear_button,groups,groups_with_separator,trigger_button,multiple_selection,custom_items,long_list,invalid,disabled,input_group,rtl}.rs`
  now expose typed app-facing `render(...)` signatures, while
  `apps/fret-ui-gallery/src/ui/pages/combobox.rs` consumes those previews through
  `DocSection::build(cx, ...)`; the old `-> AnyElement` teaching pattern is now forbidden for that
  family by
  `ui_authoring_surface_default_app::{combobox_snippets_prefer_ui_cx_on_the_default_app_surface,combobox_page_uses_typed_doc_sections_for_app_facing_snippets}`.
- the same UI Gallery default-app source gate now also records the next top-level snippet-family
  move on the carousel lane:
  `apps/fret-ui-gallery/src/ui/snippets/carousel/{demo,usage,parts,basic,sizes_thirds,sizes,spacing,spacing_responsive,orientation_vertical,options,api,events,plugin_autoplay,plugin_autoplay_controlled,plugin_autoplay_stop_on_focus,plugin_autoplay_stop_on_last_snap,plugin_autoplay_delays,plugin_wheel_gestures,rtl,loop_carousel,loop_downgrade_cannot_loop,focus_watch,duration_embla,expandable}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, while
  `apps/fret-ui-gallery/src/ui/pages/carousel.rs` consumes those previews through
  `DocSection::build(cx, ...)`; the old `pub fn render(...) -> AnyElement` teaching pattern is now
  forbidden for that family by
  `ui_authoring_surface_default_app::{carousel_snippets_prefer_ui_cx_on_the_default_app_surface,carousel_page_uses_typed_doc_sections_for_app_facing_snippets}`.
- the same UI Gallery default-app source gate now also records the next top-level snippet-family
  move on the item lane:
  `apps/fret-ui-gallery/src/ui/snippets/item/{demo,usage,variants,size,icon,avatar,image,group,header,link,dropdown,extras_rtl,gallery,link_render}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, while
  `apps/fret-ui-gallery/src/ui/pages/item.rs` consumes those previews through
  `DocSection::build(cx, ...)`; the old `pub fn render(...) -> AnyElement` teaching pattern is now
  forbidden for that family by
  `ui_authoring_surface_default_app::{item_snippets_prefer_ui_cx_on_the_default_app_surface,item_page_uses_typed_doc_sections_for_app_facing_snippets}`.
- the same UI Gallery default-app source gate now also records the next top-level snippet-family
  move on the table lane:
  `apps/fret-ui-gallery/src/ui/snippets/table/{demo,usage,footer,actions,rtl}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, while
  `apps/fret-ui-gallery/src/ui/pages/table.rs` consumes those previews through
  `DocSection::build(cx, ...)`; the old `pub fn render(...) -> AnyElement` teaching pattern is now
  forbidden for that family by
  `ui_authoring_surface_default_app::{table_app_facing_snippets_prefer_ui_cx_on_the_default_app_surface,table_page_uses_typed_doc_sections_for_app_facing_snippets}`.
- the same UI Gallery default-app source gate now also records the remaining curated tail
  snippets:
  `apps/fret-ui-gallery/src/ui/snippets/breadcrumb/responsive.rs`,
  `apps/fret-ui-gallery/src/ui/snippets/date_picker/dropdowns.rs`,
  `apps/fret-ui-gallery/src/ui/snippets/form/notes.rs`, and
  `apps/fret-ui-gallery/src/ui/snippets/sidebar/rtl.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, while the corresponding
  app-facing sections on `apps/fret-ui-gallery/src/ui/pages/{breadcrumb,date_picker,form,sidebar}.rs`
  now stay on `DocSection::build(cx, ...)`; the old `UiCx -> AnyElement` teaching pattern is now
  forbidden there by
  `ui_authoring_surface_default_app::{remaining_app_facing_tail_snippets_prefer_ui_cx_on_the_default_app_surface,remaining_app_facing_tail_pages_use_typed_doc_sections_for_app_facing_snippets}`.
- the same UI Gallery default-app source gate now also records the `badge` family:
  `apps/fret-ui-gallery/src/ui/snippets/badge/{demo,usage,spinner,rtl,counts,colors,link,icon,variants}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, while
  `apps/fret-ui-gallery/src/ui/pages/badge.rs` consumes those previews through
  `DocSection::build(cx, ...)`; the old `ElementContext<'_, H> -> AnyElement` teaching pattern is
  now forbidden there by
  `ui_authoring_surface_default_app::{badge_snippets_prefer_ui_cx_on_the_default_app_surface,badge_page_uses_typed_doc_sections_for_app_facing_snippets}`.
- the same UI Gallery default-app source gate now also records the `aspect_ratio` family:
  `apps/fret-ui-gallery/src/ui/snippets/aspect_ratio/{demo,usage,portrait,square,rtl}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`; the page surface on
  `apps/fret-ui-gallery/src/ui/pages/aspect_ratio.rs` already stays on
  `DocSection::build(cx, ...)`, while `render_preview(...)` remains the explicit asset-backed
  preview seam and the top-level `render(...)` functions stay as the copyable code-surface seam.
  This is now guarded by
  `ui_authoring_surface_default_app::{aspect_ratio_snippets_prefer_ui_cx_on_the_default_app_surface,aspect_ratio_page_uses_typed_doc_sections_for_app_facing_snippets}`.
- the same UI Gallery default-app source gate now also records the `context_menu` family:
  `apps/fret-ui-gallery/src/ui/snippets/context_menu/{demo,basic,usage,submenu,shortcuts,groups,icons,checkboxes,radio,destructive,sides,rtl}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, while
  `apps/fret-ui-gallery/src/ui/pages/context_menu.rs` consumes those previews through
  `DocSection::build(cx, ...)`; the old `ElementContext<'_, H> -> AnyElement` teaching pattern is
  now forbidden there by
  `ui_authoring_surface_default_app::{context_menu_snippets_prefer_ui_cx_on_the_default_app_surface,context_menu_page_uses_typed_doc_sections_for_app_facing_snippets}`.
- the same UI Gallery default-app source gate now also records the `dropdown_menu` family:
  `apps/fret-ui-gallery/src/ui/snippets/dropdown_menu/{avatar,basic,checkboxes,checkboxes_icons,complex,demo,destructive,icons,parts,radio_group,radio_icons,rtl,shortcuts,submenu,usage}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, keep their menu-local
  checkbox/radio/demo state beside the snippet itself, and
  `apps/fret-ui-gallery/src/ui/pages/dropdown_menu.rs` now consumes those previews through
  `DocSection::build(cx, ...)`; the old `ElementContext<'_, H> -> AnyElement` teaching pattern is
  now forbidden there by
  `ui_authoring_surface_default_app::{dropdown_menu_snippets_prefer_ui_cx_on_the_default_app_surface,dropdown_menu_page_uses_typed_doc_sections_for_app_facing_snippets}`.
- the same UI Gallery default-app source gate now also records the `menubar` family:
  `apps/fret-ui-gallery/src/ui/snippets/menubar/{checkbox,demo,parts,radio,rtl,submenu,usage,with_icons}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, keep their menubar-local
  checkbox/radio/demo state beside the snippet itself, and
  `apps/fret-ui-gallery/src/ui/pages/menubar.rs` now consumes those previews through
  `DocSection::build(cx, ...)`; the old `ElementContext<'_, H> -> AnyElement` teaching pattern is
  now forbidden there by
  `ui_authoring_surface_default_app::{menubar_snippets_prefer_ui_cx_on_the_default_app_surface,menubar_page_uses_typed_doc_sections_for_app_facing_snippets}`.
- the same UI Gallery default-app source gate now also records the `popover` family:
  `apps/fret-ui-gallery/src/ui/snippets/popover/{align,basic,demo,rtl,usage,with_form}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, keep their popover-local
  form state beside the snippet itself, and
  `apps/fret-ui-gallery/src/ui/pages/popover.rs` now consumes those previews through
  `DocSection::build(cx, ...)`; the old `ElementContext<'_, H> -> AnyElement` teaching pattern is
  now forbidden there by
  `ui_authoring_surface_default_app::{popover_snippets_prefer_ui_cx_on_the_default_app_surface,popover_page_uses_typed_doc_sections_for_app_facing_snippets}`.
- the same UI Gallery default-app source gate now also records the `hover_card` family:
  `apps/fret-ui-gallery/src/ui/snippets/hover_card/{basic,demo,positioning,rtl,sides,trigger_delays,usage}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, keep their hover-card
  demo assets and timing/placement state beside the snippet itself, and
  `apps/fret-ui-gallery/src/ui/pages/hover_card.rs` now consumes those previews through
  `DocSection::build(cx, ...)`; the old `ElementContext<'_, H> -> AnyElement` teaching pattern is
  now forbidden there by
  `ui_authoring_surface_default_app::{hover_card_snippets_prefer_ui_cx_on_the_default_app_surface,hover_card_page_uses_typed_doc_sections_for_app_facing_snippets}`.
- the same UI Gallery default-app source gate now also records the `tooltip` family:
  `apps/fret-ui-gallery/src/ui/snippets/tooltip/{demo,disabled_button,keyboard_focus,keyboard_shortcut,long_content,rtl,sides,usage}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, keep their tooltip-local
  provider/content composition beside the snippet itself, and
  `apps/fret-ui-gallery/src/ui/pages/tooltip.rs` now consumes those previews through
  `DocSection::build(cx, ...)`; the old `ElementContext<'_, H> -> AnyElement` teaching pattern is
  now forbidden there by
  `ui_authoring_surface_default_app::{tooltip_snippets_prefer_ui_cx_on_the_default_app_surface,tooltip_page_uses_typed_doc_sections_for_app_facing_snippets}`.
- the same UI Gallery default-app source gate now also records the `button` family:
  `apps/fret-ui-gallery/src/ui/snippets/button/{demo,usage,size,default,outline,secondary,ghost,destructive,link,icon,with_icon,rounded,loading,button_group,link_render,rtl,variants}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, while
  `apps/fret-ui-gallery/src/ui/pages/button.rs` consumes those previews through
  `DocSection::build(cx, ...)`; the old `ElementContext<'_, H> -> AnyElement` teaching pattern is
  now forbidden there by
  `ui_authoring_surface_default_app::{button_snippets_prefer_ui_cx_on_the_default_app_surface,button_page_uses_typed_doc_sections_for_app_facing_snippets}`.
- the same UI Gallery default-app source gate now also records the `button_group` family:
  `apps/fret-ui-gallery/src/ui/snippets/button_group/{accessibility,button_group_select,demo,dropdown_menu,flex_1_items,input,input_group,nested,orientation,popover,rtl,separator,size,split,text,usage}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, while
  `apps/fret-ui-gallery/src/ui/pages/button_group.rs` consumes those previews through
  `DocSection::build(cx, ...)`; the old `ElementContext<'_, H> -> AnyElement` teaching pattern is
  now forbidden there by
  `ui_authoring_surface_default_app::{button_group_snippets_prefer_ui_cx_on_the_default_app_surface,button_group_page_uses_typed_doc_sections_for_app_facing_snippets}`.
- the same UI Gallery default-app source gate now also records the `data_table` family:
  `apps/fret-ui-gallery/src/ui/snippets/data_table/{basic_demo,code_outline,default_demo,guide_demo,rtl_demo}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, while
  `apps/fret-ui-gallery/src/ui/pages/data_table.rs` consumes those previews through
  `DocSection::build(cx, ...)`; `guide_demo` now keeps its `TableState` inside the snippet, and
  the obsolete gallery relay fields `data_table_state` plus `image_fit_demo_streaming_image` are
  now deleted from `ui/models.rs`, `ui/content.rs`, `driver/window_bootstrap.rs`, and
  `driver/runtime_driver.rs`. The old teaching pattern is now forbidden there by
  `ui_authoring_surface_default_app::{data_table_*,selected_data_table_*}`.
- the same UI Gallery default-app source gate now also records the `motion_presets` family:
  `apps/fret-ui-gallery/src/ui/snippets/motion_presets/{preset_selector,fluid_tabs_demo,overlay_demo,stack_shift_list_demo,stagger_demo,token_snapshot}.rs`
  now expose typed top-level `render(...)` surfaces returning `impl UiChild + use<>`, while
  `apps/fret-ui-gallery/src/ui/pages/motion_presets.rs` now consumes those previews through
  `DocSection::build(cx, ...)`; `preset_selector` intentionally remains the explicit global
  motion-preset seam, while the remaining demos now keep their dialog/theme access inside the
  snippet itself. The old teaching pattern is now forbidden there by
  `ui_authoring_surface_default_app::{motion_preset_snippets_prefer_ui_cx_on_the_default_app_surface,motion_presets_page_uses_typed_doc_sections_for_app_facing_snippets,selected_motion_presets_snippet_helpers_prefer_into_ui_element_over_anyelement}`.
- after the `data_table` + `motion_presets` landings, the tracked default-app tail on this lane is
  effectively closed; remaining follow-up work is now specialized-lane work rather than baseline
  app-surface teaching drift.
- the specialized `typography` teaching lane now also matches the typed first-party posture:
  `apps/fret-ui-gallery/src/ui/snippets/typography/*.rs` now expose `UiCx -> impl UiChild`,
  `apps/fret-ui-gallery/src/ui/pages/typography.rs` now uses `DocSection::build(cx, ...)`, and
  the stale non-dev `dialog_open` relay is now gated back to `gallery-dev` only.
- the specialized `shadcn_extras` teaching lane now also matches that typed first-party posture:
  `apps/fret-ui-gallery/src/ui/snippets/shadcn_extras/*.rs` now expose `UiCx -> impl UiChild`,
  and `apps/fret-ui-gallery/src/ui/pages/shadcn_extras.rs` now uses
  `DocSection::build(cx, ...)`.
- the specialized `material3` lane has now also started converging on that posture through the
  `controls` sub-batch:
  `apps/fret-ui-gallery/src/ui/snippets/material3/{badge,button,checkbox,icon_button,radio,segmented_button,slider,switch,touch_targets}.rs`
  now expose `UiCx -> impl UiChild`, and the corresponding source gate now forbids
  `ElementContext<'_, H>` helper parameters in the affected copyable controls snippets.
- the specialized `material3` lane has now also converged further through the `inputs` sub-batch:
  `apps/fret-ui-gallery/src/ui/snippets/material3/{autocomplete,date_picker,select,text_field,time_picker}.rs`
  now expose `UiCx -> impl UiChild`, and the corresponding field-family gates now lock both the
  typed top-level teaching surface and the local uncontrolled/copyable-root ownership story.
- the specialized `material3` lane has now also converged further through the `navigation`
  sub-batch:
  `apps/fret-ui-gallery/src/ui/snippets/material3/{list,modal_navigation_drawer,navigation_bar,navigation_drawer,navigation_rail,tabs,top_app_bar}.rs`
  now expose `UiCx -> impl UiChild`, and the corresponding navigation/value-root gates now lock
  both the typed top-level teaching surface and the removal of host-bound helper spellings from
  the affected exemplar helpers.
- the specialized `material3` lane has now also converged further through the `overlays`
  sub-batch:
  `apps/fret-ui-gallery/src/ui/snippets/material3/{bottom_sheet,dialog,menu,snackbar,tooltip}.rs`
  now expose `UiCx -> impl UiChild`, and the corresponding overlay gates now lock both the typed
  top-level teaching surface and the local uncontrolled/copyable-root posture for the
  dialog/menu/bottom-sheet exemplars.
- the specialized `material3` lane is now fully aligned on the first-party default teaching
  surface:
  `apps/fret-ui-gallery/src/ui/snippets/material3/{gallery,state_matrix}.rs` now also expose the
  typed `UiCx -> impl UiChild` posture, their remaining helper signatures no longer spell
  `ElementContext<'_, H>`, and the composite source gates now close the last Material 3 teaching
  drift on this lane.
- the specialized `ai` lane has now advanced through its first curated snippet sweep:
  `apps/fret-ui-gallery/src/ui/snippets/ai/{agent_demo,artifact_demo,artifact_code_display,attachments_empty,attachments_grid,attachments_inline,attachments_list,attachments_usage,audio_player_demo,chain_of_thought_composable,chain_of_thought_demo,chat_demo,checkpoint_demo,code_block_demo,commit_custom_children,commit_demo,commit_large_demo,confirmation_accepted,confirmation_demo,confirmation_rejected,confirmation_request,context_default,context_demo,file_tree_basic,file_tree_demo,file_tree_expanded,file_tree_large,inline_citation_demo,message_demo,mic_selector_demo,model_selector_demo,open_in_chat_demo,package_info_demo,persona_basic,persona_custom_styling,persona_custom_visual,persona_demo,persona_state_management,persona_variants,plan_demo,prompt_input_action_menu_demo,prompt_input_docs_demo,prompt_input_provider_demo,prompt_input_referenced_sources_demo,reasoning_demo,schema_display_demo,shimmer_demo,shimmer_duration_demo,shimmer_elements_demo,snippet_demo,snippet_plain,sources_demo,stack_trace_collapsed,stack_trace_demo,stack_trace_large_demo,stack_trace_no_internal,task_demo,terminal_demo,test_results_basic,test_results_demo,test_results_errors,test_results_large_demo,test_results_suites,tool_demo,voice_selector_demo,web_preview_demo,workflow_canvas_demo,workflow_chrome_demo,workflow_connection_demo,workflow_controls_demo,workflow_edge_demo,workflow_node_demo,workflow_node_graph_demo,workflow_panel_demo,workflow_toolbar_demo}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, and the source gate
  `ui_authoring_surface_default_app::ai_curated_snippets_prefer_ui_cx_on_the_default_app_surface`
  now locks those exemplars to the typed default-app posture while banning the old
  `ElementContext<'_, H> -> AnyElement` spelling.
- the specialized `ai` lane has now also closed its remaining top-level tail:
  `apps/fret-ui-gallery/src/ui/snippets/ai/{canvas_world_layer_spike,conversation_demo,environment_variables_demo,image_demo,message_branch_demo,message_usage,queue_demo,sandbox_demo,speech_input_demo,suggestions_demo,transcript_torture,transcription_demo}.rs`
  now expose the same typed `UiCx -> impl UiChild` posture and are covered by the same default-app
  source gate.
- remaining specialized-lane count update on 2026-03-14:
  old-signature top-level `ai` snippet renders now fall from 87 to 0.
- the same UI Gallery default-app source gate now also records the `input_group` family:
  `apps/fret-ui-gallery/src/ui/snippets/input_group/{align_block_end,align_block_start,align_inline_end,align_inline_start,button,button_group,custom_input,demo,dropdown,icon,kbd,label,rtl,spinner,text,textarea,tooltip}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, while
  `apps/fret-ui-gallery/src/ui/pages/input_group.rs` consumes those previews through
  `DocSection::build(cx, ...)`; the old `ElementContext<'_, H> -> AnyElement` teaching pattern is
  now forbidden there by
  `ui_authoring_surface_default_app::{input_group_snippets_prefer_ui_cx_on_the_default_app_surface,input_group_page_uses_typed_doc_sections_for_app_facing_snippets}`.
- the same UI Gallery default-app source gate now also records the `toggle_group` family:
  `apps/fret-ui-gallery/src/ui/snippets/toggle_group/{custom,demo,disabled,flex_1_items,full_width_items,label,large,outline,rtl,single,size,small,spacing,usage,vertical}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, while
  `apps/fret-ui-gallery/src/ui/pages/toggle_group.rs` consumes those previews through
  `DocSection::build(cx, ...)`; the old `ElementContext<'_, H> -> AnyElement` teaching pattern is
  now forbidden there by
  `ui_authoring_surface_default_app::{toggle_group_snippets_prefer_ui_cx_on_the_default_app_surface,toggle_group_page_uses_typed_doc_sections_for_app_facing_snippets}`.
- the same UI Gallery default-app source gate now also records the `switch` family:
  `apps/fret-ui-gallery/src/ui/snippets/switch/{airplane_mode,bluetooth,choice_card,description,disabled,invalid,label,rtl,sizes,usage}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, while
  `apps/fret-ui-gallery/src/ui/pages/switch.rs` consumes those previews through
  `DocSection::build(cx, ...)`; the old `ElementContext<'_, H> -> AnyElement` teaching pattern is
  now forbidden there by
  `ui_authoring_surface_default_app::{switch_snippets_prefer_ui_cx_on_the_default_app_surface,switch_page_uses_typed_doc_sections_for_app_facing_snippets}`.
- the same UI Gallery default-app source gate now also records the `checkbox` family:
  `apps/fret-ui-gallery/src/ui/snippets/checkbox/{basic,checked_state,demo,description,disabled,group,invalid_state,label,rtl,table,usage,with_title}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, while
  `apps/fret-ui-gallery/src/ui/pages/checkbox.rs` consumes those previews through
  `DocSection::build(cx, ...)`; the old `ElementContext<'_, H> -> AnyElement` teaching pattern is
  now forbidden there by
  `ui_authoring_surface_default_app::{checkbox_snippets_prefer_ui_cx_on_the_default_app_surface,checkbox_page_uses_typed_doc_sections_for_app_facing_snippets}`.
- the same UI Gallery default-app source gate now also records the `separator` family:
  `apps/fret-ui-gallery/src/ui/snippets/separator/{demo,list,menu,rtl,usage,vertical}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, while
  `apps/fret-ui-gallery/src/ui/pages/separator.rs` consumes those previews through
  `DocSection::build(cx, ...)`; the old `ElementContext<'_, H> -> AnyElement` teaching pattern is
  now forbidden there by
  `ui_authoring_surface_default_app::{separator_snippets_prefer_ui_cx_on_the_default_app_surface,separator_page_uses_typed_doc_sections_for_app_facing_snippets}`.
- the same UI Gallery default-app source gate now also records the `input` family:
  `apps/fret-ui-gallery/src/ui/snippets/input/{badge,basic,button_group,disabled,field,field_group,file,form,grid,inline,input_group,invalid,label,required,rtl}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, while
  `apps/fret-ui-gallery/src/ui/pages/input.rs` consumes those previews through
  `DocSection::build(cx, ...)`; the old `ElementContext<'_, H> -> AnyElement` teaching pattern is
  now forbidden there by
  `ui_authoring_surface_default_app::{input_snippets_prefer_ui_cx_on_the_default_app_surface,input_page_uses_typed_doc_sections_for_app_facing_snippets}`.
- the same UI Gallery default-app source gate now also records the `field` family:
  `apps/fret-ui-gallery/src/ui/snippets/field/{checkbox,choice_card,field_group,fieldset,input,radio,responsive,rtl,select,slider,switch,textarea,validation_and_errors}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, while
  `apps/fret-ui-gallery/src/ui/pages/field.rs` consumes those previews through
  `DocSection::build(cx, ...)`; the old `ElementContext<'_, H> -> AnyElement` teaching pattern is
  now forbidden there by
  `ui_authoring_surface_default_app::{field_snippets_prefer_ui_cx_on_the_default_app_surface,field_page_uses_typed_doc_sections_for_app_facing_snippets}`.
- the same UI Gallery default-app source gate now also records the `textarea` family:
  `apps/fret-ui-gallery/src/ui/snippets/textarea/{button,demo,disabled,field,invalid,label,rtl,usage,with_text}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, while
  `apps/fret-ui-gallery/src/ui/pages/textarea.rs` consumes those previews through
  `DocSection::build(cx, ...)`; the old `ElementContext<'_, H> -> AnyElement` teaching pattern is
  now forbidden there by
  `ui_authoring_surface_default_app::{textarea_snippets_prefer_ui_cx_on_the_default_app_surface,textarea_page_uses_typed_doc_sections_for_app_facing_snippets}`.
- the same UI Gallery default-app source gate now also records the `input_otp` family:
  `apps/fret-ui-gallery/src/ui/snippets/input_otp/{alphanumeric,controlled,demo,disabled,form,four_digits,invalid,pattern,rtl,separator,usage}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, while
  `apps/fret-ui-gallery/src/ui/pages/input_otp.rs` consumes those previews through
  `DocSection::build(cx, ...)`; the old `ElementContext<'_, H> -> AnyElement` teaching pattern is
  now forbidden there by
  `ui_authoring_surface_default_app::{input_otp_snippets_prefer_ui_cx_on_the_default_app_surface,input_otp_page_uses_typed_doc_sections_for_app_facing_snippets}`.
- the same UI Gallery default-app source gate now also records the `select` family:
  `apps/fret-ui-gallery/src/ui/snippets/select/{align_item_with_trigger,demo,diag_surface,disabled,groups,invalid,label,rtl,scrollable}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, while
  `apps/fret-ui-gallery/src/ui/pages/select.rs` consumes those previews through
  `DocSection::build(cx, ...)`; the old `ElementContext<'_, H> -> AnyElement` teaching pattern is
  now forbidden there by
  `ui_authoring_surface_default_app::{select_snippets_prefer_ui_cx_on_the_default_app_surface,select_page_uses_typed_doc_sections_for_app_facing_snippets}`.
- the same UI Gallery default-app source gate now also records the `calendar` family:
  `apps/fret-ui-gallery/src/ui/snippets/calendar/{basic,booked_dates,custom_cell_size,date_and_time_picker,date_of_birth_picker,demo,hijri,locale,month_year_selector,natural_language_picker,presets,range,responsive_mixed_semantics,rtl,usage,week_numbers}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, while
  `apps/fret-ui-gallery/src/ui/pages/calendar.rs` consumes those previews through
  `DocSection::build(cx, ...)`; the old `ElementContext<'_, H> -> AnyElement` teaching pattern is
  now forbidden there by
  `ui_authoring_surface_default_app::{calendar_snippets_prefer_ui_cx_on_the_default_app_surface,calendar_page_uses_typed_doc_sections_for_app_facing_snippets}`.
- the same UI Gallery default-app source gate now also records the `alert_dialog` family:
  `apps/fret-ui-gallery/src/ui/snippets/alert_dialog/{basic,demo,destructive,detached_trigger,media,parts,rich_content,rtl,small,small_with_media,usage}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, while
  `apps/fret-ui-gallery/src/ui/pages/alert_dialog.rs` consumes those previews through
  `DocSection::build(cx, ...)`; the old `ElementContext<'_, H> -> AnyElement` teaching pattern is
  now forbidden there by
  `ui_authoring_surface_default_app::{alert_dialog_snippets_prefer_ui_cx_on_the_default_app_surface,alert_dialog_page_uses_typed_doc_sections_for_app_facing_snippets}`.
- the same UI Gallery default-app source gate now also records the `dialog` family:
  `apps/fret-ui-gallery/src/ui/snippets/dialog/{custom_close_button,demo,no_close_button,parts,rtl,scrollable_content,sticky_footer,usage}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, while
  `apps/fret-ui-gallery/src/ui/pages/dialog.rs` consumes those previews through
  `DocSection::build(cx, ...)`; the old `ElementContext<'_, H> -> AnyElement` teaching pattern is
  now forbidden there by
  `ui_authoring_surface_default_app::{dialog_snippets_prefer_ui_cx_on_the_default_app_surface,dialog_page_uses_typed_doc_sections_for_app_facing_snippets}`.
- the same UI Gallery default-app source gate now also records the `drawer` family:
  `apps/fret-ui-gallery/src/ui/snippets/drawer/{demo,responsive_dialog,rtl,scrollable_content,sides,snap_points,usage}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, while
  `apps/fret-ui-gallery/src/ui/pages/drawer.rs` consumes those previews through
  `DocSection::build(cx, ...)`; the old `ElementContext<'_, H> -> AnyElement` teaching pattern is
  now forbidden there by
  `ui_authoring_surface_default_app::{drawer_snippets_prefer_ui_cx_on_the_default_app_surface,drawer_page_uses_typed_doc_sections_for_app_facing_snippets}`.
- the same UI Gallery default-app source gate now also records the `sheet` family:
  `apps/fret-ui-gallery/src/ui/snippets/sheet/{demo,no_close_button,parts,rtl,side,usage}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, while
  `apps/fret-ui-gallery/src/ui/pages/sheet.rs` consumes those previews through
  `DocSection::build(cx, ...)`; the old `ElementContext<'_, H> -> AnyElement` teaching pattern is
  now forbidden there by
  `ui_authoring_surface_default_app::{sheet_snippets_prefer_ui_cx_on_the_default_app_surface,sheet_page_uses_typed_doc_sections_for_app_facing_snippets}`.
- the same UI Gallery default-app source gate now also records the `spinner` / `form` / `empty` /
  `breadcrumb` / `collapsible` batch:
  those snippet families now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`,
  while the corresponding pages consume the previews through `DocSection::build(cx, ...)`; the
  old `ElementContext<'_, H> -> AnyElement` teaching pattern is now forbidden there by
  `ui_authoring_surface_default_app::{spinner_snippets_prefer_ui_cx_on_the_default_app_surface,spinner_page_uses_typed_doc_sections_for_app_facing_snippets,form_snippets_prefer_ui_cx_on_the_default_app_surface,form_page_uses_typed_doc_sections_for_app_facing_snippets,empty_snippets_prefer_ui_cx_on_the_default_app_surface,empty_page_uses_typed_doc_sections_for_app_facing_snippets,breadcrumb_snippets_prefer_ui_cx_on_the_default_app_surface,breadcrumb_page_uses_typed_doc_sections_for_app_facing_snippets,collapsible_snippets_prefer_ui_cx_on_the_default_app_surface,collapsible_page_uses_typed_doc_sections_for_app_facing_snippets}`.
- the same UI Gallery default-app source gate now also records the `skeleton` / `pagination` /
  `alert` / `sidebar` / `label` batch:
  those snippet families now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`,
  while the corresponding pages consume the previews through `DocSection::build(cx, ...)`; the
  old `ElementContext<'_, H> -> AnyElement` teaching pattern is now forbidden there by
  `ui_authoring_surface_default_app::{skeleton_snippets_prefer_ui_cx_on_the_default_app_surface,skeleton_page_uses_typed_doc_sections_for_app_facing_snippets,pagination_snippets_prefer_ui_cx_on_the_default_app_surface,pagination_page_uses_typed_doc_sections_for_app_facing_snippets,alert_snippets_prefer_ui_cx_on_the_default_app_surface,alert_page_uses_typed_doc_sections_for_app_facing_snippets,sidebar_snippets_prefer_ui_cx_on_the_default_app_surface,sidebar_page_uses_typed_doc_sections_for_app_facing_snippets,label_snippets_prefer_ui_cx_on_the_default_app_surface,label_page_uses_typed_doc_sections_for_app_facing_snippets}`.
- after the recent `button_group` / `input_group` / `toggle_group` / `switch` / `checkbox` /
  `separator` / `input` / `field` / `textarea` / `input_otp` / `select` / `calendar` /
  `alert_dialog` / `dialog` / `drawer` / `sheet` / `spinner` / `form` / `empty` / `breadcrumb` /
  `collapsible` / `skeleton` / `pagination` / `alert` / `sidebar` / `label` / `kbd` / `icons` /
  `sonner` default-app sweeps, the tracked default-app workstream-local backlog now falls from 66
  to 52 top-level snippet renders still teaching `ElementContext<'_, H> -> AnyElement` on that
  lane (down from 95 before the recent high-yield family batches, 136 before the broader family
  sweeps, and 184 before the default-app migration run started).
- the same UI Gallery default-app source gate now also records the `kbd`, `icons`, and `sonner`
  families:
  `apps/fret-ui-gallery/src/ui/snippets/kbd/{button,demo,group,input_group,rtl,tooltip}.rs`,
  `apps/fret-ui-gallery/src/ui/snippets/icons/{grid,spinner}.rs`,
  and
  `apps/fret-ui-gallery/src/ui/snippets/sonner/{demo,extras,notes,position,setup,usage}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, while
  `apps/fret-ui-gallery/src/ui/pages/{kbd,icons,sonner}.rs` consume those previews through
  `DocSection::build(cx, ...)`; on the `sonner` lane, the page relay models are also removed in
  favor of snippet-local shared state plus a dedicated local toaster. The old teaching pattern is
  now forbidden there by
  `ui_authoring_surface_default_app::{kbd_*,icons_*,sonner_*}`.
- the same UI Gallery default-app source gate now also records the `date_picker` family:
  `apps/fret-ui-gallery/src/ui/snippets/date_picker/{basic,demo,dob,dropdowns,input,label,natural_language,notes,presets,range,rtl,time_picker,usage}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, while
  `apps/fret-ui-gallery/src/ui/pages/date_picker.rs` consumes those previews through
  `DocSection::build(cx, ...)` and no longer relays per-demo `open/month/selected/value` state
  through the page shell. The old teaching pattern is now forbidden there by
  `ui_authoring_surface_default_app::{date_picker_*}`.
- the same UI Gallery default-app source gate now also records the `avatar` family:
  `apps/fret-ui-gallery/src/ui/snippets/avatar/{badge_icon,basic,demo,dropdown,fallback_only,group,group_count,group_count_icon,rtl,sizes,usage,with_badge}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, while
  `apps/fret-ui-gallery/src/ui/pages/avatar.rs` consumes those previews through
  `DocSection::build(cx, ...)` and no longer relays a page-owned image model. The avatar snippet
  module now owns a self-contained `ImageSource::rgba8(...) -> ImageId` demo asset, and the old
  teaching pattern is now forbidden there by
  `ui_authoring_surface_default_app::{avatar_*,selected_avatar_*}`.
- the same UI Gallery default-app source gate now also records the `command` family:
  `apps/fret-ui-gallery/src/ui/snippets/command/{action_first_view,basic,docs_demo,groups,loading,rtl,scrollable,shortcuts,usage}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, while
  `apps/fret-ui-gallery/src/ui/pages/command.rs` consumes those previews through
  `DocSection::build(cx, ...)` and no longer relays a page-owned `last_action` model. The shared
  command local-state/action helpers now live in `snippets/command/mod.rs`, and the old teaching
  pattern is now forbidden there by
  `ui_authoring_surface_default_app::{command_*}`.
- the same UI Gallery default-app source gate now also records the `card` family:
  `apps/fret-ui-gallery/src/ui/snippets/card/{card_content,compositions,demo,image,meeting_notes,rtl,size,usage}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, while
  `apps/fret-ui-gallery/src/ui/pages/card.rs` consumes those previews through
  `DocSection::build(cx, ...)` and no longer relays a page-owned `event_cover_image` model. The
  `image` snippet now resolves its demo `ImageSource` entirely inside the snippet, and the old
  teaching pattern is now forbidden there by
  `ui_authoring_surface_default_app::{card_*,selected_card_*}`.
- the same UI Gallery default-app source gate now also records the `image_object_fit` family:
  `apps/fret-ui-gallery/src/ui/snippets/image_object_fit/{mapping,sampling}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, while
  `apps/fret-ui-gallery/src/ui/pages/image_object_fit.rs` consumes those previews through
  `DocSection::build(cx, ...)` and no longer relays gallery-owned `ImageId` models. The snippet
  module now generates its own fit/sampling demo `ImageSource`s, and the old teaching pattern is
  now forbidden there by
  `ui_authoring_surface_default_app::{image_object_fit_*}`.
- after that `image_object_fit` sweep, the tracked default-app workstream-local backlog falls from
  11 to 9
  top-level snippet renders still teaching `ElementContext<'_, H> -> AnyElement` on that lane.
- for the default-app authoring lane, the next queue should now continue on the remaining
  long-tail stateful families after `command` / `card` / `image_object_fit`, with `data_table`
  and `motion_presets` now carrying most of the remaining tracked backlog, while `ai` continues
  as the remaining specialized follow-up lane now that the dedicated `material3`, `typography`,
  and `shadcn_extras` teaching lanes have been aligned too.
- the cookbook advanced-example source gate now also records
  `apps/fret-cookbook/examples/customv1_basics.rs`,
  where `panel_shell(...)` and `preview_content(...)` now use `IntoUiElement<KernelApp>`-based
  helper signatures and reserve raw landing for the actual effect-layer assembly seams.
- the cookbook advanced-example source gate now also records
  `apps/fret-cookbook/examples/drop_shadow_basics.rs` and
  `apps/fret-cookbook/examples/icons_and_assets_basics.rs`,
  where `shadow_card(...)` and `render_image_preview(...)` now use
  `IntoUiElement<KernelApp>`-based helper signatures and reserve raw landing for effect-layer,
  image-box, and sibling child-collection seams.
- the cookbook retained-canvas source gate now also records
  `apps/fret-cookbook/examples/chart_interactions_basics.rs`,
  where `chart_canvas(...) -> AnyElement` remains intentional because it owns the
  `RetainedSubtreeProps::new::<KernelApp>(...)` and `cached_subtree_with(...)` bridge boundary.
- the advanced-example source gate now also records
  `apps/fret-examples/src/custom_effect_v3_demo.rs` and
  `apps/fret-examples/src/postprocess_theme_demo.rs`,
  where local composition helpers (`stage`, `stage_controls`, `animated_backdrop`, `lens_row`,
  `lens_shell`, `inspector`, `stage_body`, and `stage_cards`) now prefer
  `IntoUiElement<KernelApp>`-based signatures and reserve `.into_element(cx)` for the actual
  heterogenous sibling child-collection seams.
- the advanced-example source gate now also records
  `apps/fret-examples/src/async_playground_demo.rs`,
  where local composition helpers (`header_bar`, `body`, `catalog_panel`, `main_panel`,
  `inspector_panel`, `policy_editor`, `query_panel_for_mode`, `query_inputs_row`,
  `query_result_view`, and `status_badge`) now prefer `IntoUiElement<KernelApp>`-based
  signatures and reserve `.into_element(cx)` for heterogenous child arrays plus
  `TabsItem::new([..])` / `ScrollArea::new([..])` landing seams.
- the advanced-example source gate now also records
  `apps/fret-examples/src/custom_effect_v1_demo.rs` and
  `apps/fret-examples/src/custom_effect_v2_demo.rs`,
  where `stage(...)`, `lens_row(...)`, `inspector(...)`, and
  `lens_shell<B>(...) -> impl IntoUiElement<KernelApp> + use<B>` now stay on typed helper
  signatures, while the explicit raw landing step is reserved for the internal effect-layer body
  via `body.into_element(cx)`.
- the default-app/web source gate now also records
  `apps/fret-examples/src/custom_effect_v2_{identity_web,web,lut_web,glass_chrome_web}_demo.rs`,
  where reusable helpers such as `lens(...)`, `inspector(...)`, `controls_panel(...)`, and
  `label_row(...)` now prefer `IntoUiElement<fret_app::App>`-based signatures and reserve
  `.into_element(cx)` for stage child arrays, overlay child collections, and other explicit raw
  landing seams.
- the focused shadcn surface gate now also records the ecosystem-trait closeout boundary:
  `ecosystem/fret-ui-shadcn/src/ui_ext/*` stays on `IntoUiElement<H>`-based helper glue, while
  `ecosystem/fret-ui-shadcn/src/ui_builder_ext/*::into_element(...)` remains an intentional
  `AnyElement` landing seam and only its closure inputs are forbidden from regressing to
  `AnyElement`-typed signatures.
- the focused shadcn surface gate now also records the completed typography lane:
  `ecosystem/fret-ui-shadcn/src/typography.rs` keeps the `raw::typography::*` namespace but now
  exposes typed helper outputs for `h1` / `h2` / `h3` / `h4` / `p` / `lead` / `large` / `small` /
  `muted` / `inline_code` / `blockquote` / `list`, while first-party Gallery/examples and
  `ecosystem/fret-genui-shadcn` now land those helpers explicitly via `.into_element(cx)` only at
  concrete `AnyElement` seams.
- the focused shadcn surface gate now also records the prelude ergonomics follow-up:
  `ecosystem/fret-ui-shadcn/src/lib.rs::prelude` now re-exports `IntoUiElement`, so direct-crate
  first-party shadcn examples do not need local trait imports just to land typed helpers.
- the examples source-policy gate for raw shadcn escape hatches is now aligned to the reviewed
  allowlist instead of a broad `raw::*` allowance:
  `apps/fret-examples/src/lib.rs::examples_source_tree_limits_raw_shadcn_escape_hatches`
  currently permits only `shadcn::raw::typography::*`, `shadcn::raw::extras::*`,
  `fret::shadcn::raw::prelude::*`, and the documented advanced service hooks
  `raw::advanced::{sync_theme_from_environment(...), install_with_ui_services(...)}`.
- next execution order on 2026-03-13:
  1. keep M3 focused on helper-return migration outside the now-closed shadcn raw inventory,
     especially first-party app/example helpers that still leak `AnyElement` without being true
     retained/diagnostic/overlay seams;
  2. keep default-app/UI Gallery reusable helpers moving toward `impl UiChild` /
     `impl IntoUiElement<fret_app::App>` rather than broadening snippet-local raw helper returns;
  3. treat the remaining raw-explicit-IR lane as documentation/gating work unless the underlying
     runtime/storage contract actually changes.

## Hard Delete Matrix

| Old name / posture | Replacement | Delete when | Status |
| --- | --- | --- | --- |
| `UiIntoElement` as curated public conversion vocabulary | unified trait (`IntoUiElement<H>` or final equivalent) | component prelude only exports the unified trait and first-party reusable code is migrated | Migrated |
| `UiHostBoundIntoElement<H>` as curated public conversion vocabulary | unified trait (`IntoUiElement<H>` or final equivalent) | host-bound builders land through the unified trait and no curated docs teach the split | Deleted |
| `UiChildIntoElement<H>` as curated public conversion vocabulary | unified trait for component code, `UiChild` for app-facing helpers | child pipelines and curated docs no longer require the old trait name | Deleted |
| `UiBuilderHostBoundIntoElementExt<H>` as curated public bridge trait | unified trait-backed method syntax | app/component preludes stop importing the old bridge and first-party code compiles through the unified trait | Deleted |
| `AnyElement` as the default first-contact helper return type in app docs/examples | `impl UiChild` or `Ui` as appropriate | app-facing docs/examples are migrated and source gates are in place | In progress |

## Recommended Execution Order

| Order | Track | Why |
| --- | --- | --- |
| 1 | unify the public conversion contract | everything else depends on one target concept |
| 2 | migrate builder/macro landing paths | reduces churn for downstream call sites |
| 3 | migrate curated component surfaces | proves the new trait is sufficient for real reusable code |
| 4 | migrate app-facing helper teaching | sharpens the default product surface |
| 5 | delete old conversion names and add gates | locks the cleanup so drift cannot return |

## Completion Rule

This workstream is complete when:

- app-facing docs teach `Ui` / `UiChild`,
- component-facing docs teach one public conversion trait,
- raw `AnyElement` use is clearly advanced/internal rather than default teaching,
- the old split conversion traits are deleted from curated public surfaces.
