# Into-Element Surface (Fearless Refactor v1) — Milestones

This file defines milestones for the workstream in `DESIGN.md`.

## Current execution stance (2026-03-13)

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

Current readout on 2026-03-13:

| Milestone | State | Notes |
| --- | --- | --- |
| M0 | Done | target vocabulary is locked and the classification table is now recorded in `MIGRATION_MATRIX.md` |
| M1 | Done | `IntoUiElement<H>` is the curated component conversion name; docs/preludes/tests reflect it |
| M2 | Done | `UiBuilder<T>` and host-bound child builders now land through `IntoUiElement<H>`; `UiBuilderHostBoundIntoElementExt` is deleted; child collection now also consumes `IntoUiElement<H>` directly |
| M3 | In progress | curated `fret` / `fret-ui-kit` surfaces and the canonical todo/scaffold compare set are aligned; `fret::UiChild` now lands directly through `IntoUiElement<App>`; `fret-ui-shadcn` ui_ext glue, `ui_builder_ext` helper closures, overlay/single-child builders, and `fret-router-ui` outlet helpers now land through `IntoUiElement<H>`; selected advanced examples (`assets_demo`, `async_playground_demo`, `custom_effect_v1_demo`, `custom_effect_v2_demo`, `custom_effect_v3_demo`, `postprocess_theme_demo`, `drop_shadow_demo`, `markdown_demo`, `liquid_glass_demo`, `customv1_basics`, `drop_shadow_basics`, `icons_and_assets_basics`, `hello_world_compare_demo`) now also prefer `impl IntoUiElement<...>` for non-raw helpers, including `custom_effect_v1_demo::{stage,lens_row,plain_lens,custom_effect_lens,lens_shell,inspector}`, `custom_effect_v2_demo::{stage,lens_row,plain_lens,custom_effect_lens,lens_shell,inspector}`, `async_playground_demo::{header_bar,body,catalog_panel,main_panel,inspector_panel,policy_editor,query_panel_for_mode,query_inputs_row,query_result_view,status_badge}`, `custom_effect_v3_demo::{stage,stage_controls,animated_backdrop,lens_row,lens_shell}`, and `postprocess_theme_demo::{inspector,stage,stage_body,stage_cards}`; selected default-app WebGPU examples now also keep typed helper signatures, including `custom_effect_v2_identity_web_demo::{lens,inspector}`, `custom_effect_v2_web_demo::{lens,inspector}`, `custom_effect_v2_lut_web_demo::{lens,inspector}`, and `custom_effect_v2_glass_chrome_web_demo::{label_row,lens,controls_panel}`, while keeping explicit raw seams such as the internal body landing inside `custom_effect_v1_demo::lens_shell(...)` / `custom_effect_v2_demo::lens_shell(...)`, stage-tile child-array assembly in the WebGPU demos, and the retained bridge seam `chart_interactions_basics::chart_canvas(...)`; selected UI Gallery AI and Material 3 doc pages now keep page-local helpers on `impl UiChild + use<>`, including `material3/shared.rs::material3_variant_toggle_row(...)`, while `material3/shared.rs::render_material3_demo_page<D>(...)`, `doc_layout.rs::DocSection::build<P>(...)`, and `doc_layout.rs::notes_block(...)` now keep page/document wrappers on a typed lane so selected doc pages such as `pages/aspect_ratio.rs`, `pages/ai_artifact_demo.rs`, `pages/ai_context_demo.rs`, `pages/ai_model_selector_demo.rs`, `pages/ai_mic_selector_demo.rs`, `pages/ai_voice_selector_demo.rs`, `pages/ai_file_tree_demo.rs`, `pages/ai_commit_demo.rs`, `pages/ai_test_results_demo.rs`, `pages/ai_persona_demo.rs`, `pages/ai_checkpoint_demo.rs`, `pages/ai_chain_of_thought_demo.rs`, `pages/ai_shimmer_demo.rs`, `pages/ai_agent_demo.rs`, `pages/ai_attachments_demo.rs`, `pages/ai_confirmation_demo.rs`, `pages/ai_inline_citation_demo.rs`, `pages/ai_message_demo.rs`, `pages/ai_speech_input_demo.rs`, `pages/ai_stack_trace_demo.rs`, `pages/avatar.rs`, `pages/button.rs`, `pages/button_group.rs`, `pages/alert_dialog.rs`, `pages/hover_card.rs`, `pages/dropdown_menu.rs`, `pages/calendar.rs`, `pages/accordion.rs`, `pages/alert.rs`, `pages/dialog.rs`, `pages/navigation_menu.rs`, `pages/sheet.rs`, `pages/drawer.rs`, `pages/popover.rs`, `pages/select.rs`, `pages/context_menu.rs`, `pages/menubar.rs`, `pages/progress.rs`, `pages/pagination.rs`, `pages/tabs.rs`, `pages/scroll_area.rs`, `pages/command.rs`, `pages/slider.rs`, `pages/icons.rs`, `pages/typography.rs`, `pages/badge.rs`, `pages/checkbox.rs`, `pages/collapsible.rs`, `pages/empty.rs`, `pages/input.rs`, `pages/label.rs`, `pages/kbd.rs`, `pages/spinner.rs`, `pages/tooltip.rs`, `pages/switch.rs`, `pages/toggle.rs`, `pages/toggle_group.rs`, `pages/separator.rs`, `pages/textarea.rs`, `pages/radio_group.rs`, `pages/skeleton.rs`, `pages/table.rs`, `pages/image_object_fit.rs`, `pages/breadcrumb.rs`, `pages/card.rs`, `pages/input_otp.rs`, `pages/resizable.rs`, `pages/sidebar.rs`, `pages/sonner.rs`, `pages/form.rs`, `pages/carousel.rs`, `pages/chart.rs`, `pages/combobox.rs`, `pages/data_table.rs`, `pages/item.rs`, `pages/native_select.rs`, `pages/date_picker.rs`, `pages/field.rs`, and `pages/input_group.rs` now also teach typed `Features/Notes` blocks instead of eager `AnyElement` landing, the first-party Gallery docs prose helper now no longer ships the legacy `doc_layout::notes(...) -> AnyElement` shim, selected UI Gallery badge snippets now keep local `row(...)` helpers on `impl IntoUiElement<H> + use<H, F>`, selected UI Gallery avatar snippets now keep row wrappers, avatar builders, and icon/group helpers on `impl IntoUiElement<H> + use<...>`, selected UI Gallery button snippets now keep row wrappers and local size-composition helpers on `impl IntoUiElement<H> + use<...>`, selected UI Gallery card snippets now keep `meeting_notes::{marker,item}`, `compositions::cell`, and `demo::{email_field,password_field}` on `impl IntoUiElement<fret_app::App> + use<>`, and first-party card exemplars now also teach the typed helper family directly: `card(...)`, `card_header(...)`, `card_action(...)`, `card_title(...)`, `card_description(...)`, `card_content(...)`, and `card_footer(...)` plus `ui::children![cx; ...]` for heterogeneous slot child lists; selected UI Gallery navigation-menu docs snippet now keeps `list_item(...)` and `icon_row(...)` on `impl IntoUiElement<fret_app::App> + use<>`, selected UI Gallery tabs snippets now keep local `field(...)` helpers on `impl IntoUiElement<fret_app::App> + use<>`, selected UI Gallery collapsible snippets now keep `rotated_lucide(...)`, `radius_input(...)`, `details_collapsible(...)`, `file_leaf(...)`, and `folder(...)` on `impl IntoUiElement<H> + use<H>`, selected UI Gallery hover-card snippets now keep `card(...)` / `demo_content(...)` helpers on `impl IntoUiElement<H> + use<H>`, selected UI Gallery tooltip snippets now keep `make_tooltip(...)` / `make_tooltip_with_test_ids(...)` on `impl IntoUiElement<H> + use<H>`, selected UI Gallery resizable snippets now keep `panel(...)` / `box_group(...)` helpers on `impl IntoUiElement<H> + use<...>`, including `resizable/{demo,vertical,handle,rtl}.rs`, selected UI Gallery scroll-area snippets now keep `nested_scroll_routing::row(...)`, `demo::tag_row(...)`, and `expand_at_bottom::{toggle_button,empty_row}` on `impl IntoUiElement<H> + use<H>`, selected UI Gallery data-table snippets now keep `align_end(...)`, `align_inline_start(...)`, `footer(...)`, and `bottom_controls(...)` on `impl IntoUiElement<fret_app::App> + use<...>`, selected UI Gallery table-action snippets now keep `align_end(...)` and `action_row(...)` on `impl IntoUiElement<fret_app::App> + use<...>`, selected UI Gallery table snippets now keep `make_invoice_table(...)` on `impl IntoUiElement<fret_app::App> + use<>` and now drop helper-local `cx` from the `demo` / `footer` / `rtl` variants where the body can stay late-landed, selected UI Gallery separator snippets now keep `section(...)` / `row(...)` helpers on `impl IntoUiElement<H> + use<H>`, selected UI Gallery sidebar snippets now keep `menu_button(...)` helpers on `impl IntoUiElement<...>`-based signatures across `sidebar/{demo,controlled,mobile,rtl}.rs`, selected UI Gallery aspect-ratio snippets now keep `portrait_image(...)`, `square_image(...)`, `rtl_image(...)`, `ratio_example(...)`, and `render_preview(...)` helpers on `impl IntoUiElement<H> + use<H>`, including `aspect_ratio/{demo,portrait,square,rtl}.rs`, selected UI Gallery context-menu snippets now keep `trigger_surface(...)` and `side_menu(...)` helpers on `impl IntoUiElement<H>` with explicit trigger landing seams, including `context_menu/sides.rs`, selected UI Gallery combobox snippets now keep local `state_row(...)` and `state_rows(...)` helpers on `impl IntoUiElement<fret_app::App> + use<>`, selected UI Gallery pagination snippets now keep local `page_number(...)` helpers on `impl IntoUiElement<H> + use<H>`, selected UI Gallery carousel snippets now keep local `slide_card(...)` / `slide(...)` helpers on `impl IntoUiElement<fret_app::App> + use<>`, including the `api`, `duration_embla`, `rtl`, `plugin_autoplay*`, and `events` demos, selected UI Gallery skeleton snippets now keep local `round(...)` / `row(...)` helpers on `impl IntoUiElement<H> + use<H>`, selected UI Gallery popover wrapper helpers now accept/return `IntoUiElement<H>` instead of forcing `AnyElement`, and selected `popover/{demo,with_form}.rs` snippets now also teach `ui::children![cx; ...]` for `PopoverContent` / `PopoverHeader` / `FieldGroup` / `Field` assembly, selected UI Gallery dropdown-menu preview wrappers now accept/return `IntoUiElement<H>`, selected UI Gallery AI wrapper/doc-preview helpers now also accept or expose `IntoUiElement<H>`-based signatures (`centered(...)`, `preview(...)`, `progress_section(...)`, `render_grid_attachment(...)`, `render_list_attachment(...)`, `invisible_marker(...)`, `body_text(...)`, and `clear_action(...)`), including `file_tree_large.rs::preview(...)`; internal gallery wrapper shells now also keep typed wrapper seams in `doc_layout.rs::demo_shell<B>(...)` and `code_editor/mvp/gates.rs::gate_panel<B>(...)`; `fret-ui-shadcn` internal menu-slot wrappers in `context_menu.rs`, `dropdown_menu.rs`, and `menubar.rs` now also accept `IntoUiElement<H>` inputs on `menu_icon_slot(...)`; the thin public constructor/wrapper trial now covers `badge.rs::badge<H, T>(...)`, `checkbox.rs::{checkbox<H>(...), checkbox_opt<H>(...)}`, `progress.rs::progress<H>(...)`, `switch.rs::{switch<H>(...), switch_opt<H>(...)}`, `kbd.rs::kbd<H, T>(...)`, `separator.rs::separator<H>()`, `input_group.rs::input_group<H>(...)`, `input_otp.rs::input_otp<H>(...)`, `command.rs::command<H, I, F, T>(...)`, and the `card.rs` wrapper family, while `kbd.rs::kbd_icon<H>(...)` remains intentionally raw because `Kbd::from_children(...)` still owns concrete landed-child storage; combobox anchor overrides now reuse the generic `PopoverAnchor::build(...).into_anchor(cx)` path instead of a combobox-specific raw alias; the dedicated typography sweep is now landed, so `fret-ui-shadcn/src/typography.rs` keeps the `raw::typography::*` namespace but exposes typed helper outputs and first-party Gallery/examples/`fret-genui-shadcn` call sites now land those helpers explicitly via `.into_element(cx)` only where a concrete `AnyElement` seam is still required, while eager constructor examples such as the accordion snippets and selected dialog/sheet/drawer modal-form snippets now teach `ui::children![cx; ...]` instead of ad-hoc `vec![...into_element(cx)]` child assembly; selected breadcrumb helpers now keep separators on `IntoUiElement<H>`, selected button-group, toggle-group, and drawer helpers now expose `IntoUiElement`-based signatures, including `drawer/{demo,responsive_dialog,sides,scrollable_content}.rs`, selected UI Gallery sheet/dialog snippets now keep `profile_fields(...)` helpers on `impl IntoUiElement<H> + use<H>`, selected UI Gallery dialog scroll/sticky snippets now keep `lorem_block(...)` helpers on `impl IntoUiElement<H> + use<H>`, and selected item, toast, alert, slider, and motion-presets helpers now also stay on `IntoUiElement`-based signatures, including `item/{avatar,icon,link,link_render,dropdown,extras_rtl,gallery}.rs` helpers such as `icon(...)`, `icon_button(...)`, `outline_button(...)`, `outline_button_sm(...)`, `item_basic(...)`, `item_icon(...)`, `item_avatar(...)`, and `item_team(...)`, plus `toast/deprecated.rs::centered(...)`, `sonner/{demo,extras,position}.rs::wrap_controls_row(...)`, `alert/{interactive_links,demo}.rs::{interactive_link,interactive_link_text}(...)`, and `slider/demo.rs::controlled(...)`; broader shadcn/gallery/helper cleanup still remains |
| M4 | In progress | prelude gates are in place, curated component-authoring docs now teach only `IntoUiElement<H>`, stale-name source/doc guards now cover curated docs, `UiChildIntoElement` is now deleted from code, `fret_ui_shadcn::prelude::*` now re-exports `IntoUiElement` so typed direct-crate helpers do not need ad-hoc trait imports, exported `fret-ui-kit` adapter macros plus built-in primitive glue now also attach `IntoUiElement<H>` directly instead of spelling `UiIntoElement` on the first-party macro surface, the public RenderOnce helper macro is now renamed to `ui_component_render_once!`, declarative semantics wrappers (`UiElementTestIdExt`, `UiElementA11yExt`, `UiElementKeyContextExt`) now also land through `IntoUiElement<H>` directly instead of depending on `UiIntoElement` in production code, built-in text primitives (`ui::TextBox`, `ui::RawTextBox`) now also land through `IntoUiElement<H>` directly, the legacy `UiIntoElement` bridge name is now deleted from production code entirely, `docs/first-hour.md` and the `fret-ui-ai` message/workflow builder smoke tests now also stay on the public `IntoUiElement<...>` contract, the shadcn source-alignment guidance now explicitly treats typed doc/page wrapper entry points (for example `DocSection::build(...)`) as part of the first-party exemplar contract, and the focused UI Gallery source gate now covers an expanded `selected_*` helper set across AI/Material 3 pages plus internal wrapper constructors and first-party snippets/wrappers for avatar/button/card/navigation-menu/tabs/collapsible/tooltip/hover-card/context-menu/aspect-ratio/resizable/scroll-area/data-table/table-action/table/separator/sidebar/combobox/pagination/carousel/skeleton/popover/dropdown-menu/item/toast/sonner/alert/slider and other authoring surfaces; only historical docs and negative source-policy assertions still mention the old name |
| M6 | Done | the shadcn raw-seam inventory is now explicitly closed: `use_combobox_anchor(...)` is deleted in favor of `PopoverAnchor::build(...).into_anchor(cx)`, `TooltipContent::{build,text}(...)` and `state.rs::{use_selector_badge,query_status_badge,query_error_alert}` are back on the typed lane, and the only remaining deliberate raw helper contracts on that lane are `kbd.rs::kbd_icon(...)` plus the final-wrapper `text_edit_context_menu*` family, both guarded by source-policy tests and reflected in `TARGET_INTERFACE_STATE.md` / `MIGRATION_MATRIX.md` |

Verification snapshot on 2026-03-13:

- `CARGO_TARGET_DIR=target/codex-assets-reload cargo test -p fret-cookbook --lib`
- `CARGO_TARGET_DIR=target/codex-fret-examples cargo test -p fret-examples --lib`
- `CARGO_TARGET_DIR=target/codex-fretboard cargo test -p fretboard scaffold::templates::tests::todo_template_uses_default_authoring_dialect -- --exact`
- `CARGO_TARGET_DIR=target/codex-fretboard cargo test -p fretboard scaffold::templates::tests::simple_todo_template_has_low_adapter_noise_and_no_query_selector -- --exact`
- `cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app accordion_ -- --nocapture`
- `cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app tabs_ -- --nocapture`
- `cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app toggle_ -- --nocapture`
- current non-`lib.rs` source-policy readout on the compare/example lane leaves only
  `apps/fret-cookbook/examples/chart_interactions_basics.rs::chart_canvas(...) -> AnyElement` as
  the remaining intentional cookbook/examples helper seam.

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

## Update on 2026-03-13

- M3 authoring-surface convergence advanced on the first-party shadcn lane:
  the `alert` wrapper family now matches the `card` family by defaulting to a typed builder
  return (`AlertBuild<H, ...>`) instead of an eager `(cx, ...) -> AnyElement` helper, and the
  new `table(...)` helper family now gives the table docs/snippets the same typed-wrapper posture;
  `field_set(...)` / `field_group(...)` now do the same for grouped form authoring; the
  `empty(...)` / `empty_header(...)` / `empty_media(...)` / `empty_title(...)` /
  `empty_description(...)` / `empty_content(...)` family now gives empty-state authoring the same
  typed wrapper posture while preserving `EmptyMedia::variant(...)` as a fluent builder step; and
  the `pagination(...)` / `pagination_content(...)` / `pagination_item(...)` /
  `pagination_link(...)` family now removes the remaining eager child-list seam from pagination
  root/content/item/link composition while keeping `PaginationPrevious` / `PaginationNext` /
  `PaginationEllipsis` as ordinary typed leaf values.
- the public return-shape rule is now explicit for the thin-wrapper trial:
  if a helper needs to preserve fluent builder affordances before the explicit landing seam,
  it should return a concrete builder/component type rather than an opaque
  `impl IntoUiElement<H>`.
- the crate root/facade now re-export `alert(...)` plus the table wrapper family, and the UI
  Gallery alert/table snippets now teach `shadcn::alert(|cx| ui::children![cx; ...])` and
  `shadcn::table(|cx| ui::children![cx; ...])` as the default first-party authoring pattern;
  selected field/form snippets now also teach `shadcn::field_set(|cx| ...)` /
  `shadcn::field_group(|cx| ...)` instead of `FieldSet::build(...)` / `FieldGroup::build(...)`.
- the first-party Empty exemplars now follow the same lane:
  `apps/fret-ui-gallery/src/ui/snippets/empty/*`, `src/ui/pages/empty.rs`, and
  `src/ui/snippets/spinner/empty.rs` now teach the wrapper family directly, and
  `selected_empty_snippets_prefer_empty_wrapper_family` plus the new
  `empty_helpers_prefer_typed_wrapper_outputs_when_no_raw_slot_storage_is_required`
  gate lock the source policy.
- the first-party Pagination exemplars now also follow the same lane:
  `apps/fret-ui-gallery/src/ui/snippets/pagination/{demo,icons_only,rtl,simple,usage}.rs` and
  `src/ui/pages/pagination.rs` now teach the wrapper family directly, and
  `selected_pagination_snippets_prefer_pagination_wrapper_family` plus the new
  `pagination_helpers_prefer_typed_wrapper_outputs_when_no_raw_slot_storage_is_required`
  gate lock the source policy.
- page-level teaching drift cleanup is now landing behind the same M3 lane:
  selected AI doc pages now use the shared `doc_layout::text_table(...)` helper instead of
  repeating raw `Table::build(...)` teaching, and the `field` page usage/API-reference copy now
  teaches `shadcn::field_set(...)` / `shadcn::field_group(...)` as the grouped authoring default
  rather than `FieldSet::new(...)` / `FieldGroup::new(...)`.
- the same M3 cleanup now also covers a few lower-traffic first-party drifts:
  `pagination/extras.rs` now uses the pagination wrapper family instead of
  `Pagination::new(...)` / `PaginationContent::new(...)`, `checkbox/table.rs` and
  `typography/table.rs` now use the table wrapper family instead of `Table::build(...)`, and the
  `card` page notes now present `card(...)` as the default first-party teaching path while leaving
  `Card::build(...)` explicitly documented as a lower-level option.
- the same already-promoted-family cleanup now also covers the remaining alert drift on the
  first-party teaching lane:
  `alert/{demo,interactive_links,custom_colors,rich_title}.rs` and
  `motion_presets/fluid_tabs_demo.rs::panel(...)` now teach
  `shadcn::alert(|cx| ui::children![cx; ...])` instead of `Alert::new(...)`, and
  `selected_alert_snippets_prefer_alert_wrapper_family` now locks that source policy.
- the canonical compare set and scaffold teaching lane now also close the remaining card drift:
  `apps/fret-examples/src/todo_demo.rs`,
  `apps/fret-cookbook/examples/simple_todo_v2_target.rs`,
  `apps/fret-cookbook/examples/simple_todo.rs`, and
  `apps/fretboard/src/scaffold/templates.rs` now teach
  `shadcn::card(|cx| ui::children![cx; ...])`,
  `shadcn::card_header(|cx| ...)`,
  `shadcn::card_title(...)`,
  `shadcn::card_description(...)`, and
  `shadcn::card_content(|cx| ...)` instead of `Card::build(...)` /
  `CardHeader::build(...)` / `CardContent::build(...)`; template assertions now protect the
  wrapper family and explicitly reject the old build-style defaults.
- the same compare-set cleanup now also reduces one remaining transitional landing seam:
  `apps/fret-examples/src/todo_demo.rs::todo_page(...)` now stays fully late-landed on
  `impl UiChild`, so the default app-facing page shell no longer needs helper-local `cx` or
  `card.into_element(cx)` before the final `Ui` conversion; meanwhile,
  `apps/fret-examples/src/simple_todo_demo.rs::todo_row(...)` now keeps the keyed row helper on
  `impl IntoUiElement<App> + use<>` instead of `AnyElement` while preserving
  `ui::for_each_keyed_with_cx(...)` for the per-row model-watch scope.
- the same page-shell cleanup now also reaches the scaffold generation lane:
  `apps/fretboard/src/scaffold/templates.rs` now generates both `todo` and `simple-todo` page
  helpers as `impl UiChild`, drops helper-local `cx` from `todo_page(...)`, and keeps the
  explicit `.into_element(cx)` step only at the final render-root `Ui` conversion in the emitted
  templates.
- the same app-facing cleanup now also starts closing the UI Gallery top-level snippet lane:
  `apps/fret-ui-gallery/src/ui/snippets/accordion/{basic,borders,card,demo,disabled,extras,multiple,rtl,usage}.rs`
  now expose `impl UiChild + use<>` on their top-level `render(...)` surface, and
  `apps/fret-ui-gallery/src/ui/pages/accordion.rs` now routes those previews through
  `DocSection::build(cx, ...)` instead of `DocSection::new(...)`, so the default docs example path
  no longer re-teaches eager `AnyElement` landing for that family.
- the same UI Gallery app-facing cleanup now also covers the tabs family:
  `apps/fret-ui-gallery/src/ui/snippets/tabs/{demo,disabled,extras,icons,line,list,rtl,usage,vertical,vertical_line}.rs`
  now expose `impl UiChild + use<>` on their top-level `render(...)` surface, and
  `apps/fret-ui-gallery/src/ui/pages/tabs.rs` now routes those previews through
  `DocSection::build(cx, ...)` instead of `DocSection::new(...)`, so the default docs example path
  no longer re-teaches eager `AnyElement` landing for that family either.
- the same UI Gallery app-facing cleanup now also covers the toggle family:
  `apps/fret-ui-gallery/src/ui/snippets/toggle/{demo,disabled,label,outline,rtl,size,usage,with_text}.rs`
  now expose `impl UiChild + use<>` on their top-level `render(...)` surface, and
  `apps/fret-ui-gallery/src/ui/pages/toggle.rs` now routes those previews through
  `DocSection::build(cx, ...)` instead of `DocSection::new(...)`, so the default docs example path
  no longer re-teaches eager `AnyElement` landing for that family either.
- the same UI Gallery app-facing cleanup now also covers the radio-group family:
  `apps/fret-ui-gallery/src/ui/snippets/radio_group/{choice_card,demo,description,disabled,extras,fieldset,invalid,label,plans,rtl,usage}.rs`
  now expose `impl UiChild + use<>` on their top-level `render(...)` surface, and
  `apps/fret-ui-gallery/src/ui/pages/radio_group.rs` now routes those previews through
  `DocSection::build(cx, ...)` instead of `DocSection::new(...)`, so the default docs example path
  no longer re-teaches eager `AnyElement` landing for that family either.
- the same UI Gallery app-facing cleanup now also covers the slider family:
  `apps/fret-ui-gallery/src/ui/snippets/slider/{demo,extras,label,usage}.rs`
  now expose `impl UiChild + use<>` on their top-level `render(...)` surface, keep their local
  model state inside the snippet, and `apps/fret-ui-gallery/src/ui/pages/slider.rs` now routes
  those previews through `DocSection::build(cx, ...)` instead of `DocSection::new(...)`, so the
  default docs example path no longer re-teaches eager `AnyElement` landing for that family
  either.
- the same UI Gallery app-facing cleanup now also covers the native-select family:
  `apps/fret-ui-gallery/src/ui/snippets/native_select/{demo,disabled,invalid,label,rtl,usage,with_groups}.rs`
  now expose `impl UiChild + use<>` on their top-level `render(...)` surface, keep value/open
  model state inside the snippet instead of the page shell, and
  `apps/fret-ui-gallery/src/ui/pages/native_select.rs` now routes those previews through
  `DocSection::build(cx, ...)` instead of `DocSection::new(...)`, so the default docs example path
  no longer re-teaches eager `AnyElement` landing for that family either.
- the same UI Gallery app-facing cleanup now also covers the resizable family:
  `apps/fret-ui-gallery/src/ui/snippets/resizable/{demo,handle,notes,rtl,usage,vertical}.rs`
  now expose `impl UiChild + use<>` on their top-level `render(...)` surface, keep fractions model
  state inside the snippet instead of threading it through page/content/runtime-driver relay
  fields, and `apps/fret-ui-gallery/src/ui/pages/resizable.rs` now routes those previews through
  `DocSection::build(cx, ...)` instead of `DocSection::new(...)`, so the default docs example path
  no longer re-teaches eager `AnyElement` landing for that family either.
- after `accordion` / `tabs` / `toggle` / `radio_group` / `slider` / `native_select` /
  `resizable`, the highest-value remaining app-facing UI Gallery queue is now the next
  top-level-family sweep rather than more relay-state cleanup on this page.
- the advanced IMUI compare lane now also stops leaking raw helper signatures on non-raw helpers:
  `apps/fret-examples/src/imui_editor_proof_demo.rs::{render_editor_name_assist_surface,render_authoring_parity_surface,render_authoring_parity_shared_state,render_authoring_parity_declarative_group,render_authoring_parity_imui_group,render_authoring_parity_imui_host}`
  now expose `IntoUiElement<...>`-based signatures while still keeping the internal
  `PropertyGroup::into_element(...)` and `imui_build(...)` landing seams explicit; after this
  pass, the real examples/cookbook source scan now leaves `chart_interactions_basics.rs::chart_canvas(...)`
  as the only remaining intentional `-> AnyElement` helper outside `lib.rs` source-policy files.
- the thin helper typed-lane trial now also covers `radio_group(...)` /
  `radio_group_uncontrolled(...)`:
  both helpers now return the concrete `RadioGroup` value instead of eagerly landing
  `AnyElement`, preserving fluent configuration steps such as `.a11y_label(...)`,
  `.disabled(...)`, and `.style(...)` before the explicit landing seam; the
  `public_thin_constructors_or_wrappers_prefer_typed_conversion_outputs_when_no_raw_seam_is_required`
  source gate now records and protects that rule.
- the simple form-control lane now also drops its last misleading raw helper names:
  `input(...)` and `textarea(...)` now return `Input` / `Textarea` instead of exposing
  multi-argument `(cx, ...) -> AnyElement` render functions on the public surface, aligning text
  fields with the same builder-preserving authoring story as `checkbox`, `switch`, and
  `radio_group`.
- the same builder-preserving cleanup now also covers `slider(...)`:
  the public helper now returns `Slider` instead of exposing the old full-parameter
  `(cx, ...) -> AnyElement` render function, so default slider authoring stays on fluent
  component values while the raw render path remains private implementation detail.
- the same thin-helper cleanup now also covers `toggle(...)` / `toggle_uncontrolled(...)`:
  both helpers now return the concrete `Toggle` builder instead of eagerly landing
  `AnyElement`, and their closure inputs now accept typed child values so callers do not need to
  pre-land child content before the helper-owned internal child-list seam.
- the same builder-preserving cleanup now also covers `tabs(...)` / `tabs_uncontrolled(...)`:
  both helpers now return the concrete `Tabs` builder instead of eagerly landing the root helper,
  so ordinary tabs authoring can keep fluent root configuration open until the explicit landing
  seam.
- the same builder-preserving cleanup now also covers
  `accordion_single(...)` / `accordion_single_uncontrolled(...)` /
  `accordion_multiple(...)` / `accordion_multiple_uncontrolled(...)`:
  all four helpers now return the concrete `Accordion` builder instead of eagerly landing the
  root helper, so ordinary accordion authoring can still attach root-level
  `.collapsible(...)`, `.orientation(...)`, or `.loop_navigation(...)` before the explicit
  landing seam.
- the same builder-preserving cleanup now also covers
  `toggle_group_single(...)` / `toggle_group_single_uncontrolled(...)` /
  `toggle_group_multiple(...)` / `toggle_group_multiple_uncontrolled(...)`:
  all four helpers now return the concrete `ToggleGroup` builder instead of eagerly landing the
  root helper, so ordinary grouped-toggle authoring can still attach root-level
  `.variant(...)`, `.size(...)`, `.orientation(...)`, or `.roving_focus(...)` before the
  explicit landing seam.
- the same builder-preserving cleanup now also covers `resizable_panel_group(...)`:
  the helper now returns the concrete `ResizablePanelGroup` builder instead of eagerly landing the
  root helper, so ordinary resizable authoring can still attach root-level `.axis(...)`,
  `.style(...)`, or `.test_id_prefix(...)` before the explicit landing seam.
- the same builder-preserving cleanup now also covers
  `navigation_menu(...)` / `navigation_menu_uncontrolled(...)`:
  both helpers now return the concrete `NavigationMenu` builder instead of eagerly landing the
  root helper, so ordinary navigation-menu authoring can still attach root-level
  `.viewport(...)`, `.indicator(...)`, `.md_breakpoint_query(...)`, or `.delay_ms(...)` before
  the explicit landing seam.
- the same builder-preserving cleanup now also covers
  `avatar_sized(...)`, `item_sized(...)`, `item_group(...)`, `scroll_area(...)`, and
  `native_select(...)`:
  these helpers now return their concrete builder types (`Avatar`, `Item`, `ItemGroup`,
  `ScrollArea`, `NativeSelect`) instead of eagerly landing the helper output, so ordinary
  avatar/item/scroll/select authoring can still attach root-level configuration before the
  explicit landing seam.
- the first-party UI Gallery teaching surface is now converging on that same lane for the
  corresponding high-signal snippets and docs copy:
  `avatar/sizes.rs`, `item/{size,group}.rs`, `scroll_area/{usage,horizontal,nested_scroll_routing}.rs`,
  `native_select/{demo,usage,disabled,invalid,label,with_groups,rtl}.rs`, plus the
  `avatar` / `item` / `scroll_area` / `native_select` page notes now teach the helper family
  (`avatar_sized(...)`, `item_sized(...)`, `item_group(...)`, `scroll_area(...)`,
  `native_select(...)`) as the default first-party path while leaving explicit `::new(...)`
  or `new_controllable(...)` forms to advanced/raw seams.
- the same first-party teaching sweep now also closes the obvious controlled/uncontrolled drift on
  the `slider` / `radio_group` lane:
  `slider/{usage,label,demo}.rs`, `field/slider.rs`, `progress/controlled.rs`,
  `radio_group/{usage,label}.rs`, `form/upstream_demo.rs`, and the corresponding `slider` /
  `radio_group` page copy now teach `slider(model)`, `radio_group(model, items)`, and
  `radio_group_uncontrolled(default, items)` as the default first-party helper family, while
  `Slider::new_controllable(...)` remains only on the examples that explicitly need the
  default-value bridge.
- the same first-party teaching sweep now also closes the root-constructor drift on the
  `navigation_menu` / `resizable` lane:
  `navigation_menu/{usage,demo,docs_demo,link_component,rtl}.rs`,
  `resizable/{usage,demo,vertical,handle,rtl}.rs`, and the corresponding page copy now teach
  `navigation_menu(cx, model, |cx| ..)` and `resizable_panel_group(cx, model, |cx| ..)` as the
  default first-party root helper family, while the raw `NavigationMenu::new(...)` /
  `ResizablePanelGroup::new(...)` roots remain available as explicit builder seams.
- the same UI Gallery default-app top-level snippet cleanup now also records the
  `navigation_menu` family:
  `apps/fret-ui-gallery/src/ui/snippets/navigation_menu/{demo,docs_demo,link_component,rtl,usage}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, while
  `apps/fret-ui-gallery/src/ui/pages/navigation_menu.rs` consumes those previews through
  `DocSection::build(cx, ...)`; the old `pub fn render(...) -> AnyElement` teaching pattern is now
  forbidden for that family by
  `ui_authoring_surface_default_app::{navigation_menu_app_facing_snippets_prefer_ui_cx_on_the_default_app_surface,navigation_menu_page_uses_typed_doc_sections_for_app_facing_snippets}`.
- the same UI Gallery default-app top-level snippet cleanup now also records the
  `scroll_area` family:
  `apps/fret-ui-gallery/src/ui/snippets/scroll_area/{demo,usage,horizontal,nested_scroll_routing,rtl}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, while
  `apps/fret-ui-gallery/src/ui/pages/scroll_area.rs` consumes those previews through
  `DocSection::build(cx, ...)` and intentionally keeps
  `drag_baseline` / `expand_at_bottom` on `DocSection::new(...)`; the old
  `pub fn render(...) -> AnyElement` teaching pattern is now forbidden for the app-facing family by
  `ui_authoring_surface_default_app::{scroll_area_app_facing_snippets_prefer_ui_cx_on_the_default_app_surface,scroll_area_page_uses_typed_doc_sections_for_app_facing_snippets}`, while
  `selected_scroll_area_snippet_helpers_prefer_into_ui_element_over_anyelement` now also locks the
  helper-family preference for `shadcn::scroll_area(...)`.
- the same first-party teaching sweep now also closes the remaining default-root drift on the
  `tabs` / `toggle` / `accordion` lane:
  `tabs/{usage,demo,disabled,extras,icons,line,list,rtl,vertical,vertical_line}.rs`,
  `toggle/{usage,demo,outline,with_text,disabled,size,rtl,label}.rs`, and
  `accordion/{demo,basic,multiple,disabled,borders,card,extras,rtl}.rs` now teach
  `tabs_uncontrolled(cx, default, |cx| ..)`,
  `toggle_uncontrolled(cx, default, |cx| ..)` / `toggle(cx, model, |cx| ..)`, and
  `accordion_single_uncontrolled(cx, default, |cx| ..)` /
  `accordion_multiple_uncontrolled(cx, default, |cx| ..)` as the default first-party helper
  family, while `accordion/usage.rs` intentionally keeps the composable `AccordionRoot` surface
  as the explicit advanced seam and the corresponding `tabs` / `toggle` / `accordion` page copy
  now records that boundary directly.
- the canonical keyed-list compare set now also has a first dedicated helper:
  `fret_ui_kit::ui::for_each_keyed(cx, items, |item| key, |item| child)` exists specifically to
  keep dynamic keyed lists on the ordinary `ui::v_flex(|cx| ...)` / `ui::h_flex(|cx| ...)`
  authoring lane without falling back to `*_build(|cx, out| ...)` plus per-row
  `out.push_ui(cx, ui::keyed(...))` boilerplate; `apps/fret-examples/src/todo_demo.rs`,
  `apps/fret-cookbook/examples/simple_todo_v2_target.rs`, and the scaffold `simple-todo` / `todo`
  templates now all use that helper, and the scaffold README/tests teach it as the default
  first-party keyed-list story.
- the next UI Gallery app-facing snippet batch is now `progress`:
  it is currently the smallest remaining page-local family that still teaches top-level
  `UiCx -> AnyElement` returns without also carrying diagnostics-owned raw seams.
- M6 raw-seam inventory now has executable source gates in `surface_policy_tests.rs`:
  explicit raw/bridge helpers are currently limited to
  `kbd.rs::kbd_icon(...)` and
  `text_edit_context_menu.rs::{text_edit_context_menu,text_selection_context_menu,text_edit_context_menu_controllable,text_selection_context_menu_controllable}`;
  combobox anchor overrides now go through the generic `PopoverAnchor::build(...).into_anchor(cx)`
  path instead of a combobox-specific raw alias;
  the old legacy module-local root helpers (`drawer(...)`, `menubar(...)`, `combobox(...)`) are
  now deleted, so this inventory is reduced to explicit raw/bridge seams only. The text-edit
  context-menu family is now explicitly documented as a deliberate final wrapper seam rather than
  a missing typed builder path.
- `tooltip.rs::TooltipContent::{build,text}(...)` have now been promoted to typed helper outputs,
  so tooltip content authoring no longer contributes public free-function `-> AnyElement` seams.
- `state.rs::{use_selector_badge,query_status_badge}` have now been promoted back to typed
  `Badge` outputs, `query_error_alert(...)` now returns `Option<Alert>`, and the text-edit
  context-menu helpers still land as `AnyElement` only at the final wrapper seam while accepting
  typed trigger values through `IntoUiElement<H>`.
- the keyed-list lane now also has the narrow keyed-scope follow-up:
  `fret_ui_kit::ui::for_each_keyed_with_cx(cx, items, |item| key, |cx, item| child)` exists for
  row builders that really need the inner keyed child scope itself; the first concrete migration
  is `apps/fret-examples/src/simple_todo_demo.rs`, which now keeps keyed row rendering on the
  ordinary `ui::v_flex(|cx| ..)` lane without regressing to `v_flex_build(...)` +
  `cx.keyed(...)`. This keeps the keyed-list story two-tiered (`for_each_keyed` by default,
  `for_each_keyed_with_cx` when the row needs the keyed scope) without introducing a broader
  `keyed_column(...)` abstraction yet.
- the default user-facing docs now follow that same keyed-list rule:
  `docs/first-hour.md`, `docs/authoring-golden-path-v2.md`, and
  `docs/examples/todo-app-golden-path.md` now teach `ui::for_each_keyed(...)` as the default
  identity helper, mention `ui::for_each_keyed_with_cx(...)` only as the keyed-scope escape hatch,
  and demote `*_build(...)` sink collection to an explicit advanced/manual seam.
- the selected query example lane now also follows the promoted card/default composition story:
  `apps/fret-examples/src/query_demo.rs` and
  `apps/fret-examples/src/query_async_tokio_demo.rs` now teach `shadcn::card(...)` plus the slot
  helper family and use ordinary `ui::h_row(...)` / `ui::v_flex(...)` composition for fixed child
  lists, leaving only one narrow conditional-row `ui::v_flex_build(...)` seam in `query_demo`
  where optional retry/duration diagnostics still make sink-style assembly the smallest escape
  hatch.
- the cookbook high-signal example lane now also follows the same promoted card story:
  `apps/fret-cookbook/examples/query_basics.rs`,
  `apps/fret-cookbook/examples/form_basics.rs`,
  `apps/fret-cookbook/examples/async_inbox_basics.rs`, and
  `apps/fret-cookbook/examples/router_basics.rs` now teach `shadcn::card(...)` plus the slot
  helper family instead of `Card::build(...)`, while preserving only the narrow justified seams
  that still need manual child emission or explicit typed router/outlet ownership.
- the next cookbook first-contact batch now also follows that same card teaching surface:
  `apps/fret-cookbook/examples/toggle_basics.rs`,
  `apps/fret-cookbook/examples/payload_actions_basics.rs`,
  `apps/fret-cookbook/examples/hello_counter.rs`,
  `apps/fret-cookbook/examples/text_input_basics.rs`, and
  `apps/fret-cookbook/examples/commands_keymap_basics.rs` now teach `shadcn::card(...)` plus the
  slot helper family instead of `Card::build(...)`, with `hello_counter.rs` also moving its outer
  footer shell onto `card_footer(...)`.
- the next cookbook interop/state batch now also follows that same outer-card story:
  `apps/fret-cookbook/examples/undo_basics.rs`,
  `apps/fret-cookbook/examples/drag_basics.rs`,
  `apps/fret-cookbook/examples/external_texture_import_basics.rs`,
  `apps/fret-cookbook/examples/assets_reload_epoch_basics.rs`, and
  `apps/fret-cookbook/examples/date_picker_basics.rs` now teach `shadcn::card(...)` plus the
  slot helper family for their outer shells instead of `Card::build(...)`, while preserving the
  justified advanced seams that still belong to interop/assets internals and retained viewport
  ownership.
- the next cookbook visual/app-support batch now also follows that same outer-card story:
  `apps/fret-cookbook/examples/customv1_basics.rs`,
  `apps/fret-cookbook/examples/drop_shadow_basics.rs`,
  `apps/fret-cookbook/examples/effects_layer_basics.rs`,
  `apps/fret-cookbook/examples/toast_basics.rs`, and
  `apps/fret-cookbook/examples/markdown_and_code_basics.rs` now teach `shadcn::card(...)` plus
  the slot helper family instead of `Card::build(...)`, with
  `customv1_basics.rs::panel_shell(...)` also moving onto the wrapper family so advanced
  cookbook helper code no longer re-teaches the old outer-shell pattern.
- the advanced cookbook bootstrap lane now also matches the real `UiAppBuilder` surface:
  `apps/fret-cookbook/examples/{external_texture_import_basics,embedded_viewport_basics,gizmo_basics,docking_basics,chart_interactions_basics}.rs`
  now explicitly install Lucide icons through `.setup(fret_icons_lucide::app::install)` instead
  of teaching a nonexistent `.with_lucide_icons()` facade, and
  `apps/fret-cookbook/Cargo.toml` now includes `dep:fret-icons-lucide` behind the
  `cookbook-bootstrap` feature so the documented bootstrap path matches the code that actually
  compiles.
- the advanced retained/interop cookbook lane now also converges on the promoted outer-card
  family where the shell is ordinary authoring surface:
  `apps/fret-cookbook/examples/icons_and_assets_basics.rs`,
  `apps/fret-cookbook/examples/embedded_viewport_basics.rs`,
  `apps/fret-cookbook/examples/gizmo_basics.rs`,
  `apps/fret-cookbook/examples/docking_basics.rs`, and
  `apps/fret-cookbook/examples/chart_interactions_basics.rs` now teach `shadcn::card(...)` plus
  the slot helper family for their top-level shells instead of `Card::build(...)`, while
  intentionally keeping raw retained/canvas/viewport seams such as
  `chart_interactions_basics.rs::chart_canvas(...)`, dock-host retained roots, and embedded
  viewport ownership explicit.
- the remaining cookbook card-shell cleanup is now closed on the examples tree:
  `apps/fret-cookbook/examples/theme_switching_basics.rs`,
  `apps/fret-cookbook/examples/data_table_basics.rs`,
  `apps/fret-cookbook/examples/virtual_list_basics.rs`,
  `apps/fret-cookbook/examples/canvas_pan_zoom_basics.rs`,
  `apps/fret-cookbook/examples/overlay_basics.rs`, and
  `apps/fret-cookbook/examples/utility_window_materials_windows.rs` now also teach
  `shadcn::card(...)` plus the slot helper family for their ordinary shells, so
  `apps/fret-cookbook/examples/**` no longer contains first-party
  `Card::build(...)` / `CardHeader::build(...)` / `CardContent::build(...)` teaching on the
  default authoring lane.
- the cookbook source-policy lane now also locks that convergence directly in code:
  `apps/fret-cookbook/src/lib.rs::cookbook_examples_keep_card_wrapper_family_as_the_only_card_teaching_surface`
  now fails if any example reintroduces `shadcn::Card::build(...)`,
  `shadcn::CardHeader::build(...)`, or `shadcn::CardContent::build(...)`, while
  `retained_canvas_helpers_keep_raw_landing_seams` continues to record
  `chart_interactions_basics.rs::chart_canvas(...) -> AnyElement` as the one explicit retained
  bridge seam on that lane.
- the examples helper-return lane is now also tightened on the default-app side:
  `apps/fret-examples/src/{custom_effect_v2_identity_web_demo,custom_effect_v2_web_demo,custom_effect_v2_lut_web_demo,custom_effect_v2_glass_chrome_web_demo}.rs`
  now keep `stage_tile(...)` on `impl IntoUiElement<App> + use<>` and move the explicit
  `.into_element(cx)` seam back to the stage `Vec<AnyElement>` assembly site, while
  `apps/fret-examples/src/async_playground_demo.rs::catalog_item(...)` now likewise returns
  `impl IntoUiElement<KernelApp> + use<>` and lands explicitly at the `out.push(...)` sink.
  `apps/fret-examples/src/lib.rs` now records those helpers in source-policy tests so they do not
  drift back to `AnyElement`-typed authoring helpers.
- the first-party card teaching lane is now tightened beyond the docs page:
  `card/{size,card_content,image,compositions}.rs` now teach `card(...)` plus the slot helper
  family instead of eager `Card::new(...)` / `CardHeader::new(...)` / `CardContent::new(...)` /
  `CardFooter::new(...)`, `card/{demo,rtl}.rs` now also use `card_footer(...)` instead of
  `CardFooter::build(...)`, and `compositions.rs::cell(...)` now accepts generic
  `IntoUiElement<App>` card values so wrapper-family examples no longer need a concrete
  `shadcn::Card` parameter; `selected_card_snippets_prefer_card_wrapper_family` now locks that
  source policy.
- the same wrapper-family teaching rule now also applies to a few ordinary non-card compositions
  that still used card constructors as their outer shell:
  `tabs/demo.rs`, `input_otp/form.rs`, `collapsible/basic.rs`, and
  `motion_presets/fluid_tabs_demo.rs` now teach `card(...)` plus the slot helper family instead of
  `Card::new(...)` / `CardHeader::new(...)` / `CardContent::new(...)` / `CardFooter::new(...)`,
  and the existing card wrapper source gate now covers those call sites too.
- the same low-risk card-shell sweep now also covers selected calendar, accordion, and motion
  examples:
  `calendar/presets.rs`, `motion_presets/{overlay_demo,stagger_demo,stack_shift_list_demo}.rs`,
  and `accordion/card.rs` now teach `card(...)` plus the slot helper family for their outer shell,
  and `selected_card_snippets_prefer_card_wrapper_family` now guards those call sites too.
- the same first-party card-shell convergence now also reaches `toast` and `chart` demos:
  `toast/deprecated.rs` now teaches its deprecation panel through `card(...)` plus the slot helper
  family, while `chart/demo.rs` now keeps `trending_footer(...)` and `chart_card(...)` on typed
  `IntoUiElement<App>` helpers and uses `card(...)` plus the slot helper family for the chart
  shell; the Gallery source gates now cover those card-shell and typed-helper expectations too.
- the same low-risk card-shell cleanup now also touches the lightweight inset cards inside
  `form/upstream_demo.rs`:
  its `mobile_field` card and the marketing/security notification cards now teach
  `card(...)` + `card_content(...)` instead of raw `Card::new(...)` / `CardContent::new(...)`,
  with the existing card wrapper source gate extended to that file.
- the same card-family teaching convergence now also covers selected motion, skeleton,
  accordion/collapsible, and remaining in-family card examples:
  `motion_presets/{preset_selector,token_snapshot}.rs`, `skeleton/card.rs`,
  `accordion/extras.rs`, `collapsible/settings_panel.rs`, and `card/meeting_notes.rs`
  now teach `card(...)` plus the slot helper family, while `meeting_notes.rs` also teaches
  `card_action(...)`; `selected_card_snippets_prefer_card_wrapper_family` now guards those
  call sites too.
- the low-risk card-shell sweep now also closes the last ordinary AI transcript card in this lane:
  `ai/speech_input_demo.rs` now teaches `card(...)` + `card_content(...)` for the transcript
  surface instead of `Card::new(...)` / `CardContent::new(...)`, and the card wrapper source gate
  now covers that call site as well.
- the same first-party card-shell convergence now also closes the remaining ordinary `carousel`
  examples:
  `carousel/{basic,api,demo,duration_embla,events,expandable,focus_watch,loop_carousel,loop_downgrade_cannot_loop,options,orientation_vertical,parts,plugin_autoplay,plugin_autoplay_controlled,plugin_autoplay_delays,plugin_autoplay_stop_on_focus,plugin_autoplay_stop_on_last_snap,plugin_wheel_gestures,rtl,sizes,sizes_thirds,spacing,spacing_responsive,usage}.rs`
  now teach `card(...)` + `card_content(...)` instead of `Card::new(...)`, and
  `selected_card_snippets_prefer_card_wrapper_family` now guards those call sites too.
- the follow-up policy decision for the last snippet hits is also closed:
  `sidebar/*` and `material3/*` do not keep a special exemption for raw card constructors, because
  those cards are ordinary first-party showcase shells rather than low-level implementation seams.
  `sidebar/{usage,demo,rtl,use_sidebar,mobile,controlled}.rs` and
  `material3/{state_matrix,menu,text_field,autocomplete,touch_targets,snackbar,tooltip,list}.rs`
  now teach `card(...)` + `card_header(...)` + `card_content(...)`, and the UI Gallery source gate
  now locks the stronger invariant that `src/ui/snippets/**` contains no `shadcn::Card::*`
  constructor family at all.
- the same page/docs sweep now also closes the remaining card teaching ambiguity above the snippet
  tree:
  `src/ui/pages/**` no longer reintroduces eager `shadcn::Card::*` constructors, and the only
  remaining first-party mention of `Card::build(...)` on that lane is the explicit note in
  `pages/card.rs` that frames it as a lower-level late-child-collection escape hatch rather than a
  default authoring path.
- grouped-field authoring cleanup is now also extending beyond the dedicated field/form snippets:
  selected label/fieldset controls such as `combobox`, `select`, `native_select`, `date_picker`,
  `toggle_group`, `checkbox`, `slider`, `switch`, `input`, and `radio_group` now teach
  `field_group(...)` / `field_set(...)` directly on first-party snippets rather than the old
  `FieldGroup::new(...)` / `FieldSet::new(...)` constructor path.
- next M3 priority:
  cookbook outer-shell convergence and the shadcn raw-seam closure are now effectively done, so
  keep M3 focused on the still-open authoring lanes in this order:
  1. first-party app/example extracted helpers that still return raw `AnyElement` without being
     true retained/diagnostic/overlay seams;
  2. default-app/UI Gallery reusable helpers that should settle on `impl UiChild` or
     `impl IntoUiElement<fret_app::App>` instead of snippet-local raw helper returns;
  3. any remaining shadcn/component helper surface drift only when it is still teaching the wrong
     product vocabulary, not just because an internal wrapper still lands at an explicit root seam.
