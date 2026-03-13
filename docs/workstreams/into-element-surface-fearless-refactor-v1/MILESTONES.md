# Into-Element Surface (Fearless Refactor v1) — Milestones

This file defines milestones for the workstream in `DESIGN.md`.

## Current execution stance (2026-03-12)

This workstream is the **current main authoring lane**.

Reason:

- the app-facing surface is already mostly converged,
- the ecosystem trait budget is already mostly decided,
- the clearest remaining "write UI" gap is still the fragmented conversion vocabulary.

Execution rule:

- prioritize M0/M1 here before reopening broader helper design elsewhere,
- use the canonical compare set
  (`simple_todo_v2_target`, `todo_demo`, scaffold simple-todo template)
  as the first downstream proof that the new conversion surface improves authoring feel,
- let ecosystem trait/docs cleanup follow this work rather than compete with it.

Current readout on 2026-03-12:

| Milestone | State | Notes |
| --- | --- | --- |
| M0 | Done | target vocabulary is locked and the classification table is now recorded in `MIGRATION_MATRIX.md` |
| M1 | Done | `IntoUiElement<H>` is the curated component conversion name; docs/preludes/tests reflect it |
| M2 | Done | `UiBuilder<T>` and host-bound child builders now land through `IntoUiElement<H>`; `UiBuilderHostBoundIntoElementExt` is deleted; child collection now also consumes `IntoUiElement<H>` directly |
| M3 | In progress | curated `fret` / `fret-ui-kit` surfaces and the canonical todo/scaffold compare set are aligned; `fret::UiChild` now lands directly through `IntoUiElement<App>`; `fret-ui-shadcn` ui_ext glue, `ui_builder_ext` helper closures, overlay/single-child builders, and `fret-router-ui` outlet helpers now land through `IntoUiElement<H>`; selected advanced examples (`assets_demo`, `async_playground_demo`, `custom_effect_v1_demo`, `custom_effect_v2_demo`, `custom_effect_v3_demo`, `postprocess_theme_demo`, `drop_shadow_demo`, `markdown_demo`, `liquid_glass_demo`, `customv1_basics`, `drop_shadow_basics`, `icons_and_assets_basics`, `hello_world_compare_demo`) now also prefer `impl IntoUiElement<...>` for non-raw helpers, including `custom_effect_v1_demo::{stage,lens_row,plain_lens,custom_effect_lens,lens_shell,inspector}`, `custom_effect_v2_demo::{stage,lens_row,plain_lens,custom_effect_lens,lens_shell,inspector}`, `async_playground_demo::{header_bar,body,catalog_panel,main_panel,inspector_panel,policy_editor,query_panel_for_mode,query_inputs_row,query_result_view,status_badge}`, `custom_effect_v3_demo::{stage,stage_controls,animated_backdrop,lens_row,lens_shell}`, and `postprocess_theme_demo::{inspector,stage,stage_body,stage_cards}`; selected default-app WebGPU examples now also keep typed helper signatures, including `custom_effect_v2_identity_web_demo::{lens,inspector}`, `custom_effect_v2_web_demo::{lens,inspector}`, `custom_effect_v2_lut_web_demo::{lens,inspector}`, and `custom_effect_v2_glass_chrome_web_demo::{label_row,lens,controls_panel}`, while keeping explicit raw seams such as the internal body landing inside `custom_effect_v1_demo::lens_shell(...)` / `custom_effect_v2_demo::lens_shell(...)`, stage-tile child-array assembly in the WebGPU demos, and the retained bridge seam `chart_interactions_basics::chart_canvas(...)`; selected UI Gallery AI and Material 3 doc pages now keep page-local helpers on `impl UiChild + use<>`, including `material3/shared.rs::material3_variant_toggle_row(...)`, while `material3/shared.rs::render_material3_demo_page<D>(...)`, `doc_layout.rs::DocSection::build<P>(...)`, and `doc_layout.rs::notes_block(...)` now keep page/document wrappers on a typed lane so selected doc pages such as `pages/aspect_ratio.rs`, `pages/ai_artifact_demo.rs`, `pages/ai_context_demo.rs`, `pages/ai_model_selector_demo.rs`, `pages/ai_mic_selector_demo.rs`, `pages/ai_voice_selector_demo.rs`, `pages/ai_file_tree_demo.rs`, `pages/ai_commit_demo.rs`, `pages/ai_test_results_demo.rs`, `pages/ai_persona_demo.rs`, `pages/ai_checkpoint_demo.rs`, `pages/ai_chain_of_thought_demo.rs`, `pages/ai_shimmer_demo.rs`, `pages/ai_agent_demo.rs`, `pages/ai_attachments_demo.rs`, `pages/ai_confirmation_demo.rs`, `pages/ai_inline_citation_demo.rs`, `pages/ai_message_demo.rs`, `pages/ai_speech_input_demo.rs`, `pages/ai_stack_trace_demo.rs`, `pages/avatar.rs`, `pages/button.rs`, `pages/button_group.rs`, `pages/alert_dialog.rs`, `pages/hover_card.rs`, `pages/dropdown_menu.rs`, `pages/calendar.rs`, `pages/accordion.rs`, `pages/alert.rs`, `pages/dialog.rs`, `pages/navigation_menu.rs`, `pages/sheet.rs`, `pages/drawer.rs`, `pages/popover.rs`, `pages/select.rs`, `pages/context_menu.rs`, `pages/menubar.rs`, `pages/progress.rs`, `pages/pagination.rs`, `pages/tabs.rs`, `pages/scroll_area.rs`, `pages/command.rs`, `pages/slider.rs`, `pages/icons.rs`, `pages/typography.rs`, `pages/badge.rs`, `pages/checkbox.rs`, `pages/collapsible.rs`, `pages/empty.rs`, `pages/input.rs`, `pages/label.rs`, `pages/kbd.rs`, `pages/spinner.rs`, `pages/tooltip.rs`, `pages/switch.rs`, `pages/toggle.rs`, `pages/toggle_group.rs`, `pages/separator.rs`, `pages/textarea.rs`, `pages/radio_group.rs`, `pages/skeleton.rs`, `pages/table.rs`, `pages/image_object_fit.rs`, `pages/breadcrumb.rs`, `pages/card.rs`, `pages/input_otp.rs`, `pages/resizable.rs`, `pages/sidebar.rs`, `pages/sonner.rs`, `pages/form.rs`, `pages/carousel.rs`, `pages/chart.rs`, `pages/combobox.rs`, `pages/data_table.rs`, `pages/item.rs`, `pages/native_select.rs`, `pages/date_picker.rs`, `pages/field.rs`, and `pages/input_group.rs` now also teach typed `Features/Notes` blocks instead of eager `AnyElement` landing, the first-party Gallery docs prose helper now no longer ships the legacy `doc_layout::notes(...) -> AnyElement` shim, selected UI Gallery badge snippets now keep local `row(...)` helpers on `impl IntoUiElement<H> + use<H, F>`, selected UI Gallery avatar snippets now keep row wrappers, avatar builders, and icon/group helpers on `impl IntoUiElement<H> + use<...>`, selected UI Gallery button snippets now keep row wrappers and local size-composition helpers on `impl IntoUiElement<H> + use<...>`, selected UI Gallery card snippets now keep `meeting_notes::{marker,item}`, `compositions::cell`, and `demo::{email_field,password_field}` on `impl IntoUiElement<fret_app::App> + use<>`, selected UI Gallery navigation-menu docs snippet now keeps `list_item(...)` and `icon_row(...)` on `impl IntoUiElement<fret_app::App> + use<>`, selected UI Gallery tabs snippets now keep local `field(...)` helpers on `impl IntoUiElement<fret_app::App> + use<>`, selected UI Gallery collapsible snippets now keep `rotated_lucide(...)`, `radius_input(...)`, `details_collapsible(...)`, `file_leaf(...)`, and `folder(...)` on `impl IntoUiElement<H> + use<H>`, selected UI Gallery hover-card snippets now keep `card(...)` / `demo_content(...)` helpers on `impl IntoUiElement<H> + use<H>`, selected UI Gallery tooltip snippets now keep `make_tooltip(...)` / `make_tooltip_with_test_ids(...)` on `impl IntoUiElement<H> + use<H>`, selected UI Gallery resizable snippets now keep `panel(...)` / `box_group(...)` helpers on `impl IntoUiElement<H> + use<...>`, including `resizable/{demo,vertical,handle,rtl}.rs`, selected UI Gallery scroll-area snippets now keep `nested_scroll_routing::row(...)`, `demo::tag_row(...)`, and `expand_at_bottom::{toggle_button,empty_row}` on `impl IntoUiElement<H> + use<H>`, selected UI Gallery data-table snippets now keep `align_end(...)`, `align_inline_start(...)`, `footer(...)`, and `bottom_controls(...)` on `impl IntoUiElement<fret_app::App> + use<...>`, selected UI Gallery table-action snippets now keep `align_end(...)` and `action_row(...)` on `impl IntoUiElement<fret_app::App> + use<...>`, selected UI Gallery table snippets now keep `make_invoice_table(...)` on `impl IntoUiElement<fret_app::App> + use<>` and now drop helper-local `cx` from the `demo` / `footer` / `rtl` variants where the body can stay late-landed, selected UI Gallery separator snippets now keep `section(...)` / `row(...)` helpers on `impl IntoUiElement<H> + use<H>`, selected UI Gallery sidebar snippets now keep `menu_button(...)` helpers on `impl IntoUiElement<...>`-based signatures across `sidebar/{demo,controlled,mobile,rtl}.rs`, selected UI Gallery aspect-ratio snippets now keep `portrait_image(...)`, `square_image(...)`, `rtl_image(...)`, `ratio_example(...)`, and `render_preview(...)` helpers on `impl IntoUiElement<H> + use<H>`, including `aspect_ratio/{demo,portrait,square,rtl}.rs`, selected UI Gallery context-menu snippets now keep `trigger_surface(...)` and `side_menu(...)` helpers on `impl IntoUiElement<H>` with explicit trigger landing seams, including `context_menu/sides.rs`, selected UI Gallery combobox snippets now keep local `state_row(...)` and `state_rows(...)` helpers on `impl IntoUiElement<fret_app::App> + use<>`, selected UI Gallery pagination snippets now keep local `page_number(...)` helpers on `impl IntoUiElement<H> + use<H>`, selected UI Gallery carousel snippets now keep local `slide_card(...)` / `slide(...)` helpers on `impl IntoUiElement<fret_app::App> + use<>`, including the `api`, `duration_embla`, `rtl`, `plugin_autoplay*`, and `events` demos, selected UI Gallery skeleton snippets now keep local `round(...)` / `row(...)` helpers on `impl IntoUiElement<H> + use<H>`, selected UI Gallery popover wrapper helpers now accept/return `IntoUiElement<H>` instead of forcing `AnyElement`, and selected `popover/{demo,with_form}.rs` snippets now also teach `ui::children![cx; ...]` for `PopoverContent` / `PopoverHeader` / `FieldGroup` / `Field` assembly, selected UI Gallery dropdown-menu preview wrappers now accept/return `IntoUiElement<H>`, selected UI Gallery AI wrapper/doc-preview helpers now also accept or expose `IntoUiElement<H>`-based signatures (`centered(...)`, `preview(...)`, `progress_section(...)`, `render_grid_attachment(...)`, `render_list_attachment(...)`, `invisible_marker(...)`, `body_text(...)`, and `clear_action(...)`), including `file_tree_large.rs::preview(...)`; internal gallery wrapper shells now also keep typed wrapper seams in `doc_layout.rs::demo_shell<B>(...)` and `code_editor/mvp/gates.rs::gate_panel<B>(...)`; `fret-ui-shadcn` internal menu-slot wrappers in `context_menu.rs`, `dropdown_menu.rs`, and `menubar.rs` now also accept `IntoUiElement<H>` inputs on `menu_icon_slot(...)`; the thin public constructor/wrapper trial now covers `badge.rs::badge<H, T>(...)`, `kbd.rs::kbd<H, T>(...)`, `separator.rs::separator<H>()`, `input_group.rs::input_group<H>(...)`, `input_otp.rs::input_otp<H>(...)`, and `command.rs::command<H, I, F, T>(...)`, while `kbd.rs::kbd_icon<H>(...)` remains intentionally raw because `Kbd::from_children(...)` still owns a concrete `Vec<AnyElement>` child seam; the dedicated typography sweep is now landed, so `fret-ui-shadcn/src/typography.rs` keeps the `raw::typography::*` namespace but exposes typed helper outputs and first-party Gallery/examples/`fret-genui-shadcn` call sites now land those helpers explicitly via `.into_element(cx)` only where a concrete `AnyElement` seam is still required, while eager constructor examples such as the accordion snippets and selected dialog/sheet/drawer modal-form snippets now teach `ui::children![cx; ...]` instead of ad-hoc `vec![...into_element(cx)]` child assembly; selected breadcrumb helpers now keep separators on `IntoUiElement<H>`, selected button-group, toggle-group, and drawer helpers now expose `IntoUiElement`-based signatures, including `drawer/{demo,responsive_dialog,sides,scrollable_content}.rs`, selected UI Gallery sheet/dialog snippets now keep `profile_fields(...)` helpers on `impl IntoUiElement<H> + use<H>`, selected UI Gallery dialog scroll/sticky snippets now keep `lorem_block(...)` helpers on `impl IntoUiElement<H> + use<H>`, and selected item, toast, alert, slider, and motion-presets helpers now also stay on `IntoUiElement`-based signatures, including `item/{avatar,icon,link,link_render,dropdown,extras_rtl,gallery}.rs` helpers such as `icon(...)`, `icon_button(...)`, `outline_button(...)`, `outline_button_sm(...)`, `item_basic(...)`, `item_icon(...)`, `item_avatar(...)`, and `item_team(...)`, plus `toast/deprecated.rs::centered(...)`, `sonner/{demo,extras,position}.rs::wrap_controls_row(...)`, `alert/{interactive_links,demo}.rs::{interactive_link,interactive_link_text}(...)`, and `slider/demo.rs::controlled(...)`; broader shadcn/gallery/helper cleanup still remains |
| M4 | In progress | prelude gates are in place, curated component-authoring docs now teach only `IntoUiElement<H>`, stale-name source/doc guards now cover curated docs, `UiChildIntoElement` is now deleted from code, `fret_ui_shadcn::prelude::*` now re-exports `IntoUiElement` so typed direct-crate helpers do not need ad-hoc trait imports, exported `fret-ui-kit` adapter macros plus built-in primitive glue now also attach `IntoUiElement<H>` directly instead of spelling `UiIntoElement` on the first-party macro surface, the public RenderOnce helper macro is now renamed to `ui_component_render_once!`, declarative semantics wrappers (`UiElementTestIdExt`, `UiElementA11yExt`, `UiElementKeyContextExt`) now also land through `IntoUiElement<H>` directly instead of depending on `UiIntoElement` in production code, built-in text primitives (`ui::TextBox`, `ui::RawTextBox`) now also land through `IntoUiElement<H>` directly, the legacy `UiIntoElement` bridge name is now deleted from production code entirely, `docs/first-hour.md` and the `fret-ui-ai` message/workflow builder smoke tests now also stay on the public `IntoUiElement<...>` contract, the shadcn source-alignment guidance now explicitly treats typed doc/page wrapper entry points (for example `DocSection::build(...)`) as part of the first-party exemplar contract, and the focused UI Gallery source gate now covers an expanded `selected_*` helper set across AI/Material 3 pages plus internal wrapper constructors and first-party snippets/wrappers for avatar/button/card/navigation-menu/tabs/collapsible/tooltip/hover-card/context-menu/aspect-ratio/resizable/scroll-area/data-table/table-action/table/separator/sidebar/combobox/pagination/carousel/skeleton/popover/dropdown-menu/item/toast/sonner/alert/slider and other authoring surfaces; only historical docs and negative source-policy assertions still mention the old name |

## Milestone 0 — Lock the target conversion vocabulary

Outcome:

- Maintainers can answer which conversion names belong to app, component, and advanced surfaces.

Deliverables:

- `TARGET_INTERFACE_STATE.md` finalized.
- `MIGRATION_MATRIX.md` finalized.
- one decided public name for the unified component conversion trait.

Exit criteria:

- we no longer debate whether `UiIntoElement`, `UiChildIntoElement`, and
  `UiBuilderHostBoundIntoElementExt` are all part of the intended public product surface.
- classification of current names is written down rather than implied from code comments.

## Milestone 1 — Land one public conversion contract

Outcome:

- the component surface has one obvious conversion concept.

Deliverables:

- unified public conversion trait added,
- temporary internal adapters if needed,
- `.into_element(cx)` works for both host-agnostic and host-bound builder values.

Exit criteria:

- the curated component surface can teach one trait without caveats about bridge traits.
- the landing is verified in `fret-ui-kit`, `fret`, `fret-examples`, and `fretboard`.

## Milestone 2 — Migrate builders and curated first-party surfaces

Outcome:

- the new conversion contract is proven by real first-party usage.

Deliverables:

- `UiBuilder` and child pipelines migrate to the unified contract,
- `ecosystem/fret`, `fret-ui-kit`, and selected first-party component/helper surfaces migrate,
- the canonical authoring compare set migrates together:
  `apps/fret-cookbook/examples/simple_todo_v2_target.rs`,
  `apps/fret-examples/src/todo_demo.rs`, and
  `apps/fretboard/src/scaffold/templates.rs`,
- app-facing helpers continue moving toward `UiChild`.

Exit criteria:

- first-party curated examples do not need the old public conversion names to compile or teach.
- the canonical compare set shows one consistent explicit landing story instead of three
  different ad-hoc `.into_element(cx)` patterns.
- `UiBuilderHostBoundIntoElementExt` is no longer required to recover method syntax for host-bound
  builders.

## Milestone 3 — Delete the split public conversion surface

Outcome:

- public conversion vocabulary becomes materially smaller.

Deliverables:

- old curated conversion traits removed,
- stale docs/examples rewritten,
- remaining raw `AnyElement` use is intentional and scoped.

Exit criteria:

- reviewing the public surface no longer requires mentally translating several "into element"
  concepts into one operation.
- root-level scaffolding traits that survive the milestone are explicitly justified as temporary
  compatibility shims rather than silent product surface.

## Milestone 4 — Lock the surface with gates

Outcome:

- conversion-surface regressions fail fast.

Deliverables:

- prelude export gates,
- source/doc teaching gates,
- stale-name regression gates.

Exit criteria:

- new curated surfaces cannot drift back toward the old multi-trait conversion vocabulary without
  an explicit review failure.
