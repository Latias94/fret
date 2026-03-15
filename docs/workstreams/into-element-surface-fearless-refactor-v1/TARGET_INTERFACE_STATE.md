# Into-Element Surface — Target Interface State

Status: target state for the pre-release conversion-surface reset
Last updated: 2026-03-15

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
- `fret::workspace_shell::{workspace_shell_model, workspace_shell_model_default_menu}` and
  `fret_workspace::{WorkspacePaneContentFocusTarget, WorkspaceFrame, WorkspaceCommandScope}`
  now also follow the typed-helper rule:
  pane renderers and workspace shell wrappers may stay on `IntoUiElement<H>`, while the shell
  keeps the final explicit landing seam only where it must register pane-content focus targets or
  assemble heterogeneous child rows.
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
- `apps/fret-cookbook/src/scaffold.rs::{centered_page, centered_page_background, centered_page_muted}`
  now also follows the app-facing root rule: shared cookbook page shells take `&mut UiCx<'_>` plus
  `impl UiChild` and return `Ui`, so the canonical compare set (`hello_counter`, `simple_todo`,
  `simple_todo_v2_target`) no longer teaches a redundant `.into()` after the scaffold call.
- the remaining canonical compare-set roots now follow the same rule too:
  `apps/fret-examples/src/todo_demo.rs::todo_page(...)` and the generated helpers in
  `apps/fretboard/src/scaffold/templates.rs` take `&mut UiCx<'_>` and return `Ui`, so
  `todo_demo` plus both todo templates no longer teach `todo_page(...).into_element(cx).into()`
  as the default root story.
- first-party page/snippet teaching rule:
  once a wrapper/helper family is promoted onto the default authoring path, Gallery pages and
  snippets should teach that wrapper family by default and should not silently fall back to eager
  `*::new(...)` constructors or lower-level `*::build(...)` forms.
- curated namespace rule:
  if a family is taught through `fret_ui_shadcn::{facade as shadcn, prelude::*};`, every
  authoring-critical type on that lane must be reachable from both the crate root and the curated
  `facade` namespace; do not strand newly added builder steps or parts behind root-only exports.
- lower-level builder names such as `Card::build(...)` may still appear in first-party docs only
  when they are explicitly labeled as advanced or late-child-collection escape hatches rather than
  default-equal alternatives.

### Family authoring taxonomy

The app-facing typed surface does **not** require every shadcn family to converge on one identical
root story.

The target is a small, explicit taxonomy of first-party authoring lanes:

| Lane | Default teaching path | Family examples | Contract rule |
| --- | --- | --- | --- |
| Compose-root default lane | typed root builder + `compose()` | `DropdownMenu`, `ContextMenu`, `Dialog`, `Sheet`, `AlertDialog`, `Drawer` | teach one default copyable root path; keep lower-level `build_parts(...)` / `into_element_parts(...)` or `Parts` examples as focused follow-ups rather than equal defaults |
| Dual-lane family | compact typed lane **and** upstream-shaped copyable lane | `Carousel`, `Menubar`, `InputGroup` | do not force these families into an "advanced escape hatch" narrative; both lanes are legitimate first-party teaching surfaces and must be labeled explicitly |
| Direct recipe root/bridge | recipe-level root story without a generic `compose()` story | `Popover`, `HoverCard`, `Tooltip`, `Select`, `Combobox`, `Command`, `InputOtp` | keep the recipe root/bridge as the public story; when the root already owns typed builder steps, teach that compact chain first and keep `into_element_parts(...)` only as the focused upstream-shaped adapter on the same lane; do not invent a `compose()` root just for uniformity if the family is already clear and source-aligned |

This workstream therefore treats "write UI feel" as a **lane-classification** problem as much as a
trait-signature problem.

The first-party rule is:

- every family that appears on the default Gallery/docs surface must declare which lane it belongs
  to,
- every page/snippet pair must teach exactly that lane on the copyable/default path,
- every secondary lane must be labeled either as a focused follow-up or as an equal second lane,
- no fourth lane should be introduced unless the existing three are demonstrably insufficient.

Current classification queue after the 2026-03-14 follow-up:

- none on the current shadcn focus lane

Current classification snapshot:

- compose-root default lane:
  `DropdownMenu`, `ContextMenu`, `Dialog`, `Sheet`, `AlertDialog`, `Drawer`
- dual-lane family:
  `Carousel`, `Menubar`, `NavigationMenu`, `Pagination`, `InputGroup`
- direct recipe root/bridge:
  `Popover`, `HoverCard`, `Tooltip`, `Select`, `Combobox`, `Command`, `InputOtp`

Current direct-root compact-chain readout on 2026-03-14:

- `Select` keeps the direct recipe lane, but the default first-party root story is now the compact
  builder chain `.trigger(...).value(...).content(...).entries(...)`; `into_element_parts(...)`
  remains the focused upstream-shaped adapter on that same lane.
- `Combobox` keeps the direct recipe lane, but the default first-party root story is now the
  compact builder chain `.trigger(...).input(...).clear(...).content(...)`;
  `into_element_parts(...)` remains the focused upstream-shaped patch seam on that same lane.
- `InputOtp` now also keeps the direct recipe lane, and the default first-party root story is the
  compact builder using `length(...)` plus optional `group_size(...)`; `into_element_parts(...)`
  remains the upstream-shaped bridge when callers explicitly want slot/group authoring.
- `InputGroup` now sits on the dual-lane family list: first-party default snippets prefer the
  compact `InputGroup::new(model)` slot shorthand, while the explicit addon/control parts remain a
  legitimate docs-parity lane rather than an advanced-only escape hatch.
- `Carousel` now also has an explicit dual-lane readout: docs-first examples such as `Usage`,
  `Basic`, `Sizes`, `Spacing`, `Orientation`, `Options`, and `Loop`, plus ordinary diagnostics
  demos such as `Demo`, `API`, `Focus`, `Duration`, autoplay/wheel examples, and loop downgrade,
  prefer the compact `Carousel::new(items)` builder lane. Only `parts.rs` and diagnostics-specific
  snippets that need explicit control parts or control-level `test_id`s (currently `Events` and
  `RTL`) remain on the upstream-shaped parts lane.

Future families should be classified into one of the three lanes above **before** widening API
surface area again. If a family really does not fit, document the mismatch first instead of
silently adding another root-builder pattern.

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

### Intentional raw request seams (2026-03-15)

The remaining raw request constructors on this lane are intentional.

Reason:

- they assemble already-landed overlay payloads after typed children have been converted,
- that boundary no longer has a live `ElementContext<'_, H>` available,
- forcing those constructors to pretend they are typed would hide a real landing seam rather than
  improve authoring.

Final expected raw request seams:

- `ecosystem/fret-ui-kit/src/overlay_controller.rs::OverlayRequest` constructors,
- request-constructor families in
  `ecosystem/fret-ui-kit/src/primitives/dialog.rs`,
  `ecosystem/fret-ui-kit/src/primitives/popover.rs`,
  `ecosystem/fret-ui-kit/src/primitives/alert_dialog.rs`,
  `ecosystem/fret-ui-kit/src/primitives/select.rs`,
  and `ecosystem/fret-ui-kit/src/primitives/tooltip.rs`.

Typed adapters that still do have a live `ElementContext` should stay typed on top of those seams.
Current example:

- `ecosystem/fret-ui-kit/src/primitives/tooltip.rs::TooltipRoot::request(...)` accepts
  `IntoUiElement<H>` children and lands them before calling the final raw tooltip request
  constructor.

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

The same rule now also applies to shared cookbook page-shell helpers:

- `apps/fret-cookbook/src/scaffold.rs::{centered_page,centered_page_background,centered_page_muted}`
  intentionally remain named final page-root landing seams returning `AnyElement`, but they now
  accept `IntoUiElement<H>` inputs directly and no longer split the surface into separate
  `*_ui(...)` overloads for host-bound builders.

The same rule now also applies to router adoption helpers:

- `ecosystem/fret-router-ui/src/lib.rs::{router_outlet,router_outlet_with_test_id}`
- `ecosystem/fret-router-ui/src/lib.rs::RouterOutlet::{into_element,into_element_by_leaf,into_element_by_leaf_with_status}`
- `ecosystem/fret-router-ui/src/lib.rs::{router_link,router_link_to,router_link_to_with_test_id,router_link_to_typed_route,router_link_to_typed_route_with_test_id,router_link_with_props,router_link_with_test_id}`

Those helpers intentionally still return a landed `AnyElement` because they own the router
snapshot read or final pressable/outlet landing seam, but their closure outputs and iterable child
inputs now accept `IntoUiElement<App>` directly and do not split into parallel `*_ui(...)`
overloads, crate-local conversion traits, or `IntoIterator<Item = AnyElement>`-typed public child
surfaces.

The same rule also applies to first-party visual helper crates that own only a final wrapper seam:

- `ecosystem/fret-ui-magic/src/{magic_card,border_beam,lens,marquee,patterns,dock,sparkles_text}.rs`

Those helpers still return `AnyElement` because they own the final effect/material/wrapper landing
boundary, but their child closures now accept iterable `IntoUiElement<H>` values directly instead
of publishing `IntoIterator<Item = AnyElement>` on the public surface.

The same wrapper rule now also applies to `fret-ui-kit` declarative effect panels:

- `ecosystem/fret-ui-kit/src/declarative/{bloom,pixelate}.rs`

`bloom_panel(...)` and `pixelate_panel(...)` still return `AnyElement` because they own the final
effect-layer wrapper boundary, but their child closures now accept iterable `IntoUiElement<H>`
values directly and land them behind the crate-local `collect_children(...)` helper instead of
publishing raw `IntoIterator<Item = AnyElement>` child items on the public surface.

The same wrapper rule also applies to policy helpers that only bridge into a focus/a11y/runtime
wrapper:

- `ecosystem/fret-ui-kit/src/declarative/{dismissible,visually_hidden}.rs`
- `ecosystem/fret-ui-kit/src/primitives/focus_scope.rs::{focus_trap,focus_trap_with_id}`

Those helpers still return `AnyElement` or `NodeId` because they own the final dismissible root,
semantics wrapper, or focus-scope wrapper boundary, but their child closures now accept iterable
`IntoUiElement<H>` values directly and land them behind the same crate-local
`collect_children(...)` helper instead of publishing raw `IntoIterator<Item = AnyElement>` child
items on the public surface.

The same wrapper rule also applies to scroll and roving-focus policy helpers:

- `ecosystem/fret-ui-kit/src/declarative/scroll.rs`
- `ecosystem/fret-ui-kit/src/primitives/roving_focus_group.rs`
- `ecosystem/fret-ui-kit/src/primitives/toolbar.rs`
- `ecosystem/fret-ui-kit/src/primitives/dismissable_layer.rs`

Those helpers still return `AnyElement` or `NodeId` because they own the final scroll, roving
container, toolbar, or dismissable-root wrapper boundary, but their child closures now accept
iterable `IntoUiElement<H>` values directly. Direct wrappers land them behind
`collect_children(...)`, while delegate wrappers forward only to already-typed helper seams rather
than publishing raw `IntoIterator<Item = AnyElement>` child items on the public surface.

The same wrapper rule also applies to layout/effect query wrappers in `fret-ui-kit`:

- `ecosystem/fret-ui-kit/src/declarative/chrome.rs`
- `ecosystem/fret-ui-kit/src/declarative/glass.rs`
- `ecosystem/fret-ui-kit/src/declarative/container_queries.rs`

Those helpers still return `AnyElement` because they own the final control chrome, effect layer, or
layout-query wrapper boundary, but their child closures now accept iterable `IntoUiElement<H>`
values directly and land them behind `collect_children(...)` instead of publishing raw
`IntoIterator<Item = AnyElement>` child items on the public surface.

The same wrapper rule also applies to menu/popup skeleton helpers in `fret-ui-kit`:

- `ecosystem/fret-ui-kit/src/primitives/menu/{content,content_panel,sub_content}.rs`
- `ecosystem/fret-ui-kit/src/primitives/popper_content.rs`

Those helpers still return `AnyElement` or `(GlobalElementId, AnyElement)` because they own the
final menu semantics wrapper, menu panel, submenu skeleton, or popper wrapper/panel boundary, but
their child closures now accept iterable `IntoUiElement<H>` values directly. Direct wrappers land
them behind `collect_children(...)`, while delegate wrappers forward only to already-typed helper
seams instead of publishing raw `IntoIterator<Item = AnyElement>` child items on the public
surface.

The same wrapper rule also applies to cache/list helpers in `fret-ui-kit`:

- `ecosystem/fret-ui-kit/src/declarative/cached_subtree.rs`
- `ecosystem/fret-ui-kit/src/declarative/list.rs`

Those helpers still return `AnyElement` because they own the final cache-root or list-row wrapper
boundary, but their child closures now accept iterable `IntoUiElement<...>` values directly.
`cached_subtree` lands through `collect_children(...)` behind its `ViewCache` wrapper, and the
retained `list` path now lands the explicit cached row payload through the same helper instead of
publishing raw `IntoIterator<Item = AnyElement>` child items on the public surface.

The same wrapper rule also applies to the tab/toggle/accordion primitives in `fret-ui-kit`:

- `ecosystem/fret-ui-kit/src/primitives/tabs.rs`
- `ecosystem/fret-ui-kit/src/primitives/toggle.rs`
- `ecosystem/fret-ui-kit/src/primitives/accordion.rs`

Those primitives still return `AnyElement` or `Option<AnyElement>` because they own the final
tabs/accordion semantics wrapper, pressable wrapper, or content mount gate boundary, but their
public child closures now accept iterable `IntoUiElement<H>` values directly. The roving-list,
pressable, and content wrappers land those typed values behind `collect_children(...)` instead of
publishing raw `IntoIterator<Item = AnyElement>` child items on the public surface.

The same split now also applies to dialog/popover/alert-dialog/select/tooltip overlay helpers in `fret-ui-kit`:

- `ecosystem/fret-ui-kit/src/primitives/alert_dialog.rs`
- `ecosystem/fret-ui-kit/src/primitives/dialog.rs`
- `ecosystem/fret-ui-kit/src/primitives/popover.rs`
- `ecosystem/fret-ui-kit/src/primitives/select.rs`
- `ecosystem/fret-ui-kit/src/primitives/tooltip.rs`

Wrapper helpers that still receive an `ElementContext` now accept iterable `IntoUiElement<H>`
values directly and land them behind `collect_children(...)` before assembling barriers, semantics
wrappers, and layer element vectors. The final overlay-request constructors still accept
`IntoIterator<Item = AnyElement>` because they are the explicit landing seam that no longer has a
live `ElementContext` available for typed conversion.

The same wrapper rule also now applies to the sortable DnD recipe helper in `fret-ui-kit`:

- `ecosystem/fret-ui-kit/src/recipes/sortable_dnd.rs`

Its row-content closure now accepts iterable `IntoUiElement<H>` values directly and lands them
behind `collect_children(...)` inside the row container wrapper instead of publishing raw
`IntoIterator<Item = AnyElement>` child items on the public surface.

The same wrapper rule now also applies to the virtualized table helpers in `fret-ui-kit`:

- `ecosystem/fret-ui-kit/src/declarative/table.rs`

`table_virtualized(...)` and `table_virtualized_copyable(...)` now accept iterable
`IntoUiElement<H>` values for header and cell render closures directly. Their header-cell landing
now happens behind `collect_children(...)` inside the table-owned container wrapper instead of
publishing raw `IntoIterator<Item = AnyElement>` child items on the public surface.

The same wrapper rule also applies to internal gallery scaffolds:

- `src/ui/doc_layout.rs::demo_shell<B>(...)`
- `src/ui/previews/pages/editors/code_editor/mvp/gates.rs::gate_panel<B>(...)`

Current intentional raw doc-layout exceptions:

- `src/ui/doc_layout.rs::DocSection.preview` remains a landed `AnyElement` field because the docs
  scaffold still decorates preview roots, shells, and tab panels after section assembly.
- `src/ui/doc_layout.rs::gap_card(...)` remains a tuple-return raw seam because placeholder
  sections still register a concrete landed preview value alongside the section title.

The rest of the doc-layout helper family now stays on the typed lane:

- `src/ui/doc_layout.rs::{render_doc_page,wrap_preview_page,icon,render_section,preview_code_tabs,code_block_shell,section_title}`
  now return `impl UiChild` while still landing concrete child vectors internally where needed
  (page aggregation, preview harness vectors, decorated tab panels, code-block chrome, and title
  decoration).
- `src/ui/doc_layout.rs::render_doc_page(...)` intentionally still aggregates
  `Vec<AnyElement>` internally because the centered docs shell still owns the final section-body
  assembly step after the typed helper surface has already converged.
- `src/ui/doc_layout.rs::wrap_preview_page(..., elements: Vec<AnyElement>)` intentionally keeps
  the preview-root vector input explicit because the internal preview registry still dispatches
  concrete preview vectors rather than a single typed wrapper value.
- `src/ui/doc_layout.rs::{wrap_row,wrap_controls_row}` intentionally keep
  `FnOnce(&mut UiCx<'_>) -> Vec<AnyElement>` child closures because the shared flex-row scaffold
  still owns a heterogeneous late-child collection seam.
- internal preview pages now keep the final landing explicit when consuming
  `wrap_preview_page(...)`; see
  `apps/fret-ui-gallery/tests/ui_authoring_surface_internal_previews.rs::wrap_preview_page_callers_land_the_typed_preview_shell_explicitly`.
- both default-app pages and internal previews now keep the final `render_doc_page(...)` landing
  explicit at the caller side; see
  `apps/fret-ui-gallery/tests/ui_authoring_surface_default_app.rs::render_doc_page_callers_land_the_typed_doc_page_explicitly`
  and
  `apps/fret-ui-gallery/tests/ui_authoring_surface_internal_previews.rs::render_doc_page_callers_land_the_typed_doc_page_explicitly`.

Current intentional internal preview registry seam:

- `src/ui/previews/**::preview_*` remains `-> Vec<AnyElement>` because the preview registry still
  dispatches concrete page-root vectors rather than a single typed wrapper value.
- this is an internal registration boundary, not the taught authoring surface:
  page-local helpers, `DocSection::build(...)`, and `wrap_preview_page(...)` / `render_doc_page(...)`
  stay on typed helper signatures, with explicit landing occurring only at the registry return
  seam.
- until the preview registry API itself changes, do not "fix" these entry points by forcing them
  to `-> AnyElement`; keep the registry seam explicit and keep the local helper/page layers typed.

Current intentional raw internal-overlay-preview exceptions:

- `src/ui/previews/gallery/overlays/overlay.rs::preview_overlay(...)` remains
  `-> Vec<AnyElement>` because the preview still assembles cached overlay roots plus status labels
  as a concrete diagnostics result vector.
- `src/ui/previews/gallery/overlays/overlay/layout.rs::{row,row_end,compose_body}` are now back
  on `UiCx -> impl UiChild + use<>`; they now compose typed widget helpers directly, and the
  explicit landing now lives at the cached-preview seam in
  `overlay.rs`.
- `src/ui/previews/gallery/overlays/overlay/widgets.rs::{overlay_reset,dropdown,context_menu,context_menu_edge,underlay,tooltip,hover_card,popover,dialog,dialog_glass,alert_dialog,sheet,portal_geometry}`
  are now back on typed helper signatures; some helpers still lower to concrete overlay/provider
  roots internally because the current shadcn overlay APIs land roots eagerly, but that is now an
  ecosystem-layer follow-up rather than a preview-surface contract leak.
- `src/ui/previews/gallery/overlays/overlay/flags.rs::last_action_status(...)` is now back on a
  typed helper signature, with the explicit landing moved to `overlay.rs`.
- `src/ui/previews/gallery/overlays/overlay/flags.rs::status_flags(...)` remains
  `-> Vec<AnyElement>` because the conditional status labels are still appended directly onto the
  diagnostics result vector after model reads and `test_id` decoration.

Current intentional raw scroll-area diagnostics exceptions:

- `src/ui/snippets/scroll_area/drag_baseline.rs::render(...)` remains `-> AnyElement` because the
  harness owns timer-driven content growth, a retained `ScrollHandle`, explicit scrollbar
  semantics, and the final landed diagnostics root consumed by `pages/scroll_area.rs` through
  `DocSection::build(cx, ...)`.
- `src/ui/snippets/scroll_area/expand_at_bottom.rs::render(...)` remains `-> AnyElement` because
  the harness owns the pinned-extents regression probe, wrapper-budget stress tree, and the final
  landed diagnostics root consumed by `pages/scroll_area.rs` through `DocSection::build(cx, ...)`.
- those two files are now the entire audited raw-root inventory for
  `src/ui/snippets/scroll_area/**`; do not add another `render(...) -> AnyElement` there unless
  the snippet is genuinely a diagnostics-owned harness root.
- `apps/fret-ui-gallery/tests/ui_authoring_surface_default_app.rs::scroll_area_diagnostics_snippets_remain_intentional_raw_boundaries`
  is the source gate that keeps that exact two-file raw inventory explicit while the ordinary
  scroll-area docs surface stays on `UiCx -> impl UiChild`.

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
- `ecosystem/fret-ui-shadcn/src/text_edit_context_menu.rs::{text_edit_context_menu,
  text_selection_context_menu, text_edit_context_menu_controllable,
  text_selection_context_menu_controllable}` remain `-> AnyElement` because
  `ContextMenu::build(...)` / `ContextMenu::new_controllable(...).build(...)` are themselves the
  final wrapper landing seams: the helper evaluates a typed trigger and injects a fixed entry set
  in one root overlay call.

Implementation fallback rule:

- if an ecosystem builder or recipe type does not yet implement `IntoUiElement<H>` directly,
  it is still acceptable for the helper to land internally with `.into_element(cx)` and expose the
  result as `impl IntoUiElement<H>`.
- keep that fallback inside the helper so callers still see the unified conversion contract rather
  than a raw `AnyElement` signature.
- current positive example on 2026-03-14:
  `ecosystem/fret-ui-shadcn::{DialogComposition<H, _>,AlertDialogComposition<H, _>,SheetComposition<H, _>,DrawerComposition<H, _>,DropdownMenuComposition<H, _>,ContextMenuComposition<H, _>}`
  now implement
  `IntoUiElement<H>` directly, so extracted helpers can return typed menu roots without paying an
  eager root landing step.

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

Current first-party deliberate raw helper contracts on the shadcn lane are therefore limited to:

- `ecosystem/fret-ui-shadcn/src/kbd.rs::kbd_icon<H>(...)`
- `ecosystem/fret-ui-shadcn/src/text_edit_context_menu.rs::{text_edit_context_menu,
  text_selection_context_menu, text_edit_context_menu_controllable,
  text_selection_context_menu_controllable}`
