# Into-Element Surface (Fearless Refactor v1) — TODO

This TODO list tracks the work described in `DESIGN.md`.

Because this is a pre-release reset, "done" means we actually delete superseded public conversion
names rather than preserve them for inertia.

Companion docs:

- `DESIGN.md`
- `MILESTONES.md`
- `TARGET_INTERFACE_STATE.md`
- `MIGRATION_MATRIX.md`

Execution note on 2026-03-12:

- this is now the first active interface-refactor lane,
- do M0/M1 here before expanding trait-budget follow-ups elsewhere,
- use the canonical compare set (`simple_todo_v2_target`, `todo_demo`, scaffold template) as the
  first downstream migration/evidence target after the unified trait lands.

## M0 — Lock the target vocabulary

- [x] Finalize `TARGET_INTERFACE_STATE.md` as the single source of truth for conversion vocabulary.
- [x] Decide the final public name for the unified component conversion trait.
- [x] Explicitly classify each current conversion name as:
  - [x] kept publicly,
  - [x] kept internally only,
  - [x] moved to advanced/raw only,
  - [x] deleted entirely.
- [x] Confirm that `Ui = Elements` and app-facing `UiChild` are retained.

## M1 — Introduce one public conversion contract

- [x] Add one unified public conversion trait in `fret-ui-kit`.
- [x] Ensure the trait is generic over `H: UiHost` at the trait level rather than splitting host
  agnostic and host-bound concepts publicly.
- [x] Provide temporary internal adapters so current implementations can migrate incrementally.
- [x] Keep `.into_element(cx)` method syntax working on both ordinary values and host-bound
  builder values.

Implementation note after the first landing:

- `IntoUiElement<H>` is now the curated public conversion name on `fret-ui-kit` / `fret`
  component-facing surfaces.
- host-agnostic values still feed that public surface through the legacy `UiIntoElement`
  implementation path for now.
- `UiBuilderHostBoundIntoElementExt` has now been deleted from the codebase; `UiBuilder<T>`
  lands through `IntoUiElement<H>` directly.
- `UiIntoElement` is now doc-hidden scaffolding under `fret_ui_kit::ui_builder` rather than a
  root-exported curated surface.

Validation note on 2026-03-12:

- verified the landing with
  `cargo test -p fret-ui-kit --lib --no-run`,
  `cargo test -p fret --lib --no-run`,
  `cargo test -p fret-examples --lib --no-run`,
  and `cargo check -p fretboard`.

## M2 — Rewire builders and child pipelines

- [x] Migrate `UiBuilder<T>` landing paths to the unified public conversion contract without
  relying on the hidden bridge import.
- [x] Migrate `ui::children!` to consume the unified contract.
- [x] Migrate heterogeneous child builders (`FlexBox`, `ContainerBox`, `StackBox`, and related
  host-bound builders) to the unified contract.
- [x] Keep any extra bridging traits private or advanced-only if Rust still needs them
  internally.

Implementation note on 2026-03-12:

- `fret-ui-kit::imui::UiWriterUiKitExt::add_ui(...)` now lands directly through
  `IntoUiElement<H>`; there is no separate immediate-mode child conversion bridge anymore.
- `ui::children!`, `UiElementSinkExt`, and the `fret-ui-kit::ui` child-collection helpers now use
  `IntoUiElement<H>` directly, so the old child-pipeline bridge trait is gone from code.
- host-bound builders in `fret-ui-kit::ui` now implement `IntoUiElement<H>` directly, and
  `UiBuilder<T>::into_element(cx)` resolves through the unified contract.
- `UiHostBoundIntoElement<H>` has now also been deleted from `fret-ui-kit`; there is no remaining
  public host-bound compatibility alias in code.
- `fret-ui-kit` no longer re-exports `UiIntoElement` or `UiChildIntoElement` from the crate root.
- `fret::UiChild` now lands directly through `IntoUiElement<App>` rather than the child-pipeline
  bridge trait.
- first-party `fret-ui-shadcn` overlay/single-child builders and `fret-router-ui` route outlet
  helpers now also accept `IntoUiElement<H>` directly instead of spelling `UiChildIntoElement<H>`.

Validation note on 2026-03-12:

- verified with
  `cargo test -p fret-ui-shadcn --lib --no-run --message-format=short`,
  `cargo test -p fret-ui-kit --lib --no-run --message-format=short`,
  `cargo test -p fret --lib --no-run --message-format=short`,
  `cargo test -p fret-examples --lib --no-run --message-format=short`,
  `cargo check -p fretboard --message-format=short`,
  `cargo test -p fret --lib --message-format=short`,
  `cargo test -p fretboard --message-format=short`,
  `cargo test -p fret-ui-shadcn --lib dropdown_menu_trigger_build_push_ui_accepts_late_landed_child --message-format=short`,
  and `cargo test -p fret-ui-shadcn --lib popover_build_opens_on_trigger_activate_with_late_landed_parts --message-format=short`.

## M3 — Migrate first-party surfaces

- [x] Migrate `ecosystem/fret` curated app/component re-exports to the new vocabulary.
- [x] Migrate `ecosystem/fret-ui-kit` curated docs and examples.
- [ ] Migrate `ecosystem/fret-ui-shadcn` reusable helper surfaces where raw `AnyElement` is not
  conceptually required.
- [ ] Keep the canonical authoring compare set aligned on the target vocabulary:
  - [x] `apps/fret-cookbook/examples/simple_todo_v2_target.rs`
  - [x] `apps/fret-examples/src/todo_demo.rs`
  - [x] `apps/fretboard/src/scaffold/templates.rs`
- [ ] Migrate official cookbook examples toward `Ui` / `UiChild`.
- [ ] Migrate selected `apps/fret-examples` helper surfaces that are still on raw child returns.
- [ ] Migrate UI Gallery in two lanes:
  - [ ] app-facing teaching snippets toward `UiChild`,
  - [ ] generic reusable snippets toward the unified component conversion trait,
  - [ ] leave justified diagnostics/harness/raw helpers on `AnyElement`.

Implementation note on 2026-03-12:

- the canonical compare set now shares the same posture:
  app-facing imports via `fret::app::prelude::*`,
  `App` / `WindowId`,
  extracted helpers returning `impl UiChild`,
  and one explicit `card/content.into_element(cx)` landing seam before the page shell.
- `apps/fret-cookbook/examples/customv1_basics.rs` now keeps both advanced reusable helpers
  `panel_shell(...)` and `preview_content(...)` on `IntoUiElement<KernelApp>`-based signatures
  instead of returning raw `AnyElement` for non-raw composition.
- `fret-ui-shadcn` `ui_ext/support.rs` and `ui_ext/data.rs` now implement
  `IntoUiElement<H>` directly, so shadcn reusable glue no longer spells
  `UiIntoElement` on those adapters.
- `fret-ui-shadcn` `ui_builder_ext/*` reusable helper closures now accept values that land through
  `IntoUiElement<H>` instead of requiring `AnyElement`-typed closure returns up front.
- `fret-ui-shadcn` internal menu-slot wrappers in
  `context_menu.rs`, `dropdown_menu.rs`, and `menubar.rs`
  now accept `IntoUiElement<H>` inputs for `menu_icon_slot(...)` instead of forcing pre-landed
  `AnyElement`, while keeping the wrapper's own output as an explicit landed menu row slot.
- `fret-ui-shadcn` now has a thin public constructor/wrapper trial:
  `badge.rs::badge<H, T>(...)`, `kbd.rs::kbd<H, T>(...)`,
  `separator.rs::separator<H>()`, `input_group.rs::input_group<H>(...)`,
  `input_otp.rs::input_otp<H>(...)`, and `command.rs::command<H, I, F, T>(...)`
  now expose typed constructor/wrapper outputs, while
  `kbd.rs::kbd_icon<H>(...)` remains an explicit raw helper because
  `Kbd::from_children(...)` still owns a concrete `Vec<AnyElement>` child seam.
- `fret-ui-shadcn/src/typography.rs` now completes the dedicated typography lane:
  `h1` / `h2` / `h3` / `h4` / `p` / `lead` / `large` / `small` / `muted` /
  `inline_code` / `blockquote` / `list` now expose
  `(text_or_items) -> impl IntoUiElement<H> + use<...>` rather than
  `(cx, ...) -> AnyElement`, while intentionally keeping the `shadcn::raw::typography::*`
  namespace stable because typography is still a docs/helper surface rather than a promoted
  registry component contract.
- the typography sweep is now migrated across first-party call sites:
  `ecosystem/fret-genui-shadcn` resolvers, dedicated typography snippets/pages, prose-heavy
  Gallery snippets (`separator` / `sidebar` / `resizable` / `accordion` / `combobox` /
  `calendar` / `date_picker` / `scroll_area` / `sonner` / `slider` / `item` / `ai`), the
  `previews/magic.rs` page, and the affected `apps/fret-examples/*` surfaces now all land
  typography helpers explicitly via `.into_element(cx)` where a concrete `AnyElement` seam is
  still required.
- `fret-ui-shadcn::prelude::*` now re-exports `IntoUiElement`, so direct-crate first-party
  shadcn examples do not need ad-hoc trait imports just to land typed helpers such as
  `shadcn::raw::typography::*`.
- typography remains decoupled from the model-heavy constructor lane:
  this sweep does not mix with `checkbox`, `progress`, or `switch` refactors.
- selected advanced/manual-assembly examples now also keep reusable helpers off raw landed return
  types by default:
  `apps/fret-examples/src/assets_demo.rs` (`render_image_panel`, `render_svg_panel`),
  `apps/fret-examples/src/async_playground_demo.rs`
  (`header_bar`, `body`, `catalog_panel`, `main_panel`, `inspector_panel`,
  `policy_editor`, `query_panel_for_mode`, `query_inputs_row`, `query_result_view`,
  `status_badge`),
  `apps/fret-examples/src/custom_effect_v1_demo.rs`
  (`stage`, `lens_row`, `lens_shell`, `plain_lens`, `custom_effect_lens`, `inspector`),
  `apps/fret-examples/src/custom_effect_v2_demo.rs`
  (`stage`, `lens_row`, `lens_shell`, `plain_lens`, `custom_effect_lens`, `inspector`),
  `apps/fret-examples/src/custom_effect_v3_demo.rs`
  (`stage`, `stage_controls`, `animated_backdrop`, `lens_row`, `lens_shell`),
  `apps/fret-examples/src/postprocess_theme_demo.rs`
  (`inspector`, `stage`, `stage_body`, `stage_cards`),
  `apps/fret-examples/src/drop_shadow_demo.rs` (`card<H>(...)`),
  `apps/fret-examples/src/markdown_demo.rs` (`render_image_placeholder<H>(...)`),
  `apps/fret-examples/src/liquid_glass_demo.rs` (`lens_panel<H>(...)`),
  `apps/fret-examples/src/custom_effect_v2_identity_web_demo.rs`
  (`lens`, `inspector`),
  `apps/fret-examples/src/custom_effect_v2_web_demo.rs`
  (`lens`, `inspector`),
  `apps/fret-examples/src/custom_effect_v2_lut_web_demo.rs`
  (`lens`, `inspector`),
  and `apps/fret-examples/src/custom_effect_v2_glass_chrome_web_demo.rs`
  (`label_row`, `lens`, `controls_panel`),
  `apps/fret-cookbook/examples/customv1_basics.rs`
  (`panel_shell(...)`, `preview_content(...)`),
  `apps/fret-cookbook/examples/drop_shadow_basics.rs` (`shadow_card(...)`),
  and `apps/fret-cookbook/examples/icons_and_assets_basics.rs`
  (`render_image_preview(...)`)
  now return `impl IntoUiElement<...>`.
- `apps/fret-examples/src/hello_world_compare_demo.rs` now keeps its local `swatch(...)` closure
  off raw landed returns by default; it lands explicitly only where the surrounding child array
  wants raw elements.
- `apps/fret-cookbook/examples/chart_interactions_basics.rs::chart_canvas(...)` is now treated as
  an intentional raw retained seam rather than migration debt: it owns
  `RetainedSubtreeProps::new::<KernelApp>(...)` and `cached_subtree_with(...)` landing.
- selected default-app WebGPU examples now also keep reusable helpers off raw landed returns by
  default:
  `custom_effect_v2_identity_web_demo`, `custom_effect_v2_web_demo`,
  `custom_effect_v2_lut_web_demo`, and `custom_effect_v2_glass_chrome_web_demo`
  now use `impl IntoUiElement<fret_app::App> + use<>` for non-raw helper composition, with
  explicit `.into_element(cx)` reserved for stage child arrays, overlay child collections, and
  other concrete raw landing seams.
- selected UI Gallery AI doc pages now keep page-local helpers on the default app-facing child
  surface:
  `ai_persona_demo.rs`, `ai_commit_demo.rs`, `ai_context_demo.rs`,
  `ai_model_selector_demo.rs`, `ai_voice_selector_demo.rs`, `ai_mic_selector_demo.rs`,
  `ai_checkpoint_demo.rs`, `ai_shimmer_demo.rs`, `ai_test_results_demo.rs`,
  `ai_artifact_demo.rs`, and `ai_chain_of_thought_demo.rs`
  now return `impl UiChild + use<>` for page-local notes/table helpers, with explicit
  `.into_element(cx)` seams only where `DocSection::new(...)` still intentionally consumes
  `AnyElement`.
- selected UI Gallery badge snippets now also keep their local layout helper off raw landed
  returns by default:
  `src/ui/snippets/badge/{demo,spinner,rtl,counts,colors,icon,variants}.rs`
  now use `fn row<H: UiHost, F>(children: F) -> impl IntoUiElement<H> + use<H, F>`,
  while the snippet `render(...) -> AnyElement` entrypoint remains unchanged.
- selected UI Gallery context-menu snippets now also keep local trigger helpers off raw landed
  returns by default:
  `src/ui/snippets/context_menu/{basic,radio,checkboxes,groups,icons,shortcuts,destructive,demo,rtl,submenu}.rs`
  now use `trigger_surface(...) -> impl IntoUiElement<H> + use<...>`,
  with explicit `.into_element(cx)` only at the menu trigger seam.
- selected UI Gallery combobox snippets now also keep local state display helpers off raw landed
  returns by default:
  `src/ui/snippets/combobox/{long_list,input_group,trigger_button,groups_with_separator,groups,disabled,custom_items,clear_button,invalid}.rs`
  now use `state_row(...)` and `state_rows(...) -> impl IntoUiElement<fret_app::App> + use<>`,
  with explicit `.into_element(cx)` only at sibling child-collection and render-boundary seams.
- selected UI Gallery pagination snippets now also keep local page label helpers off raw landed
  returns by default:
  `src/ui/snippets/pagination/{simple,usage}.rs`
  now use `page_number(...) -> impl IntoUiElement<H> + use<H>`,
  with explicit `.into_element(cx)` only at the `PaginationLink::new([..])` seam.
- selected UI Gallery carousel snippets now also keep local slide helpers off raw landed returns
  by default:
  `src/ui/snippets/carousel/{basic,sizes,plugin_wheel_gestures,spacing_responsive,loop_carousel,options,loop_downgrade_cannot_loop,spacing,usage,sizes_thirds,parts,api,duration_embla,rtl,plugin_autoplay,plugin_autoplay_delays,plugin_autoplay_controlled,plugin_autoplay_stop_on_focus,plugin_autoplay_stop_on_last_snap,events}.rs`
  now keep `slide_card(...) -> impl IntoUiElement<fret_app::App> + use<>` wherever a card helper
  exists and `slide(...) -> impl IntoUiElement<fret_app::App> + use<>` wherever a slide wrapper
  exists, with explicit `.into_element(cx)` only at `ui::container(...)`,
  `CarouselItem::new(...)`, and final child-collection seams.
- selected UI Gallery skeleton snippets now also keep local shape/row helpers off raw landed
  returns by default:
  `src/ui/snippets/skeleton/{avatar,rtl,form,table}.rs`
  now use `round(...)` / `row(...) -> impl IntoUiElement<H> + use<H>`,
  with explicit `.into_element(cx)` only at the stack/row collection seam.
- selected UI Gallery popover snippets now also keep layout-wrapper helpers off raw landed returns
  by default:
  `src/ui/snippets/popover/{basic,demo,with_form}.rs`
  now uses `centered<H, B>(body: B) -> impl IntoUiElement<H> + use<H, B>`
  where `B: IntoUiElement<H>`,
  so the wrapper no longer forces callers to pre-land `AnyElement`.
- selected UI Gallery dropdown-menu snippets now also keep preview wrappers on the unified
  conversion contract:
  `src/ui/snippets/dropdown_menu/mod.rs`
  now uses `preview_frame<H, B>(body: B) -> impl IntoUiElement<H> + use<H, B>` and
  `preview_frame_with<H, F, B>(...) -> impl IntoUiElement<H> + use<H, F, B>`,
  and child snippets land them explicitly with `.into_element(cx)` at the render boundary.
- selected UI Gallery AI snippet wrappers and doc-preview helpers now also keep local layout and
  section helpers off raw landed returns by default:
  `src/ui/snippets/ai/{context_default,context_demo}.rs`
  now use `centered<H, B>(body: B) -> impl IntoUiElement<H> + use<H, B>` where
  `B: IntoUiElement<H>`;
  `src/ui/snippets/ai/{file_tree_basic,file_tree_expanded}.rs`
  now use `preview(...) -> impl IntoUiElement<H> + use<H>`;
  `src/ui/snippets/ai/file_tree_large.rs`
  now uses `preview(...)` and `invisible_marker(...) -> impl IntoUiElement<H> + use<H>`;
  `src/ui/snippets/ai/test_results_demo.rs`
  now uses `progress_section(...) -> impl IntoUiElement<H> + use<H>`;
  `src/ui/snippets/ai/attachments_usage.rs`
  now uses `render_grid_attachment(...) -> impl IntoUiElement<H> + use<H>`;
  `src/ui/snippets/ai/{attachments_grid,attachments_list}.rs`
  now use `render_grid_attachment(...)` / `render_list_attachment(...) -> impl IntoUiElement<H> + use<H>`;
  `src/ui/snippets/ai/file_tree_demo.rs`
  now uses `invisible_marker(...) -> impl IntoUiElement<H> + use<H>`;
  `src/ui/snippets/ai/speech_input_demo.rs`
  now uses `body_text(...)` and `clear_action(...) -> impl IntoUiElement<H> + use<H>`;
  `src/ui/snippets/breadcrumb/dropdown.rs`
  now uses `dot_separator(...) -> impl IntoUiElement<H> + use<H>`,
  with explicit `.into_element(cx)` only at doc-section, child-array, and breadcrumb-list seams.
- internal UI Gallery wrapper shells now also keep raw landing local instead of forcing pre-landed
  `AnyElement` inputs:
  `src/ui/doc_layout.rs`
  now uses `demo_shell<B>(...) -> impl IntoUiElement<fret_app::App> + use<B>` and lands the body
  at the shell boundary;
  `src/ui/previews/pages/editors/code_editor/mvp/gates.rs`
  now uses `gate_panel<B>(...) -> impl IntoUiElement<fret_app::App> + use<B>` and lands the
  editor child only at the preview-panel boundary.
- selected UI Gallery avatar snippets now also keep row wrappers, avatar builders, and group/icon
  helpers off raw landed returns by default:
  `src/ui/snippets/avatar/{demo,group,with_badge,fallback_only,sizes,group_count,group_count_icon,badge_icon,dropdown}.rs`
  now use `wrap_row(...)`, `avatar_with_image(...)`, `avatar_with_badge(...)`,
  `avatar_fallback_only(...)`, `group(...)`, `group_with_count(...)`, and `icon(...)`
  as `impl IntoUiElement<H> + use<...>` helpers,
  with explicit `.into_element(cx)` only at `AvatarGroup::new(...)`, `children([..])`, and final
  render-boundary seams.
- selected UI Gallery button snippets now also keep row wrappers and local size-composition helpers
  off raw landed returns by default:
  `src/ui/snippets/button/{demo,size,with_icon,link_render,rtl,loading,variants,button_group,rounded}.rs`
  now use `wrap_row(...) -> impl IntoUiElement<H> + use<H, F>`,
  and `src/ui/snippets/button/size.rs`
  now also keeps `row(...) -> impl IntoUiElement<H> + use<H>`,
  with explicit `.into_element(cx)` only at vector collection and final render-boundary seams.
- selected UI Gallery tabs snippets now also keep local form-field helpers off raw landed returns
  by default:
  `src/ui/snippets/tabs/demo.rs`
  now uses `field(...) -> impl IntoUiElement<fret_app::App> + use<>`,
  with explicit `.into_element(cx)` only at `CardContent::new(...)` child vectors and
  `TabsItem::new(...)` seams.
- selected UI Gallery collapsible snippets now also keep icon/field/tree helpers off raw landed
  returns by default:
  `src/ui/snippets/collapsible/{basic,settings_panel,rtl,file_tree}.rs`
  now use `rotated_lucide(...)`, `radius_input(...)`, `details_collapsible(...)`,
  `file_leaf(...)`, and `folder(...) -> impl IntoUiElement<H> + use<H>`,
  with explicit `.into_element(cx)` only at child arrays, `with_direction_provider(...)`, and
  `Collapsible::into_element_with_open_model(...)` seams.
- selected UI Gallery hover-card and tooltip snippets now also keep local overlay/content helpers
  off raw landed returns by default:
  `src/ui/snippets/hover_card/{sides,trigger_delays}.rs`
  now use `card(...)` and `demo_content(...) -> impl IntoUiElement<H> + use<H>`;
  `src/ui/snippets/tooltip/{rtl,sides}.rs`
  now use `make_tooltip(...) -> impl IntoUiElement<H> + use<H>`, and
  `src/ui/snippets/tooltip/rtl.rs`
  now also keeps `make_tooltip_with_test_ids(...) -> impl IntoUiElement<H> + use<H>`,
  with explicit `.into_element(cx)` only at row child-collection seams and the final
  render/provider boundary.
- selected UI Gallery resizable snippets now also keep panel/container wrapper helpers off raw
  landed returns by default:
  `src/ui/snippets/resizable/usage.rs`
  now uses `panel(...) -> impl IntoUiElement<H> + use<H>`;
  `src/ui/snippets/resizable/{vertical,handle}.rs`
  now use `box_group<H, B>(..., body: B) -> impl IntoUiElement<H> + use<H, B>` where
  `B: IntoUiElement<H>`,
  and `panel(...) -> impl IntoUiElement<H> + use<H>`,
  with explicit `.into_element(cx)` only at `ResizablePanel::new([..])` entry seams and the final
  render boundary.
- selected UI Gallery data-table and table-action snippets now also keep alignment/footer/row
  helpers off raw landed returns by default:
  `src/ui/snippets/data_table/{basic_demo,default_demo,guide_demo}.rs`
  now use `align_end<B>(child: B) -> impl IntoUiElement<fret_app::App> + use<B>` where
  `B: IntoUiElement<fret_app::App>`;
  `src/ui/snippets/data_table/default_demo.rs`
  now also keeps `footer(...) -> impl IntoUiElement<fret_app::App> + use<>`;
  `src/ui/snippets/data_table/{basic_demo,rtl_demo}.rs`
  now keep `bottom_controls(...) -> impl IntoUiElement<fret_app::App> + use<>`;
  `src/ui/snippets/data_table/rtl_demo.rs`
  now uses `align_inline_start<B>(cx, child) -> impl IntoUiElement<fret_app::App> + use<B>`;
  `src/ui/snippets/table/actions.rs`
  now uses `align_end<B>(child: B) -> impl IntoUiElement<fret_app::App> + use<B>` and
  `action_row(...) -> impl IntoUiElement<fret_app::App> + use<>`,
  with explicit `.into_element(cx)` only at data-table cell/table-row seams and final render
  boundaries.
- selected UI Gallery button-group, toggle-group, and drawer snippets now also expose reusable
  helpers as `IntoUiElement`-based signatures:
  `src/ui/snippets/button_group/api_reference.rs`
  now exports `basic_button_group(...)`, `button_group_with_separator(...)`, and
  `button_group_with_text(...) -> impl IntoUiElement<H> + use<H>`;
  `src/ui/snippets/toggle_group/size.rs`
  now uses `group(...) -> impl IntoUiElement<H> + use<H>`;
  `src/ui/snippets/drawer/demo.rs`
  now uses `goal_adjust_button(...)` and `goal_chart(...) -> impl IntoUiElement<H> + use<H>`;
  `src/ui/snippets/drawer/{responsive_dialog,sides,scrollable_content}.rs`
  now use `profile_field(...)`, `profile_form(...)`, `side_button(...)`, and
  `paragraph_block(...) -> impl IntoUiElement<H> + use<H>`,
  with explicit `.into_element(cx)` only at `DialogContent::new(...)`,
  `DrawerContent::new(...)`, child vectors, and scroll-area/content seams.
- selected UI Gallery sheet and dialog snippets now also keep shared form helpers off raw landed
  returns by default:
  `src/ui/snippets/sheet/{demo,rtl}.rs` and `src/ui/snippets/dialog/{demo,rtl}.rs`
  now use `profile_fields(...) -> impl IntoUiElement<H> + use<H>`,
  with explicit `.into_element(cx)` only at `SheetContent::new(...)`,
  `DialogContent::new(...)`, and intermediate container seams.
- selected UI Gallery dialog scroll-content snippets now also keep paragraph/content helpers off
  raw landed returns by default:
  `src/ui/snippets/dialog/{scrollable_content,sticky_footer}.rs`
  now use `lorem_block(...) -> impl IntoUiElement<H> + use<H>`,
  with explicit `.into_element(cx)` only at `ScrollArea::new([..])`,
  `DialogContent::new(...)`, and final dialog-content seams.
- selected UI Gallery separator snippets now also keep local section/row helpers off raw landed
  returns by default:
  `src/ui/snippets/separator/{menu,list}.rs`
  now use `section(...)` and `row(...) -> impl IntoUiElement<H> + use<H>`,
  with explicit `.into_element(cx)` only at sibling child-collection seams.
- selected UI Gallery table snippets now also keep shared table wrappers off raw landed returns by
  default:
  `src/ui/snippets/table/{demo,footer,rtl}.rs`
  now use `make_invoice_table(...) -> impl IntoUiElement<fret_app::App> + use<>`,
  with the explicit raw landing kept inside the helper at
  `Table::build(...).into_element(cx).test_id(test_id)` because the table builder still needs an
  internal semantics-decoration seam.
- selected UI Gallery sidebar snippets now also keep menu-entry helpers off raw landed returns by
  default:
  `src/ui/snippets/sidebar/{demo,controlled,mobile}.rs`
  now use `menu_button(...) -> impl IntoUiElement<H> + use<H>`;
  `src/ui/snippets/sidebar/rtl.rs`
  now uses `menu_button(...) -> impl IntoUiElement<fret_app::App> + use<>`,
  with explicit `.into_element(cx)` only at `SidebarMenuItem::new(...)` seams.
- selected UI Gallery aspect-ratio snippets now also keep image/frame helpers off raw landed
  returns by default:
  `src/ui/snippets/aspect_ratio/{portrait,square,rtl}.rs`
  now use `portrait_image(...)`, `square_image(...)`, `rtl_image(...)`, and
  `ratio_example(...) -> impl IntoUiElement<H> + use<H>`,
  with explicit `.into_element(cx)` only at `AspectRatio::with_child(...)` and render-boundary
  seams.
- selected UI Gallery item, toast, and motion-presets snippets now also keep local helpers off raw
  landed returns by default:
  `src/ui/snippets/item/{avatar,icon,link,link_render,dropdown,extras_rtl,gallery}.rs`
  now keep `icon_button(...)`, `item_team(...)`, `item_icon(...)`, `icon(...)`,
  `outline_button(...)`, `outline_button_sm(...)`, `item_basic(...)`, and `item_avatar(...)`
  helpers on `impl IntoUiElement<fret_app::App> + use<>`, with explicit `.into_element(cx)` only
  at `ItemMedia::new(...)`, `ItemActions::new(...)`, dropdown child arrays, vector collection,
  and the final render-boundary seams;
  `src/ui/snippets/toast/deprecated.rs`
  now uses `centered<B>(body: B) -> impl IntoUiElement<fret_app::App> + use<B>`;
  `src/ui/snippets/motion_presets/fluid_tabs_demo.rs`
  now uses `panel(...) -> impl IntoUiElement<fret_app::App> + use<>`.
- explicit raw seams remain where the helper is genuinely low-level composition glue, for example
  the internal body-landing step inside `custom_effect_v1_demo.rs::lens_shell(...)` and
  `custom_effect_v2_demo.rs::lens_shell(...)`, plus stage child arrays and retained-subtree
  bridges that intentionally still own raw `AnyElement` assembly.
- heterogenous sibling arrays remain valid explicit landing seams even after helper migration; for
  example `custom_effect_v3_demo.rs::{stage, stage_controls}` and
  `postprocess_theme_demo.rs::render(...)` now keep helpers on `IntoUiElement<KernelApp>` but
  still call `.into_element(cx)` where the surrounding child collection intentionally wants
  `AnyElement`.
- `async_playground_demo.rs` now follows the same rule:
  local helpers stay on `IntoUiElement<KernelApp>`, while
  `render(...)`, `body(...)`, `main_panel(...)`, `inspector_panel(...)`, and
  `query_panel_for_mode(...)` still land explicitly at heterogenous child arrays,
  `TabsItem::new([..])`, and `ScrollArea::new([..])` seams.

Validation note on 2026-03-12:

- verified the expanded UI Gallery helper gate with
  `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app selected_`;
  the focused source gate now covers 32 `selected_*` checks and passed after the AI wrapper,
  breadcrumb, avatar, button, tabs, collapsible, drawer, sheet, dialog (including scrollable and
  sticky-footer content helpers), separator, table, sidebar, aspect-ratio, popover, hover-card,
  tooltip, resizable, data-table, table-action, combobox state-row/state-rows, item
  (`extras_rtl`, `dropdown`, `gallery`), and carousel (`basic`/`usage` plus `api`,
  `duration_embla`, `rtl`, `plugin_autoplay*`, and `events`) helper migrations landed.
- verified the advanced example helper gate with
  `cargo nextest run -p fret-examples --lib`;
  the source gate now also records `custom_effect_v3_demo.rs::{stage, stage_controls,
  animated_backdrop, lens_row, lens_shell}` and `postprocess_theme_demo.rs::{inspector, stage,
  stage_body, stage_cards}` plus
  `async_playground_demo.rs::{header_bar, body, catalog_panel, main_panel, inspector_panel,
  policy_editor, query_panel_for_mode, query_inputs_row, query_result_view, status_badge}`
  on `IntoUiElement<KernelApp>`-based signatures, with explicit `.into_element(cx)` kept only at
  heterogenous sibling child-collection, tabs-item, and scroll-area seams.

## M4 — Delete the old public surface

- [x] Remove `UiIntoElement` from curated public surfaces.
- [x] Remove `UiHostBoundIntoElement` from curated public surfaces.
- [x] Remove `UiChildIntoElement` from curated public surfaces.
- [x] Remove `UiBuilderHostBoundIntoElementExt` from curated public surfaces.
- [x] Rewrite or delete stale docs that still teach the old names.

## M5 — Add guardrails

- [x] Add a gate that the app prelude does not publicly re-export old conversion traits.
- [x] Add a gate that the component prelude exports exactly one public conversion trait.
- [x] Add a source/doc gate that the canonical authoring compare set (`simple_todo_v2_target`,
  `todo_demo`, and the simple-todo scaffold template) stays on the target conversion vocabulary.
- [x] Add a source/doc gate that app-facing examples prefer `Ui` / `UiChild`.
- [x] Add a source/doc gate that generic reusable first-party helpers prefer the unified
  conversion trait over raw `AnyElement` when a raw landed element is not required.
- [x] Add a source gate that old names (`UiChildIntoElement`, `UiHostBoundIntoElement`,
  `UiBuilderHostBoundIntoElementExt`) do not return in curated surfaces.

Implementation note on 2026-03-12:

- the canonical compare set now has direct stale-name guards in:
  `apps/fret-cookbook/src/lib.rs`,
  `apps/fret-examples/src/lib.rs`,
  and `apps/fretboard/src/scaffold/templates.rs`.
- `ecosystem/fret-ui-shadcn/src/surface_policy_tests.rs` now guards that
  `ui_ext/support.rs` and `ui_ext/data.rs` stay on `IntoUiElement<H>` rather than
  reintroducing direct `UiIntoElement` glue.
- `ecosystem/fret-ui-shadcn/src/surface_policy_tests.rs` now also guards that
  `ui_builder_ext/*` reusable helper closures keep accepting `IntoUiElement<H>`.
- `ecosystem/fret-ui-shadcn/src/surface_policy_tests.rs` now also guards that
  `ui_builder_ext/*::into_element(...)` remains an explicit `AnyElement` landing seam while its
  closure inputs do not regress to `AnyElement`-typed signatures.
- `ecosystem/fret/tests/reusable_component_helper_surface.rs` now guards the facade-level
  source/doc story: shadcn reusable helpers stay on `IntoUiElement<H>` rather than requiring
  pre-landed `AnyElement` inputs.
- `docs/component-author-guide.md` and `docs/component-authoring-contracts.md` now teach only
  `IntoUiElement<H>` on the curated component-authoring lane.
- `ecosystem/fret-ui-kit/tests/curated_conversion_surface_docs.rs` now guards curated docs against
  legacy conversion trait names.
- semantic decorator helper names are now neutralized to `UiElement*Ext`, and
  `ecosystem/fret-ui-kit::source_policy_tests` guards against reintroducing the old
  `UiIntoElement*Ext` export names on declarative surfaces.
- future ergonomics follow-up: if explicit `.into_element(cx).attach_semantics(...)` starts
  showing up frequently in app/ecosystem call sites, add one unified semantics decorator helper on
  top of the public `IntoUiElement<H>` contract rather than teaching per-component builder-specific
  `.a11y(...)` APIs.
- `apps/fret-ui-gallery/tests/ui_authoring_surface_default_app.rs` now guards that selected
  `badge/*` snippet helpers stay on `IntoUiElement<H>` rather than reverting to raw `AnyElement`
  row helpers.
- `apps/fret-ui-gallery/tests/ui_authoring_surface_default_app.rs` now also guards that selected
  `context_menu/*` snippet trigger helpers stay on `IntoUiElement<H>` rather than reverting to raw
  `AnyElement`.
- `apps/fret-ui-gallery/tests/ui_authoring_surface_default_app.rs` now also guards that selected
  `combobox/*` snippet `state_row(...)` and `state_rows(...)` helpers stay on
  `IntoUiElement<fret_app::App>` rather than reverting to raw `AnyElement`.
- `apps/fret-ui-gallery/tests/ui_authoring_surface_default_app.rs` now also guards that selected
  `pagination/*` snippet page-number helpers stay on `IntoUiElement<H>` rather than reverting to
  raw `AnyElement`.
- `apps/fret-ui-gallery/tests/ui_authoring_surface_default_app.rs` now also guards that selected
  `carousel/*` snippet slide helpers stay on `IntoUiElement<fret_app::App>` rather than
  reverting to raw `AnyElement`.
- `apps/fret-ui-gallery/tests/ui_authoring_surface_default_app.rs` now also guards that selected
  `skeleton/*` snippet shape/row helpers stay on `IntoUiElement<H>` rather than reverting to raw
  `AnyElement`.
- `apps/fret-ui-gallery/tests/ui_authoring_surface_default_app.rs` now also guards that selected
  `popover/*` wrapper helpers accept/return `IntoUiElement<H>` rather than forcing pre-landed
  `AnyElement`.
- `apps/fret-ui-gallery/tests/ui_authoring_surface_default_app.rs` now also guards that
  `dropdown_menu/mod.rs` preview wrappers stay on `IntoUiElement<H>` rather than forcing
  pre-landed `AnyElement`.
- `apps/fret-ui-gallery/tests/ui_authoring_surface_default_app.rs` now also guards that selected
  `button_group/*`, `toggle_group/*`, and `drawer/*` reusable helpers stay on `IntoUiElement`
  signatures rather than reverting to raw `AnyElement`.
- `apps/fret-ui-gallery/tests/ui_authoring_surface_default_app.rs` now also guards that selected
  `hover_card/*` content helpers and `tooltip/*` helper builders stay on `IntoUiElement<H>`
  signatures rather than reverting to raw `AnyElement`.
- `apps/fret-ui-gallery/tests/ui_authoring_surface_default_app.rs` now also guards that selected
  `resizable/*` panel/container wrapper helpers stay on `IntoUiElement<H>`-based signatures
  rather than reverting to raw `AnyElement`.
- `apps/fret-ui-gallery/tests/ui_authoring_surface_default_app.rs` now also guards that selected
  `data_table/*` alignment/footer helpers and `table/actions.rs` row helpers stay on
  `IntoUiElement<fret_app::App>` rather than reverting to raw `AnyElement`.
- `apps/fret-ui-gallery/tests/ui_authoring_surface_default_app.rs` now also guards that selected
  `item/*`, `toast/*`, and `motion_presets/*` helpers stay on `IntoUiElement` signatures rather
  than reverting to raw `AnyElement`.

## M6 — Keep advanced/raw seams explicit and justified

- [ ] Document the legitimate raw `AnyElement` cases:
  - [ ] overlay/controller internals,
  - [ ] diagnostics/harness helpers,
  - [ ] low-level heterogeneous landing APIs,
  - [ ] manual assembly / advanced runtime seams.
- [ ] Ensure raw surfaces remain explicit rather than leaking back into the app-facing story.
