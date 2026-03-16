# Into-Element Surface (Fearless Refactor v1) — Milestones

This file defines milestones for the workstream in `DESIGN.md`.

## Current execution stance (2026-03-15)

This workstream should now be read as a **closeout / maintenance lane**, not the current main
authoring lane.

Closeout note on 2026-03-15:

- the broad `IntoUiElement<H>` migration is now effectively landed across the first-party compare
  set and the `fret-ui-kit` / Gallery wrapper-helper surfaces,
- the remaining raw `AnyElement` seams on this lane are now intentionally classified
  request/advanced seams rather than unresolved migration debt,
- follow-up work here should stay narrow: keep the explicit seam inventory accurate, keep source
  gates aligned, and avoid reopening broad root-builder invention unless one of those audited
  boundaries actually changes.

Historical readout on 2026-03-14:

| Milestone | State | Notes |
| --- | --- | --- |
| M0 | Done | target vocabulary is locked and the classification table is now recorded in `MIGRATION_MATRIX.md` |
| M1 | Done | `IntoUiElement<H>` is the curated component conversion name; docs/preludes/tests reflect it |
| M2 | Done | `UiBuilder<T>` and host-bound child builders now land through `IntoUiElement<H>`; `UiBuilderHostBoundIntoElementExt` is deleted; child collection now also consumes `IntoUiElement<H>` directly |
| M3 | Done | surface migration is landed across the curated app/component lanes, the canonical todo/scaffold compare set, and the first-party Gallery/doc scaffolds; the previously ambiguous families are classified (`Select` / `Combobox` / `Command` => direct recipe root/bridge, `NavigationMenu` / `Pagination` => dual-lane), the high-signal `Select` / `Combobox` snippets prefer the compact direct root builder chains instead of teaching closure-based `into_element_parts(...)`, the default-app UI Gallery `pages/` + `snippets/` surface is closed on the target posture (no `DocSection::new(...)`, no default snippet `UiCx -> AnyElement`, only intentional diagnostics/raw roots remain), and the internal preview page surface is also off `DocSection::new(...)`; remaining work is maintenance of the audited intentional seam inventory rather than active migration |
| M4 | Done | prelude gates are in place, curated component-authoring docs teach only `IntoUiElement<H>`, stale-name source/doc guards cover the curated docs, `UiChildIntoElement` / `UiIntoElement` / `UiBuilderHostBoundIntoElementExt` are deleted from production code, `fret_ui_shadcn::prelude::*` re-exports `IntoUiElement` so typed direct-crate helpers do not need ad-hoc trait imports, exported `fret-ui-kit` adapter macros plus built-in primitive glue attach `IntoUiElement<H>` directly, built-in text primitives and declarative semantics wrappers land through `IntoUiElement<H>` directly, and current follow-up is limited to keeping historical design notes accurate rather than closing code-level migration gaps |
| M6 | Done | the shadcn raw-seam inventory is now explicitly closed: `use_combobox_anchor(...)` is deleted in favor of `PopoverAnchor::build(...).into_anchor(cx)`, `TooltipContent::{build,text}(...)` and `state.rs::{use_selector_badge,query_status_badge,query_error_alert}` are back on the typed lane, and the only remaining deliberate raw helper contracts on that lane are `kbd.rs::kbd_icon(...)` plus the final-wrapper `text_edit_context_menu*` family, both guarded by source-policy tests and reflected in `TARGET_INTERFACE_STATE.md` / `MIGRATION_MATRIX.md` |

Implementation addendum on 2026-03-14:

- the workstream now explicitly recognizes three first-party family lanes:
  compose-root default lane, dual-lane families, and direct recipe root/bridge families.
- the overlay/menu teaching surface is largely stable after the current sweep:
  `DropdownMenu`, `ContextMenu`, `Dialog`, `Sheet`, `AlertDialog`, and `Drawer` now have an
  explicit default copyable root story, while `Carousel` and `Menubar` are recorded as dual-lane
  families rather than "default vs advanced" splits.
- the current follow-up classification is:
  `Select`, `Combobox`, and `Command` stay on the direct recipe root/bridge lane;
  `NavigationMenu` and `Pagination` are dual-lane families.
- the `Select` / `Combobox` follow-up has now advanced from classification to copyable default
  authoring:
  `Select` first-party snippets now prefer the compact
  `.trigger(...).value(...).content(...).entries(...)` chain,
  `Combobox` gained root-lane builder steps for
  `.trigger(...).input(...).clear(...).content(...)`,
  the `Select` gallery snippet family is now fully off `into_element_parts(...)`,
  the direct-root `Combobox::new(...)` snippets are now also off `into_element_parts(...)`,
  `ComboboxChips` now also exposes matching compact root builder steps,
  and the focused UI Gallery source gate now locks the whole first-party `combobox` snippet
  folder onto compact-chain stories without inventing a `compose()` lane.
- the `Command` direct-root lane is now also explicitly locked on the first-party teaching
  surface:
  `apps/fret-ui-gallery/src/ui/snippets/command/*.rs` now has a source gate forbidding
  `CommandInput::new(...)` and `CommandList::new(...)`, so the default snippet surface cannot
  silently drift back to split root authoring before a shared context contract exists.
- the form-input authoring lane is now more explicit too:
  `InputGroup` is recorded as a dual-lane family, the remaining `input_group/dropdown.rs`
  first-party snippet now uses the compact `InputGroup::new(model)` slot shorthand rather than
  `into_element_parts(...)`, and the page copy now keeps the compact shorthand as the first-party
  ergonomic lane while preserving parts as the direct docs-parity lane.
- `InputOtp` is now also locked onto a compact default root story:
  the first-party `apps/fret-ui-gallery/src/ui/snippets/input_otp/*.rs` surface now prefers
  `InputOTP::new(model)` plus `length(...)` and optional `group_size(...)`, the focused gallery
  source gate forbids `into_element_parts(...)` drift on that default lane, and
  `ecosystem/fret-ui-shadcn/src/input_otp.rs` now also has a compact-lane unit test for
  auto-inserted separators via `group_size(Some(...))`.
- the `Carousel` dual-lane surface is now source-aligned too:
  the first-party docs-first snippets
  `usage.rs`,
  `basic.rs`,
  `sizes_thirds.rs`,
  `sizes.rs`,
  `spacing.rs`,
  `spacing_responsive.rs`,
  `orientation_vertical.rs`,
  `options.rs`,
  and `loop_carousel.rs`
  now all stay on the compact `Carousel::new(items)` builder lane, while `parts.rs` remains the
  upstream-shaped copyable lane.
  A follow-up tightening pass moved the ordinary diagnostics snippets
  `api.rs`,
  `demo.rs`,
  `duration_embla.rs`,
  `expandable.rs`,
  `focus_watch.rs`,
  `loop_downgrade_cannot_loop.rs`,
  `plugin_autoplay.rs`,
  `plugin_autoplay_controlled.rs`,
  `plugin_autoplay_delays.rs`,
  `plugin_autoplay_stop_on_focus.rs`,
  `plugin_autoplay_stop_on_last_snap.rs`,
  and `plugin_wheel_gestures.rs`
  onto that same compact builder lane, leaving only `parts.rs` plus the custom-control
  diagnostics snippets `events.rs` and `rtl.rs` on the explicit parts lane.
  The focused gallery source gate now locks that split instead of relying only on the page notes.
- the internal preview registry seam is now explicitly classified as intentional rather than
  migration debt:
  `apps/fret-ui-gallery/src/ui/previews/**::preview_*` remains `-> Vec<AnyElement>` because the
  registry still dispatches concrete preview-root vectors, while the page-local preview helpers,
  `DocSection::build(...)`, and `wrap_preview_page(...)` / `render_doc_page(...)` stay on the
  typed lane.
- the selected first-party shadcn family lanes now also have a curated export completeness guard:
  `surface_policy_tests::authoring_critical_family_exports_stay_on_root_and_curated_facade`
  locks the crate root and `facade` exports for `Select`, `Combobox`, `ComboboxChips`, `Command`,
  `NavigationMenu`, and `Pagination`, so newly added builder parts do not strand the curated
  `shadcn::...` namespace behind root-only exports.
- there is no remaining ambiguous family on the current shadcn focus lane; future families should
  declare their lane in the audit/tracker before new root APIs are added.
- the corresponding first-party page notes are now source-gated too:
  `ui_authoring_surface_default_app::direct_recipe_root_pages_mark_their_default_lane_without_inventing_compose`
  and
  `ui_authoring_surface_default_app::navigation_menu_and_pagination_pages_keep_their_dual_lane_story`
  now lock the page-level teaching surface for those classifications.

Execution addendum on 2026-03-14:

- `apps/fret-ui-gallery/src/ui/snippets/ai/speech_input_demo.rs::{body_text,clear_action}` now use
  the default-app `UiCx -> impl UiChild` helper posture instead of host-generic helper spellings.
- `apps/fret-ui-gallery/src/ui/doc_layout.rs::{wrap_row,wrap_controls_row,text_table,muted_full_width,muted_inline}` and the local
  `notes_block(... )::muted_flex_1_min_w_0(...)` helper now also use `UiCx -> impl UiChild`,
  while `demo_shell<B>(...)` remains the explicit typed wrapper seam for late landing.
- the remaining raw doc-layout seams are now explicitly classified and gated:
  `render_doc_page(...)`, `wrap_preview_page(...)`, `icon(...)`, `render_section(...)`,
  `preview_code_tabs(...)`, `code_block_shell(...)`, and `section_title(...)` now all return typed
  helpers instead of exposing raw `AnyElement` signatures. The remaining intentional raw storage on
  this lane is the landed `DocSection.preview` field plus the tuple-return `gap_card(...)`
  placeholder seam. `gallery_doc_layout_retains_only_intentional_raw_boundaries` now prevents
  doc-layout drift on the default app-facing lane,
  `render_doc_page_callers_land_the_typed_doc_page_explicitly` locks explicit landing on the page
  lane, and `wrap_preview_page_callers_land_the_typed_preview_shell_explicitly` plus
  `render_doc_page_callers_land_the_typed_doc_page_explicitly` lock the explicit landing seams on
  the internal preview lane.
- validation addendum on 2026-03-14:
  `CARGO_TARGET_DIR=target/codex-ui-gallery cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app gallery_doc_layout_app_helpers_prefer_ui_child_over_anyelement -- --exact --nocapture`
- validation addendum on 2026-03-14:
  `CARGO_TARGET_DIR=target/codex-ui-gallery cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app gallery_doc_layout_retains_only_intentional_raw_boundaries -- --exact --nocapture`
- validation addendum on 2026-03-14:
  `CARGO_TARGET_DIR=target/codex-ui-gallery cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app render_doc_page_callers_land_the_typed_doc_page_explicitly -- --exact --nocapture`
- validation addendum on 2026-03-14:
  `CARGO_TARGET_DIR=target/codex-ui-gallery cargo test -p fret-ui-gallery --test ui_authoring_surface_internal_previews wrap_preview_page_callers_land_the_typed_preview_shell_explicitly -- --exact --nocapture`
- validation addendum on 2026-03-14:
  `CARGO_TARGET_DIR=target/codex-ui-gallery cargo test -p fret-ui-gallery --test ui_authoring_surface_internal_previews render_doc_page_callers_land_the_typed_doc_page_explicitly -- --exact --nocapture`
- validation addendum on 2026-03-14:
  `CARGO_TARGET_DIR=target/codex-ui-gallery cargo test -p fret-ui-gallery --test ui_authoring_surface_internal_previews internal_preview_scaffold_retains_only_the_audited_vec_anyelement_seams -- --exact --nocapture`
- the AI page surface now also closes its remaining section-registration tail:
  `apps/fret-ui-gallery/src/ui/pages/ai_*.rs` now use `DocSection::build(cx, ...)` instead of
  `DocSection::new(...)` for first-party demo sections, and
  `curated_ai_doc_pages_use_typed_doc_sections` now locks that default-app docs posture.
- the internal overlay preview lane now also has an audited retained-seam inventory:
  `src/ui/previews/gallery/overlays/overlay.rs`,
  `overlay/layout.rs`,
  `overlay/widgets.rs`, and
  `overlay/flags.rs` now split between typed helper shells and intentional retained seams:
  `layout.rs::{row,row_end,compose_body}` plus `flags.rs::last_action_status(...)` now expose
  `UiCx -> impl UiChild + use<>`, `widgets.rs` now keeps all widget helpers on the same typed lane
  without an `OverlayWidgets` landed inventory, while `preview_overlay(...)` and `status_flags(...)`
  remain raw only as the current diagnostics composition boundary, and
  `gallery_overlay_preview_retains_intentional_raw_boundaries` now prevents accidental drift
  while ecosystem-level overlay root APIs continue to migrate toward a more uniform typed
  composition story.
- ecosystem addendum on 2026-03-14:
  `DropdownMenu::compose()` and `ContextMenu::compose()` now exist as typed root builders on the
  shadcn layer, both implementing `IntoUiElement<H>` directly; the canonical gallery usage
  snippets and the overlay diagnostics helpers now use that path, so menu roots no longer have to
  pay an immediate `build_parts(...) -> AnyElement` cliff just to stay on the typed authoring lane.
- overlay-family addendum on 2026-03-14:
  `DialogComposition`, `AlertDialogComposition`, `SheetComposition`, and `DrawerComposition` now
  also implement `IntoUiElement<H>` directly, and the gallery overlay diagnostics helpers for
  dialog / alert-dialog / sheet now return those typed composition builders instead of calling the
  eager root `into_element(...)` closures inside the helper body.
- snippet-surface addendum on 2026-03-14:
  the first-party gallery menu snippets now use `compose()` as the default root teaching path
  across `dropdown_menu/*`, `context_menu/*`, and `avatar/dropdown.rs`; only
  `dropdown_menu/parts.rs` intentionally continues to show the lower-level `build_parts(...)`
  adapter surface.
- validation addendum on 2026-03-14:
  `CARGO_TARGET_DIR=target/codex-ui-gallery cargo test -p fret-ui-gallery --test ui_authoring_surface_internal_previews gallery_overlay_preview_retains_intentional_raw_boundaries -- --exact --nocapture`
- validation addendum on 2026-03-14:
  `CARGO_TARGET_DIR=target/codex-ui-gallery cargo test -p fret-ui-gallery --test ui_authoring_surface_internal_previews gallery_overlay_preview_modules_prefer_ui_cx_on_the_internal_gallery_surface -- --exact --nocapture`
- the notification teaching lane now also aligns with the typed helper posture:
  `src/ui/snippets/toast/deprecated.rs` now returns `impl UiChild + use<>`,
  `src/ui/pages/toast.rs` now uses `DocSection::build(cx, ...)`, and
  `src/ui/snippets/sonner/mod.rs::local_toaster(...)` now stays on `UiChild` with the explicit
  `.into_element(cx)` step living only at `pages/sonner.rs`.
- the internal code-editor MVP preview lane now also closes a helper-level raw tail:
  `src/ui/previews/pages/editors/code_editor/mvp/{header,word_boundary,gates}.rs` now keep
  `build_header(...)`, `word_boundary_controls(...)`, `word_boundary_debug_view(...)`,
  `gate_panel(...)`, and the `*_gate(...)` helpers on `impl UiChild + use<>`, with the explicit
  landing step moved back to
  `src/ui/previews/pages/editors/code_editor/mvp.rs`.
- a few lower-traffic internal preview helpers now follow the same posture:
  `src/ui/previews/pages/editors/text/outline_stroke.rs::toggle_button(...)`,
  `src/ui/previews/pages/editors/text/mixed_script_fallback.rs::sample_row(...)`,
  `src/ui/previews/pages/editors/text/feature_toggles.rs::{toggle_button,sample_text}(...)`, and
  `src/ui/previews/pages/harness/intro.rs::{card(...),preview_intro(...)}` now avoid eager
  `AnyElement` teaching where no real landing seam is intended.
- validation addendum on 2026-03-14:
  `CARGO_TARGET_DIR=target/codex-ui-gallery cargo test -p fret-ui-gallery --test ui_authoring_surface_internal_previews code_editor_mvp_internal_helpers_prefer_ui_child_over_anyelement -- --exact --nocapture`
- validation addendum on 2026-03-14:
  `CARGO_TARGET_DIR=target/codex-ui-gallery cargo test -p fret-ui-gallery --test ui_authoring_surface_internal_previews selected_internal_preview_helpers_prefer_typed_outputs -- --exact --nocapture`
- validation addendum on 2026-03-14:
  `CARGO_TARGET_DIR=target/codex-ui-gallery cargo test -p fret-ui-gallery --test ui_authoring_surface_internal_previews editor_preview_modules_prefer_ui_cx_on_the_internal_gallery_surface -- --exact --nocapture`

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
  helpers as `impl UiChild`, drops helper-local `cx` from `todo_page(...)`, and lands those page
  shells at the render root through `ui::single(cx, todo_page(...))` in the emitted templates.
- the same first-contact cleanup now also reaches the onboarding and counter samples:
  `apps/fret-cookbook/examples/hello.rs::hello_page(...)` and
  `apps/fret-examples/src/hello_counter_demo.rs::hello_counter_page(...)` now both stay on
  `impl UiChild`, drop helper-local `cx`, and let `render(...)` land them through
  `ui::single(cx, ...)` instead of helper-local `.into_element(cx)`.
- the same page-shell cleanup now also reaches the shared cookbook scaffold:
  `apps/fret-cookbook/src/scaffold.rs::{centered_page,centered_page_background,centered_page_muted}`
  now accept `IntoUiElement<H>` directly, keep `AnyElement` only as the named final page-root
  landing seam, and delete the parallel `centered_page*_ui(...)` overload family from the
  authoring surface.
- the same ecosystem helper cleanup now also reaches `fret-router-ui`:
  `router_outlet(...)`, `router_outlet_with_test_id(...)`, and
  `RouterOutlet::{into_element,into_element_by_leaf,into_element_by_leaf_with_status}` now all
  accept `IntoUiElement<App>` outputs directly on the single named outlet surface, while the
  temporary `RouterOutletIntoElement` adapter and every outlet `*_ui(...)` overload are deleted.
  `apps/fret-cookbook/examples/router_basics.rs` now teaches `.into_element_by_leaf(...)`
  directly.
- the same crate now also closes the router-link child surface:
  `router_link*` helpers accept iterable `IntoUiElement<App>` children directly instead of
  publishing `IntoIterator<Item = AnyElement>`, and the crate-level source gate now locks that
  public surface so link helpers do not regress back to raw child item types.
- the same helper cleanup now also reaches `fret-ui-magic`:
  `magic_card`, `border_beam`, `lens`, `marquee`, `dot_pattern`, `grid_pattern`, `stripe_pattern`,
  `dock`, and `sparkles_text` now accept iterable `IntoUiElement<H>` children directly while
  keeping `AnyElement` only as the final visual wrapper seam. The crate-level `collect_children(...)`
  helper and source gate now lock that policy.
- the same visual-wrapper cleanup now also reaches `fret-ui-kit` declarative effect panels:
  `ecosystem/fret-ui-kit/src/declarative/{bloom,pixelate}.rs` now keep `AnyElement` only as the
  final effect-layer wrapper seam, while `bloom_panel(...)` and `pixelate_panel(...)` accept
  iterable `IntoUiElement<H>` children directly. The shared crate-local `collect_children(...)`
  helper and a focused source-policy gate now lock that posture.
- the same wrapper cleanup now also reaches `fret-ui-kit` policy wrappers:
  `ecosystem/fret-ui-kit/src/declarative/{dismissible,visually_hidden}.rs` and
  `ecosystem/fret-ui-kit/src/primitives/focus_scope.rs::{focus_trap,focus_trap_with_id}` now keep
  `AnyElement`/`NodeId` only as the final dismissible-root or focus/a11y wrapper seam, while their
  child closures accept iterable `IntoUiElement<H>` values directly. The same shared
  `collect_children(...)` helper and the expanded source-policy gate now lock that posture.
- the same wrapper cleanup now also reaches the scroll / roving policy batch in `fret-ui-kit`:
  `ecosystem/fret-ui-kit/src/declarative/scroll.rs`,
  `ecosystem/fret-ui-kit/src/primitives/roving_focus_group.rs`,
  `ecosystem/fret-ui-kit/src/primitives/toolbar.rs`, and
  `ecosystem/fret-ui-kit/src/primitives/dismissable_layer.rs` now keep `AnyElement`/`NodeId` only
  as the final scroll, roving-container, toolbar, or dismissable-root seam, while their child
  closures accept iterable `IntoUiElement<H>` values directly. Direct wrappers land through the
  shared `collect_children(...)` helper, and delegate wrappers now forward only to already-typed
  helper seams. The expanded source-policy gate and `cargo check -p fret-ui-kit --lib` lock that
  posture.
- the same wrapper cleanup now also reaches the layout/effect query batch in `fret-ui-kit`:
  `ecosystem/fret-ui-kit/src/declarative/chrome.rs`,
  `ecosystem/fret-ui-kit/src/declarative/glass.rs`, and
  `ecosystem/fret-ui-kit/src/declarative/container_queries.rs` now keep `AnyElement` only as the
  final control-chrome, effect-layer, or layout-query wrapper seam, while their child closures
  accept iterable `IntoUiElement<H>` values directly and land through the shared
  `collect_children(...)` helper. The same expanded source-policy gate and
  `cargo check -p fret-ui-kit --lib` lock that posture.
- the same wrapper cleanup now also reaches the menu/popup skeleton batch in `fret-ui-kit`:
  `ecosystem/fret-ui-kit/src/primitives/menu/{content,content_panel,sub_content}.rs` and
  `ecosystem/fret-ui-kit/src/primitives/popper_content.rs` now keep `AnyElement` only as the final
  menu semantics/panel or popper wrapper seam, while their child closures accept iterable
  `IntoUiElement<H>` values directly. Direct wrappers land through `collect_children(...)`, and
  delegate wrappers now forward only to already-typed helper seams. The expanded source-policy gate
  and `cargo check -p fret-ui-kit --lib` lock that posture.
- the same wrapper cleanup now also reaches the cache/list batch in `fret-ui-kit`:
  `ecosystem/fret-ui-kit/src/declarative/cached_subtree.rs` and
  `ecosystem/fret-ui-kit/src/declarative/list.rs` now keep `AnyElement` only as the final
  cache-root or list-row wrapper seam, while their child closures accept iterable
  `IntoUiElement<...>` values directly. `cached_subtree` lands through `collect_children(...)`
  behind the `ViewCache` wrapper, and the retained list path now lands cached row payloads through
  the same helper. The expanded source-policy gate and `cargo check -p fret-ui-kit --lib` lock
  that posture.
- the same wrapper cleanup now also reaches the tab/toggle/accordion primitives in `fret-ui-kit`:
  `ecosystem/fret-ui-kit/src/primitives/{tabs,toggle,accordion}.rs` now keep `AnyElement` only as
  the final semantics/pressable/content wrapper seam, while their public child closures accept
  iterable `IntoUiElement<H>` values directly. The tabs/accordion roving-list wrappers, toggle and
  accordion pressable wrappers, and tab/accordion content wrappers now land those typed values
  behind `collect_children(...)`. The expanded source-policy gate and
  `cargo check -p fret-ui-kit --lib` lock that posture.
- the same wrapper cleanup now also reaches the dialog/popover/alert-dialog/select/tooltip overlay helpers in
  `fret-ui-kit`:
  `ecosystem/fret-ui-kit/src/primitives/{alert_dialog,dialog,popover,select,tooltip}.rs` now accept iterable
  `IntoUiElement<H>` values on wrapper helpers that still have an `ElementContext` available, and
  land them through `collect_children(...)` before assembling barriers, semantics wrappers, and
  modal layer vectors. The final overlay-request constructors remain the explicit raw
  `AnyElement` landing seam because they no longer have a live `ElementContext`. The dedicated
  source-policy test and `cargo check -p fret-ui-kit --lib` lock that posture.
- the same wrapper cleanup now also reaches the sortable DnD recipe helper in `fret-ui-kit`:
  `ecosystem/fret-ui-kit/src/recipes/sortable_dnd.rs` now accepts iterable `IntoUiElement<H>`
  row-content closures directly and lands them through `collect_children(...)` inside the row
  container wrapper. The expanded source-policy gate and `cargo check -p fret-ui-kit --lib` lock
  that posture.
- the same wrapper cleanup now also reaches the virtualized table helpers in `fret-ui-kit`:
  `ecosystem/fret-ui-kit/src/declarative/table.rs` now accepts iterable `IntoUiElement<H>` values
  for header and cell render closures in `table_virtualized(...)` and
  `table_virtualized_copyable(...)`, with table-owned header containers landing typed values
  through `collect_children(...)`. The expanded source-policy gate and
  `cargo check -p fret-ui-kit --lib` lock that posture.
- validation addendum on 2026-03-14:
  `CARGO_TARGET_DIR=target/codex-fret-cookbook cargo test -p fret-cookbook --lib --message-format=short`
- validation addendum on 2026-03-14:
  `CARGO_TARGET_DIR=target/codex-fret-cookbook cargo check -p fret-cookbook --examples --message-format=short`
- validation addendum on 2026-03-14:
  `CARGO_TARGET_DIR=target/codex-fret-ui-kit cargo test -p fret-ui-kit source_policy_tests::wrapper_helpers_prefer_typed_child_inputs --lib --message-format=short`
- validation addendum on 2026-03-14:
  `CARGO_TARGET_DIR=target/codex-fret-ui-kit cargo check -p fret-ui-kit --lib --message-format=short`
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
  `DocSection::build(cx, ...)`, while the Fret-only diagnostics harnesses now live in
  `apps/fret-ui-gallery/src/ui/diagnostics/scroll_area/{drag_baseline,expand_at_bottom}.rs` and
  are registered through `DocSection::build_diagnostics(cx, ...)`; the old
  `pub fn render(...) -> AnyElement` teaching pattern is now forbidden for the app-facing family by
  `ui_authoring_surface_default_app::{copyable_ui_gallery_snippet_lane_has_no_top_level_raw_render_roots,scroll_area_app_facing_snippets_prefer_ui_cx_on_the_default_app_surface,scroll_area_page_uses_typed_doc_sections_for_app_facing_snippets}`, while
  `selected_scroll_area_snippet_helpers_prefer_into_ui_element_over_anyelement` now also locks the
  helper-family preference for `shadcn::scroll_area(...)`, and
  `scroll_area_app_facing_snippet_lane_has_no_raw_boundaries`,
  `scroll_area_diagnostics_lane_keeps_intentional_raw_boundaries`, and
  `ui_gallery_diagnostics_raw_render_roots_are_explicitly_documented` prevent the two diagnostics
  harnesses from regressing back into the copyable snippet lane or silently losing their rationale.
- validation addendum on 2026-03-14:
  `CARGO_TARGET_DIR=target/codex-ui-gallery cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app scroll_area_diagnostics_lane_keeps_intentional_raw_boundaries -- --exact --nocapture`
- the same UI Gallery default-app top-level snippet cleanup now also records the
  `progress` family:
  `apps/fret-ui-gallery/src/ui/snippets/progress/{demo,usage,label,controlled,rtl}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, while
  `apps/fret-ui-gallery/src/ui/pages/progress.rs` consumes those previews through
  `DocSection::build(cx, ...)`; the old `pub fn render(...) -> AnyElement` teaching pattern is now
  forbidden for that family by
  `ui_authoring_surface_default_app::{progress_snippets_prefer_ui_cx_on_the_default_app_surface,progress_page_uses_typed_doc_sections_for_app_facing_snippets}`.
- the same UI Gallery default-app top-level snippet cleanup now also records the
  `chart` family:
  `apps/fret-ui-gallery/src/ui/snippets/chart/{demo,usage,contracts,tooltip,legend,rtl}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, while
  `apps/fret-ui-gallery/src/ui/pages/chart.rs` consumes those previews through
  `DocSection::build(cx, ...)`; the old `pub fn render(...) -> AnyElement` teaching pattern is now
  forbidden for that family by
  `ui_authoring_surface_default_app::{chart_snippets_prefer_ui_cx_on_the_default_app_surface,chart_page_uses_typed_doc_sections_for_app_facing_snippets}`.
- the same UI Gallery default-app top-level snippet cleanup now also records the
  `combobox` family:
  `apps/fret-ui-gallery/src/ui/snippets/combobox/{conformance_demo,basic,usage,label,auto_highlight,clear_button,groups,groups_with_separator,trigger_button,multiple_selection,custom_items,long_list,invalid,disabled,input_group,rtl}.rs`
  now expose typed app-facing `render(...)` signatures, while
  `apps/fret-ui-gallery/src/ui/pages/combobox.rs` consumes those previews through
  `DocSection::build(cx, ...)`; the old `-> AnyElement` teaching pattern is now forbidden for that
  family by
  `ui_authoring_surface_default_app::{combobox_snippets_prefer_ui_cx_on_the_default_app_surface,combobox_page_uses_typed_doc_sections_for_app_facing_snippets}`.
- the same UI Gallery default-app top-level snippet cleanup now also records the
  `carousel` family:
  `apps/fret-ui-gallery/src/ui/snippets/carousel/{demo,usage,parts,basic,sizes_thirds,sizes,spacing,spacing_responsive,orientation_vertical,options,api,events,plugin_autoplay,plugin_autoplay_controlled,plugin_autoplay_stop_on_focus,plugin_autoplay_stop_on_last_snap,plugin_autoplay_delays,plugin_wheel_gestures,rtl,loop_carousel,loop_downgrade_cannot_loop,focus_watch,duration_embla,expandable}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, while
  `apps/fret-ui-gallery/src/ui/pages/carousel.rs` consumes those previews through
  `DocSection::build(cx, ...)`; the old `pub fn render(...) -> AnyElement` teaching pattern is now
  forbidden for that family by
  `ui_authoring_surface_default_app::{carousel_snippets_prefer_ui_cx_on_the_default_app_surface,carousel_page_uses_typed_doc_sections_for_app_facing_snippets}`.
- the same UI Gallery default-app top-level snippet cleanup now also records the
  `item` family:
  `apps/fret-ui-gallery/src/ui/snippets/item/{demo,usage,variants,size,icon,avatar,image,group,header,link,dropdown,extras_rtl,gallery,link_render}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, while
  `apps/fret-ui-gallery/src/ui/pages/item.rs` consumes those previews through
  `DocSection::build(cx, ...)`; the old `pub fn render(...) -> AnyElement` teaching pattern is now
  forbidden for that family by
  `ui_authoring_surface_default_app::{item_snippets_prefer_ui_cx_on_the_default_app_surface,item_page_uses_typed_doc_sections_for_app_facing_snippets}`.
- the same UI Gallery default-app top-level snippet cleanup now also records the
  `table` family:
  `apps/fret-ui-gallery/src/ui/snippets/table/{demo,usage,footer,actions,rtl}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, while
  `apps/fret-ui-gallery/src/ui/pages/table.rs` consumes those previews through
  `DocSection::build(cx, ...)`; the old `pub fn render(...) -> AnyElement` teaching pattern is now
  forbidden for that family by
  `ui_authoring_surface_default_app::{table_app_facing_snippets_prefer_ui_cx_on_the_default_app_surface,table_page_uses_typed_doc_sections_for_app_facing_snippets}`.
- the same UI Gallery default-app top-level snippet cleanup now also records the remaining
  curated tail snippets:
  `breadcrumb/responsive.rs`, `date_picker/dropdowns.rs`, `form/notes.rs`, and `sidebar/rtl.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, while the corresponding
  app-facing sections on `pages/breadcrumb.rs`, `pages/date_picker.rs`, `pages/form.rs`, and
  `pages/sidebar.rs` now stay on `DocSection::build(cx, ...)`; the old `UiCx -> AnyElement`
  teaching pattern is now forbidden there by
  `ui_authoring_surface_default_app::{remaining_app_facing_tail_snippets_prefer_ui_cx_on_the_default_app_surface,remaining_app_facing_tail_pages_use_typed_doc_sections_for_app_facing_snippets}`.
- the same UI Gallery default-app top-level snippet cleanup now also records the `badge` family:
  `apps/fret-ui-gallery/src/ui/snippets/badge/{demo,usage,spinner,rtl,counts,colors,link,icon,variants}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, while
  `apps/fret-ui-gallery/src/ui/pages/badge.rs` consumes those previews through
  `DocSection::build(cx, ...)`; the old `ElementContext<'_, H> -> AnyElement` teaching pattern is
  now forbidden there by
  `ui_authoring_surface_default_app::{badge_snippets_prefer_ui_cx_on_the_default_app_surface,badge_page_uses_typed_doc_sections_for_app_facing_snippets}`.
- the same UI Gallery default-app top-level snippet cleanup now also records the
  `aspect_ratio` family:
  `apps/fret-ui-gallery/src/ui/snippets/aspect_ratio/{demo,usage,portrait,square,rtl}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`; the page surface on
  `apps/fret-ui-gallery/src/ui/pages/aspect_ratio.rs` already stays on
  `DocSection::build(cx, ...)`, while `render_preview(...)` remains the explicit asset-backed
  preview seam and the top-level `render(...)` functions stay as the copyable code-surface seam.
  This is now guarded by
  `ui_authoring_surface_default_app::{aspect_ratio_snippets_prefer_ui_cx_on_the_default_app_surface,aspect_ratio_page_uses_typed_doc_sections_for_app_facing_snippets}`.
- the same UI Gallery default-app top-level snippet cleanup now also records the
  `context_menu` family:
  `apps/fret-ui-gallery/src/ui/snippets/context_menu/{demo,basic,usage,submenu,shortcuts,groups,icons,checkboxes,radio,destructive,sides,rtl}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, while
  `apps/fret-ui-gallery/src/ui/pages/context_menu.rs` consumes those previews through
  `DocSection::build(cx, ...)`; the old `ElementContext<'_, H> -> AnyElement` teaching pattern is
  now forbidden there by
  `ui_authoring_surface_default_app::{context_menu_snippets_prefer_ui_cx_on_the_default_app_surface,context_menu_page_uses_typed_doc_sections_for_app_facing_snippets}`.
- the same UI Gallery default-app top-level snippet cleanup now also records the
  `dropdown_menu` family:
  `apps/fret-ui-gallery/src/ui/snippets/dropdown_menu/{avatar,basic,checkboxes,checkboxes_icons,complex,demo,destructive,icons,parts,radio_group,radio_icons,rtl,shortcuts,submenu,usage}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, keep their menu-local
  checkbox/radio/demo state beside the snippet itself, and
  `apps/fret-ui-gallery/src/ui/pages/dropdown_menu.rs` now routes those snippet-backed sections
  through `DocSection::build(cx, ...)` instead of `DocSection::new(...)`.
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
- the same UI Gallery default-app top-level snippet cleanup now also records the `menubar` family:
  `apps/fret-ui-gallery/src/ui/snippets/menubar/{checkbox,demo,parts,radio,rtl,submenu,usage,with_icons}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, keep their menubar-local
  checkbox/radio/demo state beside the snippet itself, and
  `apps/fret-ui-gallery/src/ui/pages/menubar.rs` now routes those snippet-backed sections through
  `DocSection::build(cx, ...)` instead of `DocSection::new(...)`.
- the same UI Gallery default-app top-level snippet cleanup now also records the `popover` family:
  `apps/fret-ui-gallery/src/ui/snippets/popover/{align,basic,demo,rtl,usage,with_form}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, keep their popover-local
  form state beside the snippet itself, and
  `apps/fret-ui-gallery/src/ui/pages/popover.rs` now routes those snippet-backed sections through
  `DocSection::build(cx, ...)` instead of `DocSection::new(...)`.
- the same UI Gallery default-app top-level snippet cleanup now also records the `hover_card` family:
  `apps/fret-ui-gallery/src/ui/snippets/hover_card/{basic,demo,positioning,rtl,sides,trigger_delays,usage}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, keep their demo assets
  and timing/placement state beside the snippet itself, and
  `apps/fret-ui-gallery/src/ui/pages/hover_card.rs` now routes those snippet-backed sections
  through `DocSection::build(cx, ...)` instead of `DocSection::new(...)`.
- the same UI Gallery default-app top-level snippet cleanup now also records the `tooltip` family:
  `apps/fret-ui-gallery/src/ui/snippets/tooltip/{demo,disabled_button,keyboard_focus,keyboard_shortcut,long_content,rtl,sides,usage}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, keep their tooltip-local
  provider/content composition beside the snippet itself, and
  `apps/fret-ui-gallery/src/ui/pages/tooltip.rs` now routes those snippet-backed sections through
  `DocSection::build(cx, ...)` instead of `DocSection::new(...)`.
- the same UI Gallery default-app top-level snippet cleanup now also records the `button` family:
  `apps/fret-ui-gallery/src/ui/snippets/button/{demo,usage,size,default,outline,secondary,ghost,destructive,link,icon,with_icon,rounded,loading,button_group,link_render,rtl,variants}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, while
  `apps/fret-ui-gallery/src/ui/pages/button.rs` consumes those previews through
  `DocSection::build(cx, ...)`; the old `ElementContext<'_, H> -> AnyElement` teaching pattern is
  now forbidden there by
  `ui_authoring_surface_default_app::{button_snippets_prefer_ui_cx_on_the_default_app_surface,button_page_uses_typed_doc_sections_for_app_facing_snippets}`.
- the same UI Gallery default-app top-level snippet cleanup now also records the `button_group`
  family:
  `apps/fret-ui-gallery/src/ui/snippets/button_group/{accessibility,button_group_select,demo,dropdown_menu,flex_1_items,input,input_group,nested,orientation,popover,rtl,separator,size,split,text,usage}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, while
  `apps/fret-ui-gallery/src/ui/pages/button_group.rs` consumes those previews through
  `DocSection::build(cx, ...)`; the old `ElementContext<'_, H> -> AnyElement` teaching pattern is
  now forbidden there by
  `ui_authoring_surface_default_app::{button_group_snippets_prefer_ui_cx_on_the_default_app_surface,button_group_page_uses_typed_doc_sections_for_app_facing_snippets}`.
- the same UI Gallery default-app top-level snippet cleanup now also records the `input_group`
  family:
  `apps/fret-ui-gallery/src/ui/snippets/input_group/{align_block_end,align_block_start,align_inline_end,align_inline_start,button,button_group,custom_input,demo,dropdown,icon,kbd,label,rtl,spinner,text,textarea,tooltip}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, while
  `apps/fret-ui-gallery/src/ui/pages/input_group.rs` consumes those previews through
  `DocSection::build(cx, ...)`; the old `ElementContext<'_, H> -> AnyElement` teaching pattern is
  now forbidden there by
  `ui_authoring_surface_default_app::{input_group_snippets_prefer_ui_cx_on_the_default_app_surface,input_group_page_uses_typed_doc_sections_for_app_facing_snippets}`.
- the same UI Gallery default-app top-level snippet cleanup now also records the `toggle_group`
  family:
  `apps/fret-ui-gallery/src/ui/snippets/toggle_group/{custom,demo,disabled,flex_1_items,full_width_items,label,large,outline,rtl,single,size,small,spacing,usage,vertical}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, while
  `apps/fret-ui-gallery/src/ui/pages/toggle_group.rs` consumes those previews through
  `DocSection::build(cx, ...)`; the old `ElementContext<'_, H> -> AnyElement` teaching pattern is
  now forbidden there by
  `ui_authoring_surface_default_app::{toggle_group_snippets_prefer_ui_cx_on_the_default_app_surface,toggle_group_page_uses_typed_doc_sections_for_app_facing_snippets}`.
- the same UI Gallery default-app top-level snippet cleanup now also records the `switch` family:
  `apps/fret-ui-gallery/src/ui/snippets/switch/{airplane_mode,bluetooth,choice_card,description,disabled,invalid,label,rtl,sizes,usage}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, while
  `apps/fret-ui-gallery/src/ui/pages/switch.rs` consumes those previews through
  `DocSection::build(cx, ...)`; the old `ElementContext<'_, H> -> AnyElement` teaching pattern is
  now forbidden there by
  `ui_authoring_surface_default_app::{switch_snippets_prefer_ui_cx_on_the_default_app_surface,switch_page_uses_typed_doc_sections_for_app_facing_snippets}`.
- the same UI Gallery default-app top-level snippet cleanup now also records the `checkbox`
  family:
  `apps/fret-ui-gallery/src/ui/snippets/checkbox/{basic,checked_state,demo,description,disabled,group,invalid_state,label,rtl,table,usage,with_title}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, while
  `apps/fret-ui-gallery/src/ui/pages/checkbox.rs` consumes those previews through
  `DocSection::build(cx, ...)`; the old `ElementContext<'_, H> -> AnyElement` teaching pattern is
  now forbidden there by
  `ui_authoring_surface_default_app::{checkbox_snippets_prefer_ui_cx_on_the_default_app_surface,checkbox_page_uses_typed_doc_sections_for_app_facing_snippets}`.
- the same UI Gallery default-app top-level snippet cleanup now also records the `separator`
  family:
  `apps/fret-ui-gallery/src/ui/snippets/separator/{demo,list,menu,rtl,usage,vertical}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, while
  `apps/fret-ui-gallery/src/ui/pages/separator.rs` consumes those previews through
  `DocSection::build(cx, ...)`; the old `ElementContext<'_, H> -> AnyElement` teaching pattern is
  now forbidden there by
  `ui_authoring_surface_default_app::{separator_snippets_prefer_ui_cx_on_the_default_app_surface,separator_page_uses_typed_doc_sections_for_app_facing_snippets}`.
- the next UI Gallery app-facing snippet batch is now `input`, `field`, `textarea`,
  `input_otp`, and `select`, followed by the remaining
  `ElementContext<'_, H> -> AnyElement` default-authoring families.
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
- UI Gallery default-app top-level snippet migration also closed a second high-yield family batch:
  `input`, `field`, `textarea`, `input_otp`, and `select` now expose
  `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, while the corresponding pages
  consume those previews through `DocSection::build(cx, ...)` and the focused
  `ui_authoring_surface_default_app::{input_*,field_*,textarea_*,input_otp_*,select_*}` tests
  now guard the teaching surface.
- UI Gallery default-app top-level snippet migration also closed a third high-yield family batch:
  `calendar`, `alert_dialog`, `dialog`, `drawer`, and `sheet` now expose
  `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, while the corresponding pages
  consume those previews through `DocSection::build(cx, ...)` and the focused
  `ui_authoring_surface_default_app::{calendar_*,alert_dialog_*,dialog_*,drawer_*,sheet_*}` tests
  now guard the teaching surface.
- UI Gallery default-app top-level snippet migration also closed a fourth high-yield family batch:
  `spinner`, `form`, `empty`, `breadcrumb`, and `collapsible` now expose
  `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, while the corresponding pages
  consume those previews through `DocSection::build(cx, ...)` and the focused
  `ui_authoring_surface_default_app::{spinner_*,form_*,empty_*,breadcrumb_*,collapsible_*}` tests
  now guard the teaching surface.
- UI Gallery default-app top-level snippet migration also closed a fifth high-yield family batch:
  `skeleton`, `pagination`, `alert`, `sidebar`, and `label` now expose
  `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, while the corresponding pages
  consume those previews through `DocSection::build(cx, ...)` and the focused
  `ui_authoring_surface_default_app::{skeleton_*,pagination_*,alert_*,sidebar_*,label_*}` tests
  now guard the teaching surface.
- UI Gallery default-app top-level snippet migration also closed a sixth family batch:
  `kbd`, `icons`, and `sonner` now expose
  `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, while the corresponding pages
  consume those previews through `DocSection::build(cx, ...)`; on the `sonner` page, the snippet
  module now also owns the local toaster id, last-action state, and toaster-position state instead
  of relaying page models through `ui/content.rs`.
- the focused
  `ui_authoring_surface_default_app::{kbd_*,icons_*,sonner_*}` tests now guard that teaching
  surface.
- UI Gallery default-app top-level snippet migration also closed the next state-heavy family:
  `date_picker` now exposes
  `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>` across
  `apps/fret-ui-gallery/src/ui/snippets/date_picker/{basic,demo,dob,dropdowns,input,label,natural_language,notes,presets,range,rtl,time_picker,usage}.rs`,
  while `apps/fret-ui-gallery/src/ui/pages/date_picker.rs` now only assembles doc sections and no
  longer relays per-demo state through the page shell.
- the focused `ui_authoring_surface_default_app::{date_picker_*}` tests now guard that teaching
  surface.
- UI Gallery default-app top-level snippet migration also closed the self-contained avatar family:
  `apps/fret-ui-gallery/src/ui/snippets/avatar/{badge_icon,basic,demo,dropdown,fallback_only,group,group_count,group_count_icon,rtl,sizes,usage,with_badge}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, while
  `apps/fret-ui-gallery/src/ui/pages/avatar.rs` now only assembles doc sections and no longer
  relays a page-owned image model through the docs shell.
- the focused `ui_authoring_surface_default_app::{avatar_*,selected_avatar_*}` tests now guard
  that teaching surface.
- the same UI Gallery default-app source gate now also records the `command` family:
  `apps/fret-ui-gallery/src/ui/snippets/command/{action_first_view,basic,docs_demo,groups,loading,rtl,scrollable,shortcuts,usage}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, while
  `apps/fret-ui-gallery/src/ui/pages/command.rs` consumes those previews through
  `DocSection::build(cx, ...)` and no longer relays a page-owned `last_action` model through the
  page shell; the shared local state/action helpers now live in `snippets/command/mod.rs`. The
  old teaching pattern is now forbidden there by
  `ui_authoring_surface_default_app::{command_*}`.
- the same UI Gallery default-app source gate now also records the `card` family:
  `apps/fret-ui-gallery/src/ui/snippets/card/{card_content,compositions,demo,image,meeting_notes,rtl,size,usage}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, while
  `apps/fret-ui-gallery/src/ui/pages/card.rs` consumes those previews through
  `DocSection::build(cx, ...)` and no longer relays a page-owned `event_cover_image` model; the
  `image` example now resolves its demo `ImageSource` inside the snippet. The old teaching
  pattern is now forbidden there by
  `ui_authoring_surface_default_app::{card_*,selected_card_*}`.
- the same UI Gallery default-app source gate now also records the `image_object_fit` family:
  `apps/fret-ui-gallery/src/ui/snippets/image_object_fit/{mapping,sampling}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, while
  `apps/fret-ui-gallery/src/ui/pages/image_object_fit.rs` consumes those previews through
  `DocSection::build(cx, ...)` and no longer relays gallery-owned `ImageId` models; the snippet
  module now generates its own fit/sampling demo `ImageSource`s. The old teaching pattern is now
  forbidden there by
  `ui_authoring_surface_default_app::{image_object_fit_*}`.
- the same UI Gallery default-app source gate now also records the `data_table` family:
  `apps/fret-ui-gallery/src/ui/snippets/data_table/{basic_demo,code_outline,default_demo,guide_demo,rtl_demo}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, while
  `apps/fret-ui-gallery/src/ui/pages/data_table.rs` consumes those previews through
  `DocSection::build(cx, ...)`; `guide_demo` now owns its `TableState` locally instead of
  relaying a gallery-wide model through `ui/content.rs`, and the obsolete gallery relay fields
  `data_table_state` plus `image_fit_demo_streaming_image` are now deleted from
  `ui/models.rs`, `driver/window_bootstrap.rs`, and `driver/runtime_driver.rs`. The old teaching
  pattern is now forbidden there by
  `ui_authoring_surface_default_app::{data_table_*,selected_data_table_*}`.
- validation addendum on 2026-03-14:
  `CARGO_TARGET_DIR=target/codex-ui-gallery cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app data_table_ -- --nocapture`
- the same UI Gallery default-app source gate now also records the `motion_presets` family:
  `apps/fret-ui-gallery/src/ui/snippets/motion_presets/{preset_selector,fluid_tabs_demo,overlay_demo,stack_shift_list_demo,stagger_demo,token_snapshot}.rs`
  now expose typed top-level `render(...)` surfaces returning `impl UiChild + use<>`, while
  `apps/fret-ui-gallery/src/ui/pages/motion_presets.rs` now consumes those previews through
  `DocSection::build(cx, ...)`; `preset_selector` intentionally keeps the global
  `motion_preset` / `motion_preset_open` model seam because it drives the page-wide theme patch,
  while `overlay_demo`, `stagger_demo`, `stack_shift_list_demo`, and `token_snapshot` now keep
  their local dialog/theme access inside the snippet itself. The old teaching pattern is now
  forbidden there by
  `ui_authoring_surface_default_app::{motion_preset_snippets_prefer_ui_cx_on_the_default_app_surface,motion_presets_page_uses_typed_doc_sections_for_app_facing_snippets,selected_motion_presets_snippet_helpers_prefer_into_ui_element_over_anyelement}`.
- validation addendum on 2026-03-14:
  `CARGO_TARGET_DIR=target/codex-ui-gallery cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app motion_preset -- --nocapture`
- the specialized `typography` teaching lane now also closes its remaining first-party drift:
  `apps/fret-ui-gallery/src/ui/snippets/typography/*.rs` now expose
  `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, while
  `apps/fret-ui-gallery/src/ui/pages/typography.rs` now consumes those previews through
  `DocSection::build(cx, ...)`; the stale non-dev `dialog_open` relay is now gated back to
  `gallery-dev` only in the gallery runtime state/model path.
- validation addendum on 2026-03-14:
  `CARGO_TARGET_DIR=target/codex-ui-gallery cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app typography_ -- --nocapture`
- the specialized `shadcn_extras` teaching lane now closes its remaining first-party drift too:
  `apps/fret-ui-gallery/src/ui/snippets/shadcn_extras/*.rs` now expose
  `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, while
  `apps/fret-ui-gallery/src/ui/pages/shadcn_extras.rs` now consumes those previews through
  `DocSection::build(cx, ...)`.
- validation addendum on 2026-03-14:
  `CARGO_TARGET_DIR=target/codex-ui-gallery cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app shadcn_extras_ -- --nocapture`
- the specialized `material3` lane has now advanced through its `controls` sub-batch:
  `apps/fret-ui-gallery/src/ui/snippets/material3/{badge,button,checkbox,icon_button,radio,segmented_button,slider,switch,touch_targets}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, and the new source gate
  locks those controls to the typed default-app teaching surface without reintroducing
  `ElementContext<'_, H>` helper parameters in the affected helper closures.
- validation addendum on 2026-03-14:
  `CARGO_TARGET_DIR=target/codex-ui-gallery cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app material3_controls_snippets_prefer_ui_cx_on_the_default_app_surface -- --nocapture`
- the specialized `material3` lane has now also advanced through its `inputs` sub-batch:
  `apps/fret-ui-gallery/src/ui/snippets/material3/{autocomplete,date_picker,select,text_field,time_picker}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, and the field-family
  source gates now lock both the local uncontrolled/copyable-root posture and the typed
  default-app teaching surface for those snippets.
- validation addendum on 2026-03-14:
  `CARGO_TARGET_DIR=target/codex-ui-gallery cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app material3_ -- --nocapture`
- the specialized `material3` lane has now also advanced through its `navigation` sub-batch:
  `apps/fret-ui-gallery/src/ui/snippets/material3/{list,modal_navigation_drawer,navigation_bar,navigation_drawer,navigation_rail,tabs,top_app_bar}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, and the corresponding
  navigation/value-root source gates now lock both the typed top-level teaching surface and the
  removal of host-bound helper parameter spellings in the affected exemplar helpers.
- validation addendum on 2026-03-14:
  `CARGO_TARGET_DIR=target/codex-ui-gallery cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app material3_ -- --nocapture`
- the specialized `material3` lane has now also advanced through its `overlays` sub-batch:
  `apps/fret-ui-gallery/src/ui/snippets/material3/{bottom_sheet,dialog,menu,snackbar,tooltip}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>` (or the same typed
  signature with the existing `last_action` model parameter), and the overlay source gates now
  lock both the typed top-level teaching surface and the local uncontrolled/copyable-root posture
  for the dialog/menu/bottom-sheet exemplars.
- validation addendum on 2026-03-14:
  `CARGO_TARGET_DIR=target/codex-ui-gallery cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app material3_ -- --nocapture`
- the specialized `material3` lane is now fully aligned on the first-party default teaching
  surface:
  `apps/fret-ui-gallery/src/ui/snippets/material3/{gallery,state_matrix}.rs` now also expose the
  typed `UiCx -> impl UiChild` posture, their remaining helper signatures no longer spell
  `ElementContext<'_, H>`, and the composite source gates now close the last Material 3 teaching
  drift on this lane.
- validation addendum on 2026-03-14:
  `CARGO_TARGET_DIR=target/codex-ui-gallery cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app material3_ -- --nocapture`
- the specialized `ai` lane has now advanced through its first curated snippet sweep:
  `apps/fret-ui-gallery/src/ui/snippets/ai/{agent_demo,artifact_demo,artifact_code_display,attachments_empty,attachments_grid,attachments_inline,attachments_list,attachments_usage,audio_player_demo,chain_of_thought_composable,chain_of_thought_demo,chat_demo,checkpoint_demo,code_block_demo,commit_custom_children,commit_demo,commit_large_demo,confirmation_accepted,confirmation_demo,confirmation_rejected,confirmation_request,context_default,context_demo,file_tree_basic,file_tree_demo,file_tree_expanded,file_tree_large,inline_citation_demo,message_demo,mic_selector_demo,model_selector_demo,open_in_chat_demo,package_info_demo,persona_basic,persona_custom_styling,persona_custom_visual,persona_demo,persona_state_management,persona_variants,plan_demo,prompt_input_action_menu_demo,prompt_input_docs_demo,prompt_input_provider_demo,prompt_input_referenced_sources_demo,reasoning_demo,schema_display_demo,shimmer_demo,shimmer_duration_demo,shimmer_elements_demo,snippet_demo,snippet_plain,sources_demo,stack_trace_collapsed,stack_trace_demo,stack_trace_large_demo,stack_trace_no_internal,task_demo,terminal_demo,test_results_basic,test_results_demo,test_results_errors,test_results_large_demo,test_results_suites,tool_demo,voice_selector_demo,web_preview_demo,workflow_canvas_demo,workflow_chrome_demo,workflow_connection_demo,workflow_controls_demo,workflow_edge_demo,workflow_node_demo,workflow_node_graph_demo,workflow_panel_demo,workflow_toolbar_demo}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, and the new
  `ui_authoring_surface_default_app::ai_curated_snippets_prefer_ui_cx_on_the_default_app_surface`
  gate forbids those exemplars from drifting back to `ElementContext<'_, H> -> AnyElement`.
- the specialized `ai` lane has now also closed its remaining top-level tail:
  `apps/fret-ui-gallery/src/ui/snippets/ai/{canvas_world_layer_spike,conversation_demo,environment_variables_demo,image_demo,message_branch_demo,message_usage,queue_demo,sandbox_demo,speech_input_demo,suggestions_demo,transcript_torture,transcription_demo}.rs`
  now expose the same typed `UiCx -> impl UiChild` posture and are covered by the same source
  gate.
- validation addendum on 2026-03-14:
  `CARGO_TARGET_DIR=target/codex-ui-gallery cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app ai_curated_snippets_prefer_ui_cx_on_the_default_app_surface -- --nocapture`
- remaining specialized-lane count update on 2026-03-14:
  old-signature top-level `ai` snippet renders now fall from 87 to 0.
- after these tracked landings, the tracked default-app workstream-local teaching-surface lane is
  now effectively closed; remaining work continues on the specialized `ai` lane plus any optional
  post-cleanup of now-nonessential gallery runtime fields.
