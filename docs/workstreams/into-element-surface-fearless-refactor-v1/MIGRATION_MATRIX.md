# Into-Element Surface — Migration Matrix

Status: execution tracker
Last updated: 2026-03-12

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

## Current Name Classification (2026-03-12)

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

Execution note on 2026-03-12:

- the focused `selected_*` source gate now also covers
  `apps/fret-ui-gallery/src/ui/snippets/ai/{attachments_usage,file_tree_demo,speech_input_demo}.rs`,
  where `render_grid_attachment(...)`, `invisible_marker(...)`, `body_text(...)`, and
  `clear_action(...)` now prefer `IntoUiElement<H>`-based helper signatures while keeping
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
