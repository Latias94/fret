# Into-Element Surface — Target Interface State

Status: target state for the pre-release conversion-surface reset
Last updated: 2026-03-12

This document records the intended end state for the authoring conversion surface.

It answers four concrete questions:

1. Which return/conversion names should ordinary app authors learn?
2. Which conversion contract should reusable component authors use?
3. Which raw types remain intentionally explicit?
4. Which current public-looking names should disappear?

## Public Surface Tiers

| Tier | Intended audience | Canonical import | Conversion vocabulary |
| --- | --- | --- | --- |
| App | ordinary app authors | `use fret::app::prelude::*;` | `Ui`, `UiChild`, `.into_element(cx)` as an operation, not a trait taxonomy |
| Component | reusable component authors | `use fret::component::prelude::*;` | one public conversion trait generic over `H: UiHost` |
| Advanced | power users / runtime/interop code | explicit raw imports | `AnyElement`, `Elements`, raw `ElementContext<'_, H>` |

## App Surface

### Target nouns

- `Ui`
- `UiChild`
- `UiCx`
- `AppUi`

### Target rule

Default app-facing docs, templates, and examples should teach:

- `fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui`
- `fn helper(cx: &mut UiCx<'_>) -> impl UiChild`

They should not teach:

- `AnyElement`
- `Elements`
- `UiChildIntoElement<App>`
- `UiIntoElement`
- `UiHostBoundIntoElement`
- `UiBuilderHostBoundIntoElementExt`

### Hidden but allowed implementation detail

The app prelude may anonymously import the unified conversion trait needed to keep
`.into_element(cx)` method syntax working.

That import support is not part of the taught product vocabulary.

Transitional note on 2026-03-12:

- when an app-facing helper wants `impl UiChild` but the caller currently holds a host-bound
  builder value, one explicit landing step such as `let card = card.into_element(cx);` is an
  acceptable temporary seam.
- keep that seam explicit at the call site rather than widening app helpers to teach component
  conversion trait names prematurely.

Advanced/manual-assembly note on 2026-03-12:

- examples that intentionally use `fret::advanced::prelude::*` do not currently teach `UiChild`
  as their primary helper vocabulary.
- in that lane, the preferred non-raw helper signature is:

```rust
fn helper(cx: &mut UiCx<'_>) -> impl fret_ui_kit::IntoUiElement<KernelApp> + use<>
```

- do not default those helpers to `AnyElement` unless the function is actually raw composition
  glue (overlay/effect/manual-assembly internals).
- current first-party examples on that lane now include
  `custom_effect_v1_demo.rs::{stage, lens_row, plain_lens, custom_effect_lens, lens_shell,
  inspector}` and
  `custom_effect_v2_demo.rs::{stage, lens_row, plain_lens, custom_effect_lens, lens_shell,
  inspector}`; those helpers now stay on `IntoUiElement<KernelApp>`, while the internal
  `body.into_element(cx)` step inside `lens_shell(...)` remains explicit because it owns
  effect-layer/body assembly.
- `apps/fret-cookbook/examples/customv1_basics.rs::{panel_shell, preview_content}` now also uses
  `impl IntoUiElement<KernelApp> + use<...>` for non-raw helper composition while the effect-layer
  body assembly remains the intentional raw landing seam.
- `apps/fret-cookbook/examples/drop_shadow_basics.rs::shadow_card(...)` and
  `apps/fret-cookbook/examples/icons_and_assets_basics.rs::render_image_preview(...)` now follow
  the same rule: non-raw helper composition stays on `IntoUiElement<KernelApp>`, while effect
  layering and sibling child-collection remain explicit landing seams.
- `apps/fret-cookbook/examples/chart_interactions_basics.rs::chart_canvas(...)` remains a valid
  raw helper example because retained-subtree adoption is itself the bridge boundary; do not hide
  `AnyElement` there unless the retained/cached subtree APIs grow a higher-level typed helper.
- `apps/fret-examples/src/custom_effect_v3_demo.rs::{stage, stage_controls, animated_backdrop,
  lens_row, lens_shell}` now also follows the typed-helper rule: these helpers return
  `impl IntoUiElement<KernelApp> + use<>`, with explicit `.into_element(cx)` kept only at the
  heterogenous sibling child-collection seams that intentionally still want `AnyElement`.
- `apps/fret-examples/src/postprocess_theme_demo.rs::{inspector, stage, stage_body, stage_cards}`
  now follows the same rule: helper composition stays on `IntoUiElement<KernelApp>`, while the
  root two-pane layout and effect-layer compare branches remain explicit landing seams.
- `apps/fret-examples/src/async_playground_demo::{header_bar, body, catalog_panel, main_panel,
  inspector_panel, policy_editor, query_panel_for_mode, query_inputs_row, query_result_view,
  status_badge}` now also follows the typed-helper rule: local composition stays on
  `IntoUiElement<KernelApp>`, while `TabsItem::new([..])`, `ScrollArea::new([..])`, and other
  heterogenous child arrays remain explicit landing seams.
- Rust 2024 precise captures note: if the helper wants `+ use<...>` and also accepts a flexible
  label/input argument, prefer a named generic such as `fn helper<L>(..., label: L) -> impl
  IntoUiElement<KernelApp> + use<L> where L: Into<Arc<str>>` rather than argument-position
  `impl Into<Arc<str>>`, because precise captures currently require all type parameters to appear
  in the `use<...>` list.
- semantics ergonomics follow-up: explicit `.into_element(cx).attach_semantics(...)` is an
  acceptable local workaround, but if that pattern becomes common the framework should add a
  unified decorator helper on top of public `IntoUiElement<H>` rather than multiplying builder-
  specific semantics methods across ecosystem components.

Default-app reusable helper note on 2026-03-12:

- if a helper lives on the default app-facing surface (`use fret::UiCx;`) but is still authored as
  a reusable typed landing helper rather than `impl UiChild`, prefer:

```rust
fn helper(cx: &mut UiCx<'_>) -> impl fret_ui_kit::IntoUiElement<fret_app::App> + use<>
```

- do not teach `KernelApp` on that lane: it is not a public type on the top-level `fret` facade,
  so Gallery/default-app snippets should spell `fret_app::App` when they need the concrete host
  type.
- current first-party examples on that lane now include
  `apps/fret-examples/src/custom_effect_v2_identity_web_demo.rs::{lens, inspector}`,
  `apps/fret-examples/src/custom_effect_v2_web_demo.rs::{lens, inspector}`,
  `apps/fret-examples/src/custom_effect_v2_lut_web_demo.rs::{lens, inspector}`, and
  `apps/fret-examples/src/custom_effect_v2_glass_chrome_web_demo.rs::{label_row, lens,
  controls_panel}`.
- keep `.into_element(cx)` explicit only where those demos intentionally still assemble raw stage
  child arrays, overlay child collections, or other concrete landing seams.
- first-party page/snippet teaching rule:
  once a wrapper/helper family is promoted onto the default authoring path, Gallery pages and
  snippets should teach that wrapper family by default and should not silently fall back to eager
  `*::new(...)` constructors or lower-level `*::build(...)` forms.
- lower-level builder names such as `Card::build(...)` may still appear in first-party docs only
  when they are explicitly labeled as advanced or late-child-collection escape hatches rather than
  default-equal alternatives.

## Component Surface

### Target public conversion contract

The curated component surface should expose one public conversion trait.

Working name:

```rust
pub trait IntoUiElement<H: UiHost>: Sized {
    fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement;
}
```

The final name can still shift during implementation, but the target contract must remain:

- host parameter lives on the trait,
- both host-agnostic values and host-bound builders implement it,
- `.into_element(cx)` keeps one obvious meaning,
- no second public bridge trait is needed just to recover method syntax.

### Target companion types

Keep on the component surface:

- `UiBuilder`
- `UiPatchTarget`
- `UiSupportsChrome`
- `UiSupportsLayout`
- layout/style refinement types
- semantics helpers
- `AnyElement` as an explicit raw type

### Target non-exports

The curated component surface should no longer teach or re-export:

- `UiIntoElement`
- `UiHostBoundIntoElement`
- `UiChildIntoElement`
- `UiBuilderHostBoundIntoElementExt`

If some of those names remain internally for a short migration window, they should be treated as
temporary scaffolding rather than stable surface.

Current execution note on 2026-03-12:

- `UiBuilderHostBoundIntoElementExt` is already deleted from the codebase.
- `UiHostBoundIntoElement<H>` is already deleted from the codebase.
- the legacy `UiIntoElement` name is now deleted from code; `fret_ui_kit::ui_builder` keeps only
  the public `IntoUiElement<H>` contract plus a direct `IntoUiElement<H> for AnyElement`
  implementation for explicit raw seams.
- exported `fret_ui_kit` adapter macros (`ui_component_*`, `ui_component_render_once!`) and the
  built-in primitive glue now implement `IntoUiElement<H>` directly, so the legacy
  `UiIntoElement` name is no longer needed anywhere in production code.
- declarative semantics helpers now also sit on the public landing trait:
  `UiElementTestIdExt`, `UiElementA11yExt`, and `UiElementKeyContextExt` wrap values that land
  through `IntoUiElement<H>` directly, so `UiIntoElement` no longer leaks into the production
  implementation of `declarative/semantics.rs`.
- built-in text primitives now also land directly through the public trait:
  `ui::TextBox` and `ui::RawTextBox` implement `IntoUiElement<H>` directly, and `ui_builder.rs`
  now keeps only the public trait plus raw `AnyElement` landing.
- `UiChildIntoElement<H>` is now deleted from the codebase; heterogeneous child collection in
  `fret_ui_kit::ui` / `imui` lands directly through `IntoUiElement<H>`.
- direct-crate shadcn authoring now also gets the same landing ergonomics:
  `fret_ui_shadcn::prelude::*` re-exports `IntoUiElement`, so typed helpers such as
  `shadcn::raw::typography::*` can land through `.into_element(cx)` without ad-hoc trait imports
  on first-party examples.
- first-party ecosystem consumers and curated docs now also avoid the old name:
  `docs/first-hour.md` and the `fret-ui-ai` message/workflow builder smoke tests now spell
  `IntoUiElement<H>` rather than `UiIntoElement`.
- when a first-party shadcn example still targets an eager constructor that owns
  `new(children: Vec<AnyElement>)`, the teaching surface should prefer
  `ui::children![cx; ...]` over `vec![...into_element(cx)]` so typed values remain the visible
  authoring vocabulary even before that constructor grows a late-landing builder path.
- this now explicitly includes first-party modal/form section examples such as
  `DialogContent` / `DialogHeader` / `DialogFooter`, `SheetContent` / `SheetHeader` /
  `SheetFooter`, and `DrawerContent` / `DrawerHeader` / `DrawerFooter`.
- the same eager child-list guidance now also covers selected popover form snippets such as
  `PopoverContent` / `PopoverHeader` / `FieldGroup` / `Field`.
- when a reusable helper can stay fully late-landed, it should not keep a render-local `cx`
  parameter just to attach layout or semantics; first-party table helpers such as
  `make_invoice_table(...)` now return typed builders and land only at the call site.

## Advanced/Raw Surface

Keep explicit and legal:

- `AnyElement`
- `Elements`
- raw `ElementContext<'_, H>`
- raw overlay/controller/helper internals

Target rule:

- raw landed-element surfaces remain available,
- but they are not the default authoring story for ordinary apps or reusable component examples.

## Target Helper Signatures

### App-facing helper

```rust
fn footer(cx: &mut UiCx<'_>) -> impl UiChild
```

Transitional equivalent while the child pipeline is still migrating:

```rust
fn page(cx: &mut UiCx<'_>, content: impl UiChild) -> AnyElement
```

### Reusable generic helper

```rust
fn footer<H: UiHost>(cx: &mut ComponentCx<'_, H>) -> impl IntoUiElement<H>
```

Wrapper/composer helper rule:

```rust
fn centered<H: UiHost, B>(body: B) -> impl IntoUiElement<H> + use<H, B>
where
    B: IntoUiElement<H>
```

If a helper only wraps/layouts child content, prefer accepting `IntoUiElement<H>` instead of
forcing callers to pre-land `AnyElement`.

First-party UI Gallery examples already using this rule now include:

- `src/ui/snippets/ai/{context_default,context_demo}.rs::centered(...)`
- `src/ui/snippets/ai/{file_tree_basic,file_tree_expanded}.rs::preview(...)`
- `src/ui/snippets/ai/file_tree_large.rs::{preview(...), invisible_marker(...)}`
- `src/ui/snippets/ai/test_results_demo.rs::progress_section(...)`
- `src/ui/snippets/ai/attachments_usage.rs::render_grid_attachment(...)`
- `src/ui/snippets/ai/attachments_grid.rs::render_grid_attachment(...)`
- `src/ui/snippets/ai/attachments_list.rs::render_list_attachment(...)`
- `src/ui/snippets/ai/file_tree_demo.rs::invisible_marker(...)`
- `src/ui/snippets/ai/speech_input_demo.rs::{body_text(...), clear_action(...)}`
- `src/ui/snippets/avatar/{with_badge,fallback_only,sizes,dropdown}.rs::wrap_row(...)`
- `src/ui/snippets/avatar/{demo,group,sizes,group_count}.rs::avatar_with_image(...)`
- `src/ui/snippets/avatar/{demo,with_badge}.rs::avatar_with_badge(...)`
- `src/ui/snippets/avatar/{group,group_count}.rs::{group(...), group_with_count(...)}`
- `src/ui/snippets/avatar/{badge_icon,group_count_icon}.rs::icon(...)`
- `src/ui/snippets/button/{demo,size,with_icon,link_render,rtl,loading,variants,button_group,rounded}.rs::wrap_row(...)`
- `src/ui/snippets/button/size.rs::row(...)`
- `src/ui/snippets/tabs/demo.rs::field(...)`
- `src/ui/snippets/collapsible/basic.rs::rotated_lucide(...)`
- `src/ui/snippets/collapsible/settings_panel.rs::radius_input(...)`
- `src/ui/snippets/collapsible/rtl.rs::details_collapsible(...)`
- `src/ui/snippets/collapsible/file_tree.rs::{rotated_lucide(...), file_leaf(...), folder(...)}`
- `src/ui/snippets/drawer/responsive_dialog.rs::{profile_field(...), profile_form(...)}`
- `src/ui/snippets/drawer/sides.rs::side_button(...)`
- `src/ui/snippets/drawer/scrollable_content.rs::paragraph_block(...)`
- `src/ui/snippets/sheet/{demo,rtl}.rs::profile_fields(...)`
- `src/ui/snippets/dialog/{demo,rtl}.rs::profile_fields(...)`
- `src/ui/snippets/dialog/{scrollable_content,sticky_footer}.rs::lorem_block(...)`
- `src/ui/snippets/separator/menu.rs::section(...)`
- `src/ui/snippets/separator/list.rs::row(...)`
- `src/ui/snippets/sidebar/{demo,controlled,mobile,rtl}.rs::menu_button(...)`
- `src/ui/snippets/aspect_ratio/{portrait,square,rtl}.rs::{portrait_image(...), square_image(...), rtl_image(...), ratio_example(...)}`
- `src/ui/snippets/combobox/{long_list,input_group,trigger_button,groups_with_separator,groups,disabled,custom_items,clear_button,invalid}.rs::{state_row(...), state_rows(...)}`
- `src/ui/snippets/popover/{basic,demo,with_form}.rs::centered(...)`
- `src/ui/snippets/resizable/{usage,vertical,handle}.rs::panel(...)`
- `src/ui/snippets/resizable/{vertical,handle}.rs::box_group(...)`
- `src/ui/snippets/data_table/{basic_demo,default_demo,guide_demo}.rs::align_end(...)`
- `src/ui/snippets/data_table/{basic_demo,rtl_demo}.rs::bottom_controls(...)`
- `src/ui/snippets/data_table/default_demo.rs::footer(...)`
- `src/ui/snippets/data_table/rtl_demo.rs::align_inline_start(...)`
- `src/ui/snippets/table/{demo,footer,rtl}.rs::make_invoice_table(...)`
- `src/ui/snippets/table/actions.rs::{align_end(...), action_row(...)}`
- `src/ui/snippets/hover_card/{sides,trigger_delays}.rs::{card(...), demo_content(...)}`
- `src/ui/snippets/tooltip/{rtl,sides}.rs::make_tooltip(...)`
- `src/ui/snippets/tooltip/rtl.rs::make_tooltip_with_test_ids(...)`
- `src/ui/snippets/breadcrumb/dropdown.rs::dot_separator(...)`
- `src/ui/snippets/carousel/{basic,sizes,plugin_wheel_gestures,spacing_responsive,loop_carousel,loop_downgrade_cannot_loop,spacing,usage,sizes_thirds,parts,duration_embla,rtl,plugin_autoplay,plugin_autoplay_controlled}.rs::{slide_card(...), slide(...)}`
- `src/ui/snippets/carousel/{options,api,plugin_autoplay_delays,plugin_autoplay_stop_on_last_snap,events}.rs::slide_card(...)`
- `src/ui/snippets/carousel/plugin_autoplay_stop_on_focus.rs::slide(...)`
- `src/ui/snippets/item/avatar.rs::{icon_button(...), item_team(...)}`
- `src/ui/snippets/item/icon.rs::{icon(...), item_icon(...)}`
- `src/ui/snippets/item/{link,link_render,dropdown}.rs::icon(...)`
- `src/ui/snippets/item/extras_rtl.rs::{outline_button_sm(...), item_basic(...)}`
- `src/ui/snippets/item/gallery.rs::{icon(...), icon_button(...), outline_button(...), outline_button_sm(...), item_basic(...), item_icon(...), item_avatar(...), item_team(...)}`

Those examples keep explicit `.into_element(cx)` seams only where the surrounding API still
intentionally consumes raw landed children, such as `DocSection::new(...)`, child arrays,
overlay/provider constructor seams, `TabsItem::new(...)`,
`Collapsible::into_element_with_open_model(...)`, `DrawerContent::new(...)`,
`SheetContent::new(...)`, `DialogContent::new(...)`,
`Table::build(...).into_element(cx).test_id(test_id)` wrapper seams, data-table/table cell-row
seams, sibling child-collection seams like the selected `combobox/*::state_rows(...)` callers,
`CarouselItem::new(...)`, `ItemMedia::new(...)`, and `ItemActions::new(...)`.

The same rule also applies to ecosystem-level explicit landing helpers:

- `ecosystem/fret-ui-shadcn/src/ui_builder_ext/*::into_element(...)` should remain
  `-> AnyElement` because those extension methods are themselves named landing seams.
- Their closure inputs should still accept values via `IntoUiElement<H>` rather than regressing to
  `AnyElement`-typed closure signatures.

The same wrapper rule also applies to internal gallery scaffolds:

- `src/ui/doc_layout.rs::demo_shell<B>(...)`
- `src/ui/previews/pages/editors/code_editor/mvp/gates.rs::gate_panel<B>(...)`

The same input rule also applies to internal shadcn menu-slot wrappers:

- `ecosystem/fret-ui-shadcn/src/context_menu.rs::menu_icon_slot<H, B>(...)`
- `ecosystem/fret-ui-shadcn/src/dropdown_menu.rs::menu_icon_slot<H, B>(...)`
- `ecosystem/fret-ui-shadcn/src/menubar.rs::menu_icon_slot<H, B>(...)`

The same typed-constructor rule now also applies to thin public shadcn constructors or wrappers
where no raw child list or explicit landing seam is conceptually required:

- `ecosystem/fret-ui-shadcn/src/badge.rs::badge<H, T>(...)`
- `ecosystem/fret-ui-shadcn/src/command.rs::command<H, I, F, T>(...)`
- `ecosystem/fret-ui-shadcn/src/input_group.rs::input_group<H>(...)`
- `ecosystem/fret-ui-shadcn/src/input_otp.rs::input_otp<H>(...)`
- `ecosystem/fret-ui-shadcn/src/kbd.rs::kbd<H, T>(...)`
- `ecosystem/fret-ui-shadcn/src/separator.rs::separator<H>()`

Intentional exception:

- `ecosystem/fret-ui-shadcn/src/kbd.rs::kbd_icon<H>(...)` remains `-> AnyElement` because
  `Kbd::from_children(...)` still owns a concrete `Vec<AnyElement>` slot for icon-first keycap
  composition.

Implementation fallback rule:

- if an ecosystem builder or recipe type does not yet implement `IntoUiElement<H>` directly,
  it is still acceptable for the helper to land internally with `.into_element(cx)` and expose the
  result as `impl IntoUiElement<H>`.
- keep that fallback inside the helper so callers still see the unified conversion contract rather
  than a raw `AnyElement` signature.

### Raw helper

```rust
fn footer_raw<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement
```

## Target Return-Type Rules

Use these rules in first-party docs/examples:

| Situation | Preferred return type |
| --- | --- |
| View `render(...)` | `Ui` |
| App-facing extracted helper | `impl UiChild` |
| Reusable generic component helper | `impl IntoUiElement<H>` |
| Raw landed helper, low-level overlay internals, diagnostics, explicit IR plumbing | `AnyElement` |

## Delete Targets

These names are delete targets for the curated public product surface:

- `UiIntoElement`
- `UiHostBoundIntoElement`
- `UiChildIntoElement`
- `UiBuilderHostBoundIntoElementExt`

These raw names are **not** delete targets:

- `AnyElement`
- `Elements`

They remain explicit raw tools, not default teaching terms.
