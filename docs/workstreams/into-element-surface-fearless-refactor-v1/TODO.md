# Into-Element Surface (Fearless Refactor v1) — TODO

This TODO list tracks the work described in `DESIGN.md`.

Because this is a pre-release reset, "done" means we actually delete superseded public conversion
names rather than preserve them for inertia.

Companion docs:

- `DESIGN.md`
- `MILESTONES.md`
- `TARGET_INTERFACE_STATE.md`
- `MIGRATION_MATRIX.md`

Execution note on 2026-03-13:

- this is now the first active interface-refactor lane,
- do M0/M1 here before expanding trait-budget follow-ups elsewhere,
- use the canonical compare set (`simple_todo_v2_target`, `todo_demo`, scaffold template) as the
  first downstream migration/evidence target after the unified trait lands.
- raw seam closure on the shadcn lane is now considered done; keep the next batches focused on
  remaining app/helper migration rather than reopening `kbd_icon(...)` or `text_edit_context_menu*`
  unless the underlying storage/builder model changes.

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
- [x] Migrate `ecosystem/fret-ui-shadcn` reusable helper surfaces where raw `AnyElement` is not
  conceptually required.
- [x] Keep the canonical authoring compare set aligned on the target vocabulary:
  - [x] `apps/fret-cookbook/examples/simple_todo_v2_target.rs`
  - [x] `apps/fret-examples/src/todo_demo.rs`
  - [x] `apps/fretboard/src/scaffold/templates.rs`
- [x] Migrate official cookbook examples toward `Ui` / `UiChild`.
- [x] Migrate selected `apps/fret-examples` helper surfaces that are still on raw child returns.
- [ ] Migrate UI Gallery in two lanes:
  - [ ] app-facing teaching snippets toward `UiChild`,
  - [ ] generic reusable snippets toward the unified component conversion trait,
  - [ ] leave justified diagnostics/harness/raw helpers on `AnyElement`.

Validation snapshot on 2026-03-13:

- `CARGO_TARGET_DIR=target/codex-assets-reload cargo test -p fret-cookbook --lib`
- `CARGO_TARGET_DIR=target/codex-fret-examples cargo test -p fret-examples --lib`
- `CARGO_TARGET_DIR=target/codex-fretboard cargo test -p fretboard scaffold::templates::tests::todo_template_uses_default_authoring_dialect -- --exact`
- `CARGO_TARGET_DIR=target/codex-fretboard cargo test -p fretboard scaffold::templates::tests::simple_todo_template_has_low_adapter_noise_and_no_query_selector -- --exact`
- `cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app accordion_ -- --nocapture`
- `cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app tabs_ -- --nocapture`
- `cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app toggle_ -- --nocapture`
- `cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app radio_group_ -- --nocapture`
- `cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app slider_ -- --nocapture`
- `cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app native_select_ -- --nocapture`
- `cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app resizable_ -- --nocapture`
- `cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app navigation_menu_ -- --nocapture`
- `cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app scroll_area_ -- --nocapture`
- `cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app progress_ -- --nocapture`
- `cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app chart_ -- --nocapture`
- `cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app combobox_ -- --nocapture`
- `cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app carousel_ -- --nocapture`
- `cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app item_ -- --nocapture`
- `cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app table_ -- --nocapture`
- `cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app remaining_app_facing_tail -- --nocapture`

Implementation note on 2026-03-13:

- the canonical compare set now shares the same posture:
  app-facing imports via `fret::app::prelude::*`,
  `App` / `WindowId`,
  extracted helpers returning `impl UiChild`,
  with page-shell helpers dropping local `cx` when the body can stay fully late-landed and the
  remaining explicit `.into_element(cx)` step living only at the final render-root `Ui`
  conversion boundary,
  `ui::for_each_keyed(...)` as the default keyed-list helper,
  and `shadcn::card(...)` / `card_header(...)` / `card_content(...)` as the default card
  teaching family instead of `Card::build(...)`.
- `apps/fret-examples/src/todo_demo.rs::todo_page(...)` now stays on `impl UiChild`, drops the
  helper-local `cx`, and lets the page shell accept the card value without an intermediate
  `.into_element(cx)` landing seam.
- `apps/fret-examples/src/simple_todo_demo.rs::todo_row(...)` now stays on
  `impl IntoUiElement<App> + use<>` instead of `AnyElement`, while intentionally keeping
  `ui::for_each_keyed_with_cx(...)` because the keyed row still watches a per-item model inside
  the keyed child scope.
- `apps/fretboard/src/scaffold/templates.rs::{todo_page(...),simple_todo::todo_page(...)}`
  now follows that same rule: page helpers stay on `impl UiChild`, drop helper-local `cx`, and
  keep the explicit `.into_element(cx)` only at the final render-root conversion for the generated
  `todo` / `simple-todo` templates.
- the remaining advanced IMUI compare lane now also hides its non-raw helper returns behind typed
  signatures:
  `apps/fret-examples/src/imui_editor_proof_demo.rs::{render_editor_name_assist_surface,render_authoring_parity_surface,render_authoring_parity_shared_state,render_authoring_parity_declarative_group,render_authoring_parity_imui_group,render_authoring_parity_imui_host}`
  now expose `IntoUiElement<...>`-based signatures while keeping the internal
  `PropertyGroup::into_element(...)` / `imui_build(...)` landing seams explicit.
- after that cleanup, the current real non-`lib.rs` source scan now leaves only one intentional
  `-> AnyElement` helper on the examples/cookbook lane:
  `apps/fret-cookbook/examples/chart_interactions_basics.rs::chart_canvas(...)`.
- the UI Gallery app-facing snippet lane now also starts landing on typed top-level snippet
  surfaces:
  `apps/fret-ui-gallery/src/ui/snippets/accordion/{basic,borders,card,demo,disabled,extras,multiple,rtl,usage}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, and
  `apps/fret-ui-gallery/src/ui/pages/accordion.rs` now routes those previews through
  `DocSection::build(cx, ...)` instead of `DocSection::new(...)`.
- the same UI Gallery top-level snippet cleanup now also covers the tabs family:
  `apps/fret-ui-gallery/src/ui/snippets/tabs/{demo,disabled,extras,icons,line,list,rtl,usage,vertical,vertical_line}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, and
  `apps/fret-ui-gallery/src/ui/pages/tabs.rs` now routes those previews through
  `DocSection::build(cx, ...)` instead of `DocSection::new(...)`.
- the same UI Gallery top-level snippet cleanup now also covers the toggle family:
  `apps/fret-ui-gallery/src/ui/snippets/toggle/{demo,disabled,label,outline,rtl,size,usage,with_text}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, and
  `apps/fret-ui-gallery/src/ui/pages/toggle.rs` now routes those previews through
  `DocSection::build(cx, ...)` instead of `DocSection::new(...)`.
- the same UI Gallery top-level snippet cleanup now also covers the radio-group family:
  `apps/fret-ui-gallery/src/ui/snippets/radio_group/{choice_card,demo,description,disabled,extras,fieldset,invalid,label,plans,rtl,usage}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, and
  `apps/fret-ui-gallery/src/ui/pages/radio_group.rs` now routes those previews through
  `DocSection::build(cx, ...)` instead of `DocSection::new(...)`.
- the same UI Gallery top-level snippet cleanup now also covers the slider family:
  `apps/fret-ui-gallery/src/ui/snippets/slider/{demo,extras,label,usage}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, keep their local model
  state inside the snippet instead of routing it through the page shell, and
  `apps/fret-ui-gallery/src/ui/pages/slider.rs` now routes those previews through
  `DocSection::build(cx, ...)` instead of `DocSection::new(...)`.
- the same UI Gallery top-level snippet cleanup now also covers the native-select family:
  `apps/fret-ui-gallery/src/ui/snippets/native_select/{demo,disabled,invalid,label,rtl,usage,with_groups}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, keep local value/open
  model state inside each snippet instead of routing it through `pages/native_select.rs`, and
  `apps/fret-ui-gallery/src/ui/pages/native_select.rs` now routes those previews through
  `DocSection::build(cx, ...)` instead of `DocSection::new(...)`.
- the same UI Gallery top-level snippet cleanup now also covers the resizable family:
  `apps/fret-ui-gallery/src/ui/snippets/resizable/{demo,handle,notes,rtl,usage,vertical}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, keep local fractions
  model state inside each snippet instead of routing it through page/content/runtime-driver relay
  state, and `apps/fret-ui-gallery/src/ui/pages/resizable.rs` now routes those previews through
  `DocSection::build(cx, ...)` instead of `DocSection::new(...)`.
- the same UI Gallery top-level snippet cleanup now also covers the navigation-menu family:
  `apps/fret-ui-gallery/src/ui/snippets/navigation_menu/{demo,docs_demo,link_component,rtl,usage}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, keep the local menu
  value state inside each snippet, and `apps/fret-ui-gallery/src/ui/pages/navigation_menu.rs`
  now routes those previews through `DocSection::build(cx, ...)` instead of `DocSection::new(...)`.
- the same UI Gallery top-level snippet cleanup now also covers the scroll-area family:
  `apps/fret-ui-gallery/src/ui/snippets/scroll_area/{demo,usage,horizontal,nested_scroll_routing,rtl}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, the app-facing examples
  now consistently teach `shadcn::scroll_area(cx, |_cx| [...])`, and
  `apps/fret-ui-gallery/src/ui/pages/scroll_area.rs` routes those previews through
  `DocSection::build(cx, ...)` while intentionally keeping `drag_baseline` /
  `expand_at_bottom` on diagnostics-owned `DocSection::new(...)` seams.
- the same UI Gallery top-level snippet cleanup now also covers the progress family:
  `apps/fret-ui-gallery/src/ui/snippets/progress/{demo,usage,label,controlled,rtl}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, and
  `apps/fret-ui-gallery/src/ui/pages/progress.rs` now routes those previews through
  `DocSection::build(cx, ...)` instead of `DocSection::new(...)`.
- the same UI Gallery top-level snippet cleanup now also covers the chart family:
  `apps/fret-ui-gallery/src/ui/snippets/chart/{demo,usage,contracts,tooltip,legend,rtl}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, and
  `apps/fret-ui-gallery/src/ui/pages/chart.rs` now routes those previews through
  `DocSection::build(cx, ...)` instead of `DocSection::new(...)`.
- the same UI Gallery top-level snippet cleanup now also covers the combobox family:
  `apps/fret-ui-gallery/src/ui/snippets/combobox/{conformance_demo,basic,usage,label,auto_highlight,clear_button,groups,groups_with_separator,trigger_button,multiple_selection,custom_items,long_list,invalid,disabled,input_group,rtl}.rs`
  now expose typed app-facing `render(...)` signatures, and
  `apps/fret-ui-gallery/src/ui/pages/combobox.rs` now routes those previews through
  `DocSection::build(cx, ...)` instead of `DocSection::new(...)`.
- the same UI Gallery top-level snippet cleanup now also covers the carousel family:
  `apps/fret-ui-gallery/src/ui/snippets/carousel/{demo,usage,parts,basic,sizes_thirds,sizes,spacing,spacing_responsive,orientation_vertical,options,api,events,plugin_autoplay,plugin_autoplay_controlled,plugin_autoplay_stop_on_focus,plugin_autoplay_stop_on_last_snap,plugin_autoplay_delays,plugin_wheel_gestures,rtl,loop_carousel,loop_downgrade_cannot_loop,focus_watch,duration_embla,expandable}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, and
  `apps/fret-ui-gallery/src/ui/pages/carousel.rs` now routes those previews through
  `DocSection::build(cx, ...)` instead of `DocSection::new(...)`.
- the same UI Gallery top-level snippet cleanup now also covers the item family:
  `apps/fret-ui-gallery/src/ui/snippets/item/{demo,usage,variants,size,icon,avatar,image,group,header,link,dropdown,extras_rtl,gallery,link_render}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, and
  `apps/fret-ui-gallery/src/ui/pages/item.rs` now routes those previews through
  `DocSection::build(cx, ...)` instead of `DocSection::new(...)`.
- the same UI Gallery top-level snippet cleanup now also covers the table family:
  `apps/fret-ui-gallery/src/ui/snippets/table/{demo,usage,footer,actions,rtl}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, and
  `apps/fret-ui-gallery/src/ui/pages/table.rs` now routes those previews through
  `DocSection::build(cx, ...)` instead of `DocSection::new(...)`.
- the remaining curated default-app tail snippets are now also closed on the same typed lane:
  `apps/fret-ui-gallery/src/ui/snippets/breadcrumb/responsive.rs`,
  `apps/fret-ui-gallery/src/ui/snippets/date_picker/dropdowns.rs`,
  `apps/fret-ui-gallery/src/ui/snippets/form/notes.rs`, and
  `apps/fret-ui-gallery/src/ui/snippets/sidebar/rtl.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`; the corresponding page
  sections on `pages/breadcrumb.rs`, `pages/date_picker.rs`, `pages/form.rs`, and
  `pages/sidebar.rs` now keep those snippet-owned app-facing sections on `DocSection::build(cx, ...)`.
- after `accordion` / `tabs` / `toggle` / `radio_group` / `slider` / `native_select` /
  `resizable` / `navigation_menu` / `scroll_area` / `progress` / `chart` / `combobox` /
  `carousel` / `item` / `table` plus the remaining curated tail snippets, the next default-app UI
  Gallery app-facing queue should move to the first full legacy page-family sweep:
  `badge`, followed by `aspect_ratio` and `context_menu`.
- `apps/fret-cookbook/examples/customv1_basics.rs` now keeps both advanced reusable helpers
  `panel_shell(...)` and `preview_content(...)` on `IntoUiElement<KernelApp>`-based signatures
  instead of returning raw `AnyElement` for non-raw composition.
- `apps/fret-cookbook/examples/assets_reload_epoch_basics.rs::{render_image_panel,render_svg_panel}`
  now also keep their advanced helper surfaces on `IntoUiElement<KernelApp>`-based signatures
  instead of returning raw `AnyElement` for ordinary card/panel composition.
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
  `input.rs::input(...)`,
  `textarea.rs::textarea(...)`,
  `slider.rs::slider(...)`,
  `progress.rs::progress<H>(...)`,
  `switch.rs::{switch<H>(...), switch_opt<H>(...)}`, and
  `toggle.rs::{toggle(...), toggle_uncontrolled(...)}`,
  now keep the helper itself on a typed surface as well, with toggle preserving the concrete
  `Toggle` builder while also accepting typed child inputs before the internal child-list seam.
  `tabs.rs::{tabs(...), tabs_uncontrolled(...)}` now also preserves the concrete `Tabs` builder
  instead of eagerly landing the root helper. Meanwhile,
  `accordion.rs::{accordion_single(...), accordion_single_uncontrolled(...), accordion_multiple(...), accordion_multiple_uncontrolled(...)}`
  now likewise preserves the concrete `Accordion` builder so root-level
  `.collapsible(...)` / `.orientation(...)` / `.loop_navigation(...)` steps stay available before
  the explicit landing seam. Meanwhile,
  `toggle_group.rs::{toggle_group_single(...), toggle_group_single_uncontrolled(...), toggle_group_multiple(...), toggle_group_multiple_uncontrolled(...)}`
  now likewise preserves the concrete `ToggleGroup` builder so root-level
  `.variant(...)` / `.size(...)` / `.orientation(...)` / `.roving_focus(...)` steps stay available
  before the explicit landing seam. Meanwhile,
  `resizable.rs::resizable_panel_group(...)` now likewise preserves the concrete
  `ResizablePanelGroup` builder so root-level `.axis(...)` / `.style(...)` /
  `.test_id_prefix(...)` steps stay available before the explicit landing seam. Meanwhile,
  `navigation_menu.rs::{navigation_menu(...), navigation_menu_uncontrolled(...)}` now likewise
  preserves the concrete `NavigationMenu` builder so root-level `.viewport(...)` /
  `.indicator(...)` / `.md_breakpoint_query(...)` / `.delay_ms(...)` steps stay available before
  the explicit landing seam. Meanwhile,
  `avatar.rs::avatar_sized(...)`, `item.rs::{item_sized(...), item_group(...)}`,
  `scroll_area.rs::scroll_area(...)`, and `native_select.rs::native_select(...)` now likewise
  preserve the concrete `Avatar`, `Item`, `ItemGroup`, `ScrollArea`, and `NativeSelect` builders
  so common size/group/scroll/select configuration can stay fluent before the explicit landing
  seam. Meanwhile, the first-party UI Gallery teaching lane now follows those same
  helpers: `avatar/sizes.rs`, `item/{size,group}.rs`, `scroll_area/{usage,horizontal,nested_scroll_routing}.rs`,
  `native_select/{demo,usage,disabled,invalid,label,with_groups,rtl}.rs`, and the corresponding
  page copy now teach the builder-preserving helper family instead of the older eager `::new(...)`
  or `new_controllable(...)` defaults. The same teaching-lane cleanup now also covers
  `slider/{usage,label,demo}.rs`, `field/slider.rs`, `progress/controlled.rs`,
  `radio_group/{usage,label}.rs`, `form/upstream_demo.rs`, and the `slider` / `radio_group`
  pages so default controlled/uncontrolled examples now teach `slider(model)`,
  `radio_group(model, items)`, and `radio_group_uncontrolled(default, items)` while keeping
  `new_controllable(...)` only on the default-value bridge examples. The same first-party
  teaching sweep now also closes the default root-helper drift on `navigation_menu` /
  `resizable`: `navigation_menu/{usage,demo,docs_demo,link_component,rtl}.rs`,
  `resizable/{usage,demo,vertical,handle,rtl}.rs`, and the corresponding pages now teach
  `navigation_menu(cx, model, |cx| ..)` and `resizable_panel_group(cx, model, |cx| ..)` on the
  default lane instead of leading with raw `::new(...)` root constructors. The same first-party
  teaching sweep now also closes the remaining default-root drift on `tabs` / `toggle` /
  `accordion`: `tabs/{usage,demo,disabled,extras,icons,line,list,rtl,vertical,vertical_line}.rs`,
  `toggle/{usage,demo,outline,with_text,disabled,size,rtl,label}.rs`, and
  `accordion/{demo,basic,multiple,disabled,borders,card,extras,rtl}.rs` now teach
  `tabs_uncontrolled(cx, default, |cx| ..)`,
  `toggle_uncontrolled(cx, default, |cx| ..)` / `toggle(cx, model, |cx| ..)`, and
  `accordion_single_uncontrolled(cx, default, |cx| ..)` /
  `accordion_multiple_uncontrolled(cx, default, |cx| ..)` as the default first-party helper
  family, while `accordion/usage.rs` intentionally keeps the composable `AccordionRoot` surface
  as the explicit advanced seam and the corresponding `tabs` / `toggle` / `accordion` page copy
  now records that boundary directly. Meanwhile,
  `radio_group.rs::{radio_group(...), radio_group_uncontrolled(...)}` now expose typed
  constructor/wrapper outputs, with the radio-group helpers returning the concrete `RadioGroup`
  value so fluent `.a11y_label(...)` / `.disabled(...)` / `.style(...)` steps remain available
  before the explicit landing seam, while
  `card.rs::{card<H, I, F, T>(...), card_sized<H, I, F, T>(...), card_header<H, I, F, T>(...), card_action<H, I, F, T>(...), card_title<H, T>(...), card_description<H, T>(...), card_description_children<H, I, F, T>(...), card_content<H, I, F, T>(...), card_footer<H, I, F, T>(...)}`
  now expose typed constructor/wrapper outputs, while
  `kbd.rs::kbd_icon<H>(...)` remains the explicit raw helper seam because
  `Kbd::from_children(...)` still owns concrete landed-child storage. Combobox anchor overrides now
  reuse `PopoverAnchor::build(...).into_anchor(cx)` instead of a combobox-specific raw alias.
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
- the keyed-list lane now also has a dedicated default helper on `fret-ui-kit`:
  `ui::for_each_keyed(cx, items, |item| key, |item| child)` intentionally serves the
  conditional-list case where a layout closure wants to stay on the ordinary
  `ui::v_flex(|cx| ...)` / `ui::h_flex(|cx| ...)` lane instead of dropping to
  `*_build(|cx, out| ...)` only to spell `for item in ... { out.push_ui(cx, ui::keyed(...)) }`.
  The canonical compare set (`todo_demo`, `simple_todo_v2_target`, scaffold `simple-todo` /
  `todo`) now uses that helper for row lists, and the scaffold README/tests record
  `ui::for_each_keyed(...)` as the default first-party keyed-list teaching surface.
- the keyed-list lane now also has the explicit keyed-scope variant:
  `ui::for_each_keyed_with_cx(cx, items, |item| key, |cx, item| child)` exists for the smaller
  but real class of row builders that need the inner keyed child scope itself (for example
  row-local `cx.text(...)`, local keyed state, or other child-scope work that should happen
  inside the keyed boundary). `apps/fret-examples/src/simple_todo_demo.rs` now uses that helper
  instead of `ui::v_flex_build(...)` + `cx.keyed(...)`, which keeps the keyed-scope escape hatch
  on the ordinary `ui::v_flex(|cx| ..)` lane without introducing a wider `keyed_column(...)`
  abstraction yet.
- the default product docs now teach the same keyed-list lane:
  `docs/first-hour.md`, `docs/authoring-golden-path-v2.md`, and
  `docs/examples/todo-app-golden-path.md` now prefer `ui::for_each_keyed(...)` as the default
  list-identity story, reserve `ui::for_each_keyed_with_cx(...)` for row builders that truly need
  the inner keyed child scope, and frame `*_build(...)` sinks as explicit advanced/manual
  collection seams rather than the first-contact default.
- the query-demo compare lane now also follows the promoted card/default composition posture:
  `apps/fret-examples/src/query_demo.rs` and
  `apps/fret-examples/src/query_async_tokio_demo.rs` now use `shadcn::card(...)` /
  `card_header(...)` / `card_content(...)` plus ordinary `ui::h_row(...)` / `ui::v_flex(...)`
  composition for fixed child lists, while keeping one narrow `ui::v_flex_build(...)` seam only
  where `query_demo` still conditionally appends optional diagnostic text rows.
- the cookbook high-signal app-authoring lane now also follows the promoted card teaching family:
  `apps/fret-cookbook/examples/query_basics.rs`,
  `apps/fret-cookbook/examples/form_basics.rs`,
  `apps/fret-cookbook/examples/async_inbox_basics.rs`, and
  `apps/fret-cookbook/examples/router_basics.rs` now all teach
  `shadcn::card(...)` / `card_header(...)` / `card_content(...)` instead of
  `Card::build(...)` / `CardHeader::build(...)` / `CardContent::build(...)`, while keeping only
  the justified narrow manual seams that are still driven by conditional child emission
  (`query_basics::lines(...)`) or typed router/outlet ownership boundaries (`router_basics`).
- the next cookbook app-authoring batch now follows the same surface:
  `apps/fret-cookbook/examples/toggle_basics.rs`,
  `apps/fret-cookbook/examples/payload_actions_basics.rs`,
  `apps/fret-cookbook/examples/hello_counter.rs`,
  `apps/fret-cookbook/examples/text_input_basics.rs`, and
  `apps/fret-cookbook/examples/commands_keymap_basics.rs` now also teach the card wrapper family
  (`card(...)`, `card_header(...)`, `card_content(...)`, and `card_footer(...)` where needed)
  instead of leading with `Card::build(...)` / section builders on the default teaching lane.
- the next cookbook interop/state batch now also follows the same outer-card teaching surface:
  `apps/fret-cookbook/examples/undo_basics.rs`,
  `apps/fret-cookbook/examples/drag_basics.rs`,
  `apps/fret-cookbook/examples/external_texture_import_basics.rs`,
  `apps/fret-cookbook/examples/assets_reload_epoch_basics.rs`, and
  `apps/fret-cookbook/examples/date_picker_basics.rs` now all teach
  `shadcn::card(...)` plus the slot helper family for their outer shells instead of
  `Card::build(...)`, while preserving the justified advanced seams that still belong to
  interop/assets internals (`Alert::build(...)`, conditional asset error callouts, and retained
  viewport ownership).
- the next cookbook visual/app-support batch now also follows the same outer-card teaching
  surface:
  `apps/fret-cookbook/examples/customv1_basics.rs`,
  `apps/fret-cookbook/examples/drop_shadow_basics.rs`,
  `apps/fret-cookbook/examples/effects_layer_basics.rs`,
  `apps/fret-cookbook/examples/toast_basics.rs`, and
  `apps/fret-cookbook/examples/markdown_and_code_basics.rs` now teach `shadcn::card(...)` plus
  the slot helper family instead of `Card::build(...)`, with
  `customv1_basics.rs::panel_shell(...)` also moving onto `card(...)` / `card_header(...)` /
  `card_content(...)` so advanced cookbook helper code no longer re-teaches the old outer-shell
  pattern.
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
- Verification update on 2026-03-13:
  `radio_group.rs::{radio_group(...), radio_group_uncontrolled(...)}` now also leaves the default
  authoring surface on a typed lane by returning `RadioGroup` instead of eagerly landing
  `AnyElement`, with the crate-level source gate updated to forbid the old `(cx, ...) ->
  AnyElement` helper shape for this wrapper pair as well.
- Verification update on 2026-03-13:
  `input.rs::input(...)` and `textarea.rs::textarea(...)` now also expose the obvious builder
  values (`Input` / `Textarea`) instead of public `(cx, ...) -> AnyElement` render functions, so
  simple text-field authoring can stay on fluent component values before the explicit landing
  seam. The same crate-level thin-helper source gate now forbids the old raw helper signatures.
- Verification update on 2026-03-13:
  `slider.rs::slider(...)` now also returns the concrete `Slider` builder instead of exposing the
  old parameter-heavy `(cx, ...) -> AnyElement` render function on the public surface, keeping
  ordinary slider authoring aligned with the same builder-preserving rule as `input`,
  `textarea`, and `radio_group`.
- Verification update on 2026-03-13:
  `toggle.rs::{toggle(...), toggle_uncontrolled(...)}` now also returns the concrete `Toggle`
  builder instead of eagerly landing the helper output, and the closure inputs now accept typed
  child values (`IntoUiElement<H>`) instead of forcing `AnyElement` before the helper-owned
  internal child-list seam.
- Verification update on 2026-03-13:
  `tabs.rs::{tabs(...), tabs_uncontrolled(...)}` now also returns the concrete `Tabs` builder
  instead of eagerly landing the root helper, aligning the helper posture with the same
  builder-preserving story already used by `slider`, `toggle`, and `radio_group`.
- Verification update on 2026-03-13:
  `accordion.rs::{accordion_single(...), accordion_single_uncontrolled(...), accordion_multiple(...), accordion_multiple_uncontrolled(...)}`
  now also returns the concrete `Accordion` builder instead of eagerly landing the root helper,
  keeping common accordion configuration on the same builder-preserving lane as `tabs` while the
  explicit `.into_element(cx)` seam remains the only place where the root lands.
- Verification update on 2026-03-13:
  `toggle_group.rs::{toggle_group_single(...), toggle_group_single_uncontrolled(...), toggle_group_multiple(...), toggle_group_multiple_uncontrolled(...)}`
  now also returns the concrete `ToggleGroup` builder instead of eagerly landing the root helper,
  keeping grouped toggle configuration on the same builder-preserving lane as `accordion` and
  `tabs` while the explicit `.into_element(cx)` seam remains the only root landing point.
- Verification update on 2026-03-13:
  `resizable.rs::resizable_panel_group(...)` now also returns the concrete
  `ResizablePanelGroup` builder instead of eagerly landing the root helper, so common resizable
  root configuration can stay fluent until the explicit `.into_element(cx)` seam.
- Verification update on 2026-03-13:
  `navigation_menu.rs::{navigation_menu(...), navigation_menu_uncontrolled(...)}` now also
  returns the concrete `NavigationMenu` builder instead of eagerly landing the root helper, so
  common navigation-menu root configuration can stay fluent until the explicit
  `.into_element(cx)` seam.
- Verification update on 2026-03-13:
  `avatar.rs::avatar_sized(...)`, `item.rs::{item_sized(...), item_group(...)}`,
  `scroll_area.rs::scroll_area(...)`, and `native_select.rs::native_select(...)` now also stay on
  the builder-preserving lane instead of eagerly landing the obvious helper surface, so ordinary
  avatar/item/scroll/select authoring can keep fluent root configuration open until the explicit
  `.into_element(cx)` seam.
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
  - [x] overlay/controller internals.
    Evidence: `ecosystem/fret-ui-shadcn/src/ui_builder_ext/*.rs` still keeps
    `into_element(...) -> AnyElement` as the explicit landing seam while closure inputs are already
    typed via `IntoUiElement<H>`.
  - [x] diagnostics/harness helpers.
    Evidence: retained/manual-assembly seams such as
    `apps/fret-cookbook/examples/chart_interactions_basics.rs::chart_canvas(...) -> AnyElement`
    remain explicitly called out in this workstream rather than reintroduced into default teaching
    surfaces.
  - [x] low-level heterogeneous landing APIs.
    Evidence: `kbd.rs::kbd_icon(...)` remains inventoried by
    `surface_policy_tests::{explicit_raw_or_bridge_public_anyelement_helpers_stay_small_and_reviewable,kbd_icon_stays_an_explicit_raw_helper_for_kbd_child_lists}`,
    while combobox anchor overrides now route through
    `surface_policy_tests::combobox_surface_uses_generic_popover_anchor_builder_not_combobox_specific_raw_alias`.
  - [x] manual assembly / advanced runtime seams.
    Evidence: `text_edit_context_menu.rs::{text_edit_context_menu,text_selection_context_menu,text_edit_context_menu_controllable,text_selection_context_menu_controllable}`
    now keep the final wrapper landing seam explicit while accepting typed trigger values through
    `IntoUiElement<H>`; `state.rs::{use_selector_badge,query_status_badge}` have been promoted
    back to typed `Badge` outputs, `query_error_alert(...)` now returns `Option<Alert>`, and this
    lane is covered by
    `surface_policy_tests::state_helpers_prefer_typed_badge_outputs_when_no_runtime_landing_seam_is_required`.
  - [x] slot-scoped typed helper surfaces.
    Evidence: `tooltip.rs::TooltipContent::{build,text}(...)` now stay on the typed lane and are
    covered by
    `surface_policy_tests::tooltip_content_helpers_prefer_typed_build_and_text_outputs_when_slot_scope_is_required`.
- [x] Ensure raw surfaces remain explicit rather than leaking back into the app-facing story.
  - Current status: legacy module-local root helpers are cleared (`drawer(...)`, `menubar(...)`,
    `combobox(...)` deleted).
  - These helpers are now inventoried by
    `surface_policy_tests::legacy_public_anyelement_helper_inventory_is_explicit_until_promoted_or_deleted`
    and the inventory is expected to stay empty unless a new helper is added by explicit review.
  - Current shadcn deliberate-raw helper contracts are now fixed to
    `kbd.rs::kbd_icon(...)` and
    `text_edit_context_menu.rs::{text_edit_context_menu,text_selection_context_menu,text_edit_context_menu_controllable,text_selection_context_menu_controllable}`,
    with `surface_policy_tests::text_edit_context_menu_helpers_keep_landing_seam_explicit_but_accept_typed_triggers`
    documenting the final wrapper-seam rationale directly in source.
