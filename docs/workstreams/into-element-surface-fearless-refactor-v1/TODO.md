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
- already-landed raw values (`AnyElement`) now feed that public surface directly via
  `IntoUiElement<H>`; there is no remaining `UiIntoElement` bridge in production code.
- `UiBuilderHostBoundIntoElementExt` has now been deleted from the codebase; `UiBuilder<T>`
  lands through `IntoUiElement<H>` directly.
- the legacy `UiIntoElement` name is now deleted from code; `fret_ui_kit::ui_builder` keeps only
  the public `IntoUiElement<H>` contract plus a direct raw `AnyElement` implementation.

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
  `input_otp.rs::input_otp<H>(...)`, `command.rs::command<H, I, F, T>(...)`,
  `checkbox.rs::{checkbox<H>(...), checkbox_opt<H>(...)}`,
  `progress.rs::progress<H>(...)`,
  `switch.rs::{switch<H>(...), switch_opt<H>(...)}`, and
  `card.rs::{card<H, I, F, T>(...), card_sized<H, I, F, T>(...), card_header<H, I, F, T>(...), card_action<H, I, F, T>(...), card_title<H, T>(...), card_description<H, T>(...), card_description_children<H, I, F, T>(...), card_content<H, I, F, T>(...), card_footer<H, I, F, T>(...)}`
  now expose typed constructor/wrapper outputs, while
  `kbd.rs::kbd_icon<H>(...)` and `combobox.rs::use_combobox_anchor(...)` remain explicit raw
  helper seams because `Kbd::from_children(...)` and `PopoverAnchor::new(...)` still own concrete
  landed-child storage.
  The card lane now also has builder support where the wrapper still needs late child collection:
  `CardAction::build(...)` and `CardDescription::build(...)`.
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
- first-party direct-crate shadcn teaching is now starting to normalize the eager child-list path
  as well: where a component still intentionally owns `new(children: Vec<AnyElement>)`, Gallery
  snippets should prefer `ui::children![cx; ...]` over ad-hoc `vec![...into_element(cx)]`
  assembly so typed helpers can still read like component values at the call site.
- that eager child-list guidance now also covers first-party card wrappers:
  `apps/fret-ui-gallery/src/ui/snippets/card/{usage,demo,rtl}.rs`
  now teach `shadcn::card(|cx| ui::children![cx; ...])` and
  `shadcn::card_header(|cx| ui::children![cx; ...])`
  instead of the old `card(cx, ...)` / `card_header(cx, ...)` eager-landing pattern.
- that eager child-list guidance now also covers the first-party modal/form exemplars:
  selected `dialog`, `sheet`, and `drawer` snippets now build `Field`, `FieldSet`,
  `DialogContent`, `DialogHeader`, `DialogFooter`, `SheetContent`, `SheetHeader`,
  `SheetFooter`, `DrawerContent`, `DrawerHeader`, and `DrawerFooter` children through
  `ui::children![cx; ...]` instead of manually pre-landing every child.
- the same child-list normalization now also covers selected `popover` form snippets:
  `PopoverContent`, `PopoverHeader`, `FieldGroup`, and `Field` now teach
  `ui::children![cx; ...]` rather than manual `AnyElement` assembly on first-party examples.
- helper ergonomics are also tightening around render-local `cx` ownership:
  where a helper can remain fully late-landed, Gallery examples should drop the `cx` parameter
  instead of carrying it only to attach `.into_element(cx)` / `test_id(...)` at helper scope.
- `fret-ui-shadcn::prelude::*` now re-exports `IntoUiElement`, so direct-crate first-party
  shadcn examples do not need ad-hoc trait imports just to land typed helpers such as
  `shadcn::raw::typography::*`.
- Verification update on 2026-03-13:
  the remaining thin helper family (`checkbox`, `progress`, `switch`) now also stays on typed
  `IntoUiElement<H>` outputs all the way through the current first-party teaching lane, with
  `fret-ui-shadcn` crate docs calling out the rule explicitly, `fret_ui_shadcn::prelude::*`
  also re-exporting `UiElementTestIdExt` / `UiElementA11yExt` / `UiElementKeyContextExt`, and the
  affected UI Gallery snippets (`tooltip`, `input_group`, `kbd`, `label`, `typography`, selected
  AI attachment hover-card examples) no longer re-teach eager helper landing just to attach
  decoration or diagnostics hooks. Validation for the production surface: `cargo check -p
  fret-ui-shadcn --lib`.
- first-party docs/tests no longer need to mention the old scaffold name for ordinary authoring:
  `docs/first-hour.md` now teaches `IntoUiElement<H>`, and `fret-ui-ai` builder smoke tests
  (`elements/message.rs`, `elements/workflow/panel.rs`) now assert against the public
  `IntoUiElement<fret_app::App>` contract instead of `UiIntoElement`.
- exported `fret-ui-kit` adapter macros now attach `IntoUiElement<H>` directly, and the legacy
  `UiIntoElement` production bridge is now deleted from `ui_builder.rs`; the old name survives
  only in source-policy assertions and historical workstream docs.
- the RenderOnce helper macro is now also renamed onto the public vocabulary:
  component-authoring docs should teach `fret_ui_kit::ui_component_render_once!(Ty)` rather than
  the old `ui_into_element_render_once!` name.
- declarative semantics decorators now also sit on the public landing trait:
  `UiElementTestIdExt`, `UiElementA11yExt`, and `UiElementKeyContextExt` wrappers land through
  `IntoUiElement<H>` directly, so `declarative/semantics.rs` no longer depends on
  `UiIntoElement` outside tests.
- built-in text primitives now also bypass the hidden bridge:
  `ui::TextBox` and `ui::RawTextBox` implement `IntoUiElement<H>` directly, so the remaining
  `UiIntoElement` production usage is now concentrated in `ui_builder.rs`.
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

Update on 2026-03-13:

- the first-party wrapper return-shape rule is now explicit:
  when a helper should preserve fluent builder affordances before the explicit landing seam,
  prefer returning the concrete builder/component type rather than an opaque
  `impl IntoUiElement<H>`.
- this rule is now landed across the `card`, `alert`, `table`, and `field` families:
  `card(...)` / `card_header(...)` / `card_action(...)` / `card_content(...)` / `card_footer(...)`
  return concrete builder or component types, `alert(...)` now returns `AlertBuild<H, ...>`
  while `AlertAction::build(...)` stays on the builder-first typed lane, and the new
  `table(...)` / `table_header(...)` / `table_body(...)` / `table_footer(...)` /
  `table_row(...)` / `table_head(...)` / `table_cell(...)` / `table_caption(...)` helpers now
  cover the first-party table authoring lane without forcing `Table::build(...)` /
  `TableRow::build(...)` boilerplate at the call site, while `field_set(...)` /
  `field_group(...)` now remove the remaining raw/eager wrapper seam around grouped form authoring.
- the same typed-wrapper rule now also covers the `empty` family:
  `empty(...)` / `empty_header(...)` / `empty_media(...)` / `empty_title(...)` /
  `empty_description(...)` / `empty_content(...)` now keep the default empty-state slot surface on
  the typed lane, while `EmptyMedia::variant(...)` remains available on the builder/component
  surface when call sites still need to tweak media chrome before the explicit landing seam.
- page-level/default-teaching cleanup is now an explicit sub-lane of M3:
  once a family is promoted onto a typed wrapper path, first-party Gallery page prose/code blocks
  should stop teaching the old constructor/builder surface as the default authoring path.

Update on 2026-03-13 (page/docs teaching drift cleanup):

- selected AI Gallery doc pages now standardize their small reference tables on
  `doc_layout::text_table(...)` instead of open-coding `Table::build(...)` /
  `TableHeader::build(...)` / `TableRow::build(...)` / `TableCell::build(...)` boilerplate.
- the `field` page usage/code block now teaches `shadcn::field_set(|cx| { ... })` and
  `shadcn::field_group(|cx| { ... })` as the default grouped authoring path while keeping
  `Field::new([...])` as the low-level single-field root.
- lower-traffic first-party snippets are now being swept under the same rule:
  `src/ui/snippets/pagination/extras.rs` teaches the pagination wrapper family instead of
  `Pagination::new(...)` / `PaginationContent::new(...)`, while
  `src/ui/snippets/checkbox/table.rs` and `src/ui/snippets/typography/table.rs` now teach the
  table wrapper family instead of raw `Table::build(...)`.
- the first-party alert teaching lane is now also tightened for already-promoted wrappers:
  `src/ui/snippets/alert/{demo,interactive_links,custom_colors,rich_title}.rs` and
  `src/ui/snippets/motion_presets/fluid_tabs_demo.rs` now teach
  `shadcn::alert(|cx| ui::children![cx; ...])` instead of `Alert::new(...)`, and
  `selected_alert_snippets_prefer_alert_wrapper_family` now locks that source policy.
- the same wrapper-family cleanup now also covers grouped-field control snippets beyond the
  dedicated field/form pages:
  `combobox/label.rs`, `input/{form,field_group}.rs`, `toggle_group/label.rs`,
  `date_picker/{label,time_picker}.rs`, `native_select/label.rs`, `select/label.rs`,
  `select/align_item_with_trigger.rs`, `checkbox/with_title.rs`, `slider/label.rs`,
  `switch/{label,choice_card}.rs`, and `radio_group/{label,fieldset,invalid,extras}.rs` now teach
  `field_group(...)` / `field_set(...)` instead of `FieldGroup::new(...)` / `FieldSet::new(...)`.
- the `card` page notes now also present `card(...)` as the default first-party teaching path,
  with `Card::build(...)` explicitly framed as a lower-level option rather than a default-equal
  recommendation.
- selected first-party card snippets now also follow the wrapper-family lane end-to-end:
  `card/{size,card_content,image,compositions}.rs` now teach `card(...)` plus the slot helper
  family instead of `Card::new(...)` / `CardHeader::new(...)` / `CardContent::new(...)` /
  `CardFooter::new(...)`, while `card/{demo,rtl}.rs` now also teach `card_footer(...)` instead of
  `CardFooter::build(...)`; `compositions.rs::cell(...)` now accepts any `IntoUiElement<App>`
  card input so the example no longer needs a concrete `shadcn::Card` parameter to stay typed.
- the same card-family cleanup now also reaches a few non-card composition snippets that were still
  teaching `Card::*` as the outer container:
  `tabs/demo.rs`, `input_otp/form.rs`, `collapsible/basic.rs`, and
  `motion_presets/fluid_tabs_demo.rs` now teach `card(...)` plus the slot helper family instead of
  `Card::new(...)` / `CardHeader::new(...)` / `CardContent::new(...)` / `CardFooter::new(...)`.
- another low-risk composition sweep now lands on the same lane:
  `calendar/presets.rs`, `motion_presets/{overlay_demo,stagger_demo,stack_shift_list_demo}.rs`,
  and `accordion/card.rs` now also teach `card(...)` plus the slot helper family instead of
  `Card::new(...)` / `CardHeader::new(...)` / `CardContent::new(...)` / `CardFooter::new(...)`
  for their outer showcase shell.
- the card-shell cleanup now also reaches two ordinary non-card demo surfaces:
  `toast/deprecated.rs` now teaches its deprecation card through `card(...)` plus the slot helper
  family, and `chart/demo.rs` now teaches `chart_card(...)` / `trending_footer(...)` on typed
  `IntoUiElement<App>` helpers while using `card(...)` plus the slot helper family for the chart
  shell instead of eager `Card::*` constructors.
- `form/upstream_demo.rs` now joins the same card-shell lane for its lightweight inset rows:
  the `mobile_field` card plus the marketing/security email-notification cards now teach
  `card(...)` + `card_content(...)` instead of raw `Card::new(...)` / `CardContent::new(...)`
  while leaving the broader upstream form/state logic untouched.
- another low-risk sweep now lands on the same card-family teaching rule:
  `motion_presets/{preset_selector,token_snapshot}.rs`, `skeleton/card.rs`,
  `accordion/extras.rs`, `collapsible/settings_panel.rs`, and `card/meeting_notes.rs`
  now teach `card(...)` plus the slot helper family instead of eager `Card::*` constructors,
  with `meeting_notes.rs` also switching its header action to `card_action(...)`.
- the last remaining low-risk non-material/non-carousel card shell in this sweep is now gone too:
  `ai/speech_input_demo.rs` now teaches its transcript surface through
  `card(...)` + `card_content(...)` instead of `Card::new(...)` / `CardContent::new(...)`.
- the remaining ordinary `carousel` teaching drift is now closed too:
  `carousel/{basic,api,demo,duration_embla,events,expandable,focus_watch,loop_carousel,loop_downgrade_cannot_loop,options,orientation_vertical,parts,plugin_autoplay,plugin_autoplay_controlled,plugin_autoplay_delays,plugin_autoplay_stop_on_focus,plugin_autoplay_stop_on_last_snap,plugin_wheel_gestures,rtl,sizes,sizes_thirds,spacing,spacing_responsive,usage}.rs`
  now teach `card(...)` + `card_content(...)` instead of eager `Card::new(...)` /
  `CardContent::new(...)`, and the selected card-wrapper source gate now covers those carousel
  call sites too.
- the follow-up policy decision is now made explicitly:
  `sidebar/*` and `material3/*` should also teach the wrapper family for ordinary showcase shells,
  because those cards are still first-party teaching surfaces rather than low-level recipe
  implementation examples.
- with that rule applied, the remaining `sidebar/{usage,demo,rtl,use_sidebar,mobile,controlled}.rs`
  and `material3/{state_matrix,menu,text_field,autocomplete,touch_targets,snackbar,tooltip,list}.rs`
  card shells now also teach `card(...)` + `card_header(...)` + `card_content(...)` instead of
  eager `Card::*` constructors, and the source-policy gate now locks the stronger invariant that
  `src/ui/snippets/**` no longer reintroduces any `shadcn::Card::*` constructor family at all.
- a follow-up page/docs sweep now confirms the same policy at the page layer too:
  `src/ui/pages/**` no longer reintroduces `shadcn::Card::*` constructors either, while
  `pages/card.rs` intentionally keeps one note that frames `Card::build(...)` as a lower-level
  late-child-collection escape hatch rather than a default teaching path.
- focused source-policy gates now also cover this page-level teaching lane, so already-promoted
  families cannot silently regress back to legacy `FieldSet::new(...)` / `Table::build(...)`
  teaching on first-party doc pages.
- the same typed-wrapper rule now also covers the `pagination` family where the old surface still
  forced eager child landing:
  `pagination(...)` / `pagination_content(...)` / `pagination_item(...)` /
  `pagination_link(...)` now keep pagination root/content/item/link authoring on the typed lane,
  while `PaginationPrevious::new()`, `PaginationNext::new()`, and `PaginationEllipsis::new()`
  remain ordinary typed leaf constructors because they never required a raw child-list seam.
- the crate root/facade now also expose `alert(...)` and the table helper family, and the UI
  Gallery alert/table snippets now teach `shadcn::alert(|cx| ui::children![cx; ...])` and
  `shadcn::table(|cx| ui::children![cx; ...])` as the default first-party authoring pattern, and
  the selected field/form snippets now also teach `shadcn::field_set(|cx| ...)` /
  `shadcn::field_group(|cx| ...)` instead of `FieldSet::build(...)` / `FieldGroup::build(...)`.
- the first-party Empty docs/snippets now follow that same teaching lane:
  `apps/fret-ui-gallery/src/ui/snippets/empty/*` and `src/ui/snippets/spinner/empty.rs`
  now teach `shadcn::empty(|cx| ui::children![cx; ...])` plus the slot helper family rather than
  `Empty::new([...])` / `EmptyHeader::new([...])` / `EmptyContent::new([...])`,
  and the gallery source gate now locks that posture with
  `selected_empty_snippets_prefer_empty_wrapper_family`.
- the first-party Pagination docs/snippets now follow the same teaching lane:
  `apps/fret-ui-gallery/src/ui/snippets/pagination/{demo,icons_only,rtl,simple,usage}.rs`
  now teach `shadcn::pagination(|cx| ui::children![cx; ...])` plus the content/item/link helper
  family rather than `Pagination::new([ PaginationContent::new([ ... ]) ])`, and the gallery
  source gate now locks that posture with
  `selected_pagination_snippets_prefer_pagination_wrapper_family`.
- remaining follow-up after the current pagination landing:
  finish auditing page-level docs/code blocks that still teach old type-first examples for
  already-promoted families beyond the now-updated `field` page and AI page-local table helpers,
  before widening the thin-wrapper trial much further.
- next priority after the `card` + `alert` + `table` + `field` sweep:
  review the remaining lower-frequency wrappers and decide which ones still justify promotion,
  versus leaving them as explicit type-first APIs.
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
