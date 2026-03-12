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
| M3 | In progress | curated `fret` / `fret-ui-kit` surfaces and the canonical todo/scaffold compare set are aligned; `fret::UiChild` now lands directly through `IntoUiElement<App>`; `fret-ui-shadcn` ui_ext glue, `ui_builder_ext` helper closures, overlay/single-child builders, and `fret-router-ui` outlet helpers now land through `IntoUiElement<H>`; selected advanced examples (`assets_demo`, `async_playground_demo`, `custom_effect_v1_demo`, `custom_effect_v2_demo`, `custom_effect_v3_demo`, `postprocess_theme_demo`, `drop_shadow_demo`, `markdown_demo`, `liquid_glass_demo`, `customv1_basics`, `drop_shadow_basics`, `icons_and_assets_basics`, `hello_world_compare_demo`) now also prefer `impl IntoUiElement<...>` for non-raw helpers, including `custom_effect_v1_demo::{stage,lens_row,plain_lens,custom_effect_lens,lens_shell,inspector}`, `custom_effect_v2_demo::{stage,lens_row,plain_lens,custom_effect_lens,lens_shell,inspector}`, `async_playground_demo::{header_bar,body,catalog_panel,main_panel,inspector_panel,policy_editor,query_panel_for_mode,query_inputs_row,query_result_view,status_badge}`, `custom_effect_v3_demo::{stage,stage_controls,animated_backdrop,lens_row,lens_shell}`, and `postprocess_theme_demo::{inspector,stage,stage_body,stage_cards}`; selected default-app WebGPU examples now also keep typed helper signatures, including `custom_effect_v2_identity_web_demo::{lens,inspector}`, `custom_effect_v2_web_demo::{lens,inspector}`, `custom_effect_v2_lut_web_demo::{lens,inspector}`, and `custom_effect_v2_glass_chrome_web_demo::{label_row,lens,controls_panel}`, while keeping explicit raw seams such as the internal body landing inside `custom_effect_v1_demo::lens_shell(...)` / `custom_effect_v2_demo::lens_shell(...)`, stage-tile child-array assembly in the WebGPU demos, and the retained bridge seam `chart_interactions_basics::chart_canvas(...)`; selected UI Gallery AI doc pages now keep page-local helpers on `impl UiChild + use<>`, selected UI Gallery badge snippets now keep local `row(...)` helpers on `impl IntoUiElement<H> + use<H, F>`, selected UI Gallery avatar snippets now keep row wrappers, avatar builders, and icon/group helpers on `impl IntoUiElement<H> + use<...>`, selected UI Gallery button snippets now keep row wrappers and local size-composition helpers on `impl IntoUiElement<H> + use<...>`, selected UI Gallery tabs snippets now keep local `field(...)` helpers on `impl IntoUiElement<fret_app::App> + use<>`, selected UI Gallery collapsible snippets now keep `rotated_lucide(...)`, `radius_input(...)`, `details_collapsible(...)`, `file_leaf(...)`, and `folder(...)` on `impl IntoUiElement<H> + use<H>`, selected UI Gallery hover-card snippets now keep `card(...)` / `demo_content(...)` helpers on `impl IntoUiElement<H> + use<H>`, selected UI Gallery tooltip snippets now keep `make_tooltip(...)` / `make_tooltip_with_test_ids(...)` on `impl IntoUiElement<H> + use<H>`, selected UI Gallery resizable snippets now keep `panel(...)` / `box_group(...)` helpers on `impl IntoUiElement<H> + use<...>`, selected UI Gallery data-table snippets now keep `align_end(...)`, `align_inline_start(...)`, `footer(...)`, and `bottom_controls(...)` on `impl IntoUiElement<fret_app::App> + use<...>`, selected UI Gallery table-action snippets now keep `align_end(...)` and `action_row(...)` on `impl IntoUiElement<fret_app::App> + use<...>`, selected UI Gallery table snippets now keep `make_invoice_table(...)` on `impl IntoUiElement<fret_app::App> + use<>`, selected UI Gallery separator snippets now keep `section(...)` / `row(...)` helpers on `impl IntoUiElement<H> + use<H>`, selected UI Gallery sidebar snippets now keep `menu_button(...)` helpers on `impl IntoUiElement<...>`-based signatures across `sidebar/{demo,controlled,mobile,rtl}.rs`, selected UI Gallery aspect-ratio snippets now keep `portrait_image(...)`, `square_image(...)`, `rtl_image(...)`, and `ratio_example(...)` helpers on `impl IntoUiElement<H> + use<H>`, selected UI Gallery context-menu snippets now keep `trigger_surface(...)` helpers on `impl IntoUiElement<H>` with explicit trigger landing seams, selected UI Gallery combobox snippets now keep local `state_row(...)` and `state_rows(...)` helpers on `impl IntoUiElement<fret_app::App> + use<>`, selected UI Gallery pagination snippets now keep local `page_number(...)` helpers on `impl IntoUiElement<H> + use<H>`, selected UI Gallery carousel snippets now keep local `slide_card(...)` / `slide(...)` helpers on `impl IntoUiElement<fret_app::App> + use<>`, including the `api`, `duration_embla`, `rtl`, `plugin_autoplay*`, and `events` demos, selected UI Gallery skeleton snippets now keep local `round(...)` / `row(...)` helpers on `impl IntoUiElement<H> + use<H>`, selected UI Gallery popover wrapper helpers now accept/return `IntoUiElement<H>` instead of forcing `AnyElement`, selected UI Gallery dropdown-menu preview wrappers now accept/return `IntoUiElement<H>`, selected UI Gallery AI wrapper/doc-preview helpers now also accept or expose `IntoUiElement<H>`-based signatures (`centered(...)`, `preview(...)`, `progress_section(...)`, `render_grid_attachment(...)`, `render_list_attachment(...)`, `invisible_marker(...)`, `body_text(...)`, and `clear_action(...)`), including `file_tree_large.rs::preview(...)`; internal gallery wrapper shells now also keep typed wrapper seams in `doc_layout.rs::demo_shell<B>(...)` and `code_editor/mvp/gates.rs::gate_panel<B>(...)`; `fret-ui-shadcn` internal menu-slot wrappers in `context_menu.rs`, `dropdown_menu.rs`, and `menubar.rs` now also accept `IntoUiElement<H>` inputs on `menu_icon_slot(...)`; the thin public constructor/wrapper trial now covers `badge.rs::badge<H, T>(...)`, `kbd.rs::kbd<H, T>(...)`, `separator.rs::separator<H>()`, `input_group.rs::input_group<H>(...)`, `input_otp.rs::input_otp<H>(...)`, and `command.rs::command<H, I, F, T>(...)`, while `kbd.rs::kbd_icon<H>(...)` remains intentionally raw because `Kbd::from_children(...)` still owns a concrete `Vec<AnyElement>` child seam; the dedicated typography sweep is now landed, so `fret-ui-shadcn/src/typography.rs` keeps the `raw::typography::*` namespace but exposes typed helper outputs and first-party Gallery/examples/`fret-genui-shadcn` call sites now land those helpers explicitly via `.into_element(cx)` only where a concrete `AnyElement` seam is still required; selected breadcrumb helpers now keep separators on `IntoUiElement<H>`, selected button-group, toggle-group, and drawer helpers now expose `IntoUiElement`-based signatures, including `drawer/{demo,responsive_dialog,sides,scrollable_content}.rs`, selected UI Gallery sheet/dialog snippets now keep `profile_fields(...)` helpers on `impl IntoUiElement<H> + use<H>`, selected UI Gallery dialog scroll/sticky snippets now keep `lorem_block(...)` helpers on `impl IntoUiElement<H> + use<H>`, and selected item, toast, and motion-presets helpers now also stay on `IntoUiElement`-based signatures, including `item/{avatar,icon,link,link_render,dropdown,extras_rtl,gallery}.rs` helpers such as `icon(...)`, `icon_button(...)`, `outline_button(...)`, `outline_button_sm(...)`, `item_basic(...)`, `item_icon(...)`, `item_avatar(...)`, and `item_team(...)`; broader shadcn/gallery/helper cleanup still remains |
| M4 | In progress | prelude gates are in place, curated component-authoring docs now teach only `IntoUiElement<H>`, stale-name source/doc guards now cover curated docs, `UiChildIntoElement` is now deleted from code, `fret_ui_shadcn::prelude::*` now re-exports `IntoUiElement` so typed direct-crate helpers do not need ad-hoc trait imports, and the focused UI Gallery source gate now covers 32 `selected_*` helper assertions across AI pages/snippets, avatar/button wrappers and builders, tabs form helpers, collapsible tree/settings helpers, drawer/sheet/dialog form helpers plus dialog scroll/sticky content helpers, separator/table wrappers, sidebar menu helpers, aspect-ratio helpers, popover/resizable wrappers, hover-card/tooltip overlays, data-table/table-action helpers, dropdown/context-menu wrappers, combobox `state_row(...)`/`state_rows(...)` helpers, carousel helpers spanning `api`/autoplay/events/rtl/duration demos, item helpers spanning `dropdown`/`gallery`/`extras_rtl` variants, pagination/skeleton helpers, and other first-party authoring surfaces; `UiIntoElement` still survives as internal doc-hidden scaffolding, so the delete phase is not complete yet |

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
